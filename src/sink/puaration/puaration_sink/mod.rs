use std::collections::HashMap;

use crate::{
    data::data::Data,
    error::Error,
    sink::{
        data_sink::DataSink,
        puaration::{
            PURATION_SINK_CONFIG_PATH, process_vector::ProcessVector,
            puaration_sink_config::PuarationSinkConfig, puaration_stat::PuarationStat,
        },
    },
};

mod db_persistence;
mod json_persistence;
mod stats;

pub struct PuarationSink {
    next_sink: Box<dyn DataSink>,
    config: PuarationSinkConfig,
    counts_by_material: HashMap<String, HashMap<ProcessVector, usize>>,
}

impl PuarationSink {
    fn load_config() -> PuarationSinkConfig {
        let Ok(text) = std::fs::read_to_string(PURATION_SINK_CONFIG_PATH) else {
            return PuarationSinkConfig::default();
        };

        serde_json::from_str::<PuarationSinkConfig>(&text).unwrap_or_default()
    }

    pub fn new(next_sink: Box<dyn DataSink>) -> Self {
        Self {
            next_sink,
            config: Self::load_config(),
            counts_by_material: HashMap::new(),
        }
    }

    pub fn stats(&self) -> Vec<PuarationStat> {
        self.collect_stats()
    }
}

impl DataSink for PuarationSink {
    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        self.collect_valid_data(data);
        self.next_sink.sink(data)
    }

    fn get_next_sink(&self) -> Result<Option<Box<dyn DataSink>>, Error> {
        Ok(None)
    }
}

impl Drop for PuarationSink {
    fn drop(&mut self) {
        if let Err(e) = self.persist_stats_to_json() {
            eprintln!("puaration sink write json failed: {}", e);
        }
        if let Err(e) = self.persist_stats_to_database() {
            eprintln!("puaration sink write database failed: {}", e);
        }
    }
}
