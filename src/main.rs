// Standard I/O module for basic input and output operations.
use std::io;
// AsyncWrite trait from tokio, required for asynchronous write operations.
use tokio::io::AsyncWrite;

// Importing necessary components from the opensrv_mysql crate.
use opensrv_mysql::*;
// TcpListener from tokio for listening to TCP connections.
use tokio::net::TcpListener;

// Backend struct that will implement the AsyncMysqlShim trait.
// This struct represents the logic of your MySQL server.
struct Backend;

// Implementation of the AsyncMysqlShim trait for the Backend.
// This trait requires you to define how your server handles different MySQL commands.
#[async_trait::async_trait]
impl<W: AsyncWrite + Send + Unpin> AsyncMysqlShim<W> for Backend {
    type Error = io::Error;

    // Called when a new prepare statement request is received.
    async fn on_prepare<'a>(
        &'a mut self,
        _: &'a str,                       // The SQL query string to prepare.
        info: StatementMetaWriter<'a, W>, // Used to write the response back to the client.
    ) -> io::Result<()> {
        // Here you would normally prepare the SQL query and return any metadata associated with it.
        // For simplicity, this example just sends a dummy reply.
        info.reply(42, &[], &[]).await
    }

    // Called when an execute statement request is received.
    async fn on_execute<'a>(
        &'a mut self,
        _: u32,                            // The statement ID to execute.
        _: opensrv_mysql::ParamParser<'a>, // Parameters for the statement.
        results: QueryResultWriter<'a, W>, // Used to write the execution results back to the client.
    ) -> io::Result<()> {
        // Normally, you would execute the prepared statement here and send back the results.
        // This example simply returns a success message with no actual data.
        results.completed(OkResponse::default()).await
    }

    // Called when a close statement request is received.
    async fn on_close(&mut self, _: u32) {
        // Here you can clean up resources related to a prepared statement.
    }

    // Called when a new query is received.
    async fn on_query<'a>(
        &'a mut self,
        sql: &'a str,                      // The SQL query string.
        results: QueryResultWriter<'a, W>, // Used to write the query results back to the client.
    ) -> io::Result<()> {
        // Log the received SQL query for debugging.
        println!("execute sql {:?}", sql);
        // Here you would normally execute the SQL query and return the results.
        // This example just sends a response indicating that the query execution has completed, but with no rows.
        results.start(&[]).await?.finish().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start listening for incoming TCP connections on port 3306 (default MySQL port).
    let listener = TcpListener::bind("0.0.0.0:3306").await?;

    loop {
        // Accept a new connection.
        let (stream, _) = listener.accept().await?;
        // Split the TCP stream into a read half and write half.
        let (r, w) = stream.into_split();
        // Spawn a new asynchronous task to handle the MySQL protocol for this connection.
        tokio::spawn(async move {
            // Run the MySQL protocol logic defined in the Backend struct.
            // This involves waiting for client requests and sending responses.
            AsyncMysqlIntermediary::run_on(Backend, r, w).await
        });
    }
}
