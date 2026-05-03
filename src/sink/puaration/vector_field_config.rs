use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct VectorFieldConfig {
    pub aliases: Vec<String>,
    pub output_column: String,
}
