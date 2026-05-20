use serde::Deserialize;

use crate::sink::operator_puaration::operator_output_metric_columns::OperatorOutputMetricColumns;
use crate::sink::puaration::vector_field_config::VectorFieldConfig;

// 操作工分离Sink的配置
// 包含：备注列别名、工艺字段映射、输出指标列名、数据库连接信息
#[derive(Clone, Debug, Deserialize)]
pub struct OperatorPuarationSinkConfig {
    pub remark_aliases: Vec<String>,          // 备注数据字段的别名列表，用于模糊匹配查找
    pub vector_fields: Vec<VectorFieldConfig>,  // 参与工艺向量计算的字段配置列表
    pub output_metrics: OperatorOutputMetricColumns,  // 输出指标使用的列名称
    pub database_config_path: String,         // 数据库连接配置文件路径
}

impl Default for OperatorPuarationSinkConfig {
    fn default() -> Self {
        Self {
            // 默认备注字段别名
            remark_aliases: vec!["备注信息".to_string(), "备注".to_string()],
            // 默认参与向量计算的字段列表，引用公共默认值
            vector_fields: VectorFieldConfig::default_vector_fields(),
            // 默认输出指标列名
            output_metrics: OperatorOutputMetricColumns {
                operator_name: "操机手".to_string(),
                process_vector: "工艺参数向量".to_string(),
                occurrence_count: "出现次数".to_string(),
                distinct_vector_count: "不同向量数量".to_string(),
                purity: "纯度".to_string(),
            },
            // 默认数据库配置文件路径
            database_config_path: "config/database/operator_puaration_database_config.json"
                .to_string(),
        }
    }
}
