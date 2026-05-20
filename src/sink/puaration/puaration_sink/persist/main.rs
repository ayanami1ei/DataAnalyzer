use std::collections::HashMap;
use std::path::Path;

use rusqlite::types::Value as SqlValue;

use crate::error::Error;
use crate::sink::database::sqlite_writer::{self, SqliteDbConfig};
use crate::sink::puaration::process_vector::ProcessVector;
use crate::sink::puaration::puaration_sink::PuarationSink;
use crate::sink::quality_inspection::quality_stat::BatchQuality;

impl PuarationSink {
    // 根据配置的列名列表构建一行数据库插入值；
    // 指标列使用对应数值，向量字段列从 vector.values 按索引匹配 output_column
    fn build_db_row_values(
        &self,
        columns: &[String],
        material_code: &str,
        vector: &ProcessVector,
        occurrence_count: usize,
        distinct_vector_count: usize,
        purity: f64,
        quant_isok_pct: f64,
        qual_isok_pct: f64,
    ) -> Vec<SqlValue> {
        // 构建向量字段输出列名到值的映射
        let mut vector_output_map = HashMap::new();
        for (idx, field_cfg) in self.config.vector_fields.iter().enumerate() {
            if let Some(value) = vector.values.get(idx) {
                vector_output_map.insert(field_cfg.output_column.clone(), value.clone());
            }
        }

        let mut values = Vec::with_capacity(columns.len());
        // 按列名依次赋值：指标列匹配对应输出列名，其他列从向量字段映射中获取
        for col in columns {
            if col == &self.config.output_metrics.occurrence_count {
                values.push(SqlValue::Integer(occurrence_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.distinct_vector_count {
                values.push(SqlValue::Integer(distinct_vector_count as i64));
                continue;
            }
            if col == &self.config.output_metrics.purity {
                values.push(SqlValue::Real(purity));
                continue;
            }
            if col == &self.config.output_metrics.material_code {
                values.push(SqlValue::Text(material_code.to_string()));
                continue;
            }
            if col == &self.config.output_metrics.process_vector {
                values.push(SqlValue::Text(vector.signature()));
                continue;
            }
            if col == &self.config.output_metrics.quant_isok_pct {
                values.push(SqlValue::Real(quant_isok_pct));
                continue;
            }
            if col == &self.config.output_metrics.qual_isok_pct {
                values.push(SqlValue::Real(qual_isok_pct));
                continue;
            }

            let text = vector_output_map.get(col).cloned().unwrap_or_default();
            values.push(SqlValue::Text(text));
        }

        values
    }

    // 根据列名列表构建 SQLite 列定义（INTEGER / REAL / TEXT）
    fn build_column_defs(&self, columns: &[String]) -> Vec<sqlite_writer::ColumnDef> {
        columns
            .iter()
            .map(|col| {
                // 计数类列为 INTEGER，百分比类列为 REAL，其余为 TEXT
                if col == &self.config.output_metrics.occurrence_count
                    || col == &self.config.output_metrics.distinct_vector_count
                {
                    sqlite_writer::ColumnDef::integer(col)
                } else if col == &self.config.output_metrics.purity
                    || col == &self.config.output_metrics.quant_isok_pct
                    || col == &self.config.output_metrics.qual_isok_pct
                {
                    sqlite_writer::ColumnDef::real(col)
                } else {
                    sqlite_writer::ColumnDef::text(col)
                }
            })
            .collect()
    }

    // 将全量统计结果（含质检数据关联）写入 SQLite 数据库
    pub(crate) fn persist_stats_to_database(
        &self,
        quality_results: &HashMap<String, BatchQuality>,
    ) -> Result<(), Error> {
        let map = self.counts_by_material.lock().unwrap();
        // 无数据时直接返回
        if map.is_empty() {
            return Ok(());
        }

        // 从内存计数收集统计，关联质检数据
        let mut stats = Self::collect_stats_from_map(&self.config, &map);
        let b2v = self.batch_to_vector.lock().unwrap();
        Self::enrich_stats_with_quality(&mut stats, &b2v, quality_results);
        drop(b2v);
        drop(map);

        if stats.is_empty() {
            return Ok(());
        }

        // 数据库配置文件不存在时跳过持久化
        if !Path::new(&self.config.database_config_path).exists() {
            return Ok(());
        }

        // 读取并解析数据库配置
        let db_cfg_text = std::fs::read_to_string(&self.config.database_config_path)?;
        let db_cfg: SqliteDbConfig = serde_json::from_str(&db_cfg_text)?;
        let sqlite_db_path = db_cfg
            .sqlite_db_path
            .unwrap_or_else(|| "data/puaration_stats.db".to_string());
        let table_name = db_cfg
            .table_name
            .unwrap_or_else(|| "puaration_stats".to_string());
        // 列名列表不能为空
        let columns = db_cfg
            .columns
            .filter(|x| !x.is_empty())
            .ok_or_else(|| Error::SourceDataFileError("missing database columns".to_string()))?;

        // 确保数据库目录存在，打开连接（关闭同步模式以提升写入性能）
        sqlite_writer::prepare_directory(&sqlite_db_path)?;
        let mut conn = sqlite_writer::open_sqlite(
            &sqlite_db_path,
            &[("synchronous", "OFF"), ("journal_mode", "MEMORY")],
        )?;

        // 创建表（若不存在）
        let col_defs = self.build_column_defs(&columns);
        sqlite_writer::create_table(&conn, &table_name, &col_defs)?;

        // 构建数据行，将工艺向量签名按 "|" 拆回值列表以匹配字段输出列
        let rows: Vec<Vec<SqlValue>> = stats
            .iter()
            .map(|stat| {
                let vector = ProcessVector {
                    values: stat.process_vector.split('|').map(|s| s.to_string()).collect(),
                };
                self.build_db_row_values(
                    &columns,
                    &stat.material_code,
                    &vector,
                    stat.occurrence_count,
                    stat.distinct_vector_count,
                    stat.purity,
                    stat.quant_isok_pct,
                    stat.qual_isok_pct,
                )
            })
            .collect();

        // 批量插入数据
        sqlite_writer::batch_insert_values(&mut conn, &table_name, &columns, &rows)?;

        Ok(())
    }
}
