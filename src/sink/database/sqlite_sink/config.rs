// 配置管理：定义 SQLite 数据汇的配置结构及加载逻辑
use serde::Deserialize;

use crate::{config, error::Error};

use super::SqliteDatabaseSink;

// 默认 SQLite 数据库文件路径
const DEFAULT_SQLITE_PATH: &str = "data/work_record.db";

// JSON 配置文件的内部结构，对应磁盘上的数据库配置文件
#[derive(Debug, Deserialize)]
struct DatabaseConfigFile {
    // SQLite 数据库文件路径
    sqlite_db_path: Option<String>,
    // 有效数据表名
    table_name: Option<String>,
    // 无效数据表名
    invalid_table_name: Option<String>,
    // 列名列表
    columns: Option<Vec<String>>,
    // 缓冲区容量
    buffer_capacity: Option<usize>,
    // 刷新阈值
    flush_threshold: Option<usize>,
}

// 数据库配置：解析并验证后的配置结构，提供默认值
pub struct DbConfig {
    // SQLite 数据库文件路径
    pub sqlite_db_path: String,
    // 有序列名列表
    pub ordered_columns: Vec<String>,
    // 缓冲区容量
    pub buffer_capacity: usize,
    // 刷新阈值
    pub flush_threshold: usize,
}

impl SqliteDatabaseSink {
    // 从 JSON 配置文件加载数据库配置，验证列名不为空并提供默认值
    pub fn load_config(config_path: &str) -> Result<DbConfig, Error> {
        // 读取并解析 JSON 配置文件
        let text = std::fs::read_to_string(config_path)?;
        let cfg: DatabaseConfigFile = serde_json::from_str(&text)?;

        // 提取列名列表，必须存在且非空
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

        let pc = config::pipeline_constants();
        Ok(DbConfig {
            sqlite_db_path: cfg
                .sqlite_db_path
                .unwrap_or_else(|| DEFAULT_SQLITE_PATH.to_string()),
            ordered_columns,
            buffer_capacity: cfg
                .buffer_capacity
                .unwrap_or(pc.db_buffer_capacity),
            flush_threshold: cfg
                .flush_threshold
                .unwrap_or(pc.db_flush_threshold),
        })
    }

    // 从配置文件中读取表名，根据 use_invalid 决定返回有效表名或无效表名
    pub fn read_table_name(
        config_path: &str,
        use_invalid: bool,
    ) -> Result<String, Error> {
        let text = std::fs::read_to_string(config_path)?;
        let cfg: DatabaseConfigFile = serde_json::from_str(&text)?;
        let valid =
            cfg.table_name.unwrap_or_else(|| "work_record".to_string());
        if use_invalid {
            Ok(cfg
                .invalid_table_name
                .unwrap_or_else(|| format!("{}_invalid", valid)))
        } else {
            Ok(valid)
        }
    }
}
