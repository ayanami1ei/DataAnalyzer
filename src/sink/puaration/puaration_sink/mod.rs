use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Deserialize;
use sink_macro::DataSink;
use tracing::{info, warn};

use crate::{
    data::data::Data,
    error::Error,
    sink::{
        data_sink::DataSink,
        puaration::{
            process_vector::ProcessVector,
            puaration_sink_config::PuarationSinkConfig, puaration_stat::PuarationStat,
        },
    },
};

mod persist;
mod stats;

#[derive(Deserialize)]
struct PuarationNewConfig {
    sink_config: String,
}

#[derive(DataSink)]
pub struct PuarationSink {
    pub(crate) config: PuarationSinkConfig,
    pub(crate) counts_by_material: Mutex<HashMap<String, HashMap<ProcessVector, usize>>>,
    pub(crate) batch_to_vector: Mutex<HashMap<String, (String, ProcessVector)>>,
}

impl PuarationSink {
    fn load_config(path: &str) -> PuarationSinkConfig {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("PuarationSink: cannot read config: {}", path));
        serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("PuarationSink: invalid config: {}", path))
    }

    pub fn new(config_path: &str) -> Self {
        let text = std::fs::read_to_string(config_path)
            .unwrap_or_else(|_| panic!("PuarationSink: cannot read new config: {}", config_path));
        let cfg: PuarationNewConfig = serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("PuarationSink: invalid new config: {}", config_path));
        Self {
            config: Self::load_config(&cfg.sink_config),
            counts_by_material: Mutex::new(HashMap::new()),
            batch_to_vector: Mutex::new(HashMap::new()),
        }
    }

    pub fn stats(&self) -> Vec<PuarationStat> {
        self.collect_stats(&HashMap::new())
    }

    pub fn finish(&self) -> Result<(), Error> {
        self.finish_with_quality(&HashMap::new())
    }

    pub fn finish_with_quality(
        &self,
        quality_results: &HashMap<String, crate::sink::quality_inspection::quality_stat::BatchQuality>,
    ) -> Result<(), Error> {
        if let Err(e) = self.persist_stats_to_database(quality_results) {
            warn!("puaration: db persistence failed: {}", e);
        }
        if let Err(e) = self.persist_top_n_to_database(quality_results) {
            warn!("puaration: top-n persistence failed: {}", e);
        }
        if let Ok(mut c) = self.counts_by_material.lock() { c.clear(); }
        if let Ok(mut b) = self.batch_to_vector.lock() { b.clear(); }
        Ok(())
    }

    pub(crate) fn collect_stats_from_map(
        _config: &PuarationSinkConfig,
        map: &HashMap<String, HashMap<ProcessVector, usize>>,
    ) -> Vec<PuarationStat> {
        let mut stats = Vec::new();
        for (material_code, vector_counts) in map {
            let distinct_vector_count = vector_counts.len();
            if distinct_vector_count == 0 { continue; }
            for (vector, occurrence_count) in vector_counts {
                stats.push(PuarationStat {
                    material_code: material_code.clone(),
                    process_vector: vector.signature(),
                    occurrence_count: *occurrence_count,
                    distinct_vector_count,
                    purity: *occurrence_count as f64 / distinct_vector_count as f64,
                    quant_isok_pct: 100.0,
                    qual_isok_pct: 100.0,
                });
            }
        }
        stats.sort_by(|a, b| {
            a.material_code.cmp(&b.material_code)
                .then(a.process_vector.cmp(&b.process_vector))
        });
        stats
    }

    pub fn enrich_stats_with_quality(
        stats: &mut [PuarationStat],
        batch_to_vector: &HashMap<String, (String, ProcessVector)>,
        quality_results: &HashMap<String, crate::sink::quality_inspection::quality_stat::BatchQuality>,
    ) {
        if quality_results.is_empty() { return; }
        let mut mvs: HashMap<String, HashMap<String, Vec<(f64, f64)>>> = HashMap::new();
        for (batch_no, (mat_code, vector)) in batch_to_vector {
            let Some(bq) = quality_results.get(batch_no) else { continue; };
            mvs.entry(mat_code.clone()).or_default()
                .entry(vector.signature()).or_default()
                .push((bq.quant_isok_pct, bq.qual_isok_pct));
        }
        for stat in stats.iter_mut() {
            let Some(scores) = mvs.get(&stat.material_code)
                .and_then(|m| m.get(&stat.process_vector)) else { continue; };
            let n = scores.len();
            if n > 0 {
                let (sum_qn, sum_ql) = scores
                    .iter().fold((0.0, 0.0), |(a, b), (qn, ql)| (a + qn, b + ql));
                stat.quant_isok_pct = sum_qn / n as f64;
                stat.qual_isok_pct = sum_ql / n as f64;
            }
        }
    }
}

#[async_trait]
impl DataSink for PuarationSink {
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
            let mut counts = self.counts_by_material.lock().unwrap();
            self.collect_data_into(&snapshot, &mut counts);
        }
        {
            let mut data = data.lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            data.add_flow_state("puaration".to_string());
        }
        Ok(())
    }
}

impl Drop for PuarationSink {
    fn drop(&mut self) {
        let material_count = self.counts_by_material.lock().unwrap().len();
        info!("puaration: Drop called, {} materials with stats", material_count);
        if let Err(e) = self.persist_stats_to_database(&HashMap::new()) {
            eprintln!("puaration sink write database failed: {}", e);
        }
        info!("puaration: persistence done");
    }
}
