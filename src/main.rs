// Standard I/O module for basic input and output operations.
use std::io;
use std::sync::Arc; // For shared ownership of the PostgreSQL client.

// AsyncWrite trait from tokio, required for asynchronous write operations.
use tokio::io::AsyncWrite;
use tokio::net::TcpListener; // TcpListener from tokio for listening to TCP connections.

// Importing necessary components from the opensrv_mysql crate.
use async_trait::async_trait;
use opensrv_mysql::*;

// Additional imports for PostgreSQL support and environment variables handling.
use dotenv::dotenv;
use std::env;
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

                // Create a default OkResponse and modify the affected_rows field
                let mut response = OkResponse::default();
                response.affected_rows = row_count; // Set the actual number of affected rows

                // Use this updated response
                results.completed(response).await
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
    dotenv().ok(); // Load environment variables from .env file.

    let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
    let db_user = env::var("DB_USER").expect("DB_USER must be set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");

    let connection_string = format!("host={} user={} password={}", db_host, db_user, db_password);

    // Connect to PostgreSQL database.
    let (pg_client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

    // The connection object performs the communication with the database, so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let pg_client = Arc::new(pg_client); // Wrap the client in an Arc for shared ownership.
    let listener = TcpListener::bind("0.0.0.0:3306").await?;

    println!(
        r#"
________             ___________  ___       ________              _____             ______      ______
___  __ \______________  /___   |/  /____  ____  __ \___  __________  /______ _________  /_________  /
__  /_/ /  __ \_  ___/  __/_  /|_/ /__  / / /_  /_/ /  / / /_  ___/  __/  __ `/  ___/_  __ \  _ \_  /
_  ____// /_/ /(__  )/ /_ _  /  / / _  /_/ /_  _, _// /_/ /_(__  )/ /_ / /_/ // /__ _  / / /  __//_/
/_/     \____//____/ \__/ /_/  /_/  _\__, / /_/ |_| \__,_/ /____/ \__/ \__,_/ \___/ /_/ /_/\___/(_)
                                    /____/
"#
    );

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
