use std::env;

#[derive(Clone)]
pub struct Config {
    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
    pub mysql_username: String,
    pub mysql_password: String,
    pub bind_address: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let db_host = env::var("DB_HOST").map_err(|_| ConfigError::MissingEnvVar("DB_HOST"))?;
        let db_user = env::var("DB_USER").map_err(|_| ConfigError::MissingEnvVar("DB_USER"))?;
        let db_password =
            env::var("DB_PASSWORD").map_err(|_| ConfigError::MissingEnvVar("DB_PASSWORD"))?;
        let mysql_username =
            env::var("MYSQL_USERNAME").map_err(|_| ConfigError::MissingEnvVar("MYSQL_USERNAME"))?;
        let mysql_password =
            env::var("MYSQL_PASSWORD").map_err(|_| ConfigError::MissingEnvVar("MYSQL_PASSWORD"))?;
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3306".to_string());

        Ok(Config {
            db_host,
            db_user,
            db_password,
            mysql_username,
            mysql_password,
            bind_address,
        })
    }

    pub fn postgres_connection_string(&self) -> String {
        format!(
            "host={} user={} password={}",
            self.db_host, self.db_user, self.db_password
        )
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => {
                write!(f, "Missing required environment variable: {var}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_success() {
        // Save original values
        let original_values = [
            ("DB_HOST", env::var("DB_HOST").ok()),
            ("DB_USER", env::var("DB_USER").ok()),
            ("DB_PASSWORD", env::var("DB_PASSWORD").ok()),
            ("MYSQL_USERNAME", env::var("MYSQL_USERNAME").ok()),
            ("MYSQL_PASSWORD", env::var("MYSQL_PASSWORD").ok()),
            ("BIND_ADDRESS", env::var("BIND_ADDRESS").ok()),
        ];

        // Set up environment variables
        env::set_var("DB_HOST", "test_host");
        env::set_var("DB_USER", "test_user");
        env::set_var("DB_PASSWORD", "test_password");
        env::set_var("MYSQL_USERNAME", "test_mysql_user");
        env::set_var("MYSQL_PASSWORD", "test_mysql_password");
        env::set_var("BIND_ADDRESS", "127.0.0.1:3307");

        let config = Config::from_env().unwrap();

        assert_eq!(config.db_host, "test_host");
        assert_eq!(config.db_user, "test_user");
        assert_eq!(config.db_password, "test_password");
        assert_eq!(config.mysql_username, "test_mysql_user");
        assert_eq!(config.mysql_password, "test_mysql_password");
        assert_eq!(config.bind_address, "127.0.0.1:3307");

        // Restore original values
        for (key, value) in original_values {
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn test_config_from_env_default_bind_address() {
        // Save original values
        let original_values = [
            ("DB_HOST", env::var("DB_HOST").ok()),
            ("DB_USER", env::var("DB_USER").ok()),
            ("DB_PASSWORD", env::var("DB_PASSWORD").ok()),
            ("MYSQL_USERNAME", env::var("MYSQL_USERNAME").ok()),
            ("MYSQL_PASSWORD", env::var("MYSQL_PASSWORD").ok()),
            ("BIND_ADDRESS", env::var("BIND_ADDRESS").ok()),
        ];

        // Set up required environment variables
        env::set_var("DB_HOST", "test_host");
        env::set_var("DB_USER", "test_user");
        env::set_var("DB_PASSWORD", "test_password");
        env::set_var("MYSQL_USERNAME", "test_mysql_user");
        env::set_var("MYSQL_PASSWORD", "test_mysql_password");
        env::remove_var("BIND_ADDRESS"); // Remove to test default

        let config = Config::from_env().unwrap();

        assert_eq!(config.bind_address, "0.0.0.0:3306");

        // Restore original values
        for (key, value) in original_values {
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn test_config_from_env_missing_required() {
        // Save original values
        let original_values = [
            ("DB_HOST", env::var("DB_HOST").ok()),
            ("DB_USER", env::var("DB_USER").ok()),
            ("DB_PASSWORD", env::var("DB_PASSWORD").ok()),
            ("MYSQL_USERNAME", env::var("MYSQL_USERNAME").ok()),
            ("MYSQL_PASSWORD", env::var("MYSQL_PASSWORD").ok()),
        ];

        // Remove all environment variables to test error handling
        env::remove_var("DB_HOST");
        env::remove_var("DB_USER");
        env::remove_var("DB_PASSWORD");
        env::remove_var("MYSQL_USERNAME");
        env::remove_var("MYSQL_PASSWORD");

        let result = Config::from_env();
        assert!(result.is_err());

        // Restore original values
        for (key, value) in original_values {
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn test_postgres_connection_string() {
        let config = Config {
            db_host: "localhost".to_string(),
            db_user: "postgres".to_string(),
            db_password: "password123".to_string(),
            mysql_username: "admin".to_string(),
            mysql_password: "secret".to_string(),
            bind_address: "0.0.0.0:3306".to_string(),
        };

        let connection_string = config.postgres_connection_string();
        assert_eq!(
            connection_string,
            "host=localhost user=postgres password=password123"
        );
    }
}
