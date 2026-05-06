use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, params_from_iter, types::Value as SqlValue};

use crate::{
    data::data::Data,
    error::Error,
    sink::{
        data_sink::DataSink,
        operator_puaration::{
            OPERATOR_PUARATION_SINK_CONFIG_PATH, operator_parser::OperatorParser,
            operator_puaration_db_config::OperatorPuarationDbConfig,
            operator_puaration_sink_config::OperatorPuarationSinkConfig,
            operator_puaration_stat::OperatorPuarationStat,
        },
    },
};

pub struct OperatorPuarationSink {
    next_sink: Box<dyn DataSink>,
    config: OperatorPuarationSinkConfig,
    counts_by_operator: HashMap<String, HashMap<String, usize>>,
}

impl OperatorPuarationSink {
    fn load_config() -> OperatorPuarationSinkConfig {
        let Ok(text) = std::fs::read_to_string(OPERATOR_PUARATION_SINK_CONFIG_PATH) else {
            return OperatorPuarationSinkConfig::default();
        };

        serde_json::from_str::<OperatorPuarationSinkConfig>(&text).unwrap_or_default()
    }

    pub fn new(next_sink: Box<dyn DataSink>) -> Self {
        Self {
            next_sink,
            config: Self::load_config(),
            counts_by_operator: HashMap::new(),
        }
    }

    pub fn stats(&self) -> Vec<OperatorPuarationStat> {
        let mut stats = Vec::new();

        for (operator_name, vector_counts) in &self.counts_by_operator {
            let distinct_vector_count = vector_counts.len();
            if distinct_vector_count == 0 {
                continue;
            }

            for (process_vector, occurrence_count) in vector_counts {
                stats.push(OperatorPuarationStat {
                    operator_name: operator_name.clone(),
                    process_vector: process_vector.clone(),
                    occurrence_count: *occurrence_count,
                    distinct_vector_count,
                    purity: *occurrence_count as f64 / distinct_vector_count as f64,
                });
            }
        }

        stats.sort_by(|a, b| {
            a.operator_name
                .cmp(&b.operator_name)
                .then(a.process_vector.cmp(&b.process_vector))
        });
        stats
    }

    fn read_value_by_aliases(data: &Data, aliases: &[String]) -> Option<String> {
        for alias in aliases {
            let Some(pair) = data.get_pair(alias) else {
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

    fn build_process_vector(&self, data: &Data) -> Option<String> {
        let mut values = Vec::with_capacity(self.config.vector_fields.len());
        for field in &self.config.vector_fields {
            values.push(Self::read_value_by_aliases(data, &field.aliases)?);
        }

        Some(values.join("|"))
    }

    fn collect_valid_data(&mut self, data: &Data) {
        if !data.is_valid() {
            return;
        }

        let Some(remark) = Self::read_value_by_aliases(data, &self.config.remark_aliases) else {
            return;
        };
        let Some(operator_name) = OperatorParser::extract_main_operator(&remark) else {
            return;
        };
        let Some(process_vector) = self.build_process_vector(data) else {
            return;
        };

        let vector_counts = self.counts_by_operator.entry(operator_name).or_default();
        let count = vector_counts.entry(process_vector).or_insert(0);
        *count += 1;
    }

    fn persist_stats_to_json(&self) -> Result<(), Error> {
        if self.counts_by_operator.is_empty() {
            return Ok(());
        }

        let path = Path::new(&self.config.result_json_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let rows = self.build_json_rows();
        let text = serde_json::to_string_pretty(&rows)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    fn persist_stats_to_database(&self) -> Result<(), Error> {
        if self.counts_by_operator.is_empty() {
            return Ok(());
        }

        if !Path::new(&self.config.database_config_path).exists() {
            return Ok(());
        }

        let db_cfg_text = std::fs::read_to_string(&self.config.database_config_path)?;
        let db_cfg: OperatorPuarationDbConfig = serde_json::from_str(&db_cfg_text)?;
        let sqlite_db_path = db_cfg
            .sqlite_db_path
            .unwrap_or_else(|| "data/operator_puaration_stats.db".to_string());
        let table_name = db_cfg
            .table_name
            .unwrap_or_else(|| "operator_puaration_stats".to_string());
        let columns = db_cfg
            .columns
            .filter(|x| !x.is_empty())
            .ok_or_else(|| Error::SourceDataFileError("missing database columns".to_string()))?;

        let db_path = Path::new(&sqlite_db_path);
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut conn = Connection::open(sqlite_db_path)?;
        let create_sql = self.build_create_table_sql(&table_name, &columns);
        conn.execute_batch(&create_sql)?;

        let insert_sql = self.build_insert_sql(&table_name, &columns);
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare(&insert_sql)?;
        for stat in self.stats() {
            let row = self.build_db_row_values(&columns, &stat);
            stmt.execute(params_from_iter(row))?;
        }
        drop(stmt);
        tx.commit()?;

        Ok(())
    }

    fn quote_ident(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn build_create_table_sql(&self, table_name: &str, columns: &[String]) -> String {
        let mut defs = Vec::with_capacity(columns.len() + 1);
        defs.push("id INTEGER PRIMARY KEY AUTOINCREMENT".to_string());

        for col in columns {
            let col_type = if col == &self.config.output_metrics.occurrence_count
                || col == &self.config.output_metrics.distinct_vector_count
            {
                "INTEGER"
            } else if col == &self.config.output_metrics.purity {
                "REAL"
            } else {
                "TEXT"
            };
            defs.push(format!("{} {} NULL", self.quote_ident(col), col_type));
        }

        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            self.quote_ident(table_name),
            defs.join(", ")
        )
    }

    fn build_insert_sql(&self, table_name: &str, columns: &[String]) -> String {
        let cols = columns
            .iter()
            .map(|x| self.quote_ident(x))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; columns.len()].join(", ");

        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.quote_ident(table_name),
            cols,
            placeholders
        )
    }

    fn build_db_row_values(
        &self,
        columns: &[String],
        stat: &OperatorPuarationStat,
    ) -> Vec<SqlValue> {
        let mut values = Vec::with_capacity(columns.len());

        for col in columns {
            if col == &self.config.output_metrics.operator_name {
                values.push(SqlValue::Text(stat.operator_name.clone()));
                continue;
            }
            if col == &self.config.output_metrics.process_vector {
                values.push(SqlValue::Text(stat.process_vector.clone()));
                continue;
            }
            if col == &self.config.output_metrics.occurrence_count {
                values.push(SqlValue::Integer(stat.occurrence_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.distinct_vector_count {
                values.push(SqlValue::Integer(stat.distinct_vector_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.purity {
                values.push(SqlValue::Real(stat.purity));
                continue;
            }

            values.push(SqlValue::Text(String::new()));
        }

        values
    }

    fn build_json_rows(&self) -> Vec<serde_json::Value> {
        let mut rows = Vec::new();
        for stat in self.stats() {
            let mut row = serde_json::Map::new();
            row.insert(
                self.config.output_metrics.operator_name.clone(),
                serde_json::Value::String(stat.operator_name),
            );
            row.insert(
                self.config.output_metrics.process_vector.clone(),
                serde_json::Value::String(stat.process_vector),
            );
            row.insert(
                self.config.output_metrics.occurrence_count.clone(),
                serde_json::Value::Number((stat.occurrence_count as u64).into()),
            );
            row.insert(
                self.config.output_metrics.distinct_vector_count.clone(),
                serde_json::Value::Number((stat.distinct_vector_count as u64).into()),
            );

            let purity_value = serde_json::Number::from_f64(stat.purity)
                .map(serde_json::Value::Number)
                .unwrap_or_else(|| serde_json::Value::String(format!("{:.6}", stat.purity)));
            row.insert(self.config.output_metrics.purity.clone(), purity_value);

            rows.push(serde_json::Value::Object(row));
        }

        rows
    }
}

impl DataSink for OperatorPuarationSink {
    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        self.collect_valid_data(data);
        self.next_sink.sink(data)
    }

    fn get_next_sink(&self) -> Result<Option<Box<dyn DataSink>>, Error> {
        Ok(None)
    }
}

impl Drop for OperatorPuarationSink {
    fn drop(&mut self) {
        if let Err(e) = self.persist_stats_to_json() {
            eprintln!("operator puaration sink write json failed: {}", e);
        }
        if let Err(e) = self.persist_stats_to_database() {
            eprintln!("operator puaration sink write database failed: {}", e);
        }
    }
}
