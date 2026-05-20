// SQLite 写入工具：提供与 SQLite 数据库交互的底层函数，包括建表、插入、批处理写入等
use std::path::Path;

use rusqlite::{Connection, params_from_iter, types::Value};
use serde::Deserialize;

use crate::error::Error;

// SQLite 数据库配置文件结构，对应 JSON 配置中的字段
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SqliteDbConfig {
    // SQLite 数据库文件路径
    pub sqlite_db_path: Option<String>,
    // 数据表名称
    pub table_name: Option<String>,
    // 数据表的列名列表
    pub columns: Option<Vec<String>>,
}

// 确保数据库文件所在目录存在，如不存在则递归创建
pub fn prepare_directory(path: &str) -> Result<(), Error> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

// 对标识符（表名、列名）进行双引号转义，防止 SQL 注入
pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

// 打开 SQLite 连接，并执行指定的 PRAGMA 设置
pub fn open_sqlite(path: &str, pragmas: &[(&str, &str)]) -> Result<Connection, Error> {
    let conn = Connection::open(path)?;
    for (key, value) in pragmas {
        conn.execute_batch(&format!("PRAGMA {} = {}", key, value))?;
    }
    Ok(conn)
}

// 列定义：包含列名和 SQL 数据类型
pub struct ColumnDef {
    // 列名
    pub name: String,
    // SQL 数据类型，如 TEXT / INTEGER / REAL
    pub col_type: &'static str,
}

impl ColumnDef {
    // 创建文本类型的列定义
    pub fn text(name: &str) -> Self {
        Self { name: name.to_string(), col_type: "TEXT" }
    }

    // 创建整数类型的列定义
    pub fn integer(name: &str) -> Self {
        Self { name: name.to_string(), col_type: "INTEGER" }
    }

    // 创建浮点数类型的列定义
    pub fn real(name: &str) -> Self {
        Self { name: name.to_string(), col_type: "REAL" }
    }
}

// 根据列定义创建数据表（如果不存在），自动添加自增主键 id 列
pub fn create_table(
    conn: &Connection,
    table_name: &str,
    columns: &[ColumnDef],
) -> Result<(), Error> {
    // 构建列定义列表，包含自增主键和用户定义的列
    let mut defs = Vec::with_capacity(columns.len() + 1);
    defs.push("id INTEGER PRIMARY KEY AUTOINCREMENT".to_string());
    for col in columns {
        defs.push(format!("{} {} NULL", quote_ident(&col.name), col.col_type));
    }
    // 拼接 CREATE TABLE 语句并执行
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        quote_ident(table_name),
        defs.join(", ")
    );
    conn.execute_batch(&sql)?;
    Ok(())
}

// 使用全文本类型快速创建数据表
pub fn create_table_all_text(
    conn: &Connection,
    table_name: &str,
    columns: &[String],
) -> Result<(), Error> {
    let defs: Vec<ColumnDef> = columns.iter().map(|c| ColumnDef::text(c)).collect();
    create_table(conn, table_name, &defs)
}

// 构建 INSERT 语句模板，使用 ? 作为值占位符
pub fn build_insert_sql(table_name: &str, columns: &[String]) -> String {
    let cols = columns
        .iter()
        .map(|c| quote_ident(c))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = vec!["?"; columns.len()].join(", ");
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_ident(table_name),
        cols,
        placeholders
    )
}

// 批量插入数据（Value 类型），使用事务保证写入性能
pub fn batch_insert_values(
    conn: &mut Connection,
    table_name: &str,
    columns: &[String],
    rows: &[Vec<Value>],
) -> Result<(), Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let sql = build_insert_sql(table_name, columns);
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare(&sql)?;
    for row in rows {
        stmt.execute(params_from_iter(row.iter()))?;
    }
    drop(stmt);
    tx.commit()?;
    Ok(())
}

// 批量插入字符串数据，适用于全文本类型的表
pub fn batch_insert_str(
    conn: &mut Connection,
    table_name: &str,
    columns: &[String],
    rows: &[Vec<String>],
) -> Result<(), Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let sql = build_insert_sql(table_name, columns);
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare(&sql)?;
    for row in rows {
        stmt.execute(params_from_iter(row.iter()))?;
    }
    drop(stmt);
    tx.commit()?;
    Ok(())
}

// 便捷函数：先建表（全文本类型）再批量写入字符串数据
pub fn write_batch_all_text(
    conn: &mut Connection,
    table_name: &str,
    columns: &[String],
    rows: &[Vec<String>],
) -> Result<(), Error> {
    create_table_all_text(conn, table_name, columns)?;
    batch_insert_str(conn, table_name, columns, rows)
}
