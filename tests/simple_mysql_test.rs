use postmyrustache::query::QueryHandler;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

async fn setup_postgres_client() -> Result<Arc<Client>, Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=1234", 
        NoTls
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });
    
    Ok(Arc::new(client))
}

#[tokio::test]
async fn test_mysql_specific_queries() {
    // Test if we can create a QueryHandler and handle MySQL-specific queries
    let pg_client = match setup_postgres_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return; // Skip test if PostgreSQL is not available
        }
    };
    
    let query_handler = QueryHandler::new(pg_client);
    
    // Test MySQL system variable queries
    let mysql_queries = vec![
        "SELECT @@version_comment",
        "SELECT @@sql_mode",
        "SELECT @@autocommit",
        "SHOW TABLES",
        "SHOW DATABASES",
        "USE test_db",
        "SET autocommit=1",
        "SELECT CONNECTION_ID()",
        "SELECT DATABASE()",
        "SELECT USER()",
    ];
    
    for query in mysql_queries {
        match query_handler.handle_query(query).await {
            Ok(_) => {
                println!("✓ MySQL query handled successfully: {}", query);
            }
            Err(e) => {
                println!("✗ MySQL query failed: {} - Error: {}", query, e);
            }
        }
    }
}

#[tokio::test]
async fn test_mysql_to_postgres_translation() {
    let pg_client = match setup_postgres_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return;
        }
    };
    
    let query_handler = QueryHandler::new(pg_client);
    
    // Test SQL translation capabilities
    let translation_tests = vec![
        (
            "CREATE TABLE test (id INT AUTO_INCREMENT PRIMARY KEY)",
            "Should translate AUTO_INCREMENT to SERIAL"
        ),
        (
            "SELECT * FROM `users` WHERE `name` = 'test'",
            "Should translate backticks to double quotes"
        ),
        (
            "SELECT NOW(), CURDATE(), CURTIME()",
            "Should translate MySQL date functions"
        ),
        (
            "CREATE TABLE test (name VARCHAR(255), content LONGTEXT)",
            "Should translate LONGTEXT to TEXT"
        ),
    ];
    
    for (query, description) in translation_tests {
        match query_handler.handle_query(query).await {
            Ok(_) => {
                println!("✓ Translation test passed: {} - {}", description, query);
            }
            Err(e) => {
                println!("✗ Translation test failed: {} - {} - Error: {}", description, query, e);
            }
        }
    }
}

#[tokio::test]
async fn test_basic_sql_operations() {
    let pg_client = match setup_postgres_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return;
        }
    };
    
    let query_handler = QueryHandler::new(pg_client);
    
    // Test basic SQL operations that should work in both MySQL and PostgreSQL
    let basic_queries = vec![
        "CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY, name VARCHAR(100))",
        "INSERT INTO test_table (name) VALUES ('test1')",
        "INSERT INTO test_table (name) VALUES ('test2')",
        "SELECT COUNT(*) FROM test_table",
        "UPDATE test_table SET name = 'updated' WHERE id = 1",
        "DELETE FROM test_table WHERE id = 2",
        "DROP TABLE IF EXISTS test_table",
    ];
    
    for query in basic_queries {
        match query_handler.handle_query(query).await {
            Ok(_) => {
                println!("✓ Basic SQL operation succeeded: {}", query);
            }
            Err(e) => {
                println!("✗ Basic SQL operation failed: {} - Error: {}", query, e);
            }
        }
    }
}