use serde::Deserialize;

use crate::sink::puaration::{
    output_metric_columns::OutputMetricColumns,
    vector_field_config::VectorFieldConfig,
};

// 工艺参数汇聚（puaration）下沉器的全部配置
#[derive(Clone, Debug, Deserialize)]
pub struct PuarationSinkConfig {
    // 物料品号的字段别名列表
    pub material_code_aliases: Vec<String>,
    // 批号的字段别名列表
    pub batch_code_aliases: Vec<String>,
    // 构成工艺参数向量的各个字段配置
    pub vector_fields: Vec<VectorFieldConfig>,
    // 输出指标列的名称配置
    pub output_metrics: OutputMetricColumns,
    // 数据库配置文件路径
    pub database_config_path: String,
}

impl Default for PuarationSinkConfig {
    fn default() -> Self {
        Self {
            material_code_aliases: vec!["物料品号".to_string(), "物料品名".to_string()],
            batch_code_aliases: vec!["批号".to_string()],
            // 默认参与向量计算的字段列表，引用公共默认值
            vector_fields: VectorFieldConfig::default_vector_fields(),
            output_metrics: OutputMetricColumns {
                material_code: "物料品号".to_string(),
                process_vector: "工艺参数向量".to_string(),
                occurrence_count: "出现次数".to_string(),
                distinct_vector_count: "不同向量数量".to_string(),
                purity: "纯度".to_string(),
                quant_isok_pct: "定量IsOK百分比".to_string(),
                qual_isok_pct: "定性IsOK百分比".to_string(),
            },
            database_config_path: "config/database/puaration_database_config.json".to_string(),
        }
    }
}
