use opensrv_mysql::AsyncMysqlIntermediary;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_postgres::{Client, NoTls};

use crate::auth::AuthProvider;
use crate::backend::Backend;
use crate::config::Config;

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Connect to PostgreSQL
        let pg_client = self.connect_to_postgres().await?;

        // Create TCP listener
        let listener = TcpListener::bind(&self.config.bind_address).await?;

        self.print_startup_banner();
        log::info!("MySQL server is running on {}", self.config.bind_address);

        // Accept connections
        loop {
            let (stream, addr) = listener.accept().await?;
            log::debug!("New connection from: {addr}");

            let (r, w) = stream.into_split();
            let pg_client_clone = Arc::clone(&pg_client);
            let auth_provider = AuthProvider::new(self.config.clone());

            tokio::spawn(async move {
                let backend = Backend::new(pg_client_clone, auth_provider);

                if let Err(e) = AsyncMysqlIntermediary::run_on(backend, r, w).await {
                    log::error!("Connection error: {e}");
                }
            });
        }
    }

    async fn connect_to_postgres(&self) -> Result<Arc<Client>, Box<dyn std::error::Error>> {
        let connection_string = self.config.postgres_connection_string();
        log::info!("Connecting to PostgreSQL: {connection_string}");

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

        // Spawn the connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                log::error!("PostgreSQL connection error: {e}");
            }
        });

        Ok(Arc::new(client))
    }

    fn print_startup_banner(&self) {
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
    }
}
