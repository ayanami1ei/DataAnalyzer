use serde::Serialize;

/// 单个 IsOK 不一致的错误记录
#[derive(Debug, Clone, Serialize)]
pub struct IsokError {
    /// 所属批号
    pub batch_no: String,
    /// 物料品名
    pub material_code: String,
    /// 检查项目名称
    pub check_item: String,
    /// 原始检验参数字符串
    pub param_raw: String,
    /// 实际检验值
    pub test_value: String,
    /// 根据标准重新计算应得的 IsOK 结果
    pub expected_isok: bool,
    /// 原始数据中记录的 IsOK 结果
    pub actual_isok: bool,
}

/// 单条检验明细记录
#[derive(Debug, Clone, Serialize)]
pub struct QualityItem {
    /// 所属批号
    pub batch_no: String,
    /// 物料品名
    pub material_code: String,
    /// 检查项目名称
    pub check_item: String,
    /// 原始检验参数表达式，如 "0.5≤V≤1.0"
    pub param_raw: String,
    /// 参数类型中文描述："定量" 或 "定性"
    pub param_type: String,
    /// 定量下界，无下界时为 None
    pub lower_bound: Option<f64>,
    /// 定量上界，无上界时为 None
    pub upper_bound: Option<f64>,
    /// 实际检验值字符串
    pub test_value: String,
    /// 原始数据中记录的 IsOK 值
    pub original_isok: bool,
    /// 根据检验参数标准重新计算的正确 IsOK 值
    pub correct_isok: bool,
}

/// 单个批次的质检汇总结果
#[derive(Debug, Clone, Serialize)]
pub struct BatchQuality {
    /// 批号
    pub batch_no: String,
    /// 物料品名
    pub material_code: String,
    /// 该批次下所有检验明细
    pub items: Vec<QualityItem>,
    /// 定量检验总次数
    pub quant_total: usize,
    /// 定量检验通过次数
    pub quant_ok: usize,
    /// 定性检验总次数
    pub qual_total: usize,
    /// 定性检验通过次数
    pub qual_ok: usize,
    /// 定量检验通过百分比
    pub quant_isok_pct: f64,
    /// 定性检验通过百分比
    pub qual_isok_pct: f64,
    /// 该批次中 IsOK 不一致的错误列表
    pub errors: Vec<IsokError>,
}
