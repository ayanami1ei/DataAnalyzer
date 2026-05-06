use std::{collections::HashMap, path::Path};

use rusqlite::{Connection, params_from_iter, types::Value as SqlValue};

use crate::{
    error::Error,
    sink::puaration::{
        process_vector::ProcessVector, puaration_db_config::PuarationDbConfig,
        puaration_sink::PuarationSink,
    },
};

impl PuarationSink {
    fn build_db_row_values(
        &self,
        columns: &[String],
        material_code: &str,
        vector: &ProcessVector,
        occurrence_count: usize,
        distinct_vector_count: usize,
        purity: f64,
    ) -> Vec<SqlValue> {
        let mut vector_output_map = HashMap::new();
        for (idx, field_cfg) in self.config.vector_fields.iter().enumerate() {
            if let Some(value) = vector.values.get(idx) {
                vector_output_map.insert(field_cfg.output_column.clone(), value.clone());
            }
        }

        let mut values = Vec::with_capacity(columns.len());
        for col in columns {
            if col == &self.config.output_metrics.occurrence_count {
                values.push(SqlValue::Integer(occurrence_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.distinct_vector_count {
                values.push(SqlValue::Integer(distinct_vector_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.purity {
                values.push(SqlValue::Real(purity));
                continue;
            }
            if col == &self.config.output_metrics.material_code {
                values.push(SqlValue::Text(material_code.to_string()));
                continue;
            }
            if col == &self.config.output_metrics.process_vector {
                values.push(SqlValue::Text(vector.signature()));
                continue;
            }

            let text = vector_output_map.get(col).cloned().unwrap_or_default();
            values.push(SqlValue::Text(text));
        }

        values
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

    pub(super) fn persist_stats_to_database(&self) -> Result<(), Error> {
        if self.counts_by_material.is_empty() {
            return Ok(());
        }

        if !Path::new(&self.config.database_config_path).exists() {
            return Ok(());
        }

        let db_cfg_text = std::fs::read_to_string(&self.config.database_config_path)?;
        let db_cfg: PuarationDbConfig = serde_json::from_str(&db_cfg_text)?;
        let sqlite_db_path = db_cfg
            .sqlite_db_path
            .unwrap_or_else(|| "data/puaration_stats.db".to_string());
        let table_name = db_cfg
            .table_name
            .unwrap_or_else(|| "puaration_stats".to_string());
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

        let mut conn = Connection::open(&sqlite_db_path)?;
        let create_sql = self.build_create_table_sql(&table_name, &columns);
        conn.execute_batch(&create_sql)?;

        let insert_sql = self.build_insert_sql(&table_name, &columns);
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare(&insert_sql)?;

        for (material_code, vector_counts) in &self.counts_by_material {
            let distinct_vector_count = vector_counts.len();
            if distinct_vector_count == 0 {
                continue;
            }

            for (vector, occurrence_count) in vector_counts {
                let purity = *occurrence_count as f64 / distinct_vector_count as f64;
                let row = self.build_db_row_values(
                    &columns,
                    material_code,
                    vector,
                    *occurrence_count,
                    distinct_vector_count,
                    purity,
                );
                stmt.execute(params_from_iter(row))?;
            }
        }

        drop(stmt);
        tx.commit()?;

        Ok(())
    }
}
