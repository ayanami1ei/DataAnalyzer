use serde::Deserialize;

// 操作工分离输出指标列名配置
// 定义各统计指标在输出中的列名称
#[derive(Clone, Debug, Deserialize)]
pub struct OperatorOutputMetricColumns {
    pub operator_name: String,           // 操作工姓名列名
    pub process_vector: String,          // 工艺参数向量列名
    pub occurrence_count: String,        // 出现次数列名
    pub distinct_vector_count: String,   // 不同向量数量列名
    pub purity: String,                  // 纯度列名
}
