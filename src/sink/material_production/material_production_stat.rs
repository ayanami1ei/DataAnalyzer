use serde::Serialize;

// 物料生产统计记录：记录单个物料的总生产次数
#[derive(Clone, Debug, Serialize)]
pub struct MaterialProductionStat {
    // 物料品号
    pub material_code: String,
    // 该物料的生产次数（批号去重后的数量）
    pub production_count: usize,
}
