use std::collections::HashMap;

use rusqlite::types::Value as SqlValue;

use crate::{
    error::Error,
    sink::database::sqlite_writer::{self, SqliteDbConfig},
    sink::operator_puaration::operator_puaration_stat::OperatorPuarationStat,
};

use super::OperatorPuarationSink;

impl OperatorPuarationSink {
    // 将内存中的统计结果持久化到SQLite数据库
    // 步骤：读取数据库配置 -> 创建表 -> 写入统计行数据
    pub(crate) fn persist_stats_to_database(&self) -> Result<(), Error> {
        // 锁定统计映射，若为空则直接返回
        let map = self.counts_by_operator.lock().unwrap();
        if map.is_empty() {
            return Ok(());
        }

        // 从配置文件中读取数据库连接配置
        let db_cfg_text = std::fs::read_to_string(&self.config.database_config_path)?;
        let db_cfg: SqliteDbConfig = serde_json::from_str(&db_cfg_text)?;
        // 使用配置中的数据库路径和表名，缺失时使用默认值
        let sqlite_db_path = db_cfg
            .sqlite_db_path
            .unwrap_or_else(|| "data/operator_puaration_stats.db".to_string());
        let table_name = db_cfg
            .table_name
            .unwrap_or_else(|| "operator_puaration_stats".to_string());
        let columns = db_cfg
            .columns
            .filter(|x| !x.is_empty())
            .ok_or_else(|| Error::SourceDataFileError("missing database columns".to_string()))?;

        // 创建数据库目录并打开SQLite连接（关闭同步与日志以提升写入性能）
        sqlite_writer::prepare_directory(&sqlite_db_path)?;
        let mut conn = sqlite_writer::open_sqlite(
            &sqlite_db_path,
            &[("synchronous", "OFF"), ("journal_mode", "MEMORY")],
        )?;

        // 根据输出指标配置构建列定义并创建表
        let col_defs = self.build_column_defs(&columns);
        sqlite_writer::create_table(&conn, &table_name, &col_defs)?;

        // 从统计映射生成统计记录列表并构建数据库行数据
        let stats = Self::collect_stats_from_map(&map);
        let rows: Vec<Vec<SqlValue>> = stats
            .iter()
            .map(|stat| self.build_db_row_values(&columns, stat))
            .collect();
        sqlite_writer::batch_insert_values(&mut conn, &table_name, &columns, &rows)?;

        Ok(())
    }

    // 根据输出指标配置构建数据库表的列定义
    // occurrence_count 和 distinct_vector_count 映射为INTEGER
    // purity 映射为REAL，其余列映射为TEXT
    pub(crate) fn build_column_defs(
        &self,
        columns: &[String],
    ) -> Vec<sqlite_writer::ColumnDef> {
        columns
            .iter()
            .map(|col| {
                if col == &self.config.output_metrics.occurrence_count
                    || col == &self.config.output_metrics.distinct_vector_count
                {
                    sqlite_writer::ColumnDef::integer(col)
                } else if col == &self.config.output_metrics.purity {
                    sqlite_writer::ColumnDef::real(col)
                } else {
                    sqlite_writer::ColumnDef::text(col)
                }
            })
            .collect()
    }

    // 将单条OperatorPuarationStat记录转换为SQLite行值
    // 根据输出列名从stat中提取对应字段，工艺参数向量字段从vector_fields配置匹配
    pub(crate) fn build_db_row_values(
        &self,
        columns: &[String],
        stat: &OperatorPuarationStat,
    ) -> Vec<SqlValue> {
        // 构建工艺向量字段名到值的映射，用于后续查找
        let mut vector_output_map = HashMap::new();
        for (idx, field_cfg) in self.config.vector_fields.iter().enumerate() {
            if let Some(value) = stat.vector_values.get(idx) {
                vector_output_map.insert(field_cfg.output_column.clone(), value.clone());
            }
        }

        // 按输出列定义顺序逐个构建值
        let mut values = Vec::with_capacity(columns.len());
        for col in columns {
            if col == &self.config.output_metrics.operator_name {
                values.push(SqlValue::Text(stat.operator_name.clone()));
                continue;
            }
            if col == &self.config.output_metrics.occurrence_count {
                values.push(SqlValue::Integer(stat.occurrence_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.distinct_vector_count {
                values.push(SqlValue::Integer(stat.distinct_vector_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.purity {
                values.push(SqlValue::Real(stat.purity));
                continue;
            }

            // 工艺向量字段：从映射中取值，缺失则使用空字符串
            let text = vector_output_map.get(col).cloned().unwrap_or_default();
            values.push(SqlValue::Text(text));
        }

        values
    }
}
