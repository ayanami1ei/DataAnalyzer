use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::Connection;

use crate::{
    data::data::Data,
    error::Error,
    sink::{
        data_sink::DataSink,
        material_production::material_production_stat::MaterialProductionStat,
    },
};

const MATERIAL_ALIASES: [&str; 2] = ["物料品号", "物料品名"];
const BATCH_ALIASES: [&str; 1] = ["批号"];
const OUTPUT_JSON_PATH: &str = "data/material_production_stats.json";
const OUTPUT_DB_PATH: &str = "data/material_production_stats.db";
const OUTPUT_TABLE: &str = "material_production_stats";

pub struct MaterialProductionSink<SinkType: DataSink> {
    next_sink: SinkType,
    batches_by_material: HashMap<String, HashSet<String>>,
}

impl<SinkType: DataSink> MaterialProductionSink<SinkType> {
    pub fn new(next_sink: SinkType) -> Self {
        Self {
            next_sink,
            batches_by_material: HashMap::new(),
        }
    }

    pub fn stats(&self) -> Vec<MaterialProductionStat> {
        let mut stats = self
            .batches_by_material
            .iter()
            .map(|(material_code, batches)| MaterialProductionStat {
                material_code: material_code.clone(),
                production_count: batches.len(),
            })
            .collect::<Vec<_>>();

        stats.sort_by(|a, b| a.material_code.cmp(&b.material_code));
        stats
    }

    fn read_value_by_aliases(data: &Data, aliases: &[&str]) -> Option<String> {
        for alias in aliases {
            let key = alias.to_string();
            let Some(pair) = data.get_pair(&key) else {
                continue;
            };
            let value = pair.value.trim();
            if value.is_empty() {
                continue;
            }
            return Some(value.to_string());
        }

        None
    }

    fn collect_valid_data(&mut self, data: &Data) {
        if !data.is_valid() {
            return;
        }

        let Some(material_code) = Self::read_value_by_aliases(data, &MATERIAL_ALIASES) else {
            return;
        };
        let Some(batch_no) = Self::read_value_by_aliases(data, &BATCH_ALIASES) else {
            return;
        };

        self.batches_by_material
            .entry(material_code)
            .or_default()
            .insert(batch_no);
    }

    fn persist_stats_to_json(&self) -> Result<(), Error> {
        if self.batches_by_material.is_empty() {
            return Ok(());
        }

        let path = Path::new(OUTPUT_JSON_PATH);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let text = serde_json::to_string_pretty(&self.stats())?;
        std::fs::write(path, text)?;
        Ok(())
    }

    fn persist_stats_to_database(&self) -> Result<(), Error> {
        if self.batches_by_material.is_empty() {
            return Ok(());
        }

        let db_path = Path::new(OUTPUT_DB_PATH);
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(OUTPUT_DB_PATH)?;
        conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (id INTEGER PRIMARY KEY AUTOINCREMENT, \"物料品号\" TEXT NULL, \"生产次数\" INTEGER NULL)",
            OUTPUT_TABLE
        ))?;

        let sql = format!(
            "INSERT INTO \"{}\" (\"物料品号\", \"生产次数\") VALUES (?1, ?2)",
            OUTPUT_TABLE
        );

        let mut stmt = conn.prepare(&sql)?;
        for stat in self.stats() {
            stmt.execute(rusqlite::params![stat.material_code, stat.production_count as i64])?;
        }

        Ok(())
    }
}

impl<SinkType: DataSink> DataSink for MaterialProductionSink<SinkType> {
    type NextType = SinkType;

    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        self.collect_valid_data(data);
        self.next_sink.sink(data)
    }

    fn get_next_sink(&self) -> Result<Option<Self::NextType>, Error> {
        Ok(None)
    }
}

impl<SinkType: DataSink> Drop for MaterialProductionSink<SinkType> {
    fn drop(&mut self) {
        if let Err(e) = self.persist_stats_to_json() {
            eprintln!("material production sink write json failed: {}", e);
        }
        if let Err(e) = self.persist_stats_to_database() {
            eprintln!("material production sink write database failed: {}", e);
        }
    }
}
