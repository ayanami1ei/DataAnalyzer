use crate::{data::data::Data, error::Error, traits::DataCreater};
use calamine::{ExcelDateTime, ExcelDateTimeType};
use std::collections::HashMap;

pub struct WorkRecordCreater {
    data_map: HashMap<Vec<String>, Data>,
    data_template: Data,
    index_keys: Vec<String>,
    project_name_column: String,
    project_value_column: String,
    indexing_elements_to_index: HashMap<String, usize>,
    indexing_elements: Vec<String>,
    last_indexing: Option<Vec<String>>,
}

impl WorkRecordCreater {
    pub fn new(data_template: Data) -> WorkRecordCreater {
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

    fn key_aliases(logical_key: &str) -> Vec<&str> {
        match logical_key {
            "物料键" | "物料品号" => vec!["物料品号", "物料品名"],
            "物料品名" => vec!["物料品名", "物料品号"],
            "创建日期" => vec!["创建日期", "生产日期"],
            _ => vec![logical_key],
        }
    }

    fn get_index(&self, logical_key: &str) -> Result<usize, Error> {
        let aliases = Self::key_aliases(logical_key);
        for alias in aliases {
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

    fn get_or_create_data(&mut self, index: &[String]) -> &mut Data {
        let template = self.data_template.clone();
        self.data_map
            .entry(index.to_vec())
            .or_insert_with(|| Data::from_template(&template))
    }

    fn build_indexing(&self, batch: &[String]) -> Result<Vec<String>, Error> {
        if self.index_keys.is_empty() {
            return Err(Error::SourceDataFileError(
                "index_key_set is empty in data config".to_string(),
            ));
        }

        let mut indexing = Vec::with_capacity(self.index_keys.len());
        for index_key in &self.index_keys {
            indexing.push(self.get_value_by_col_name(batch, index_key)?.to_string());
        }

        Ok(indexing)
    }

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

    fn get_optional_value_by_col_name<'a>(
        &self,
        batch: &'a [String],
        logical_key: &str,
    ) -> Option<&'a str> {
        let idx = self.get_index(logical_key).ok()?;
        batch.get(idx).map(|x| x.as_str())
    }

    fn normalize_column_name(col_name: &str) -> String {
        col_name
            .replace('\u{feff}', "")
            .replace('\u{a0}', " ")
            .trim()
            .to_string()
    }

    fn normalize_object_value(object_key: &str, raw_value: &str) -> String {
        if !object_key.contains("日期") {
            return raw_value.to_string();
        }

        let trimmed = raw_value.trim();
        let Ok(serial) = trimmed.parse::<f64>() else {
            return raw_value.to_string();
        };
        if !(1.0..=2_958_465.0).contains(&serial) {
            return raw_value.to_string();
        }

        let dt = ExcelDateTime::new(serial, ExcelDateTimeType::DateTime, false);
        let (year, month, day, hour, minute, second, milli) = dt.to_ymd_hms_milli();
        if hour == 0 && minute == 0 && second == 0 && milli == 0 {
            return format!("{:04}-{:02}-{:02}", year, month, day);
        }

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    }
}

impl DataCreater for WorkRecordCreater {
    fn set_row_elements(&mut self, indexing_elements: Vec<String>) -> Result<(), Error> {
        self.indexing_elements_to_index.clear();
        for i in 0..indexing_elements.len() {
            let normalized = Self::normalize_column_name(&indexing_elements[i]);
            self.indexing_elements_to_index.insert(normalized, i);
        }

        self.indexing_elements = indexing_elements;

        Ok(())
    }

    fn create_by_batch(&mut self, batch: Vec<String>) -> Result<(), Error> {
        let project_name = self
            .get_value_by_col_name(&batch, &self.project_name_column)?
            .to_string();
        let project_value = self
            .get_value_by_col_name(&batch, &self.project_value_column)?
            .to_string();

        let indexing = self.build_indexing(&batch)?;
        self.last_indexing = Some(indexing.clone());

        let object_keys = self.data_template.all_object_keys();
        let mut object_key_values: Vec<(String, String)> = Vec::with_capacity(object_keys.len());
        for object_key in &object_keys {
            let Some(key_value) = self.get_optional_value_by_col_name(&batch, object_key) else {
                continue;
            };
            let normalized_value = Self::normalize_object_value(object_key, key_value);
            object_key_values.push((object_key.clone(), normalized_value));
        }

        let data = self.get_or_create_data(&indexing);

        // 对象区分键（来自不同列）与变量名/变量值（来自另一组列）分开写入。
        for (object_key, key_value) in &object_key_values {
            if key_value.trim().is_empty() {
                continue;
            }
            data.set_object_key_value(object_key, key_value)?;
        }
        let _ = data.set_variable_value(&project_name, &project_value)?;

        Ok(())
    }

    fn get_data(&self) -> Result<Data, Error> {
        let index = self
            .last_indexing
            .as_ref()
            .ok_or_else(|| Error::SourceDataFileError("no data created yet".to_string()))?;

        let data_ptr = self.data_map.get(index).ok_or_else(|| {
            Error::SourceDataFileError("cannot find data by last indexing".to_string())
        })?;

        Ok(data_ptr.clone())
    }

    fn drain_all_data(&mut self) -> Result<Vec<Data>, Error> {
        let all = self
            .data_map
            .drain()
            .map(|(_, data)| data)
            .collect::<Vec<_>>();
        Ok(all)
    }
}
