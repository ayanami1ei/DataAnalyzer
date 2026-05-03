use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OutputMetricColumns {
    pub material_code: String,
    pub process_vector: String,
    pub occurrence_count: String,
    pub distinct_vector_count: String,
    pub purity: String,
}
