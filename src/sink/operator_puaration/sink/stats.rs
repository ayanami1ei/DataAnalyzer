use std::collections::HashMap;

use crate::{
    data::data::Data,
    sink::{
        operator_puaration::{
            operator_parser::OperatorParser,
            operator_puaration_stat::OperatorPuarationStat,
        },
        puaration::process_vector::ProcessVector,
    },
};

use super::OperatorPuarationSink;

impl OperatorPuarationSink {
    // 将嵌套的统计映射（操作工 -> 向量 -> 次数）展开为扁平化的统计记录列表
    // 按操作工姓名和向量值排序后返回
    pub(crate) fn collect_stats_from_map(
        map: &HashMap<String, HashMap<ProcessVector, usize>>,
    ) -> Vec<OperatorPuarationStat> {
        let mut stats = Vec::new();

        // 遍历每个操作工及其下辖的工艺向量与计数值
        for (operator_name, vector_counts) in map {
            let distinct_vector_count = vector_counts.len();
            // 跳过没有任何向量的操作工记录
            if distinct_vector_count == 0 {
                continue;
            }

            // 为每条向量生成一条统计记录，纯度 = 该向量频次 / 向量种类数
            for (vector, occurrence_count) in vector_counts {
                stats.push(OperatorPuarationStat {
                    operator_name: operator_name.clone(),
                    vector_values: vector.values.clone(),
                    occurrence_count: *occurrence_count,
                    distinct_vector_count,
                    purity: *occurrence_count as f64 / distinct_vector_count as f64,
                });
            }
        }

        // 按操作工姓名排序，同类操作工内按向量值排序
        stats.sort_by(|a, b| {
            a.operator_name
                .cmp(&b.operator_name)
                .then(a.vector_values.cmp(&b.vector_values))
        });
        stats
    }

    // 从数据行中构建工艺参数向量
    // 按配置的vector_fields顺序，通过字段别名从数据中提取各维度的值
    pub(crate) fn build_process_vector(&self, data: &Data) -> Option<ProcessVector> {
        let mut values = Vec::with_capacity(self.config.vector_fields.len());
        for field in &self.config.vector_fields {
            values.push(data.get_value_by_aliases(&field.aliases)?);
        }

        Some(ProcessVector { values })
    }

    // 将单条数据中的操作工-向量信息收集到统计映射中
    // 步骤：提取备注 -> 识别操作工 -> 构建向量 -> 计数累加
    pub(crate) fn collect_data_into(
        &self,
        data: &Data,
        counts_by_operator: &mut HashMap<String, HashMap<ProcessVector, usize>>,
    ) {
        // 通过别名查找备注字段值，若不存在则跳过
        let Some(remark) = data.get_value_by_aliases(&self.config.remark_aliases) else {
            return;
        };
        // 从备注中解析操作工姓名，解析失败则跳过
        let Some(operator_name) = OperatorParser::extract_main_operator(&remark) else {
            return;
        };
        // 从数据中构建工艺向量，构建失败则跳过
        let Some(vector) = self.build_process_vector(data) else {
            return;
        };

        // 在嵌套映射中累加当前向量的出现次数
        let vector_counts = counts_by_operator.entry(operator_name).or_default();
        let count = vector_counts.entry(vector).or_insert(0);
        *count += 1;
    }
}
