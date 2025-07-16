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
        let db_password = env::var("DB_PASSWORD").map_err(|_| ConfigError::MissingEnvVar("DB_PASSWORD"))?;
        let mysql_username = env::var("MYSQL_USERNAME").map_err(|_| ConfigError::MissingEnvVar("MYSQL_USERNAME"))?;
        let mysql_password = env::var("MYSQL_PASSWORD").map_err(|_| ConfigError::MissingEnvVar("MYSQL_PASSWORD"))?;
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
        format!("host={} user={} password={}", self.db_host, self.db_user, self.db_password)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => write!(f, "Missing required environment variable: {}", var),
        }
    }
}

impl std::error::Error for ConfigError {}