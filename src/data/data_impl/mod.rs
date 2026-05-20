use std::collections::{BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::data::data_pair::DataPair;

mod flow;
mod schema;
mod value;

// Data：核心数据对象，维护列模式（索引键/对象键/变量键）和运行时键值存储
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Data {
    // 数据对象的名称标识
    pub name: String,
    #[serde(default)]
    // 索引键集合，用于标识行级索引字段
    index_key_set: HashSet<String>,
    // 对象键集合，用于标识对象级元数据字段
    object_key_set: HashSet<String>,
    // 变量键集合，用于标识动态变量字段
    variable_key_set: HashSet<String>,
    // 用于投影输出的名称列名
    project_name_column: String,
    // 用于投影输出的值列名
    project_value_column: String,
    #[serde(skip)]
    // 运行时流程状态集合，记录该数据经过的流程阶段
    flow_state_set: BTreeSet<String>,
    #[serde(skip)]
    // 标记该数据是否已被发射（输出）到下一个处理阶段
    emitted: bool,
    #[serde(skip)]
    // 标记数据是否通过校验
    is_valid: bool,
    #[serde(skip)]
    // 校验失败时的失败原因列表
    invalid_reasons: Vec<String>,
    #[serde(skip)]
    // 键值对字典，存储所有字段（索引/对象/变量）的当前值
    value_dict: HashMap<String, DataPair>,
}

impl Data {
    // 创建一个新的 Data 实例，所有集合和字典初始化为空
    pub fn new(name: String) -> Self {
        Self {
            name,
            index_key_set: HashSet::new(),
            object_key_set: HashSet::new(),
            variable_key_set: HashSet::new(),
            project_name_column: String::new(),
            project_value_column: String::new(),
            flow_state_set: BTreeSet::new(),
            emitted: false,
            is_valid: true,
            invalid_reasons: Vec::new(),
            value_dict: HashMap::new(),
        }
    }

    // 返回数据是否通过校验
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    // 返回校验失败的原因列表
    pub fn invalid_reasons(&self) -> &[String] {
        &self.invalid_reasons
    }

    // 重置校验状态：恢复为有效，清空失败原因
    pub fn reset_validation_state(&mut self) {
        self.is_valid = true;
        self.invalid_reasons.clear();
    }

    // 将数据标记为无效，并记录失败原因
    pub fn mark_invalid(&mut self, reason: String) {
        self.is_valid = false;
        self.invalid_reasons.push(reason);
    }

    // 返回是否已被发射到下一阶段
    pub fn emitted(&self) -> bool {
        self.emitted
    }

    // 设置发射标记
    pub fn set_emitted(&mut self, value: bool) {
        self.emitted = value;
    }

    // 返回投影名称列名
    pub fn project_name_column(&self) -> &str {
        &self.project_name_column
    }

    // 返回投影值列名
    pub fn project_value_column(&self) -> &str {
        &self.project_value_column
    }

    // 返回排序后的对象键列表
    pub fn object_keys(&self) -> Vec<String> {
        let mut keys = self.object_key_set.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    // 返回排序后的索引键列表
    pub fn index_keys(&self) -> Vec<String> {
        let mut keys = self.index_key_set.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    // 将一个键加入到变量键集合中
    pub fn add_key(&mut self, key: String) {
        self.variable_key_set.insert(key);
    }

    // 根据键名获取对应的 DataPair 不可变引用
    pub fn get_pair(&self, key: &String) -> Option<&DataPair> {
        self.value_dict.get(key)
    }

    // 根据键名获取对应的 DataPair 可变引用
    pub fn get_pair_mut(&mut self, key: &String) -> Option<&mut DataPair> {
        self.value_dict.get_mut(key)
    }

    // 设置指定键的 ready 标记
    pub fn set_pair_ready(&mut self, key: &String, ready: bool) {
        if let Some(pair) = self.value_dict.get_mut(key) {
            pair.ready = ready;
        }
    }

    // 根据模板数据克隆一份新 Data，并重置运行时状态
    pub fn from_template(template: &Data) -> Data {
        let mut data = template.clone();
        data.reset_runtime_state();
        data
    }
}
