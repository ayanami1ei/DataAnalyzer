use crate::{
    data::data::Data,
    sink::puaration::{
        process_vector::ProcessVector, puaration_sink::PuarationSink, puaration_stat::PuarationStat,
    },
};

impl<SinkType: crate::sink::data_sink::DataSink> PuarationSink<SinkType> {
    pub(super) fn read_value_by_aliases(data: &Data, aliases: &[String]) -> Option<String> {
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

    pub(super) fn build_record(&self, data: &Data) -> Option<(String, ProcessVector)> {
        let material_code = Self::read_value_by_aliases(data, &self.config.material_code_aliases)?;

        let mut values = Vec::with_capacity(self.config.vector_fields.len());
        for field_cfg in &self.config.vector_fields {
            values.push(Self::read_value_by_aliases(data, &field_cfg.aliases)?);
        }

        Some((material_code, ProcessVector { values }))
    }

    pub(super) fn collect_valid_data(&mut self, data: &Data) {
        if !data.is_valid() {
            return;
        }

        let Some((material_code, vector)) = self.build_record(data) else {
            return;
        };

        let vector_counts = self.counts_by_material.entry(material_code).or_default();
        let count = vector_counts.entry(vector).or_insert(0);
        *count += 1;
    }

    pub(super) fn collect_stats(&self) -> Vec<PuarationStat> {
        let mut stats = Vec::new();

        for (material_code, vector_counts) in &self.counts_by_material {
            let distinct_vector_count = vector_counts.len();
            if distinct_vector_count == 0 {
                continue;
            }

            for (vector, occurrence_count) in vector_counts {
                stats.push(PuarationStat {
                    material_code: material_code.clone(),
                    process_vector: vector.signature(),
                    occurrence_count: *occurrence_count,
                    distinct_vector_count,
                    purity: *occurrence_count as f64 / distinct_vector_count as f64,
                });
            }
        }

        stats.sort_by(|a, b| {
            a.material_code
                .cmp(&b.material_code)
                .then(a.process_vector.cmp(&b.process_vector))
        });

        stats
    }
}
