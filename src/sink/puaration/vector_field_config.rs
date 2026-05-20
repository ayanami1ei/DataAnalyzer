use serde::Deserialize;

// 单个工艺参数字段的配置：字段别名列表及输出列名
#[derive(Clone, Debug, Deserialize)]
pub struct VectorFieldConfig {
    // 该字段可能使用的名称别名，用于从原始数据中匹配
    pub aliases: Vec<String>,
    // 在输出数据库表中使用的列名
    pub output_column: String,
}

impl VectorFieldConfig {
    // 公共默认工艺向量字段列表，OperatorPuarationSink 和 PuarationSink 共用
    // 修改此处可同步更新两个 Sink 的默认配置
    pub fn default_vector_fields() -> Vec<VectorFieldConfig> {
    vec![
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
    ]
}
}
