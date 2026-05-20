// 操作工分离的统计记录
// 描述单个操作工在特定工艺参数向量下的统计结果
#[derive(Clone, Debug)]
pub struct OperatorPuarationStat {
    pub operator_name: String,          // 操作工姓名
    pub vector_values: Vec<String>,     // 工艺参数向量的各维度值
    pub occurrence_count: usize,         // 该操作工使用该向量的总次数
    pub distinct_vector_count: usize,   // 该操作工使用的不同向量种类数
    pub purity: f64,                    // 纯度 = 该向量出现次数 / 该操作工的不同向量数
}
