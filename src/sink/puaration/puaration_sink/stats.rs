use std::collections::HashMap;

use crate::{
    data::data::Data,
    sink::{
        puaration::{
            process_vector::ProcessVector, puaration_sink::PuarationSink,
            puaration_stat::{PuarationStat, TopRankedVector},
        },
        quality_inspection::quality_stat::BatchQuality,
    },
};

impl PuarationSink {
    // 从单行数据中提取物料品号和工艺参数向量；
    // 若缺少任何必要字段则返回 None
    pub(super) fn build_record(&self, data: &Data) -> Option<(String, ProcessVector)> {
        let material_code = data.get_value_by_aliases(&self.config.material_code_aliases)?;

        let mut values = Vec::with_capacity(self.config.vector_fields.len());
        for field_cfg in &self.config.vector_fields {
            values.push(data.get_value_by_aliases(&field_cfg.aliases)?);
        }

        Some((material_code, ProcessVector { values }))
    }

    // 将一行数据归入物料的向量计数，并记录批次号到向量的映射（仅首次）
    pub(super) fn collect_data_into(
        &self,
        data: &Data,
        counts_by_material: &mut HashMap<String, HashMap<ProcessVector, usize>>,
    ) {
        // 提取物料品号和工艺参数向量，提取失败则跳过此行数据
        let Some((material_code, vector)) = self.build_record(data) else {
            return;
        };

        // 在对应物料的映射中递增该向量的计数
        let vector_counts = counts_by_material.entry(material_code).or_default();
        let count = vector_counts.entry(vector).or_insert(0);
        *count += 1;

        // 若数据中包含批号字段，则将批次号与物料/向量关联，用于后续质检数据聚合
        if let Some(batch_no) = data.get_value_by_aliases(&self.config.batch_code_aliases) {
            let mut b2v = self.batch_to_vector.lock().unwrap_or_else(|e| e.into_inner());
            b2v.entry(batch_no).or_insert_with(|| {
                let material_code =
                    data.get_value_by_aliases(&self.config.material_code_aliases)
                        .unwrap_or_default();
                let mut values = Vec::with_capacity(self.config.vector_fields.len());
                for field_cfg in &self.config.vector_fields {
                    values.push(
                        data.get_value_by_aliases(&field_cfg.aliases)
                            .unwrap_or_default(),
                    );
                }
                (material_code, ProcessVector { values })
            });
        }
    }

    // 收集统计结果，并将质检数据关联到每条统计记录后返回
    pub(super) fn collect_stats(&self, quality_results: &HashMap<String, BatchQuality>) -> Vec<PuarationStat> {
        let map = self.counts_by_material.lock().unwrap_or_else(|e| e.into_inner());
        let mut stats = Self::collect_stats_from_map(&self.config, &map);
        let b2v = self.batch_to_vector.lock().unwrap_or_else(|e| e.into_inner());
        Self::enrich_stats_with_quality(&mut stats, &b2v, quality_results);
        stats
    }

    // 计算每个物料下综合得分最高的前 n 个向量，返回带排名的结果列表
    pub fn top_vectors_per_material(
        &self,
        n: usize,
        quality_results: &HashMap<String, BatchQuality>,
    ) -> Vec<TopRankedVector> {
        let map = self.counts_by_material.lock().unwrap_or_else(|e| e.into_inner());
        let mut stats = Self::collect_stats_from_map(&self.config, &map);
        let b2v = self.batch_to_vector.lock().unwrap_or_else(|e| e.into_inner());
        Self::enrich_stats_with_quality(&mut stats, &b2v, quality_results);

        // 按物料品号分组
        let mut by_material: std::collections::HashMap<String, Vec<PuarationStat>> =
            std::collections::HashMap::new();
        for stat in &stats {
            by_material.entry(stat.material_code.clone()).or_default().push(stat.clone());
        }

        let mut result = Vec::new();
        // 对每个物料的统计按综合得分降序排列，取前 n 个
        for mut vec_stats in by_material.into_values() {
            vec_stats.sort_by(|a, b| {
                b.composite_score()
                    .partial_cmp(&a.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for (i, stat) in vec_stats.into_iter().take(n).enumerate() {
                result.push(TopRankedVector::from_stat(&stat, i + 1));
            }
        }

        // 按物料品号 + 排名排序输出
        result.sort_by(|a, b| {
            a.material_code
                .cmp(&b.material_code)
                .then(a.rank.cmp(&b.rank))
        });
        result
    }
}
