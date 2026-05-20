// SQLite 数据汇核心实现：提供数据的缓冲、批量写入、配置加载及生命周期管理
pub mod config;
mod flush;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex, Weak,
};
use async_trait::async_trait;
use rusqlite::Connection;

use crate::{
    data::data::Data,
    error::Error,
    router::Router,
    sink::database::sqlite_writer,
    sink::data_sink::DataSink,
};

pub use config::DbConfig;

// SQLite 运行时状态：记录当前使用的表名、列顺序和配置文件路径
struct SqliteState {
    // 数据库表名
    table_name: String,
    // 有序的列名列表，决定写入时各字段的排列顺序
    ordered_columns: Vec<String>,
    // 配置文件路径
    config_path: String,
}

// SQLite 数据库数据汇：管理数据库连接、数据缓冲、批量刷新和路由分发
pub struct SqliteDatabaseSink {
    // 数据库连接（可选），未配置时为 None
    conn: Mutex<Option<Connection>>,
    // 运行时状态，包括表名、列顺序和配置路径
    state: Mutex<SqliteState>,
    // 数据缓冲区，暂存待写入的数据对象
    buffer: Mutex<Vec<Data>>,
    // 触发刷新的缓冲区大小阈值
    flush_threshold: AtomicUsize,
    // 是否已成功配置
    configured: AtomicBool,
    // 是否正在执行刷新操作
    flushing: AtomicBool,
    router: Mutex<Option<Weak<Router>>>,
}

impl SqliteDatabaseSink {
    // 应用数据库配置：创建目录、打开连接、设置 PRAGMA、建表、更新状态
    fn apply_config(
        &self,
        cfg: &DbConfig,
        table_name: &str,
    ) -> Result<(), Error> {
        // 确保数据库文件所在目录存在
        let db_path = Path::new(&cfg.sqlite_db_path);
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // 打开数据库连接并设置性能优化 PRAGMA
        let conn = Connection::open(&cfg.sqlite_db_path)?;
        conn.execute_batch("PRAGMA synchronous = OFF")?;
        conn.execute_batch("PRAGMA journal_mode = MEMORY")?;
        // 创建数据表（所有列均为 TEXT 类型）
        sqlite_writer::create_table_all_text(
            &conn,
            table_name,
            &cfg.ordered_columns,
        )?;

        // 更新运行时状态
        *self.state.lock().unwrap() = SqliteState {
            table_name: table_name.to_string(),
            ordered_columns: cfg.ordered_columns.clone(),
            config_path: String::new(),
        };
        self.flush_threshold
            .store(cfg.flush_threshold, Ordering::Release);
        *self.conn.lock().unwrap() = Some(conn);
        self.configured.store(true, Ordering::Release);
        Ok(())
    }

    // 配置数据汇：校验配置路径、刷新已有缓冲区、加载配置并应用
    pub fn configure(
        &self,
        config_path: &str,
        table_name: &str,
    ) -> Result<(), Error> {
        // 校验配置路径不为空
        if config_path.trim().is_empty() {
            return Err(Error::SourceDataFileError(
                "sqlite config path is empty".to_string(),
            ));
        }
        // 校验配置文件存在
        if !Path::new(config_path).exists() {
            return Err(Error::SourceDataFileError(format!(
                "sqlite config file not found: {}",
                config_path
            )));
        }

        // 先刷新当前缓冲区中的数据，再加载新配置
        self.flush_pending_buffer_sync()?;
        let cfg = Self::load_config(config_path)?;
        self.apply_config(&cfg, table_name)?;
        self.state.lock().unwrap().config_path =
            config_path.to_string();
        Ok(())
    }

    // 根据列顺序从数据对象中提取对应值，按顺序返回字符串向量
    fn row_values_for_columns(
        ordered_columns: &[String],
        data: &Data,
    ) -> Vec<String> {
        let map = data.column_index_value_map();
        ordered_columns
            .iter()
            .map(|col| map.get(col).cloned().unwrap_or_default())
            .collect()
    }

    // 异步处理数据：将数据加入缓冲区，超过阈值时触发刷新，标记流程状态并路由到下游
    pub async fn sink_impl(
        &self,
        data: Arc<Mutex<Data>>,
        flow_state_label: &str,
    ) -> Result<(), Error> {
        let data_handle = Arc::clone(&data);

        // 将数据克隆后加入缓冲区，并检查是否需要触发刷新
        let need_flush = {
            let mut buffer = self.buffer.lock().unwrap();
            if !self.configured.load(Ordering::Acquire) {
                return Err(Error::SourceDataFileError(
                    "sqlite sink is not configured".to_string(),
                ));
            }
            let data_guard = data.lock().map_err(|_| {
                Error::PipelineError("data mutex poisoned".into())
            })?;
            buffer.push(data_guard.clone());
            drop(data_guard);
            buffer.len()
                >= self.flush_threshold.load(Ordering::Acquire)
        };

        // 达到阈值则异步刷新缓冲区
        if need_flush {
            self.try_flush().await;
        }

        // 标记数据的流程状态
        {
            let mut data_guard = data_handle.lock().map_err(|_| {
                Error::PipelineError("data mutex poisoned".into())
            })?;
            data_guard.add_flow_state(flow_state_label.to_string());
        }

        let router = self.router.lock().unwrap().as_ref().and_then(|w| w.upgrade());
        if let Some(router) = router {
            router.route_data(data_handle).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl DataSink for SqliteDatabaseSink {
    fn new(config_path: &str) -> Self {
        Self::new(config_path)
    }

    async fn sink(
        &self,
        _data: Arc<Mutex<Data>>,
    ) -> Result<(), Error> {
        Err(Error::PipelineError(
            "SqliteDatabaseSink is not a direct sink; \
             use ValidSqliteSink or InvalidSqliteSink"
                .into(),
        ))
    }

    fn set_router(&self, router: Weak<Router>) {
        *self.router.lock().unwrap() = Some(router);
    }

    fn finish(&self) -> Result<(), Error> {
        self.flush_pending_buffer_sync()
    }
}

impl SqliteDatabaseSink {
    pub fn new(_config_path: &str) -> Self {
        Self {
            conn: Mutex::new(None),
            state: Mutex::new(SqliteState {
                table_name: String::new(),
                ordered_columns: Vec::new(),
                config_path: String::new(),
            }),
            buffer: Mutex::new(Vec::with_capacity(
                crate::config::pipeline_constants().db_buffer_capacity,
            )),
            flush_threshold: AtomicUsize::new(
                crate::config::pipeline_constants().db_flush_threshold,
            ),
            configured: AtomicBool::new(false),
            flushing: AtomicBool::new(false),
            router: Mutex::new(None),
        }
    }
}

impl Drop for SqliteDatabaseSink {
    // 析构时刷新缓冲区中残留的数据
    fn drop(&mut self) {
        if !self.configured.load(Ordering::Acquire) {
            return;
        }
        if let Err(e) = self.flush_pending_buffer_sync() {
            eprintln!("[sqlite] flush pending buffer on drop failed: {}", e);
        }
    }
}
