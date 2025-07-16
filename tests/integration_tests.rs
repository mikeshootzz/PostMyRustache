use postmyrustache::config::{Config, ConfigError};
use std::env;

#[test]
fn test_config_integration() {
    // Save original env vars
    let original_db_host = env::var("DB_HOST").ok();
    let original_db_user = env::var("DB_USER").ok();
    let original_db_password = env::var("DB_PASSWORD").ok();
    let original_mysql_username = env::var("MYSQL_USERNAME").ok();
    let original_mysql_password = env::var("MYSQL_PASSWORD").ok();

    // Set test environment variables
    env::set_var("DB_HOST", "integration_test_host");
    env::set_var("DB_USER", "integration_test_user");
    env::set_var("DB_PASSWORD", "integration_test_password");
    env::set_var("MYSQL_USERNAME", "integration_mysql_user");
    env::set_var("MYSQL_PASSWORD", "integration_mysql_password");

    // Test config creation
    let config = Config::from_env().expect("Failed to create config from env");

    assert_eq!(config.db_host, "integration_test_host");
    assert_eq!(config.db_user, "integration_test_user");
    assert_eq!(config.db_password, "integration_test_password");
    assert_eq!(config.mysql_username, "integration_mysql_user");
    assert_eq!(config.mysql_password, "integration_mysql_password");

    // Test connection string generation
    let connection_string = config.postgres_connection_string();
    assert_eq!(
        connection_string,
        "host=integration_test_host user=integration_test_user password=integration_test_password"
    );

    // Restore original env vars
    if let Some(val) = original_db_host {
        env::set_var("DB_HOST", val);
    } else {
        env::remove_var("DB_HOST");
    }
    if let Some(val) = original_db_user {
        env::set_var("DB_USER", val);
    } else {
        env::remove_var("DB_USER");
    }
    if let Some(val) = original_db_password {
        env::set_var("DB_PASSWORD", val);
    } else {
        env::remove_var("DB_PASSWORD");
    }
    if let Some(val) = original_mysql_username {
        env::set_var("MYSQL_USERNAME", val);
    } else {
        env::remove_var("MYSQL_USERNAME");
    }
    if let Some(val) = original_mysql_password {
        env::set_var("MYSQL_PASSWORD", val);
    } else {
        env::remove_var("MYSQL_PASSWORD");
    }
}

#[test]
fn test_config_error_handling() {
    // Save original env vars
    let original_vars = [
        ("DB_HOST", env::var("DB_HOST").ok()),
        ("DB_USER", env::var("DB_USER").ok()),
        ("DB_PASSWORD", env::var("DB_PASSWORD").ok()),
        ("MYSQL_USERNAME", env::var("MYSQL_USERNAME").ok()),
        ("MYSQL_PASSWORD", env::var("MYSQL_PASSWORD").ok()),
    ];

    // Remove all required env vars
    env::remove_var("DB_HOST");
    env::remove_var("DB_USER");
    env::remove_var("DB_PASSWORD");
    env::remove_var("MYSQL_USERNAME");
    env::remove_var("MYSQL_PASSWORD");

    // Test error handling
    let result = Config::from_env();
    assert!(result.is_err());

    if let Err(ConfigError::MissingEnvVar(var)) = result {
        // Should be one of the required variables
        assert!([
            "DB_HOST",
            "DB_USER",
            "DB_PASSWORD",
            "MYSQL_USERNAME",
            "MYSQL_PASSWORD"
        ]
        .contains(&var));
    } else {
        panic!("Expected ConfigError::MissingEnvVar");
    }

    // Restore original env vars
    for (key, value) in original_vars {
        match value {
            Some(val) => env::set_var(key, val),
            None => env::remove_var(key),
        }
    }
}
