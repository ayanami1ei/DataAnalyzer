use std::{fs::File, io::BufReader};

use calamine::{Data, Reader, Sheets, open_workbook_auto};

use crate::{error::Error, traits::DataReader};

pub struct ExcelReader {
    rows: Vec<Vec<String>>,
}

impl ExcelReader {
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

    pub fn new(path: &str) -> Result<ExcelReader, Error> {
        let mut workbook: Sheets<BufReader<File>> = open_workbook_auto(path)?;
        let sheet = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| Error::SourceDataFileError("no worksheet found".into()))?;
        let range = workbook.worksheet_range(&sheet)?;
        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(|cell: &Data| cell.to_string()).collect())
            .collect();

        Ok(ExcelReader {
            rows: Self::normalize_rows(rows),
        })
    }

    pub fn new_from_paths(paths: &[String]) -> Result<ExcelReader, Error> {
        let mut merged_rows: Vec<Vec<String>> = Vec::new();

        for (idx, path) in paths.iter().enumerate() {
            let mut reader = ExcelReader::new(path)?;
            if idx == 0 {
                merged_rows.append(&mut reader.rows);
                continue;
            }

            if reader.rows.is_empty() {
                continue;
            }

            // 后续文件跳过表头，避免重复列名行进入数据流。
            let mut rows = reader.rows;
            rows.remove(0);
            merged_rows.append(&mut rows);
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
    fn read_line(&mut self, index: usize) -> Result<Vec<String>, Error> {
        let row = self
            .rows
            .get(index)
            .ok_or_else(|| Error::SourceDataFileError(format!("row {} out of range", index)))?;

        Ok(row.clone())
    }

    fn max_line(&mut self) -> Result<usize, Error> {
        Ok(self.rows.len())
    }
}
