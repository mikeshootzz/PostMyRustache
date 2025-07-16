pub mod auth;
pub mod backend;
pub mod config;
pub mod query;
pub mod server;

use dotenv::dotenv;

use config::Config;
use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = Config::from_env()?;

    // Create and start server
    let server = Server::new(config);
    server.start().await?;

    Ok(())
}
