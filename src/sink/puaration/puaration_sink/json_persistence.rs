use std::path::Path;

use serde_json::{Map, Value};

use crate::{
    error::Error,
    sink::puaration::{
        process_vector::ProcessVector, puaration_sink::PuarationSink,
        puaration_sink_config::PuarationSinkConfig,
    },
};

impl PuarationSink {
    fn build_json_row(
        config: &PuarationSinkConfig,
        material_code: &str,
        vector: &ProcessVector,
        occurrence_count: usize,
        distinct_vector_count: usize,
    ) -> Value {
        let purity = occurrence_count as f64 / distinct_vector_count as f64;
        let mut row = Map::new();

        row.insert(
            config.output_metrics.material_code.clone(),
            Value::String(material_code.to_string()),
        );
        for (idx, field_cfg) in config.vector_fields.iter().enumerate() {
            if let Some(value) = vector.values.get(idx) {
                row.insert(
                    field_cfg.output_column.clone(),
                    Value::String(value.clone()),
                );
            }
        }
        row.insert(
            config.output_metrics.process_vector.clone(),
            Value::String(vector.signature()),
        );
        row.insert(
            config.output_metrics.occurrence_count.clone(),
            Value::Number((occurrence_count as u64).into()),
        );
        row.insert(
            config.output_metrics.distinct_vector_count.clone(),
            Value::Number((distinct_vector_count as u64).into()),
        );

        let purity_value = serde_json::Number::from_f64(purity)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(format!("{:.6}", purity)));
        row.insert(config.output_metrics.purity.clone(), purity_value);

        Value::Object(row)
    }

    pub(super) fn persist_stats_to_json(&self) -> Result<(), Error> {
        if self.counts_by_material.is_empty() {
            return Ok(());
        }

        let mut rows = Vec::new();
        for (material_code, vector_counts) in &self.counts_by_material {
            let distinct_vector_count = vector_counts.len();
            if distinct_vector_count == 0 {
                continue;
            }

            for (vector, occurrence_count) in vector_counts {
                rows.push(Self::build_json_row(
                    &self.config,
                    material_code,
                    vector,
                    *occurrence_count,
                    distinct_vector_count,
                ));
            }
        }

        let json_path = Path::new(&self.config.result_json_path);
        if let Some(parent) = json_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let text = serde_json::to_string_pretty(&rows)?;
        std::fs::write(json_path, text)?;
        Ok(())
    }
}
