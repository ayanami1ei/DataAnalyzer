use serde::Deserialize;

use crate::sink::puaration::{
    output_metric_columns::OutputMetricColumns, vector_field_config::VectorFieldConfig,
};

#[derive(Clone, Debug, Deserialize)]
pub struct PuarationSinkConfig {
    pub material_code_aliases: Vec<String>,
    pub vector_fields: Vec<VectorFieldConfig>,
    pub output_metrics: OutputMetricColumns,
    pub result_json_path: String,
    pub database_config_path: String,
}

impl Default for PuarationSinkConfig {
    fn default() -> Self {
        Self {
            material_code_aliases: vec!["物料品号".to_string(), "物料品名".to_string()],
            vector_fields: vec![
                VectorFieldConfig {
                    aliases: vec!["缆芯外径".to_string(), "缆芯外径（mm)".to_string()],
                    output_column: "缆芯外径".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec!["护套外径".to_string(), "护套外径(mm)".to_string()],
                    output_column: "护套外径".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec!["挤出内模".to_string(), "挤出内模(mm)".to_string()],
                    output_column: "挤出内模".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec!["挤出外模".to_string(), "挤出外模(mm)".to_string()],
                    output_column: "挤出外模".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec![
                        "螺杆速度".to_string(),
                        "螺杆速度(rpm)-(挤塑主机速度)（转/分）".to_string(),
                    ],
                    output_column: "螺杆速度".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec!["螺杆电流".to_string(), "螺杆电流（A）".to_string()],
                    output_column: "螺杆电流".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec![
                        "实际生产速度".to_string(),
                        "实际生产速度（m/min）".to_string(),
                    ],
                    output_column: "实际生产速度".to_string(),
                },
                VectorFieldConfig {
                    aliases: vec!["设备名称".to_string()],
                    output_column: "设备名称".to_string(),
                },
            ],
            output_metrics: OutputMetricColumns {
                material_code: "物料品号".to_string(),
                process_vector: "工艺参数向量".to_string(),
                occurrence_count: "出现次数".to_string(),
                distinct_vector_count: "不同向量数量".to_string(),
                purity: "纯度".to_string(),
            },
            result_json_path: "data/puaration_stats.json".to_string(),
            database_config_path: "config/database/puaration_database_config.json".to_string(),
        }
    }
}
