#[derive(Clone, Debug)]
pub struct PuarationStat {
    pub material_code: String,
    pub process_vector: String,
    pub occurrence_count: usize,
    pub distinct_vector_count: usize,
    pub purity: f64,
}
