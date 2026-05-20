use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use sink_macro::DataSink;
use tracing::{info, warn};

use crate::{
    config, data::data::Data,
    error::Error,
    router::Router,
    sink::data_sink::DataSink,
};

mod persist;
mod process;

pub(crate) const QUALITY_DB_CONFIG_PATH: &str = "config/database/quality_inspection_database_config.json";

#[derive(DataSink)]
pub struct QualityInspectionSink {
    records: Mutex<Vec<Data>>,
    results: Mutex<HashMap<String, crate::sink::quality_inspection::quality_stat::BatchQuality>>,
    router: Mutex<Option<Weak<Router>>>,
}

impl QualityInspectionSink {
    pub fn new(_config_path: &str) -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            results: Mutex::new(HashMap::new()),
            router: Mutex::new(None),
        }
    }

    pub fn get_results(&self) -> HashMap<String, crate::sink::quality_inspection::quality_stat::BatchQuality> {
        self.results.lock().unwrap().clone()
    }

    fn collect_data_into(&self, data: &Data, records: &mut Vec<Data>) {
        let qi = config::quality_inspection_fields();
        if data.get_pair(&qi.param_raw).is_none() { return; }
        records.push(data.clone());
    }

    pub fn extract_str(data: &Data, key: &str) -> String {
        data.get_pair(&key.to_string())
            .map(|p| p.value.clone()).unwrap_or_default()
    }

    pub fn extract_bool(data: &Data, key: &str) -> bool {
        let v = Self::extract_str(data, key);
        v == "True" || v == "true" || v == "1"
    }

    fn log_errors(all_errors: &[crate::sink::quality_inspection::quality_stat::IsokError]) {
        warn!("=== IsOK ERRORS DETECTED ===");
        for err in all_errors {
            warn!(
                "batch={}, item={}, param={}, value={}, expected={}, actual={}",
                err.batch_no, err.check_item, err.param_raw,
                err.test_value, err.expected_isok, err.actual_isok
            );
        }
    }

    pub fn finish(&self) -> Result<(), Error> {
        let records = self.records.lock()
            .unwrap_or_else(|e| e.into_inner()).clone();
        if records.is_empty() {
            info!("quality_inspection: no records to process");
            return Ok(());
        }

        let (batches, detail_rows, error_rows) = QualityInspectionSink::process_records(&records);
        let errors: Vec<_> = batches.iter().flat_map(|b| b.errors.clone()).collect();

        let mut cache = self.results.lock().unwrap();
        cache.clear();
        for bq in &batches {
            cache.insert(bq.batch_no.clone(), bq.clone());
        }

        if let Err(e) = QualityInspectionSink::persist_database(&detail_rows, &error_rows) {
            warn!("quality: db persistence failed: {}", e);
        }

        info!("quality: processed {} batches, {} items, {} IsOK errors",
            batches.len(), detail_rows.len(), errors.len());

        if !errors.is_empty() { Self::log_errors(&errors); }
        Ok(())
    }
}

#[async_trait]
impl DataSink for QualityInspectionSink {
    fn new(config_path: &str) -> Self {
        Self::new(config_path)
    }

    fn set_router(&self, router: Weak<Router>) {
        *self.router.lock().unwrap() = Some(router);
    }

    fn finish(&self) -> Result<(), Error> {
        self.finish()
    }

    async fn sink(&self, data: Arc<Mutex<Data>>) -> Result<(), Error> {
        let snapshot = {
            let d = data.lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            d.clone()
        };
        {
            let Ok(mut records) = self.records.lock() else { return Ok(()); };
            self.collect_data_into(&snapshot, &mut records);
        }
        {
            let mut d = data.lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            d.add_flow_state("quality_inspection".to_string());
        }
        let router = self.router.lock().unwrap().as_ref().and_then(|w| w.upgrade());
        if let Some(router) = router {
            router.route_data(data).await?;
        }
        Ok(())
    }
}
