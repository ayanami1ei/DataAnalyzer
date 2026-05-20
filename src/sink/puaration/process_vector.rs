// 表示多字段组合而成的工艺参数向量
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProcessVector {
    // 各工艺参数字段的值列表，顺序与配置中 vector_fields 一致
    pub values: Vec<String>,
}

impl ProcessVector {
    // 将多个字段值用 "|" 拼接，生成向量的唯一标识签名
    pub fn signature(&self) -> String {
        self.values.join("|")
    }
}
