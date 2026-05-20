use std::collections::HashMap;

use crate::config;
use crate::error::Error;
use crate::sink::database::sqlite_writer::{self, SqliteDbConfig};
use crate::sink::puaration::puaration_sink::PuarationSink;
use crate::sink::quality_inspection::quality_stat::BatchQuality;

impl PuarationSink {
    // 将每个物料下综合得分前 5 的工艺向量排名写入 SQLite 数据库
    pub(crate) fn persist_top_n_to_database(
        &self,
        quality_results: &HashMap<String, BatchQuality>,
    ) -> Result<(), Error> {
        let pc = config::pipeline_constants();
        let top_n = self.top_vectors_per_material(pc.top_n_count, quality_results);
        if top_n.is_empty() {
            return Ok(());
        }

        // 数据库配置文件不存在时跳过
        if !std::path::Path::new(&self.config.database_config_path).exists() {
            return Ok(());
        }

        // 读取数据库配置，确定 SQLite 文件路径
        let db_cfg_text = std::fs::read_to_string(&self.config.database_config_path)?;
        let db_cfg: SqliteDbConfig = serde_json::from_str(&db_cfg_text)?;
        let sqlite_db_path = db_cfg
            .sqlite_db_path
            .unwrap_or_else(|| "data/puaration_stats.db".to_string());
        let pc = config::pipeline_constants();
        let table_name = &pc.top_n_table_name;

        // 确保目录存在，打开数据库连接（关闭同步模式提升性能）
        sqlite_writer::prepare_directory(&sqlite_db_path)?;
        let mut conn = sqlite_writer::open_sqlite(
            &sqlite_db_path,
            &[("synchronous", "OFF"), ("journal_mode", "MEMORY")],
        )?;

        let columns = pc.top_n_columns.clone();
        let col_defs: Vec<sqlite_writer::ColumnDef> = columns.iter().map(|c| {
            if c.contains("IsOK") || c == "纯度" || c == "综合得分" {
                sqlite_writer::ColumnDef::real(c)
            } else if c == "出现次数" || c == "不同向量数量" || c == "排名" {
                sqlite_writer::ColumnDef::integer(c)
            } else {
                sqlite_writer::ColumnDef::text(c)
            }
        }).collect();
        sqlite_writer::create_table(&conn, table_name, &col_defs)?;

        // 将 TopRankedVector 列表转换为 SQLite 行数据
        let rows: Vec<Vec<rusqlite::types::Value>> = top_n
            .iter()
            .map(|v| {
                vec![
                    rusqlite::types::Value::Text(v.material_code.clone()),
                    rusqlite::types::Value::Text(v.process_vector.clone()),
                    rusqlite::types::Value::Integer(v.occurrence_count as i64),
                    rusqlite::types::Value::Integer(v.distinct_vector_count as i64),
                    rusqlite::types::Value::Real(v.purity),
                    rusqlite::types::Value::Real(v.quant_isok_pct),
                    rusqlite::types::Value::Real(v.qual_isok_pct),
                    rusqlite::types::Value::Real(v.composite_score),
                    rusqlite::types::Value::Integer(v.rank as i64),
                ]
            })
            .collect();

        // 批量插入排名数据
        sqlite_writer::batch_insert_values(&mut conn, table_name, &columns, &rows)?;

        Ok(())
    }
}
