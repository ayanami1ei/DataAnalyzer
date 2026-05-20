// Excel 文件读取器模块：使用 calamine 引擎解析 .xls/.xlsx 文件，按行返回原始字符串数据
use std::{fs::File, io::BufReader};

use calamine::{Data, Reader, Sheets, open_workbook_auto};

use crate::{error::Error, traits::DataReader};

// Excel 读取器：将工作簿内容缓存为二维字符串矩阵
pub struct ExcelReader {
    // 按行存储的所有单元格数据，第一行为表头
    rows: Vec<Vec<String>>,
}

impl ExcelReader {
    // 在表头行中查找指定列名的索引位置，未找到时返回 SourceDataFileError
    pub fn get_column_index(header: &[String], column_name: &str) -> Result<usize, Error> {
        for (idx, col) in header.iter().enumerate() {
            if col.trim() == column_name {
                return Ok(idx);
            }
        }

        Err(Error::SourceDataFileError(format!(
            "missing column '{}'",
            column_name
        )))
    }

    // 将所有行的列数统一为第一行的宽度，不足的列用空字符串补齐
    fn normalize_rows(mut rows: Vec<Vec<String>>) -> Vec<Vec<String>> {
        let expected_width = rows.first().map(|r| r.len()).unwrap_or(0);
        if expected_width == 0 {
            return rows;
        }

        for row in &mut rows {
            if row.len() < expected_width {
                row.resize(expected_width, String::new());
            }
        }

        rows
    }

    // 从单个 Excel 文件创建读取器，自动识别文件格式并读取第一个工作表
    pub fn new(path: &str) -> Result<ExcelReader, Error> {
        let mut workbook: Sheets<BufReader<File>> = open_workbook_auto(path)?;
        let sheet = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| Error::SourceDataFileError("no worksheet found".into()))?;
        let range = workbook.worksheet_range(&sheet)?;
        // 将工作表中的每个单元格转换为字符串，按行收集
        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(|cell: &Data| cell.to_string()).collect())
            .collect();

        Ok(ExcelReader {
            rows: Self::normalize_rows(rows),
        })
    }

    // 并发读取多个 Excel 文件，合并所有工作表的数据行
    // 后续文件的第一行（表头）会被自动跳过，仅保留数据行
    pub async fn new_from_paths_parallel(paths: &[String]) -> Result<ExcelReader, Error> {
        let handles: Vec<_> = paths
            .iter()
            .enumerate()
            .map(|(_, path)| {
                let path = path.clone();
                tokio::task::spawn_blocking(move || ExcelReader::new(&path))
            })
            .collect();

        let mut merged_rows: Vec<Vec<String>> = Vec::new();

        // 依次收集各文件的结果，第一个文件保留表头，其余文件跳过表头行
        for (i, handle) in handles.into_iter().enumerate() {
            let mut reader = handle
                .await
                .map_err(|e| Error::SourceDataFileError(format!("reader task panicked: {}", e)))??;
            if i == 0 {
                merged_rows.append(&mut reader.rows);
                continue;
            }
            if reader.rows.is_empty() {
                continue;
            }
            // 后续文件跳过表头
            reader.rows.remove(0);
            merged_rows.append(&mut reader.rows);
        }

        if merged_rows.is_empty() {
            return Err(Error::SourceDataFileError(
                "no rows found in all source files".to_string(),
            ));
        }

        Ok(ExcelReader {
            rows: Self::normalize_rows(merged_rows),
        })
    }
}

impl DataReader for ExcelReader {
    // 按索引读取一行数据，索引越界时返回错误
    fn read_line(&mut self, index: usize) -> Result<Vec<String>, Error> {
        let row = self
            .rows
            .get(index)
            .ok_or_else(|| Error::SourceDataFileError(format!("row {} out of range", index)))?;

        Ok(row.clone())
    }

    // 返回当前读取器的总行数
    fn max_line(&mut self) -> Result<usize, Error> {
        Ok(self.rows.len())
    }
}
