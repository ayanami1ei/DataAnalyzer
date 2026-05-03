use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{data::data_pair::DataPair, error::Error};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Data {
    pub name: String,
    #[serde(default)]
    index_key_set: HashSet<String>,
    object_key_set: HashSet<String>,
    variable_key_set: HashSet<String>,
    project_name_column: String,
    project_value_column: String,
    #[serde(skip)]
    is_valid: bool,
    #[serde(skip)]
    invalid_reasons: Vec<String>,
    #[serde(skip)]
    value_dict: HashMap<String, DataPair>,
}

impl Data {
    pub fn new(name: String) -> Self {
        Self {
            name,
            index_key_set: HashSet::new(),
            object_key_set: HashSet::new(),
            variable_key_set: HashSet::new(),
            project_name_column: String::new(),
            project_value_column: String::new(),
            is_valid: true,
            invalid_reasons: Vec::new(),
            value_dict: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn invalid_reasons(&self) -> &[String] {
        &self.invalid_reasons
    }

    pub fn reset_validation_state(&mut self) {
        self.is_valid = true;
        self.invalid_reasons.clear();
    }

    pub fn mark_invalid(&mut self, reason: String) {
        self.is_valid = false;
        self.invalid_reasons.push(reason);
    }

    pub fn project_name_column(&self) -> &str {
        &self.project_name_column
    }

    pub fn project_value_column(&self) -> &str {
        &self.project_value_column
    }

    pub fn object_keys(&self) -> Vec<String> {
        let mut keys = self.object_key_set.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    pub fn index_keys(&self) -> Vec<String> {
        let mut keys = self.index_key_set.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    pub fn all_object_keys(&self) -> Vec<String> {
        let mut keys = self.object_schema_keys().into_iter().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    pub fn object_key_str(&self) -> Result<String, Error> {
        let mut res = String::new();
        let mut object_keys = self.object_schema_keys().into_iter().collect::<Vec<_>>();
        object_keys.sort();

        for key in object_keys {
            let value = self
                .value_dict
                .get(&key)
                .ok_or_else(|| Error::Other("Key not found".into()))?
                .value
                .clone();
            res.push_str(&format!("{}: {}, ", key, value));
        }

        Ok(res)
    }

    fn object_schema_keys(&self) -> HashSet<String> {
        self.index_key_set
            .union(&self.object_key_set)
            .cloned()
            .collect()
    }

    fn schema_keys(&self) -> HashSet<String> {
        let object_keys = self.object_schema_keys();

        object_keys.union(&self.variable_key_set).cloned().collect()
    }

    pub fn is_ready(&self) -> bool {
        for key in self.schema_keys().iter() {
            let Some(pair) = self.value_dict.get(key) else {
                return false;
            };
            if !pair.ready {
                return false;
            }
        }
        true
    }

    pub fn column_index_value_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        // 仅导出配置声明过的字段，避免把临时字段写入数据库。
        for key in self.schema_keys().iter() {
            if let Some(pair) = self.value_dict.get(key) {
                map.insert(key.clone(), pair.value.clone());
            }
        }

        map
    }

    pub fn get_key_str(&self) -> Result<String, Error> {
        let mut res = String::new();
        let mut object_keys = self.object_schema_keys().into_iter().collect::<Vec<_>>();
        object_keys.sort();

        let mut variable_keys = self.variable_key_set.iter().cloned().collect::<Vec<_>>();
        variable_keys.sort();

        for key in object_keys.iter().chain(variable_keys.iter()) {
            res.push_str(&format!(
                "{}: {}, ",
                key,
                self.value_dict
                    .get(key)
                    .ok_or_else(|| Error::Other("Key not found".into()))?
                    .value
            ));
        }
        Ok(res)
    }

    pub fn add_pair(&mut self, key: String, value: String) {
        let pair = DataPair {
            key: key.clone(),
            value,
            ready: false,
        };
        self.variable_key_set.insert(key.clone());
        self.value_dict.insert(key, pair);
    }

    pub fn add_key(&mut self, key: String) {
        self.variable_key_set.insert(key);
    }

    pub fn get_pair(&self, key: &String) -> Option<&DataPair> {
        self.value_dict.get(key)
    }

    pub fn get_pair_mut(&mut self, key: &String) -> Option<&mut DataPair> {
        self.value_dict.get_mut(key)
    }

    pub fn set_pair_ready(&mut self, key: &String, ready: bool) {
        if let Some(pair) = self.value_dict.get_mut(key) {
            pair.ready = ready;
        }
    }

    pub fn reset_runtime_state(&mut self) {
        // 每次创建新业务记录前，把 value/ready 清空，避免复用时污染。
        self.reset_validation_state();
        for key in self.schema_keys().iter() {
            self.value_dict
                .entry(key.clone())
                .and_modify(|pair| {
                    pair.value.clear();
                    pair.ready = false;
                })
                .or_insert(DataPair {
                    key: key.clone(),
                    value: String::new(),
                    ready: false,
                });
        }
    }

    pub fn set_object_key_value(&mut self, key: &str, value: &str) -> Result<(), Error> {
        if !self.object_schema_keys().contains(key) {
            return Err(Error::DataError(format!(
                "'{}' is not configured as object key field",
                key
            )));
        }

        let pair = self
            .value_dict
            .get_mut(key)
            .ok_or_else(|| Error::DataError(format!("cannot find pair by key '{}'", key)))?;

        pair.value = value.to_string();
        pair.ready = !value.trim().is_empty();
        Ok(())
    }

    pub fn set_variable_value(
        &mut self,
        variable_name: &str,
        variable_value: &str,
    ) -> Result<bool, Error> {
        if !self.variable_key_set.contains(variable_name) {
            return Ok(false);
        }

        let pair = self.value_dict.get_mut(variable_name).ok_or_else(|| {
            Error::DataError(format!("cannot find pair by variable '{}'", variable_name))
        })?;

        pair.value = variable_value.to_string();
        pair.ready = !variable_value.trim().is_empty();
        Ok(true)
    }

    pub fn from_template(template: &Data) -> Data {
        let mut data = template.clone();
        data.reset_runtime_state();
        data
    }
}
