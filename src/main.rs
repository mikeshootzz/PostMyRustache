// Standard I/O module for basic input and output operations.
use std::io;
use std::sync::Arc; // For shared ownership of the PostgreSQL client.

// AsyncWrite trait from tokio, required for asynchronous write operations.
use tokio::io::AsyncWrite;
use tokio::net::TcpListener; // TcpListener from tokio for listening to TCP connections.

// Importing necessary components from the opensrv_mysql crate.
use async_trait::async_trait;
use opensrv_mysql::*;
use mysql_common as myc;

// Additional imports for PostgreSQL support and environment variables handling.
use dotenv::dotenv;
use std::env;
use tokio_postgres::{Client, NoTls};

// Backend struct that will implement the AsyncMysqlShim trait and hold a PostgreSQL client.
struct Backend {
    pg_client: Arc<Client>,
}

#[async_trait]
impl<W: AsyncWrite + Send + Unpin> AsyncMysqlShim<W> for Backend {
    type Error = io::Error;

    async fn on_prepare<'a>(
        &'a mut self,
        _: &'a str,
        _info: StatementMetaWriter<'a, W>,
    ) -> io::Result<()> {
        todo!()
    }

    async fn on_execute<'a>(
        &'a mut self,
        _: u32,
        _: opensrv_mysql::ParamParser<'a>,
        _: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        todo!()
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
        } else if sql.trim().to_lowercase().starts_with("create database if not exists") {
            // Intercepting a MySQL-specific CREATE DATABASE IF NOT EXISTS query.
            let db_name = sql.trim().split_whitespace().last().unwrap();
            let check_db_exists = format!("SELECT 1 FROM pg_database WHERE datname = '{}'", db_name);
            match self.pg_client.execute(&check_db_exists, &[]).await {
                Ok(_) => {
                    println!("Database {} already exists, skipping creation.", db_name);
                    return results.completed(OkResponse::default()).await;
                },
                Err(_) => {
                    // Handle error...
                }
            } // Add closing brace here
        } else if sql.trim().to_lowercase().contains("database()") {
            // Intercepting a query that contains the MySQL-specific `database()` function.
            let modified_sql = sql.to_lowercase().replace("database()", "current_database()");
            match self.pg_client.execute(&modified_sql, &[]).await {
                Ok(_) => {
                    println!("Query executed successfully.");
                    return results.completed(OkResponse::default()).await;
                },
                Err(err) => {
                    println!("Error executing query: {:?}", err);
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to execute query."));
                }
            }
        }
    
        // Rest of the function...

    
        // Forward other queries to PostgreSQL.
        match self.pg_client.execute(sql, &[]).await {
            Ok(row_count) => {
                println!("Query executed successfully, {} rows affected.", row_count);
    
                if sql.trim().to_lowercase().starts_with("select") {
                    println!("SELECT query was found"); 
            // Start the resultset response with columns information
            //let mut row_writer = results.start(&[]).await?;
            let pg_results_raw = self.pg_client.execute(sql, &[]).await;
            println!("{:?}", pg_results_raw);

            // Execute the same query against PostgreSQL to get the results
            let pg_results = self.pg_client.query(sql, &[]).await.map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Error executing query: {:?}", e),
                )
            })?;

            println!("result: {:?}", pg_results);

            let mut column_names: Vec<String> = Vec::new();
            let mut cols: Vec<Column> = Vec::new();


            if let Some(first_row) = pg_results.get(0) {
                let columns = first_row.columns();
                column_names = columns.iter().map(|col| col.name().to_string()).collect();
            
                // Populate cols vector here, outside of the row iteration loop
                for column_name in &column_names {
                    cols.push(Column {
                        table: String::new(),
                        column: column_name.to_string(),
                        coltype: myc::constants::ColumnType::MYSQL_TYPE_LONG,
                        colflags: myc::constants::ColumnFlags::UNSIGNED_FLAG,
                    });
                }
            
                // Iterate over rows and send each row to the MySQL client
                let mut w = results.start(&cols).await?;
                for row in &pg_results {
                    let mut row_values = Vec::new();
                    for (i, column_name) in column_names.iter().enumerate() {
                        let column_type = row.columns()[i].type_();
                        let value = match *column_type {
                            tokio_postgres::types::Type::INT4 => {
                                let value: i32 = row.get(i);
                                myc::Value::Int(value.into())
                            },
                            tokio_postgres::types::Type::VARCHAR => {
                                let value: String = row.get(i);
                                myc::Value::Bytes(value.into_bytes())
                            },
                            tokio_postgres::types::Type::BOOL => {
                                let value: bool = row.get(i);
                                myc::Value::Bytes(value.to_string().into_bytes())
                            },
                            tokio_postgres::types::Type::FLOAT4 => {
                                let value: f32 = row.get(i);
                                myc::Value::Float(value)
                            },
                            tokio_postgres::types::Type::FLOAT8 => {
                                let value: f64 = row.get(i);
                                myc::Value::Double(value)
                            },
                            // Add more match arms for other types as needed
                            _ => return Err(io::Error::new(io::ErrorKind::Other, "Unsupported type")),
                        };
                        println!("Column: '{}', Value being sent: {:?}", column_name, value); // Debugging line
                        row_values.push(value);
}
                    // Write each row separately
                    w.write_row(row_values).await?;
                }
                w.finish().await?;
            }
                } else {
                    // For non-SELECT queries, send response indicating rows affected
                    let mut response = OkResponse::default();
                    response.affected_rows = row_count; // Set the actual number of affected rows
                    results.completed(response).await?;
                }
            }
            Err(e) => {
                println!("Error executing query: {:?}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to execute query.",
                ));
            }
        }

        Ok(())
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
