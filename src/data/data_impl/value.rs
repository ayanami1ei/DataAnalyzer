use crate::{data::data_pair::DataPair, error::Error};

use super::Data;

impl Data {
    // 添加一个键值对到 value_dict 中，同时将键加入变量键集合，初始 ready 为 false
    pub fn add_pair(&mut self, key: String, value: String) {
        let pair = DataPair {
            key: key.clone(),
            value,
            ready: false,
        };
        self.variable_key_set.insert(key.clone());
        self.value_dict.insert(key, pair);
    }

    // 设置对象键对应的值，仅在键已被配置为对象键时成功。若值非空则标记 ready。
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

    // 设置变量键对应的值，仅在键存在于变量键集合中时成功。若值非空则标记 ready。
    // 返回 true 表示该变量存在且已更新，false 表示不存在该变量。
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

    // 按优先级遍历别名列表，返回第一个非空的值；若均为空则返回 None
    pub fn get_value_by_aliases(&self, aliases: &[String]) -> Option<String> {
        for alias in aliases {
            let Some(pair) = self.get_pair(alias) else {
                continue;
            };
            let value = pair.value.trim();
            if value.is_empty() {
                continue;
            }
            return Some(value.to_string());
        }
        None
    }
}
