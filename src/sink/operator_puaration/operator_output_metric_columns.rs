use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OperatorOutputMetricColumns {
    pub operator_name: String,
    pub process_vector: String,
    pub occurrence_count: String,
    pub distinct_vector_count: String,
    pub purity: String,
}
