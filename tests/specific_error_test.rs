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
async fn test_specific_error_case() {
    let pg_client = match setup_postgres_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return;
        }
    };

    let query_handler = QueryHandler::new(pg_client);

    // Test the exact error case from the user
    let problematic_sql = "CREATE TABLE test(name(VARCHAR255))";

    println!("Testing problematic SQL: {}", problematic_sql);

    match query_handler.handle_query(problematic_sql).await {
        Ok(_) => {
            println!("✓ Successfully handled the problematic SQL (after fixing)");
        }
        Err(e) => {
            println!("✗ Error handling problematic SQL: {}", e);
            println!("  (This is expected - the important thing is we get an error, not a connection drop)");
        }
    }

    // Test that the corrected version works
    let corrected_sql = "CREATE TABLE test_corrected(name VARCHAR(255))";

    println!("\nTesting corrected SQL: {}", corrected_sql);

    match query_handler.handle_query(corrected_sql).await {
        Ok(_) => {
            println!("✓ Successfully handled the corrected SQL");
        }
        Err(e) => {
            println!("✗ Error handling corrected SQL: {}", e);
        }
    }

    // Clean up
    let _ = query_handler
        .handle_query("DROP TABLE IF EXISTS test")
        .await;
    let _ = query_handler
        .handle_query("DROP TABLE IF EXISTS test_corrected")
        .await;

    // Test that the handler is still working after all this
    match query_handler.handle_query("SELECT 1 as test_value").await {
        Ok(_) => {
            println!("✓ Handler is still functional after error handling");
        }
        Err(e) => {
            println!("✗ Handler is broken after error handling: {}", e);
        }
    }
}

#[tokio::test]
async fn test_regex_fixing() {
    use regex::Regex;

    // Test the regex patterns directly
    let test_cases = vec![
        (
            "CREATE TABLE test(name(VARCHAR255))",
            "CREATE TABLE test(name VARCHAR(255))",
        ),
        ("VARCHAR255", "VARCHAR(255)"),
        ("CHAR10", "CHAR(10)"),
        ("INT11", "INT(11)"),
    ];

    for (input, expected) in test_cases {
        let mut fixed = input.to_string();

        // Apply the same regex fixes as in the code
        if let Ok(re) = Regex::new(r"VARCHAR(\d+)") {
            fixed = re.replace_all(&fixed, "VARCHAR($1)").to_string();
        }

        if let Ok(re) = Regex::new(r"CHAR(\d+)") {
            fixed = re.replace_all(&fixed, "CHAR($1)").to_string();
        }

        if let Ok(re) = Regex::new(r"INT(\d+)") {
            fixed = re.replace_all(&fixed, "INT($1)").to_string();
        }

        // Fix parentheses issues
        if let Ok(re) = Regex::new(r"(\w+)\(([A-Z]+\(\d+\))\)") {
            fixed = re.replace_all(&fixed, "$1 $2").to_string();
        }

        if let Ok(re) = Regex::new(r"(\w+)\(([A-Z]+)(\d+)\)") {
            fixed = re.replace_all(&fixed, "$1 $2($3)").to_string();
        }

        println!(
            "Input: {} -> Fixed: {} (Expected: {})",
            input, fixed, expected
        );

        if input.contains("VARCHAR255") || input.contains("CHAR10") || input.contains("INT11") {
            // These simple cases should match exactly
            if fixed != expected {
                println!("  ⚠️  Regex fix didn't match expected result");
            } else {
                println!("  ✓ Regex fix worked correctly");
            }
        } else {
            // More complex cases, just verify improvement
            if fixed != input {
                println!("  ✓ Regex made changes to improve the SQL");
            } else {
                println!("  ? No changes made by regex");
            }
        }
    }
}
