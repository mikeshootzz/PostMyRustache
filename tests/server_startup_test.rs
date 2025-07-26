use postmyrustache::config::Config;
use postmyrustache::server::Server;
use std::env;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

// Helper function to set up test environment
fn setup_test_environment() {
    env::set_var("DB_HOST", "localhost");
    env::set_var("DB_USER", "postgres");
    env::set_var("DB_PASSWORD", "1234");
    env::set_var("MYSQL_USERNAME", "testuser");
    env::set_var("MYSQL_PASSWORD", "testpass");
    env::set_var("BIND_ADDRESS", "127.0.0.1:3308"); // Use different port for this test
    env::set_var("RUST_LOG", "info");
}

#[tokio::test]
async fn test_server_startup_and_connection() {
    setup_test_environment();

    // Try to connect to PostgreSQL first
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            return;
        }
    };

    // Test PostgreSQL connection
    let connection_string = config.postgres_connection_string();
    let postgres_result = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await;

    let (_pg_client, pg_connection) = match postgres_result {
        Ok((client, connection)) => (client, connection),
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {e}");
            eprintln!("Make sure PostgreSQL is running on localhost:5432 with user 'postgres' and password '1234'");
            return;
        }
    };

    // Spawn PostgreSQL connection
    tokio::spawn(async move {
        if let Err(e) = pg_connection.await {
            eprintln!("PostgreSQL connection error: {e}");
        }
    });

    println!("✓ PostgreSQL connection successful");

    // Start the server in a background task
    let server = Server::new(config.clone());
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {e}");
        }
    });

    // Give the server a moment to start
    sleep(Duration::from_millis(500)).await;

    // Test if we can connect to the server
    let connection_result = timeout(
        Duration::from_secs(5),
        TcpStream::connect(&config.bind_address),
    )
    .await;

    match connection_result {
        Ok(Ok(_stream)) => {
            println!(
                "✓ Successfully connected to PostMyRustache server on {}",
                config.bind_address
            );

            // Close the connection gracefully
            drop(_stream);
        }
        Ok(Err(e)) => {
            eprintln!("✗ Failed to connect to server: {e}");
            panic!("Could not connect to PostMyRustache server");
        }
        Err(_) => {
            eprintln!("✗ Connection attempt timed out");
            panic!("Connection to PostMyRustache server timed out");
        }
    }

    // Abort the server task
    server_handle.abort();

    println!("✓ Server startup and connection test completed successfully");
}

#[tokio::test]
async fn test_multiple_connections() {
    setup_test_environment();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            return;
        }
    };

    // Test PostgreSQL connection first
    let connection_string = config.postgres_connection_string();
    if let Err(e) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await {
        eprintln!("Failed to connect to PostgreSQL: {e}");
        return;
    }

    // Set different port for this test
    env::set_var("BIND_ADDRESS", "127.0.0.1:3309");
    let config = Config::from_env().unwrap();

    // Start the server
    let server = Server::new(config.clone());
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {e}");
        }
    });

    // Give the server time to start
    sleep(Duration::from_millis(500)).await;

    // Test multiple simultaneous connections
    let mut connection_handles = Vec::new();

    for i in 0..3 {
        let bind_address = config.bind_address.clone();
        let handle = tokio::spawn(async move {
            match timeout(Duration::from_secs(3), TcpStream::connect(&bind_address)).await {
                Ok(Ok(_stream)) => {
                    println!("✓ Connection {} successful", i);
                    // Keep connection open briefly
                    sleep(Duration::from_millis(100)).await;
                    drop(_stream);
                    true
                }
                Ok(Err(e)) => {
                    eprintln!("✗ Connection {i} failed: {e}");
                    false
                }
                Err(_) => {
                    eprintln!("✗ Connection {} timed out", i);
                    false
                }
            }
        });
        connection_handles.push(handle);
    }

    // Wait for all connections to complete
    let mut successful_connections = 0;
    for handle in connection_handles {
        if let Ok(success) = handle.await {
            if success {
                successful_connections += 1;
            }
        }
    }

    println!(
        "✓ Successfully handled {}/3 concurrent connections",
        successful_connections
    );

    // Abort the server
    server_handle.abort();

    if successful_connections == 3 {
        println!("✓ Multiple connections test passed");
    } else {
        eprintln!("✗ Only {}/3 connections succeeded", successful_connections);
    }
}
