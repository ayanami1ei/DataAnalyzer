use std::collections::{HashMap, HashSet};

use crate::{data::data_pair::DataPair, error::Error};

use super::Data;

impl Data {
    // 返回排序后的对象模式键列表（索引键 + 对象键）
    pub fn all_object_keys(&self) -> Vec<String> {
        let mut keys = self.object_schema_keys().into_iter().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    // 生成对象键及其值的格式化字符串，格式为 "key: value, key: value, "
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

    // 检查所有模式键（索引键+对象键+变量键）是否都已就绪
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

    // 构建模式键到当前值的映射表
    pub fn column_index_value_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        for key in self.schema_keys().iter() {
            if let Some(pair) = self.value_dict.get(key) {
                map.insert(key.clone(), pair.value.clone());
            }
        }

        map
    }

    // 生成所有键（对象键+变量键）及其值的格式化字符串
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

    // 重置运行时状态：恢复校验状态、清空流程状态、重置发射标记，并将所有模式键的值清空/ready 置 false
    pub fn reset_runtime_state(&mut self) {
        self.reset_validation_state();
        self.clear_flow_state_set();
        self.emitted = false;
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

    // 返回对象模式键集合：索引键与对象键的并集（仅内部使用）
    pub(crate) fn object_schema_keys(&self) -> HashSet<String> {
        self.index_key_set
            .union(&self.object_key_set)
            .cloned()
            .collect()
    }

    // 返回完整模式键集合：对象模式键与变量键的并集（仅内部使用）
    pub(crate) fn schema_keys(&self) -> HashSet<String> {
        let object_keys = self.object_schema_keys();
        object_keys.union(&self.variable_key_set).cloned().collect()
    }
}
