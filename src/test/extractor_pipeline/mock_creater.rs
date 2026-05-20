// MockCreater：用于提取器管道测试的模拟数据创建器
// 根据 batch 中第二列是否为 "1" 决定数据是否 ready，不 ready 的数据积压后在 drain 时输出

use crate::data::data::Data;
use crate::error::Error;
use crate::traits::DataCreater;

// 模拟数据创建器
pub struct MockCreater {
    // 暂存未 ready 的数据
    pub all_data: Vec<Data>,
}

impl MockCreater {
    // 创建新的空 MockCreater
    pub fn new() -> Self {
        Self { all_data: vec![] }
    }
}

impl DataCreater for MockCreater {
    // 接收索引元素（当前无操作）
    fn set_row_elements(&mut self, _indexing_elements: Vec<String>) -> Result<(), Error> {
        Ok(())
    }

    // 根据 batch 创建数据
    // batch[0]: 值, batch[1]: ready 标记（"1" 表示 ready）
    // ready 的数据直接返回并标记 emitted；不 ready 的数据暂存到 all_data
    fn create_by_batch(&mut self, batch: Vec<String>) -> Result<Vec<Data>, Error> {
        let value = batch.first().cloned().unwrap_or_default();
        let is_ready = batch.get(1).is_some_and(|x| x == "1");

        let mut data = Data::new("mock_data".to_string());
        let key = "k".to_string();
        data.add_pair(key.clone(), value);
        data.set_pair_ready(&key, is_ready);
        if is_ready {
            data.add_flow_state("valid".to_string());
            data.set_emitted(true);
            Ok(vec![data])
        } else {
            // 不 ready 的数据暂存
            self.all_data.push(data);
            Ok(Vec::new())
        }
    }

    // 获取最后一条暂存的数据
    fn get_data(&self) -> Result<Data, Error> {
        self.all_data
            .last()
            .cloned()
            .ok_or_else(|| Error::SourceDataFileError("no data".to_string()))
    }

    // 清空暂存区，将所有未 ready 的数据标记为无效并返回
    fn drain_all_data(&mut self) -> Result<Vec<Data>, Error> {
        let mut all = std::mem::take(&mut self.all_data);
        for data in &mut all {
            data.mark_invalid("source ended before data became complete".to_string());
            data.add_flow_state("invalid".to_string());
        }
        Ok(all)
    }
}
