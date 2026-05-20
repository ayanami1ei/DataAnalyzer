// DataCreater trait 实现：列映射配置、数据创建、数据刷出

use crate::{data::data::Data, error::Error, traits::DataCreater};

use super::WorkRecordCreater;

impl DataCreater for WorkRecordCreater {
    /// 设置索引行元素：将列名（标准化后）映射到列号
    ///
    /// # 参数
    /// - `indexing_elements`: 索引行的列名列表
    fn set_row_elements(&mut self, indexing_elements: Vec<String>) -> Result<(), Error> {
        self.indexing_elements_to_index.clear();
        for i in 0..indexing_elements.len() {
            let normalized = Self::normalize_column_name(&indexing_elements[i]);
            self.indexing_elements_to_index.insert(normalized, i);
        }

        self.indexing_elements = indexing_elements;

        Ok(())
    }

    /// 获取最后一条已创建的 Data 的副本
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

    /// 刷出所有残留的 Data
    ///
    /// 遍历 data_map 中尚未发出的数据，将其标记为无效（因为源文件结束时数据仍未完成），
    /// 然后返回这些数据供后续处理。
    fn drain_all_data(&mut self) -> Result<Vec<Data>, Error> {
        // 取出所有 Data，清空 data_map
        let all = self
            .data_map
            .drain()
            .map(|(_, data)| data)
            .collect::<Vec<_>>();

        // 过滤：已发出的数据不再处理，未发出的标记为无效
        let mut remaining = Vec::new();
        for mut data in all {
            if data.emitted() {
                continue;
            }

            data.mark_invalid("source ended before data became complete".to_string());
            data.add_flow_state("invalid".to_string());
            remaining.push(data);
        }

        Ok(remaining)
    }

    /// 将一行原始数据转换为 Data（同时更新缓存）
    ///
    /// 处理流程：
    /// 1. 提取项目名称和数值
    /// 2. 构建索引值
    /// 3. 收集对象键值对
    /// 4. 将值应用到对应的 Data 对象
    /// 5. 检查该 Data 是否已就绪，若是则发出
    ///
    /// # 参数
    /// - `batch`: 一行原始数据（按列分割的字符串数组）
    ///
    /// # 返回
    /// - 就绪的 Data 列表（通常为空或含一个元素）
    fn create_by_batch(&mut self, batch: Vec<String>) -> Result<Vec<Data>, Error> {
        // 提取项目名称和项目数值
        let project_name = self
            .get_value_by_col_name(&batch, &self.project_name_column)?
            .to_string();
        let project_value = self
            .get_value_by_col_name(&batch, &self.project_value_column)?
            .to_string();

        // 构建当前行的索引值
        let indexing = self.build_indexing(&batch)?;
        self.last_indexing = Some(indexing.clone());

        // 收集该行中所有对象键的值
        let object_key_values = self.collect_object_key_values(&batch);

        // 将值应用到索引对应的 Data 对象上
        self.apply_values_to_data(&indexing, &object_key_values, &project_name, &project_value)?;

        // 检查该 Data 是否已收集完所有必要字段，是则发出
        let mut ready_data = Vec::new();
        if let Some(data) = self.try_emit_ready_data(&indexing)? {
            ready_data.push(data);
        }

        Ok(ready_data)
    }
}
