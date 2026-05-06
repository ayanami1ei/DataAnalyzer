use crate::{
    data::data::Data, error::Error, progress_table::ProgressTable, sink::data_sink::DataSink,
};
use rusqlite::{Connection, params_from_iter};
use serde::Deserialize;

const DEFAULT_DB_BUFFER_CAPACITY: usize = 3000;
const DEFAULT_DB_FLUSH_THRESHOLD: usize = 2500;
const DEFAULT_DB_PRINT_EVERY: u64 = 100;
const DEFAULT_SQLITE_PATH: &str = "data/work_record.db";

#[derive(Debug, Deserialize)]
struct DatabaseConfigFile {
    sqlite_db_path: Option<String>,
    table_name: Option<String>,
    invalid_table_name: Option<String>,
    columns: Option<Vec<String>>,
    buffer_capacity: Option<usize>,
    flush_threshold: Option<usize>,
    progress_print_every: Option<u64>,
}

struct DbRuntimeConfig {
    sqlite_db_path: String,
    valid_table_name: String,
    invalid_table_name: String,
    ordered_columns: Vec<String>,
    buffer_capacity: usize,
    flush_threshold: usize,
    progress_print_every: u64,
}

pub struct SqliteDatabaseSink {
    next_sink: Box<dyn DataSink>,
    conn: Connection,
    valid_table_name: String,
    invalid_table_name: String,
    ordered_columns: Vec<String>,
    flush_threshold: usize,
    progress: ProgressTable,
    valid_pending_buffer: Vec<Data>,
    invalid_pending_buffer: Vec<Data>,
}

impl SqliteDatabaseSink {
    fn load_db_runtime_config(config_path: &str) -> Result<DbRuntimeConfig, Error> {
        let text = std::fs::read_to_string(config_path)?;
        let cfg: DatabaseConfigFile = serde_json::from_str(&text)?;

        let valid_table_name = cfg
            .table_name
            .clone()
            .unwrap_or_else(|| "work_record".to_string());
        let invalid_table_name = cfg
            .invalid_table_name
            .clone()
            .unwrap_or_else(|| format!("{}_invalid", valid_table_name));

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

        let sqlite_db_path = cfg
            .sqlite_db_path
            .clone()
            .unwrap_or_else(|| DEFAULT_SQLITE_PATH.to_string());
        let buffer_capacity = cfg.buffer_capacity.unwrap_or(DEFAULT_DB_BUFFER_CAPACITY);
        let flush_threshold = cfg.flush_threshold.unwrap_or(DEFAULT_DB_FLUSH_THRESHOLD);
        let progress_print_every = cfg.progress_print_every.unwrap_or(DEFAULT_DB_PRINT_EVERY);

        Ok(DbRuntimeConfig {
            sqlite_db_path,
            valid_table_name,
            invalid_table_name,
            ordered_columns,
            buffer_capacity,
            flush_threshold,
            progress_print_every,
        })
    }

    pub fn new(
        next_sink: Box<dyn DataSink>,
        config_path: &str,
    ) -> Result<SqliteDatabaseSink, Error> {
        let runtime_cfg = Self::load_db_runtime_config(config_path)?;

        let db_path = std::path::Path::new(&runtime_cfg.sqlite_db_path);
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(&runtime_cfg.sqlite_db_path)?;

        let db = SqliteDatabaseSink {
            next_sink,
            conn,
            valid_table_name: runtime_cfg.valid_table_name,
            invalid_table_name: runtime_cfg.invalid_table_name,
            ordered_columns: runtime_cfg.ordered_columns,
            flush_threshold: runtime_cfg.flush_threshold,
            progress: ProgressTable::new(runtime_cfg.progress_print_every),
            valid_pending_buffer: Vec::with_capacity(runtime_cfg.buffer_capacity),
            invalid_pending_buffer: Vec::with_capacity(runtime_cfg.buffer_capacity),
        };

        db.ensure_table_exists(&db.valid_table_name)?;
        db.ensure_table_exists(&db.invalid_table_name)?;
        Ok(db)
    }

    fn quote_ident(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn ensure_table_exists(&self, table_name: &str) -> Result<(), Error> {
        let mut defs = Vec::with_capacity(self.ordered_columns.len());
        for col in &self.ordered_columns {
            defs.push(format!("{} TEXT NULL", Self::quote_ident(col)));
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, {})",
            Self::quote_ident(table_name),
            defs.join(", ")
        );
        self.conn.execute_batch(&sql)?;
        Ok(())
    }

    fn row_values_for_columns(ordered_columns: &[String], data: &Data) -> Vec<String> {
        let map = data.column_index_value_map();
        ordered_columns
            .iter()
            .map(|col| map.get(col).cloned().unwrap_or_default())
            .collect()
    }

    fn flush_buffer_to_table(
        &mut self,
        table_name: &str,
        rows: &[Vec<String>],
    ) -> Result<(), Error> {
        if rows.is_empty() {
            return Ok(());
        }

        self.ensure_table_exists(table_name)?;

        let columns = self
            .ordered_columns
            .iter()
            .map(|col| Self::quote_ident(col))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; self.ordered_columns.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::quote_ident(table_name),
            columns,
            placeholders
        );

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(&sql)?;
            for row_values in rows {
                stmt.execute(params_from_iter(row_values.iter()))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn flush_pending_buffer(&mut self) -> Result<(), Error> {
        if self.valid_pending_buffer.is_empty() && self.invalid_pending_buffer.is_empty() {
            return Ok(());
        }

        let ordered_columns = self.ordered_columns.clone();
        let valid_rows: Vec<Vec<String>> = self
            .valid_pending_buffer
            .iter()
            .map(|row| Self::row_values_for_columns(&ordered_columns, row))
            .collect();
        let invalid_rows: Vec<Vec<String>> = self
            .invalid_pending_buffer
            .iter()
            .map(|row| Self::row_values_for_columns(&ordered_columns, row))
            .collect();

        let valid_table = self.valid_table_name.clone();
        let invalid_table = self.invalid_table_name.clone();
        self.flush_buffer_to_table(&valid_table, &valid_rows)?;
        self.flush_buffer_to_table(&invalid_table, &invalid_rows)?;

        self.valid_pending_buffer.clear();
        self.invalid_pending_buffer.clear();
        Ok(())
    }
}

impl DataSink for SqliteDatabaseSink {
    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        if data.is_valid() {
            self.valid_pending_buffer.push(data.clone());
        } else {
            self.invalid_pending_buffer.push(data.clone());
        }

        let mut result = self.next_sink.sink(data);
        let pending_total = self.valid_pending_buffer.len() + self.invalid_pending_buffer.len();
        if result.is_ok() && pending_total >= self.flush_threshold {
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

impl Drop for SqliteDatabaseSink {
    fn drop(&mut self) {
        if let Err(e) = self.flush_pending_buffer() {
            eprintln!("[sqlite] flush pending buffer on drop failed: {}", e);
        }
        self.progress.print_final();
    }
}
