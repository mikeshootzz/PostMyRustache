use std::io;
use std::sync::Arc;
use tokio::io::AsyncWrite;
use tokio_postgres::Client;
use opensrv_mysql::*;
use async_trait::async_trait;

use crate::auth::AuthProvider;
use crate::query::{QueryHandler, QueryResult};

pub struct Backend {
    auth_provider: AuthProvider,
    query_handler: QueryHandler,
}

impl Backend {
    pub fn new(pg_client: Arc<Client>, auth_provider: AuthProvider) -> Self {
        let query_handler = QueryHandler::new(Arc::clone(&pg_client));
        
        Self {
            auth_provider,
            query_handler,
        }
    }
}

#[async_trait]
impl<W: AsyncWrite + Send + Unpin> AsyncMysqlShim<W> for Backend {
    type Error = io::Error;

    async fn authenticate(
        &self,
        _auth_plugin: &str,
        username: &[u8],
        _salt: &[u8],
        _auth_data: &[u8],
    ) -> bool {
        let username_str = String::from_utf8_lossy(username);
        self.auth_provider.authenticate(&username_str)
    }

    fn default_auth_plugin(&self) -> &str {
        self.auth_provider.default_auth_plugin()
    }

    async fn auth_plugin_for_username(&self, _user: &[u8]) -> &str {
        self.auth_provider.default_auth_plugin()
    }

    fn salt(&self) -> [u8; 20] {
        self.auth_provider.generate_salt()
    }

    async fn on_init<'a>(
        &'a mut self,
        _: &'a str,
        _: InitWriter<'a, W>
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_prepare<'a>(
        &'a mut self,
        _: &'a str,
        info: StatementMetaWriter<'a, W>,
    ) -> io::Result<()> {
        info.reply(42, &[], &[]).await
    }

    async fn on_execute<'a>(
        &'a mut self,
        _: u32,
        _: opensrv_mysql::ParamParser<'a>,
        results: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        results.completed(OkResponse::default()).await
    }

    async fn on_close(&mut self, _: u32) {
        // Clean up resources here, if necessary.
    }

    async fn on_query<'a>(
        &'a mut self,
        sql: &'a str,
        results: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        match self.query_handler.handle_query(sql).await? {
            QueryResult::Ok(response) => {
                results.completed(response).await
            }
        }
    }
}