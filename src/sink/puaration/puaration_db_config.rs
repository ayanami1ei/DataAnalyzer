use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PuarationDbConfig {
    pub sqlite_db_path: Option<String>,
    pub table_name: Option<String>,
    pub columns: Option<Vec<String>>,
}
