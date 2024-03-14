// Standard I/O module for basic input and output operations.
use std::io;
use std::sync::Arc; // For shared ownership of the PostgreSQL client.

// AsyncWrite trait from tokio, required for asynchronous write operations.
use tokio::io::AsyncWrite;
use tokio::net::TcpListener; // TcpListener from tokio for listening to TCP connections.

// Importing necessary components from the opensrv_mysql crate.
use async_trait::async_trait;
use opensrv_mysql::*;

// Additional imports for PostgreSQL support.
use tokio_postgres::{Client, NoTls};

// Backend struct that will implement the AsyncMysqlShim trait and hold a PostgreSQL client.
struct Backend {
    pg_client: Arc<Client>,
}

// Implementation of the AsyncMysqlShim trait for the Backend.
#[async_trait]
impl<W: AsyncWrite + Send + Unpin> AsyncMysqlShim<W> for Backend {
    type Error = io::Error;

    async fn on_prepare<'a>(
        &'a mut self,
        _: &'a str,
        info: StatementMetaWriter<'a, W>,
    ) -> io::Result<()> {
        info.reply(42, &[], &[]).await
    }

    async fn on_execute<'a>(
        &'a mut self,
        _: u32,
        _: opensrv_mysql::ParamParser<'a>,
        results: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        results.completed(OkResponse::default()).await
    }

    async fn on_close(&mut self, _: u32) {
        // Clean up resources here, if necessary.
    }

    async fn on_query<'a>(
        &'a mut self,
        sql: &'a str,
        results: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        println!("Received SQL query: {:?}", sql);

        // Check and handle MySQL-specific system variable queries or other incompatible queries.
        if sql
            .trim()
            .eq_ignore_ascii_case("select @@version_comment limit 1")
        {
            println!("Intercepted MySQL-specific query, returning dummy response.");
            return results.completed(OkResponse::default()).await;
        } else if sql.trim().starts_with("select $$") {
            // Intercepting a query that's not compatible with PostgreSQL.
            println!("Intercepted query with unsupported syntax, returning dummy response.");
            return results.completed(OkResponse::default()).await;
        }

        // Forward other queries to PostgreSQL.
        match self.pg_client.execute(sql, &[]).await {
            Ok(row_count) => {
                println!("Query executed successfully, {} rows affected.", row_count);
                results.completed(OkResponse::default()).await
            }
            Err(e) => {
                println!("Error executing query: {:?}", e);
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to execute query.",
                ))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PostgreSQL database.
    let (pg_client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1234", NoTls).await?;

    // The connection object performs the communication with the database, so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let pg_client = Arc::new(pg_client); // Wrap the client in an Arc for shared ownership.
    let listener = TcpListener::bind("0.0.0.0:3306").await?;
    println!("MySQL server is running on port 3306");

    loop {
        let (stream, _) = listener.accept().await?;
        let (r, w) = stream.into_split();
        let pg_client_clone = Arc::clone(&pg_client); // Clone the Arc, not the Client.
        tokio::spawn(async move {
            if let Err(e) = AsyncMysqlIntermediary::run_on(
                Backend {
                    pg_client: pg_client_clone,
                },
                r,
                w,
            )
            .await
            {
                eprintln!("Error: {}", e);
            }
        });
    }
}
