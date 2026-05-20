// 批次处理子模块：收集对象键值、应用到 Data、检查并就绪发出

use crate::data::data::Data;

use super::WorkRecordCreater;

impl WorkRecordCreater {
    /// 从一行原始数据中收集所有对象键的值
    ///
    /// 遍历模板定义的所有对象键，在行数据中查找对应值，
    /// 并对日期类型的值进行标准化（Excel 序列号 → 日期字符串）。
    ///
    /// # 参数
    /// - `batch`: 一行原始数据（按列分割的字符串数组）
    ///
    /// # 返回
    /// - 非空键值对列表
    pub(super) fn collect_object_key_values(&self, batch: &[String]) -> Vec<(String, String)> {
        let object_keys = self.data_template.all_object_keys();
        let mut object_key_values = Vec::with_capacity(object_keys.len());
        for object_key in &object_keys {
            let Some(key_value) = self.get_optional_value_by_col_name(batch, object_key) else {
                continue;
            };
            let normalized_value = Self::normalize_object_value(object_key, key_value);
            object_key_values.push((object_key.clone(), normalized_value));
        }
        object_key_values
    }

    /// 将对象键值对和变量值应用到指定索引对应的 Data 对象上
    ///
    /// # 参数
    /// - `indexing`: 索引值数组，用于定位 Data 对象
    /// - `object_key_values`: 对象字段的键值对列表
    /// - `project_name`: 项目名称（变量名）
    /// - `project_value`: 项目数值（变量值）
    pub(super) fn apply_values_to_data(
        &mut self,
        indexing: &[String],
        object_key_values: &[(String, String)],
        project_name: &str,
        project_value: &str,
    ) -> Result<(), super::Error> {
        let data = self.get_or_create_data(indexing);

        // 设置对象字段值，跳过空值
        for (object_key, key_value) in object_key_values {
            if key_value.trim().is_empty() {
                continue;
            }
            data.set_object_key_value(object_key, key_value)?;
        }
        // 设置变量（项目名称 → 项目数值）
        let _ = data.set_variable_value(project_name, project_value)?;
        Ok(())
    }

    /// 检查索引对应的 Data 是否已就绪且未发出，若是则标记为已发出并返回副本
    ///
    /// 就绪条件由 Data 内部定义（通常指所有必需字段均已赋值）。
    ///
    /// # 参数
    /// - `indexing`: 索引值数组
    ///
    /// # 返回
    /// - `Ok(Some(Data))`: 就绪的 Data 副本
    /// - `Ok(None)`: 数据未就绪或已发出
    pub(super) fn try_emit_ready_data(
        &mut self,
        indexing: &[String],
    ) -> Result<Option<Data>, super::Error> {
        let Some(data) = self.data_map.get(indexing) else {
            return Ok(None);
        };
        // 数据未就绪或已发出则跳过
        if !data.is_ready() || data.emitted() {
            return Ok(None);
        }

        // 标记为有效并发出
        let completed_data = self.data_map.get_mut(indexing).ok_or_else(|| {
            super::Error::SourceDataFileError("cannot find ready data".to_string())
        })?;
        completed_data.add_flow_state("valid".to_string());
        completed_data.set_emitted(true);
        Ok(Some(completed_data.clone()))
    }
}
