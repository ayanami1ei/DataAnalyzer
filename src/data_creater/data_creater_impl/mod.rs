// 工作记录数据创建器实现：将原始行数据按模板组装为结构化 Data 对象

mod batch;
mod drain;

use crate::{config, data::data::Data, error::Error};
use calamine::{ExcelDateTime, ExcelDateTimeType};
use std::collections::HashMap;

/// 工作记录数据创建器
///
/// 负责将每一行原始数据（String 数组）根据 DataTemplate 的定义组装为 Data 对象，
/// 并在数据完整时标记为"就绪"状态以供后续落库。
pub struct WorkRecordCreater {
    // 索引键 → Data 对象的映射缓存（以索引值数组为键）
    data_map: HashMap<Vec<String>, Data>,
    // 数据模板，定义了字段结构、索引键等元信息
    data_template: Data,
    // 用于唯一标识一条记录的索引键列表
    index_keys: Vec<String>,
    // 存储项目名称的列名
    project_name_column: String,
    // 存储项目数值的列名
    project_value_column: String,
    // 列名（经过标准化）→ 列号的映射
    indexing_elements_to_index: HashMap<String, usize>,
    // 原始索引行内容（未经标准化的列名列表）
    indexing_elements: Vec<String>,
    // 最后处理的索引值，用于 get_data 查询
    last_indexing: Option<Vec<String>>,
}

impl WorkRecordCreater {
    /// 使用数据模板创建新的 WorkRecordCreater
    ///
    /// # 参数
    /// - `data_template`: 数据模板，定义了记录的字段结构
    pub fn new(data_template: Data) -> WorkRecordCreater {
        // 从模板中提取索引键和项目列名
        let index_keys = data_template.index_keys();
        let project_name_column = data_template.project_name_column().to_string();
        let project_value_column = data_template.project_value_column().to_string();
        WorkRecordCreater {
            data_map: HashMap::new(),
            data_template,
            index_keys,
            project_name_column,
            project_value_column,
            indexing_elements_to_index: HashMap::new(),
            indexing_elements: vec![],
            last_indexing: None,
        }
    }

    /// 返回逻辑列名的别名列表，用于模糊匹配
    ///
    /// 例如 "物料键" 也可以匹配 "物料品号" 列。
    fn key_aliases(logical_key: &str) -> Vec<String> {
        let aliases = config::key_aliases();
        aliases
            .get(logical_key)
            .cloned()
            .unwrap_or_else(|| vec![logical_key.to_string()])
    }

    /// 根据逻辑列名查找列号（尝试所有别名）
    ///
    /// # 参数
    /// - `logical_key`: 逻辑列名
    ///
    /// # 返回
    /// - `Ok(usize)`: 列号
    /// - `Err`: 未找到匹配的列
    fn get_index(&self, logical_key: &str) -> Result<usize, Error> {
        let aliases = Self::key_aliases(logical_key);
        for alias in &aliases {
            let normalized = Self::normalize_column_name(alias);
            if let Some(index) = self.indexing_elements_to_index.get(&normalized) {
                return Ok(*index);
            }
        }

        Err(Error::SourceDataFileError(format!(
            "no such indexing element, {}",
            logical_key
        )))
    }

    /// 根据索引值获取或创建对应的 Data 对象（不存在时从模板克隆）
    fn get_or_create_data(&mut self, index: &[String]) -> &mut Data {
        let template = self.data_template.clone();
        self.data_map
            .entry(index.to_vec())
            .or_insert_with(|| Data::from_template(&template))
    }

    /// 根据索引键列表从一行数据中构建索引值数组
    ///
    /// # 参数
    /// - `batch`: 一行原始数据（按列分割的字符串数组）
    ///
    /// # 返回
    /// - `Ok(Vec<String>)`: 索引值数组
    fn build_indexing(&self, batch: &[String]) -> Result<Vec<String>, Error> {
        if self.index_keys.is_empty() {
            return Err(Error::SourceDataFileError(
                "index_key_set is empty in data config".to_string(),
            ));
        }

        // 逐列查找索引值
        let mut indexing = Vec::with_capacity(self.index_keys.len());
        for index_key in &self.index_keys {
            indexing.push(self.get_value_by_col_name(batch, index_key)?.to_string());
        }

        Ok(indexing)
    }

    /// 根据逻辑列名从一行数据中获取值（必需字段，不存在时返回错误）
    fn get_value_by_col_name<'a>(
        &self,
        batch: &'a [String],
        logical_key: &str,
    ) -> Result<&'a str, Error> {
        let idx = self.get_index(logical_key)?;
        batch.get(idx).map(|x| x.as_str()).ok_or_else(|| {
            Error::SourceDataFileError(format!(
                "column '{}' index {} out of range",
                logical_key, idx
            ))
        })
    }

    /// 根据逻辑列名从一行数据中获取值（可选字段，不存在时返回 None）
    fn get_optional_value_by_col_name<'a>(
        &self,
        batch: &'a [String],
        logical_key: &str,
    ) -> Option<&'a str> {
        let idx = self.get_index(logical_key).ok()?;
        batch.get(idx).map(|x| x.as_str())
    }

    /// 标准化列名：移除 BOM 标记、非断行空格，并去除首尾空白
    fn normalize_column_name(col_name: &str) -> String {
        col_name
            .replace('\u{feff}', "")
            .replace('\u{a0}', " ")
            .trim()
            .to_string()
    }

    /// 标准化对象值：对于包含"日期"的字段，将 Excel 序列号转换为日期字符串
    ///
    /// # 参数
    /// - `object_key`: 对象键名（用于判断是否包含日期）
    /// - `raw_value`: 原始字符串值
    ///
    /// # 返回
    /// - 标准化后的值，日期序列号会被转换为 "YYYY-MM-DD" 或 "YYYY-MM-DD HH:MM:SS" 格式
    fn normalize_object_value(object_key: &str, raw_value: &str) -> String {
        // 非日期字段直接返回原始值
        if !object_key.contains("日期") {
            return raw_value.to_string();
        }

        let trimmed = raw_value.trim();
        // 尝试解析为浮点数（Excel 日期序列号）
        let Ok(serial) = trimmed.parse::<f64>() else {
            return raw_value.to_string();
        };
        // Excel 日期序列号的有效范围
        if !(1.0..=2_958_465.0).contains(&serial) {
            return raw_value.to_string();
        }

        // 将序列号转换为日期时间
        let dt = ExcelDateTime::new(serial, ExcelDateTimeType::DateTime, false);
        let (year, month, day, hour, minute, second, milli) = dt.to_ymd_hms_milli();
        // 无时间部分时只返回日期
        if hour == 0 && minute == 0 && second == 0 && milli == 0 {
            return format!("{:04}-{:02}-{:02}", year, month, day);
        }

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    }
}
