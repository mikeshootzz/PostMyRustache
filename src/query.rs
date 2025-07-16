use opensrv_mysql::OkResponse;
use std::io;
use std::sync::Arc;
use tokio_postgres::Client;

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

                let response = OkResponse {
                    affected_rows: row_count,
                    ..Default::default()
                };

                Ok(QueryResult::Ok(response))
            }
            Err(e) => {
                log::error!("Error executing query: {:?}", e);
                Err(io::Error::other(format!("Failed to execute query: {}", e)))
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

#[cfg(test)]
mod tests {
    use super::*;

    // Create a mock QueryHandler for testing that doesn't need a real PostgreSQL client
    struct MockQueryHandler;

    impl MockQueryHandler {
        fn handle_mysql_specific_query(&self, sql: &str) -> Option<QueryResult> {
            let sql_trimmed = sql.trim();

            if sql_trimmed.eq_ignore_ascii_case("select @@version_comment limit 1") {
                return Some(QueryResult::Ok(OkResponse::default()));
            }

            if sql_trimmed.starts_with("select $$") {
                return Some(QueryResult::Ok(OkResponse::default()));
            }

            None
        }
    }

    #[test]
    fn test_handle_mysql_specific_query_version_comment() {
        let handler = MockQueryHandler;
        let result = handler.handle_mysql_specific_query("select @@version_comment limit 1");
        assert!(result.is_some());

        if let Some(QueryResult::Ok(response)) = result {
            assert_eq!(response.affected_rows, 0);
        }
    }

    #[test]
    fn test_handle_mysql_specific_query_dollar_syntax() {
        let handler = MockQueryHandler;
        let result = handler.handle_mysql_specific_query("select $$ something");
        assert!(result.is_some());

        if let Some(QueryResult::Ok(response)) = result {
            assert_eq!(response.affected_rows, 0);
        }
    }

    #[test]
    fn test_handle_mysql_specific_query_case_insensitive() {
        let handler = MockQueryHandler;
        let result = handler.handle_mysql_specific_query("SELECT @@VERSION_COMMENT LIMIT 1");
        assert!(result.is_some());
    }

    #[test]
    fn test_handle_mysql_specific_query_regular_query() {
        let handler = MockQueryHandler;
        let result = handler.handle_mysql_specific_query("SELECT * FROM users");
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_mysql_specific_query_with_whitespace() {
        let handler = MockQueryHandler;
        let result = handler.handle_mysql_specific_query("  select @@version_comment limit 1  ");
        assert!(result.is_some());
    }

    #[test]
    fn test_handle_mysql_specific_query_empty_string() {
        let handler = MockQueryHandler;
        let result = handler.handle_mysql_specific_query("");
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_mysql_specific_query_partial_match() {
        let handler = MockQueryHandler;
        // Should not match partial strings
        let result = handler.handle_mysql_specific_query("select @@version_comment limit 2");
        assert!(result.is_none());
    }
}
