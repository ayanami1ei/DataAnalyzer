// 刷新逻辑：实现 SQLite 数据汇的异步和同步缓冲区刷新机制
use std::sync::atomic::Ordering;

use crate::{
    error::Error,
    sink::database::sqlite_writer,
};

use super::SqliteDatabaseSink;

impl SqliteDatabaseSink {
    // 尝试异步刷新缓冲区：仅当未在刷新时执行，将缓冲区数据批量写入数据库
    pub(super) async fn try_flush(&self) {
        // 如果已在刷新中则直接返回，防止并发刷新
        if self.flushing.swap(true, Ordering::Acquire) {
            return;
        }

        // 取出缓冲区数据并清空，准备写入数据库
        let (ordered_columns, table_name, rows) = {
            let state = self.state.lock().unwrap();
            if !self.configured.load(Ordering::Acquire) {
                self.flushing.store(false, Ordering::Release);
                return;
            }
            let mut buffer = self.buffer.lock().unwrap();
            if buffer.is_empty() {
                self.flushing.store(false, Ordering::Release);
                return;
            }
            let ordered_columns = state.ordered_columns.clone();
            let table_name = state.table_name.clone();
            let rows: Vec<Vec<String>> = buffer
                .iter()
                .map(|row| {
                    Self::row_values_for_columns(&ordered_columns, row)
                })
                .collect();
            buffer.clear();
            (ordered_columns, table_name, rows)
        };

        // 获取数据库连接，在独立线程中执行批量写入以避免阻塞异步运行时
        let maybe_conn = self.conn.lock().unwrap().take();
        if let Some(mut conn) = maybe_conn {
            let result = tokio::task::spawn_blocking(move || {
                sqlite_writer::write_batch_all_text(
                    &mut conn,
                    &table_name,
                    &ordered_columns,
                    &rows,
                )?;
                Ok::<_, Error>(conn)
            })
            .await;

            // 处理写入结果：成功则归还连接，失败则记录警告
            match result {
                Ok(Ok(c)) => {
                    *self.conn.lock().unwrap() = Some(c);
                }
                Ok(Err(e)) => {
                    tracing::warn!("sqlite flush failed: {}", e);
                }
                Err(e) => {
                    tracing::warn!(
                        "spawn_blocking panicked: {}",
                        e
                    );
                }
            }
        }

        // 标记刷新结束
        self.flushing.store(false, Ordering::Release);
    }

    // 收尾操作：同步刷新缓冲区中的所有残留数据
    pub fn finish(&self) -> Result<(), Error> {
        self.flush_pending_buffer_sync()
    }

    // 同步刷新缓冲区：直接在当前线程中执行批量写入
    pub(super) fn flush_pending_buffer_sync(&self) -> Result<(), Error> {
        // 未配置时直接返回
        if !self.configured.load(Ordering::Acquire) {
            return Ok(());
        }

        // 取出缓冲区数据并清空
        let (ordered_columns, table_name, rows) = {
            let state = self.state.lock().unwrap();
            let mut buffer = self.buffer.lock().unwrap();
            if buffer.is_empty() {
                return Ok(());
            }
            let ordered_columns = state.ordered_columns.clone();
            let table_name = state.table_name.clone();
            let rows: Vec<Vec<String>> = buffer
                .iter()
                .map(|row| {
                    Self::row_values_for_columns(&ordered_columns, row)
                })
                .collect();
            buffer.clear();
            (ordered_columns, table_name, rows)
        };

        // 获取数据库连接并执行批量写入
        let mut conn_guard = self.conn.lock().unwrap();
        let Some(conn) = conn_guard.as_mut() else {
            return Err(Error::SourceDataFileError(
                "sqlite sink is not configured".to_string(),
            ));
        };

        sqlite_writer::write_batch_all_text(
            conn,
            &table_name,
            &ordered_columns,
            &rows,
        )?;
        Ok(())
    }
}
