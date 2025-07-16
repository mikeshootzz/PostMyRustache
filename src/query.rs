use std::io;
use std::sync::Arc;
use tokio_postgres::Client;
use opensrv_mysql::OkResponse;

pub struct QueryHandler {
    pg_client: Arc<Client>,
}

impl QueryHandler {
    pub fn new(pg_client: Arc<Client>) -> Self {
        Self { pg_client }
    }

    pub async fn handle_query(&self, sql: &str) -> io::Result<QueryResult> {
        log::info!("Received SQL query: {:?}", sql);

        // Check for MySQL-specific queries that need special handling
        if let Some(response) = self.handle_mysql_specific_query(sql) {
            return Ok(response);
        }

        // Forward other queries to PostgreSQL
        match self.pg_client.execute(sql, &[]).await {
            Ok(row_count) => {
                log::info!("Query executed successfully, {} rows affected.", row_count);
                
                let mut response = OkResponse::default();
                response.affected_rows = row_count;
                
                Ok(QueryResult::Ok(response))
            }
            Err(e) => {
                log::error!("Error executing query: {:?}", e);
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to execute query: {}", e),
                ))
            }
        }
    }

    fn handle_mysql_specific_query(&self, sql: &str) -> Option<QueryResult> {
        let sql_trimmed = sql.trim();

        if sql_trimmed.eq_ignore_ascii_case("select @@version_comment limit 1") {
            log::info!("Intercepted MySQL-specific query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.starts_with("select $$") {
            log::info!("Intercepted query with unsupported syntax, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        None
    }
}

pub enum QueryResult {
    Ok(OkResponse),
}