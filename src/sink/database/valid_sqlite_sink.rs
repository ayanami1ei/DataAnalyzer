use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Deserialize;
use sink_macro::DataSink;
use tracing::info;

use std::sync::Weak;

use crate::{
    data::data::Data, error::Error, router::Router,
    sink::{
        data_sink::DataSink,
        database::sqlite_database_sink::SqliteDatabaseSink,
    },
};

#[derive(Deserialize)]
struct ValidSqliteConfig {
    database_config: String,
    table_name: String,
    flow_state: String,
}

#[derive(DataSink)]
pub struct ValidSqliteSink {
    inner: SqliteDatabaseSink,
    flow_state: String,
}

impl ValidSqliteSink {
    pub fn new(config_path: &str) -> Self {
        let text = std::fs::read_to_string(config_path)
            .unwrap_or_else(|_| panic!("ValidSqliteSink: cannot read config: {}", config_path));
        let cfg: ValidSqliteConfig = serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("ValidSqliteSink: invalid config: {}", config_path));
        let inner = SqliteDatabaseSink::new(&cfg.database_config);
        inner.configure(&cfg.database_config, &cfg.table_name)
            .expect("ValidSqliteSink: failed to configure database");
        info!("valid_sqlite: configured, table={}", cfg.table_name);
        Self { inner, flow_state: cfg.flow_state }
    }
}

#[async_trait]
impl DataSink for ValidSqliteSink {
    fn new(config_path: &str) -> Self {
        Self::new(config_path)
    }

    fn set_router(&self, router: Weak<Router>) {
        self.inner.set_router(router);
    }

    async fn sink(&self, data: Arc<Mutex<Data>>) -> Result<(), Error> {
        self.inner.sink_impl(data, &self.flow_state).await
    }

    fn finish(&self) -> Result<(), Error> {
        self.inner.finish()
    }
}
