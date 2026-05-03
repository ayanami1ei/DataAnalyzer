use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct MaterialProductionStat {
    pub material_code: String,
    pub production_count: usize,
}
