use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct StatsFlowConfig {
    pub sink_order: Vec<String>,
}

impl Default for StatsFlowConfig {
    fn default() -> Self {
        Self {
            sink_order: vec![
                "puaration".to_string(),
                "operator_puaration".to_string(),
                "material_production".to_string(),
            ],
        }
    }
}
