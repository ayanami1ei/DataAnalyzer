use std::collections::BTreeSet;

use super::Data;

impl Data {
    // 返回流程状态集合的克隆副本
    pub fn flow_state_set(&self) -> BTreeSet<String> {
        self.flow_state_set.clone()
    }

    // 添加一个流程状态，返回 true 表示该状态此前不存在并已插入
    pub fn add_flow_state(&mut self, state: String) -> bool {
        self.flow_state_set.insert(state)
    }

    // 移除一个流程状态，返回 true 表示该状态存在并被删除
    pub fn remove_flow_state(&mut self, state: &str) -> bool {
        self.flow_state_set.remove(state)
    }

    // 清空所有流程状态
    pub fn clear_flow_state_set(&mut self) {
        self.flow_state_set.clear();
    }
}
