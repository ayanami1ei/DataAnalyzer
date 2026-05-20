use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rusqlite::types::Value as SqlValue;
use serde::Deserialize;
use sink_macro::DataSink;
use tracing::warn;

use crate::{
    config, data::data::Data,
    error::Error,
    sink::{
        database::sqlite_writer,
        data_sink::DataSink, material_production::material_production_stat::MaterialProductionStat,
    },
};

#[derive(Deserialize)]
struct MaterialProductionConfig {
    output_db_path: String,
    table_name: String,
}

#[derive(DataSink)]
pub struct MaterialProductionSink {
    output_db_path: String,
    output_table: String,
    batches_by_material: Mutex<HashMap<String, HashSet<String>>>,
}

impl MaterialProductionSink {
    pub fn new(config_path: &str) -> Self {
        let text = std::fs::read_to_string(config_path)
            .unwrap_or_else(|_| panic!("MaterialProductionSink: cannot read config: {}", config_path));
        let cfg: MaterialProductionConfig = serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("MaterialProductionSink: invalid config: {}", config_path));
        Self {
            output_db_path: cfg.output_db_path,
            output_table: cfg.table_name,
            batches_by_material: Mutex::new(HashMap::new()),
        }
    }

    pub fn stats(&self) -> Vec<MaterialProductionStat> {
        let map = self.batches_by_material.lock().unwrap();
        Self::collect_stats(&map)
    }

    pub fn finish(&self) -> Result<(), Error> {
        if let Err(e) = self.persist_stats_to_database() {
            warn!("material_production: db persistence failed: {}", e);
        }
        self.batches_by_material.lock().unwrap().clear();
        Ok(())
    }

    fn collect_stats(map: &HashMap<String, HashSet<String>>) -> Vec<MaterialProductionStat> {
        let mut stats: Vec<MaterialProductionStat> = map
            .iter()
            .map(|(material_code, batches)| MaterialProductionStat {
                material_code: material_code.clone(),
                production_count: batches.len(),
            })
            .collect();
        stats.sort_by(|a, b| a.material_code.cmp(&b.material_code));
        stats
    }

    fn collect_data_into(
        data: &Data,
        batches_by_material: &mut HashMap<String, HashSet<String>>,
    ) {
        let mf = config::material_production_fields();
        let Some(material_code) = data.get_value_by_aliases(&mf.material_aliases) else { return; };
        let Some(batch_no) = data.get_value_by_aliases(&mf.batch_aliases) else { return; };
        batches_by_material.entry(material_code).or_default().insert(batch_no);
    }

    fn persist_stats_to_database(&self) -> Result<(), Error> {
        let map = self.batches_by_material.lock().unwrap();
        if map.is_empty() { return Ok(()); }

        sqlite_writer::prepare_directory(&self.output_db_path)?;
        let mut conn = sqlite_writer::open_sqlite(
            &self.output_db_path,
            &[("synchronous", "OFF"), ("journal_mode", "MEMORY")],
        )?;

        let mf = config::material_production_fields();
        let columns = mf.columns.clone();
        let col_defs = vec![
            sqlite_writer::ColumnDef::text(&columns[0]),
            sqlite_writer::ColumnDef::integer(&columns[1]),
        ];
        sqlite_writer::create_table(&conn, &self.output_table, &col_defs)?;

        let rows: Vec<Vec<SqlValue>> = map
            .iter()
            .map(|(material_code, batches)| {
                vec![
                    SqlValue::Text(material_code.clone()),
                    SqlValue::Integer(batches.len() as i64),
                ]
            })
            .collect();

        sqlite_writer::batch_insert_values(&mut conn, &self.output_table, &columns, &rows)?;
        Ok(())
    }
}

#[async_trait]
impl DataSink for MaterialProductionSink {
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
            let mut map = self.batches_by_material.lock().unwrap();
            Self::collect_data_into(&snapshot, &mut map);
        }
        {
            let mut data = data.lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            data.add_flow_state("material_production".to_string());
        }
        Ok(())
    }
}

impl Drop for MaterialProductionSink {
    fn drop(&mut self) {
        if let Err(e) = self.persist_stats_to_database() {
            eprintln!("material production sink write database failed: {}", e);
        }
    }
}
