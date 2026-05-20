use serde::Deserialize;

// 输出数据库表中各指标列的列名配置
#[derive(Clone, Debug, Deserialize)]
pub struct OutputMetricColumns {
    // 物料品号列的列名
    pub material_code: String,
    // 工艺参数向量列的列名
    pub process_vector: String,
    // 出现次数列的列名
    pub occurrence_count: String,
    // 不同向量数量列的列名
    pub distinct_vector_count: String,
    // 纯度列的列名
    pub purity: String,
    // 定量 IsOK 百分比列的列名
    pub quant_isok_pct: String,
    // 定性 IsOK 百分比列的列名
    pub qual_isok_pct: String,
}
