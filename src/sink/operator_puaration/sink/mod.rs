use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Deserialize;
use sink_macro::DataSink;
use tracing::warn;

use crate::{
    data::data::Data,
    error::Error,
    sink::{
        data_sink::DataSink,
        operator_puaration::{
            operator_puaration_sink_config::OperatorPuarationSinkConfig,
            operator_puaration_stat::OperatorPuarationStat,
        },
        puaration::process_vector::ProcessVector,
    },
};

mod persist;
mod stats;

#[derive(Deserialize)]
struct OperatorPuarationNewConfig {
    sink_config: String,
}

#[derive(DataSink)]
pub struct OperatorPuarationSink {
    pub(crate) config: OperatorPuarationSinkConfig,
    pub(crate) counts_by_operator: Mutex<HashMap<String, HashMap<ProcessVector, usize>>>,
}

impl OperatorPuarationSink {
    fn load_config(path: &str) -> OperatorPuarationSinkConfig {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("OperatorPuarationSink: cannot read config: {}", path));
        serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("OperatorPuarationSink: invalid config: {}", path))
    }

    pub fn new(config_path: &str) -> Self {
        let text = std::fs::read_to_string(config_path)
            .unwrap_or_else(|_| panic!("OperatorPuarationSink: cannot read new config: {}", config_path));
        let cfg: OperatorPuarationNewConfig = serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("OperatorPuarationSink: invalid new config: {}", config_path));
        Self {
            config: Self::load_config(&cfg.sink_config),
            counts_by_operator: Mutex::new(HashMap::new()),
        }
    }

    pub fn stats(&self) -> Vec<OperatorPuarationStat> {
        let map = self.counts_by_operator.lock().unwrap();
        Self::collect_stats_from_map(&map)
    }

    pub fn finish(&self) -> Result<(), Error> {
        if let Err(e) = self.persist_stats_to_database() {
            warn!("operator_puaration: db persistence failed: {}", e);
        }
        self.counts_by_operator.lock().unwrap().clear();
        Ok(())
    }
}

#[async_trait]
impl DataSink for OperatorPuarationSink {
    fn new(config_path: &str) -> Self {
        Self::new(config_path)
    }

    async fn sink(&self, data: Arc<Mutex<Data>>) -> Result<(), Error> {
        let snapshot = {
            let data = data.lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            data.clone()
        };
        {
            let mut counts = self.counts_by_operator.lock().unwrap();
            self.collect_data_into(&snapshot, &mut counts);
        }
        {
            let mut data = data.lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            data.add_flow_state("operator_puaration".to_string());
        }
        Ok(())
    }
}

impl Drop for OperatorPuarationSink {
    fn drop(&mut self) {
        if let Err(e) = self.persist_stats_to_database() {
            eprintln!("operator puaration sink write database failed: {}", e);
        }
    }
}
