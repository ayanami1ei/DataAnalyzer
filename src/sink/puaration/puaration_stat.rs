use crate::config;

// 单条工艺参数汇聚统计结果
#[derive(Clone, Debug)]
pub struct PuarationStat {
    // 物料品号
    pub material_code: String,
    // 工艺参数向量签名（由各字段值按 "|" 拼接而成）
    pub process_vector: String,
    // 该向量出现的总次数
    pub occurrence_count: usize,
    // 该物料下不同向量的总数
    pub distinct_vector_count: usize,
    // 纯度 = occurrence_count / distinct_vector_count
    pub purity: f64,
    // 定量 IsOK 百分比（平均值）
    pub quant_isok_pct: f64,
    // 定性 IsOK 百分比（平均值）
    pub qual_isok_pct: f64,
}

// 每个物料下得分最高的前 N 个工艺参数向量
#[derive(Clone, Debug)]
pub struct TopRankedVector {
    pub material_code: String,
    pub process_vector: String,
    pub occurrence_count: usize,
    pub distinct_vector_count: usize,
    pub purity: f64,
    pub quant_isok_pct: f64,
    pub qual_isok_pct: f64,
    // 根据 purity、quant_isok_pct、qual_isok_pct 计算的综合得分
    pub composite_score: f64,
    // 在所属物料中的排名（从 1 开始）
    pub rank: usize,
}

impl TopRankedVector {
    // 从 PuarationStat 构造排名向量，计算综合得分并记录排名
    pub fn from_stat(stat: &PuarationStat, rank: usize) -> Self {
        let composite = stat.composite_score();
        Self {
            material_code: stat.material_code.clone(),
            process_vector: stat.process_vector.clone(),
            occurrence_count: stat.occurrence_count,
            distinct_vector_count: stat.distinct_vector_count,
            purity: stat.purity,
            quant_isok_pct: stat.quant_isok_pct,
            qual_isok_pct: stat.qual_isok_pct,
            composite_score: composite,
            rank,
        }
    }
}

impl PuarationStat {
    // 计算纯度、定量 IsOK%、定性 IsOK% 三者的算术平均作为综合得分
    pub fn composite_score(&self) -> f64 {
        let sc = config::score_constants();
        let p = self.purity.min(sc.purity_cap);
        let qn = (self.quant_isok_pct / sc.pct_divisor).clamp(0.0, 1.0);
        let ql = (self.qual_isok_pct / sc.pct_divisor).clamp(0.0, 1.0);
        (p + qn + ql) / sc.score_divisor
    }
}
