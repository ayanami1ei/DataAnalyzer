use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OperatorVectorFieldConfig {
    pub aliases: Vec<String>,
    pub output_column: String,
}
