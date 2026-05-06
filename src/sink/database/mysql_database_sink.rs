use crate::{
    data::data::Data, error::Error, progress_table::ProgressTable, sink::data_sink::DataSink,
};
use mysql::{OptsBuilder, Params, Pool, PooledConn, Value, prelude::Queryable};
use serde::Deserialize;

use super::database_row_mapping::DatabaseRowMapping;

const DEFAULT_DB_BUFFER_CAPACITY: usize = 3000;
const DEFAULT_DB_FLUSH_THRESHOLD: usize = 2500;
const DEFAULT_DB_PRINT_EVERY: u64 = 100;

#[derive(Debug, Deserialize)]
struct DatabaseConfigFile {
    host: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db_name: Option<String>,
    table_name: Option<String>,
    columns: Option<Vec<String>>,
    buffer_capacity: Option<usize>,
    flush_threshold: Option<usize>,
    progress_print_every: Option<u64>,
}

struct DbRuntimeConfig {
    host: String,
    user: String,
    password: String,
    db_name: String,
    table_name: String,
    ordered_columns: Vec<String>,
    buffer_capacity: usize,
    flush_threshold: usize,
    progress_print_every: u64,
}

pub struct MysqlDatabaseSink {
    next_sink: Box<dyn DataSink>,
    pool: Pool,
    table_name: String,
    ordered_columns: Vec<String>,
    flush_threshold: usize,
    progress: ProgressTable,
    pending_buffer: Vec<Data>,
}

impl MysqlDatabaseSink {
    fn load_db_runtime_config(config_path: &str) -> Result<DbRuntimeConfig, Error> {
        let text = std::fs::read_to_string(config_path)?;
        let cfg: DatabaseConfigFile = serde_json::from_str(&text)?;

        let host = cfg.host.clone().ok_or_else(|| {
            Error::SourceDataFileError("missing database.host in config".to_string())
        })?;
        let user = cfg.user.clone().ok_or_else(|| {
            Error::SourceDataFileError("missing database.user in config".to_string())
        })?;
        let password = cfg.password.clone().ok_or_else(|| {
            Error::SourceDataFileError("missing database.password in config".to_string())
        })?;
        let db_name = cfg.db_name.clone().ok_or_else(|| {
            Error::SourceDataFileError("missing database.db_name in config".to_string())
        })?;
        let table_name = cfg
            .table_name
            .clone()
            .unwrap_or_else(|| "work_record".to_string());

        let ordered_columns = cfg
            .columns
            .clone()
            .filter(|x| !x.is_empty())
            .ok_or_else(|| {
                Error::SourceDataFileError(
                    "missing or empty database.columns in config".to_string(),
                )
            })?;

        if ordered_columns.is_empty() {
            return Err(Error::SourceDataFileError(
                "database.columns is empty in config".to_string(),
            ));
        }

        let buffer_capacity = cfg.buffer_capacity.unwrap_or(DEFAULT_DB_BUFFER_CAPACITY);
        let flush_threshold = cfg.flush_threshold.unwrap_or(DEFAULT_DB_FLUSH_THRESHOLD);
        let progress_print_every = cfg.progress_print_every.unwrap_or(DEFAULT_DB_PRINT_EVERY);

        Ok(DbRuntimeConfig {
            host,
            user,
            password,
            db_name,
            table_name,
            ordered_columns,
            buffer_capacity,
            flush_threshold,
            progress_print_every,
        })
    }

    pub fn new(
        next_sink: Box<dyn DataSink>,
        config_path: &str,
    ) -> Result<MysqlDatabaseSink, Error> {
        let runtime_cfg = Self::load_db_runtime_config(config_path)?;

        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(runtime_cfg.host.clone()))
            .user(Some(runtime_cfg.user.clone()))
            .pass(Some(runtime_cfg.password.clone()))
            .db_name(Some(runtime_cfg.db_name.clone()));
        let pool = Pool::new(opts)?;

        Ok(MysqlDatabaseSink {
            pool,
            next_sink,
            table_name: runtime_cfg.table_name,
            ordered_columns: runtime_cfg.ordered_columns,
            flush_threshold: runtime_cfg.flush_threshold,
            progress: ProgressTable::new(runtime_cfg.progress_print_every),
            pending_buffer: Vec::with_capacity(runtime_cfg.buffer_capacity),
        })
    }

    fn quote_ident(ident: &str) -> String {
        format!("`{}`", ident.replace('`', "``"))
    }

    fn ensure_table_exists(&self, conn: &mut PooledConn, table_name: &str) -> Result<(), Error> {
        let mut defs = Vec::with_capacity(self.ordered_columns.len());
        for col in &self.ordered_columns {
            defs.push(format!("{} TEXT NULL", Self::quote_ident(col)));
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (id BIGINT PRIMARY KEY AUTO_INCREMENT, {})",
            Self::quote_ident(table_name),
            defs.join(", ")
        );
        conn.query_drop(sql)?;
        Ok(())
    }

    pub fn insert<T: DatabaseRowMapping>(&mut self, data: &mut T) -> Result<(), Error> {
        let map = data.column_index_value_map();
        if map.is_empty() {
            return Ok(());
        }

        data.check_legal();

        let mut conn = self.pool.get_conn()?;
        self.ensure_table_exists(&mut conn, data.table_name())?;

        let columns = self
            .ordered_columns
            .iter()
            .map(|col| Self::quote_ident(col))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; self.ordered_columns.len()].join(", ");
        let params: Vec<Value> = self
            .ordered_columns
            .iter()
            .map(|col| Value::from(map.get(col).cloned().unwrap_or_default()))
            .collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::quote_ident(data.table_name()),
            columns,
            placeholders,
        );

        conn.exec_drop(sql, Params::Positional(params))?;
        Ok(())
    }

    fn row_params(&self, data: &Data) -> Vec<Value> {
        let map = data.column_index_value_map();
        self.ordered_columns
            .iter()
            .map(|col| Value::from(map.get(col).cloned().unwrap_or_default()))
            .collect()
    }

    fn flush_pending_buffer(&mut self) -> Result<(), Error> {
        if self.pending_buffer.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get_conn()?;
        self.ensure_table_exists(&mut conn, &self.table_name)?;

        let columns = self
            .ordered_columns
            .iter()
            .map(|col| Self::quote_ident(col))
            .collect::<Vec<_>>()
            .join(", ");

        let row_placeholders = format!("({})", vec!["?"; self.ordered_columns.len()].join(", "));
        let values_clause = vec![row_placeholders; self.pending_buffer.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            Self::quote_ident(&self.table_name),
            columns,
            values_clause
        );

        let mut all_params: Vec<Value> =
            Vec::with_capacity(self.pending_buffer.len() * self.ordered_columns.len());
        for row in &self.pending_buffer {
            all_params.extend(self.row_params(row));
        }

        conn.exec_drop(sql, Params::Positional(all_params))?;
        self.pending_buffer.clear();
        Ok(())
    }
}

impl DataSink for MysqlDatabaseSink {
    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        self.pending_buffer.push(data.clone());

        let mut result = self.next_sink.sink(data);
        if result.is_ok() && self.pending_buffer.len() >= self.flush_threshold {
            result = self.flush_pending_buffer();
        }

        match result {
            Ok(()) => {
                self.progress.record_success();
                if self.progress.should_print() {
                    self.progress.print_table();
                }
                Ok(())
            }
            Err(e) => {
                self.progress.record_failed();
                if self.progress.should_print() {
                    self.progress.print_table();
                }
                Err(e)
            }
        }
    }

    fn get_next_sink(&self) -> Result<Option<Box<dyn DataSink>>, Error> {
        Ok(None)
    }
}

impl Drop for MysqlDatabaseSink {
    fn drop(&mut self) {
        if let Err(e) = self.flush_pending_buffer() {
            eprintln!("[database] flush pending buffer on drop failed: {}", e);
        }
        self.progress.print_final();
    }
}
