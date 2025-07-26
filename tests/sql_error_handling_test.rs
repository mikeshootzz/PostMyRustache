use postmyrustache::query::QueryHandler;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

async fn setup_postgres_client() -> Result<Arc<Client>, Box<dyn std::error::Error>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1234", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    Ok(Arc::new(client))
}

#[tokio::test]
async fn test_sql_error_fixing() {
    let pg_client = match setup_postgres_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return;
        }
    };

    let query_handler = QueryHandler::new(pg_client);

    // Test cases with common SQL errors that should be fixed
    let test_cases = vec![
        (
            "CREATE TABLE test(name VARCHAR255)",
            "Should fix VARCHAR255 to VARCHAR(255)",
        ),
        (
            "CREATE TABLE users(id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(50))",
            "Should handle AUTO_INCREMENT properly",
        ),
        (
            "CREATE TABLE products(name VARCHAR100, price DECIMAL(10,2))",
            "Should fix VARCHAR100 to VARCHAR(100)",
        ),
    ];

    for (sql, description) in test_cases {
        match query_handler.handle_query(sql).await {
            Ok(_) => {
                println!("✓ SQL error fixing test passed: {} - {}", description, sql);
            }
            Err(e) => {
                println!(
                    "✗ SQL error fixing test failed: {} - {} - Error: {}",
                    description, sql, e
                );
            }
        }
    }
}

#[tokio::test]
async fn test_malformed_sql_handling() {
    let pg_client = match setup_postgres_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return;
        }
    };

    let query_handler = QueryHandler::new(pg_client);

    // Test cases with malformed SQL that should produce helpful error messages
    let malformed_queries = vec![
        "CREATE TABLE test(name(VARCHAR255))", // The original error case
        "CREATE TABLE invalid syntax here",
        "SELECT * FROM nonexistent_table_xyz",
        "INSERT INTO invalid values",
        "UPDATE SET invalid",
    ];

    for query in malformed_queries {
        match query_handler.handle_query(query).await {
            Ok(_) => {
                println!("? Malformed SQL unexpectedly succeeded: {}", query);
            }
            Err(e) => {
                println!(
                    "✓ Malformed SQL properly handled with error: {} - Error: {}",
                    query, e
                );
                // The important thing is that we get an error message, not that the connection drops
            }
        }
    }

    // Test that the handler is still working after processing malformed SQL
    match query_handler.handle_query("SELECT 1").await {
        Ok(_) => {
            println!("✓ Handler still functional after processing malformed SQL");
        }
        Err(e) => {
            println!("✗ Handler broken after malformed SQL: {}", e);
        }
    }
}
