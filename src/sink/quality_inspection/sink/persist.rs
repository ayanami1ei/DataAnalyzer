use rusqlite::types::Value as SqlValue;

use crate::{
    config, error::Error,
    sink::database::sqlite_writer::{self, SqliteDbConfig},
};

use super::{QualityInspectionSink, QUALITY_DB_CONFIG_PATH};

impl QualityInspectionSink {
    /// 加载数据库配置，若配置文件不存在则使用从字段配置中导出的默认值
    fn load_db_config() -> Result<SqliteDbConfig, Error> {
        let qi = config::quality_inspection_fields();
        if std::path::Path::new(QUALITY_DB_CONFIG_PATH).exists() {
            let text = std::fs::read_to_string(QUALITY_DB_CONFIG_PATH)?;
            Ok(serde_json::from_str::<SqliteDbConfig>(&text).unwrap_or_default())
        } else {
            Ok(SqliteDbConfig {
                sqlite_db_path: Some("data/quality_inspection.db".to_string()),
                table_name: Some("quality_inspection".to_string()),
                columns: Some(qi.detail_columns.clone()),
            })
        }
    }

    /// 根据列名返回对应的 SQLite 列类型定义
    fn col_def_for_name(name: &str) -> sqlite_writer::ColumnDef {
        if name.contains("IsOK") {
            sqlite_writer::ColumnDef::integer(name)
        } else if name.contains("下界") || name.contains("上界") {
            sqlite_writer::ColumnDef::real(name)
        } else {
            sqlite_writer::ColumnDef::text(name)
        }
    }

    /// 在事务中将明细数据逐行写入主表
    fn write_detail_data(
        tx: &rusqlite::Transaction<'_>,
        table_name: &str,
        columns: &[String],
        detail_rows: &[Vec<SqlValue>],
    ) -> Result<(), Error> {
        let insert_sql = sqlite_writer::build_insert_sql(table_name, columns);
        let mut stmt = tx.prepare(&insert_sql)?;
        for row in detail_rows {
            stmt.execute(rusqlite::params_from_iter(row.iter()))?;
        }
        Ok(())
    }

    /// 在事务中将 IsOK 不一致的错误记录写入 `{表名}_isok_errors` 附表
    fn write_error_data(
        tx: &rusqlite::Transaction<'_>,
        table_name: &str,
        error_rows: &[Vec<SqlValue>],
    ) -> Result<(), Error> {
        let qi = config::quality_inspection_fields();
        let err_table = format!("{}_isok_errors", table_name);
        let err_columns = qi.error_columns.clone();
        let err_insert_sql = sqlite_writer::build_insert_sql(&err_table, &err_columns);
        let mut err_stmt = tx.prepare(&err_insert_sql)?;
        for row in error_rows {
            err_stmt.execute(rusqlite::params_from_iter(row.iter()))?;
        }
        Ok(())
    }

    /// 将明细数据和错误数据持久化到 SQLite 数据库
    pub(super) fn persist_database(
        detail_rows: &[Vec<SqlValue>],
        error_rows: &[Vec<SqlValue>],
    ) -> Result<(), Error> {
        let db_cfg = Self::load_db_config()?;
        let sqlite_db_path = db_cfg.sqlite_db_path
            .unwrap_or_else(|| "data/quality_inspection.db".to_string());
        let table_name = db_cfg.table_name
            .unwrap_or_else(|| "quality_inspection".to_string());
        let columns = db_cfg.columns.filter(|x| !x.is_empty()).unwrap_or_default();
        if columns.is_empty() { return Ok(()); }

        sqlite_writer::prepare_directory(&sqlite_db_path)?;
        let mut conn = sqlite_writer::open_sqlite(
            &sqlite_db_path,
            &[("synchronous", "OFF"), ("journal_mode", "MEMORY")],
        )?;

        let col_defs: Vec<sqlite_writer::ColumnDef> = columns.iter()
            .map(|c| Self::col_def_for_name(c)).collect();
        sqlite_writer::create_table(&conn, &table_name, &col_defs)?;

        let has_errors = !error_rows.is_empty();
        if has_errors {
            let qi = config::quality_inspection_fields();
            let err_table = format!("{}_isok_errors", table_name);
            let err_col_defs: Vec<sqlite_writer::ColumnDef> = qi.error_columns.iter()
                .map(|c| Self::col_def_for_name(c)).collect();
            sqlite_writer::create_table(&conn, &err_table, &err_col_defs)?;
        }

        let tx = conn.transaction()?;
        Self::write_detail_data(&tx, &table_name, &columns, detail_rows)?;
        if has_errors { Self::write_error_data(&tx, &table_name, error_rows)?; }
        tx.commit()?;

        tracing::info!("quality: wrote {} detail rows to {}", detail_rows.len(), sqlite_db_path);
        Ok(())
    }
}
