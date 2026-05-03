use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct OperatorPuarationStat {
    pub operator_name: String,
    pub process_vector: String,
    pub occurrence_count: usize,
    pub distinct_vector_count: usize,
    pub purity: f64,
}
