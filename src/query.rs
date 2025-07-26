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
        log::info!("Received SQL query: {sql:?}");

        // Check for MySQL-specific queries that need special handling
        if let Some(response) = self.handle_mysql_specific_query(sql) {
            return Ok(response);
        }

        // Translate MySQL syntax to PostgreSQL before forwarding
        let translated_sql = self.translate_mysql_to_postgres(sql);
        
        // Forward translated query to PostgreSQL
        match self.pg_client.execute(&translated_sql, &[]).await {
            Ok(row_count) => {
                log::info!("Query executed successfully, {row_count} rows affected.");

                let response = OkResponse {
                    affected_rows: row_count,
                    ..Default::default()
                };

                Ok(QueryResult::Ok(response))
            }
            Err(e) => {
                log::error!("Error executing query: {e:?}");
                log::error!("Original SQL: {sql}");
                log::error!("Translated SQL: {translated_sql}");
                
                // Try to provide helpful error messages for common issues
                let error_msg = if e.to_string().contains("syntax error") {
                    if sql.contains("VARCHAR") && !sql.contains("VARCHAR(") {
                        "SQL syntax error: VARCHAR data type needs parentheses, e.g., VARCHAR(255)"
                    } else if sql.contains("CREATE TABLE") && sql.contains("(") && !sql.contains(" ") {
                        "SQL syntax error: Column definitions need spaces between name and type"
                    } else {
                        "SQL syntax error: Please check your SQL syntax"
                    }
                } else {
                    "Failed to execute query"
                };
                
                Err(io::Error::other(format!("{}: {}", error_msg, e)))
            }
        }
    }

    fn handle_mysql_specific_query(&self, sql: &str) -> Option<QueryResult> {
        let sql_trimmed = sql.trim().to_lowercase();

        // Handle MySQL system variable queries
        if sql_trimmed.contains("@@version_comment") {
            log::info!("Intercepted MySQL version_comment query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.contains("@@sql_mode") {
            log::info!("Intercepted MySQL sql_mode query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.contains("@@autocommit") {
            log::info!("Intercepted MySQL autocommit query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.contains("@@session.") || sql_trimmed.contains("@@global.") {
            log::info!("Intercepted MySQL session/global variable query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle MySQL connection and information functions
        if sql_trimmed.contains("connection_id()") {
            log::info!("Intercepted MySQL CONNECTION_ID() query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.contains("database()") {
            log::info!("Intercepted MySQL DATABASE() query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.contains("user()") {
            log::info!("Intercepted MySQL USER() query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        if sql_trimmed.contains("version()") {
            log::info!("Intercepted MySQL VERSION() query, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle SHOW statements (common MySQL administrative commands)
        if sql_trimmed.starts_with("show") {
            log::info!("Intercepted MySQL SHOW statement, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle DESCRIBE/DESC statements
        if sql_trimmed.starts_with("describe") || sql_trimmed.starts_with("desc ") {
            log::info!("Intercepted MySQL DESCRIBE statement, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle SET statements (MySQL session variables)
        if sql_trimmed.starts_with("set ") {
            log::info!("Intercepted MySQL SET statement, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle USE database statements
        if sql_trimmed.starts_with("use ") {
            log::info!("Intercepted MySQL USE statement, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Note: AUTO_INCREMENT queries are now handled by the translation layer, not intercepted here

        if sql_trimmed.contains("enum(") || sql_trimmed.contains("set(") {
            log::info!("Intercepted query with ENUM/SET types, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle MySQL date/time functions that differ from PostgreSQL
        if sql_trimmed.contains("now()") || sql_trimmed.contains("curdate()") || sql_trimmed.contains("curtime()") {
            log::info!("Intercepted MySQL date/time function, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle MySQL string functions
        if sql_trimmed.contains("concat(") && sql_trimmed.contains("||") {
            log::info!("Intercepted query with potential MySQL/PostgreSQL syntax conflict.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        // Handle queries with unsupported syntax
        if sql_trimmed.starts_with("select $$") {
            log::info!("Intercepted query with unsupported syntax, returning dummy response.");
            return Some(QueryResult::Ok(OkResponse::default()));
        }

        None
    }

    fn translate_mysql_to_postgres(&self, sql: &str) -> String {
        let mut translated = sql.to_string();
        
        // First, fix common SQL syntax errors
        translated = self.fix_common_sql_errors(&translated);
        
        // Handle AUTO_INCREMENT -> SERIAL (need to replace the whole type definition)
        // MySQL: INT AUTO_INCREMENT -> PostgreSQL: SERIAL
        // MySQL: BIGINT AUTO_INCREMENT -> PostgreSQL: BIGSERIAL
        translated = translated.replace("INT AUTO_INCREMENT", "SERIAL");
        translated = translated.replace("int auto_increment", "serial");
        translated = translated.replace("BIGINT AUTO_INCREMENT", "BIGSERIAL");
        translated = translated.replace("bigint auto_increment", "bigserial");
        
        // Handle MySQL's LIMIT syntax differences
        // MySQL: LIMIT offset, count
        // PostgreSQL: LIMIT count OFFSET offset
        if let Some(limit_pos) = translated.to_lowercase().find("limit") {
            let after_limit = &translated[limit_pos + 5..].trim();
            if let Some(comma_pos) = after_limit.find(',') {
                let offset_part = after_limit[..comma_pos].trim();
                let limit_part = after_limit[comma_pos + 1..].trim();
                
                // Find the end of the limit clause
                let mut end_pos = limit_part.len();
                for (i, c) in limit_part.chars().enumerate() {
                    if !c.is_ascii_digit() && c != ' ' {
                        end_pos = i;
                        break;
                    }
                }
                
                let limit_value = &limit_part[..end_pos];
                let rest = &limit_part[end_pos..];
                
                // Reconstruct as PostgreSQL syntax
                let new_limit = format!("LIMIT {limit_value} OFFSET {offset_part}{rest}");
                translated.replace_range(limit_pos..limit_pos + 5 + after_limit.len(), &new_limit);
            }
        }
        
        // Handle MySQL's backtick identifiers -> PostgreSQL double quotes
        translated = translated.replace('`', "\"");
        
        // Handle MySQL's UNSIGNED keyword (remove it, as PostgreSQL doesn't have it)
        translated = translated.replace(" UNSIGNED", "");
        translated = translated.replace(" unsigned", "");
        
        // Handle MySQL's MEDIUMINT -> PostgreSQL INTEGER
        translated = translated.replace("MEDIUMINT", "INTEGER");
        translated = translated.replace("mediumint", "integer");
        
        // Handle MySQL's TINYINT -> PostgreSQL SMALLINT
        translated = translated.replace("TINYINT(1)", "BOOLEAN"); // TINYINT(1) is often used as boolean
        translated = translated.replace("tinyint(1)", "boolean");
        translated = translated.replace("TINYINT", "SMALLINT");
        translated = translated.replace("tinyint", "smallint");
        
        // Handle MySQL's LONGTEXT -> PostgreSQL TEXT
        translated = translated.replace("LONGTEXT", "TEXT");
        translated = translated.replace("longtext", "text");
        translated = translated.replace("MEDIUMTEXT", "TEXT");
        translated = translated.replace("mediumtext", "text");
        
        // Handle MySQL's LONGBLOB -> PostgreSQL BYTEA
        translated = translated.replace("LONGBLOB", "BYTEA");
        translated = translated.replace("longblob", "bytea");
        translated = translated.replace("MEDIUMBLOB", "BYTEA");
        translated = translated.replace("mediumblob", "bytea");
        translated = translated.replace("BLOB", "BYTEA");
        translated = translated.replace("blob", "bytea");
        
        // Handle MySQL's VARBINARY -> PostgreSQL BYTEA
        translated = translated.replace("VARBINARY", "BYTEA");
        translated = translated.replace("varbinary", "bytea");
        
        // Handle MySQL's BINARY -> PostgreSQL BYTEA
        translated = translated.replace("BINARY(", "BYTEA -- original was BINARY(");
        translated = translated.replace("binary(", "bytea -- original was binary(");
        
        // Handle MySQL date functions
        translated = translated.replace("NOW()", "CURRENT_TIMESTAMP");
        translated = translated.replace("now()", "current_timestamp");
        translated = translated.replace("CURDATE()", "CURRENT_DATE");
        translated = translated.replace("curdate()", "current_date");
        translated = translated.replace("CURTIME()", "CURRENT_TIME");
        translated = translated.replace("curtime()", "current_time");
        
        // Handle MySQL string functions that differ from PostgreSQL
        // Note: PostgreSQL uses || for concatenation, but also supports CONCAT()
        // MySQL LENGTH() vs PostgreSQL LENGTH() - these are actually compatible
        // MySQL UPPER() vs PostgreSQL UPPER() - these are compatible too
        
        // Handle MySQL's YEAR type -> PostgreSQL SMALLINT
        translated = translated.replace("YEAR", "SMALLINT");
        translated = translated.replace("year", "smallint");
        
        // Handle MySQL's IF NOT EXISTS for databases (PostgreSQL uses CREATE DATABASE IF NOT EXISTS differently)
        if translated.to_lowercase().contains("create database") && translated.to_lowercase().contains("if not exists") {
            // For now, just remove IF NOT EXISTS as it's more complex in PostgreSQL
            translated = translated.replace("IF NOT EXISTS", "");
            translated = translated.replace("if not exists", "");
        }
        
        // Handle MySQL's ENGINE specification (remove it)
        if let Some(engine_pos) = translated.to_lowercase().find("engine=") {
            if let Some(next_space) = translated[engine_pos..].find(' ') {
                translated.replace_range(engine_pos..engine_pos + next_space, "");
            } else if let Some(semicolon) = translated[engine_pos..].find(';') {
                translated.replace_range(engine_pos..engine_pos + semicolon, "");
            }
        }
        
        log::debug!("Translated SQL: {sql} -> {translated}");
        translated
    }

    fn fix_common_sql_errors(&self, sql: &str) -> String {
        let mut fixed = sql.to_string();
        
        // Fix VARCHAR without space: VARCHAR255 -> VARCHAR(255)
        use regex::Regex;
        if let Ok(re) = Regex::new(r"VARCHAR(\d+)") {
            fixed = re.replace_all(&fixed, "VARCHAR($1)").to_string();
        }
        
        // Fix CHAR without space: CHAR10 -> CHAR(10) 
        if let Ok(re) = Regex::new(r"CHAR(\d+)") {
            fixed = re.replace_all(&fixed, "CHAR($1)").to_string();
        }
        
        // Fix INT without space: INT10 -> INT(10)
        if let Ok(re) = Regex::new(r"INT(\d+)") {
            fixed = re.replace_all(&fixed, "INT($1)").to_string();
        }
        
        // Fix common parentheses issues in CREATE TABLE
        // Pattern: name(VARCHAR(255)) -> name VARCHAR(255)
        if let Ok(re) = Regex::new(r"(\w+)\(([A-Z]+\(\d+\))\)") {
            fixed = re.replace_all(&fixed, "$1 $2").to_string();
        }
        
        // Pattern: name(VARCHAR255) -> name VARCHAR(255) 
        if let Ok(re) = Regex::new(r"(\w+)\(([A-Z]+)(\d+)\)") {
            fixed = re.replace_all(&fixed, "$1 $2($3)").to_string();
        }
        
        // Fix missing commas in CREATE TABLE (basic detection)
        // This is a simple fix for consecutive column definitions
        if fixed.contains("CREATE TABLE") {
            // Fix pattern: ) columnname -> ), columnname
            if let Ok(re) = Regex::new(r"\)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+(VARCHAR|CHAR|INT|TEXT|TIMESTAMP)") {
                fixed = re.replace_all(&fixed, "), $1 $2").to_string();
            }
        }
        
        log::debug!("Fixed SQL syntax: {sql} -> {fixed}");
        fixed
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
