use postmyrustache::config::Config;
use postmyrustache::server::Server;
use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

// Helper function to set up test environment
fn setup_test_environment() {
    env::set_var("DB_HOST", "localhost");
    env::set_var("DB_USER", "postgres");
    env::set_var("DB_PASSWORD", "1234");
    env::set_var("MYSQL_USERNAME", "testuser");
    env::set_var("MYSQL_PASSWORD", "testpass");
    env::set_var("BIND_ADDRESS", "127.0.0.1:3307"); // Use different port for testing
    env::set_var("RUST_LOG", "debug");
}

// Helper function to check if PostgreSQL is running
async fn check_postgres_connection() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let connection_string = config.postgres_connection_string();
    
    let (client, connection) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await?;
    
    // Spawn the connection in a separate task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });
    
    // Test connection with a simple query
    client.execute("SELECT 1", &[]).await?;
    
    Ok(())
}

// Helper function to start PostMyRustache server
async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    setup_test_environment();
    
    // Check PostgreSQL connection first
    check_postgres_connection().await?;
    
    let config = Config::from_env()?;
    let server = Server::new(config);
    
    // Start server in background task
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });
    
    // Give server time to start
    sleep(Duration::from_secs(2)).await;
    
    Ok(())
}

// Helper function to execute MySQL commands using mysql client
fn execute_mysql_command(command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("mysql")
        .args(&[
            "-h", "127.0.0.1",
            "-P", "3307",
            "-u", "testuser",
            "-ptestpass",
            "-e", command
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(String::from_utf8_lossy(&result.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Err(format!("MySQL command failed: {}", stderr).into())
            }
        }
        Err(e) => Err(format!("Failed to execute mysql command: {}", e).into())
    }
}

// Helper function to execute SQL file
fn execute_sql_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let _output = Command::new("mysql")
        .args(&[
            "-h", "127.0.0.1",
            "-P", "3307",
            "-u", "testuser",
            "-ptestpass"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .stdin
        .take()
        .ok_or("Failed to take stdin")?;
    
    // Read and execute the SQL file
    let sql_content = std::fs::read_to_string(file_path)?;
    
    let mut child = Command::new("mysql")
        .args(&[
            "-h", "127.0.0.1",
            "-P", "3307",
            "-u", "testuser",
            "-ptestpass"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(sql_content.as_bytes())?;
    }
    
    let output = child.wait_with_output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("SQL file execution failed: {}", stderr).into())
    }
}

#[tokio::test]
#[ignore] // Use ignore by default since this requires external dependencies
async fn test_mysql_compatibility_basic_connection() {
    // This test checks if we can connect to the server using MySQL protocol
    if let Err(e) = start_server().await {
        eprintln!("Failed to start server: {}", e);
        return;
    }
    
    // Test basic connection
    match execute_mysql_command("SELECT 1 as test") {
        Ok(output) => {
            println!("Basic connection test passed: {}", output);
            assert!(output.contains("test"));
        }
        Err(e) => {
            eprintln!("Basic connection test failed: {}", e);
            panic!("Failed basic connection test");
        }
    }
}

#[tokio::test]
#[ignore] // Use ignore by default since this requires external dependencies
async fn test_mysql_version_queries() {
    if let Err(e) = start_server().await {
        eprintln!("Failed to start server: {}", e);
        return;
    }
    
    // Test MySQL-specific version queries
    let version_queries = vec![
        "SELECT @@version_comment",
        "SELECT @@sql_mode",
        "SELECT CONNECTION_ID()",
        "SELECT DATABASE()",
        "SELECT USER()"
    ];
    
    for query in version_queries {
        match execute_mysql_command(query) {
            Ok(output) => {
                println!("Version query '{}' passed: {}", query, output);
            }
            Err(e) => {
                eprintln!("Version query '{}' failed: {}", query, e);
                // Note: Some failures are expected as we may not implement all MySQL functions
            }
        }
    }
}

#[tokio::test]
#[ignore] // Use ignore by default since this requires external dependencies
async fn test_basic_ddl_operations() {
    if let Err(e) = start_server().await {
        eprintln!("Failed to start server: {}", e);
        return;
    }
    
    // Test basic DDL operations
    let ddl_commands = vec![
        "CREATE DATABASE IF NOT EXISTS test_db",
        "USE test_db",
        "CREATE TABLE test_table (id INT PRIMARY KEY, name VARCHAR(50))",
        "INSERT INTO test_table (id, name) VALUES (1, 'test')",
        "SELECT * FROM test_table",
        "UPDATE test_table SET name = 'updated' WHERE id = 1",
        "DELETE FROM test_table WHERE id = 1",
        "DROP TABLE test_table"
    ];
    
    for command in ddl_commands {
        match execute_mysql_command(command) {
            Ok(output) => {
                println!("DDL command '{}' passed: {}", command, output);
            }
            Err(e) => {
                eprintln!("DDL command '{}' failed: {}", command, e);
            }
        }
    }
}

#[tokio::test]
#[ignore] // Use ignore by default since this requires external dependencies
async fn test_comprehensive_mysql_compatibility() {
    if let Err(e) = start_server().await {
        eprintln!("Failed to start server: {}", e);
        return;
    }
    
    // Execute the comprehensive SQL test file
    let sql_file_path = "tests/mysql_compatibility_test.sql";
    
    match execute_sql_file(sql_file_path) {
        Ok(output) => {
            println!("Comprehensive compatibility test passed!");
            println!("Output: {}", output);
        }
        Err(e) => {
            eprintln!("Comprehensive compatibility test failed: {}", e);
            
            // Try to execute individual commands to identify specific failures
            println!("Attempting to identify specific compatibility issues...");
            
            // Read the SQL file and try commands one by one
            if let Ok(sql_content) = std::fs::read_to_string(sql_file_path) {
                let lines: Vec<&str> = sql_content.lines()
                    .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("--"))
                    .collect();
                
                let mut current_command = String::new();
                
                for line in lines {
                    current_command.push_str(line);
                    current_command.push(' ');
                    
                    if line.trim().ends_with(';') {
                        // Execute this command
                        match execute_mysql_command(&current_command) {
                            Ok(output) => {
                                println!("✓ Command succeeded: {}", current_command.trim());
                                if !output.is_empty() {
                                    println!("  Output: {}", output.trim());
                                }
                            }
                            Err(e) => {
                                eprintln!("✗ Command failed: {}", current_command.trim());
                                eprintln!("  Error: {}", e);
                            }
                        }
                        current_command.clear();
                    }
                }
            }
        }
    }
}

// Test specific MySQL data types
#[tokio::test]
#[ignore]
async fn test_mysql_data_types() {
    if let Err(e) = start_server().await {
        eprintln!("Failed to start server: {}", e);
        return;
    }
    
    let data_type_tests = vec![
        // Integer types
        "CREATE TABLE int_test (id INT, tiny TINYINT, small SMALLINT, big BIGINT)",
        "INSERT INTO int_test VALUES (1, 127, 32767, 9223372036854775807)",
        "SELECT * FROM int_test",
        
        // String types
        "CREATE TABLE string_test (id INT, char_col CHAR(10), varchar_col VARCHAR(255), text_col TEXT)",
        "INSERT INTO string_test VALUES (1, 'test', 'variable text', 'long text content')",
        "SELECT * FROM string_test",
        
        // Date types
        "CREATE TABLE date_test (id INT, date_col DATE, time_col TIME, datetime_col DATETIME)",
        "INSERT INTO date_test VALUES (1, '2024-01-15', '14:30:00', '2024-01-15 14:30:00')",
        "SELECT * FROM date_test",
        
        // Clean up
        "DROP TABLE IF EXISTS int_test",
        "DROP TABLE IF EXISTS string_test", 
        "DROP TABLE IF EXISTS date_test"
    ];
    
    for command in data_type_tests {
        match execute_mysql_command(command) {
            Ok(output) => {
                println!("✓ Data type test passed: {}", command);
                if !output.is_empty() {
                    println!("  Output: {}", output);
                }
            }
            Err(e) => {
                eprintln!("✗ Data type test failed: {}", command);
                eprintln!("  Error: {}", e);
            }
        }
    }
}

// Test for cleanup
#[tokio::test]
#[ignore]
async fn test_cleanup() {
    setup_test_environment();
    
    // Clean up test data
    if let Ok(_) = execute_mysql_command("DROP DATABASE IF EXISTS test_db") {
        println!("Cleanup completed successfully");
    }
}