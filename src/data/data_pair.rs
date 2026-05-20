use serde::{Deserialize, Serialize};

// DataPair：表示一个键值对，携带就绪标记 ready
// 序列化时跳过 value（滞后加载）和 ready（运行时状态），仅持久化 key
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPair{
    // 键名，用于在 value_dict 中唯一标识此数据对
    pub key:String,

    // 键对应的实际值，序列化时跳过
    #[serde(skip)]
    pub value:String,
    // 标记该值是否已就绪（非空），序列化时跳过
    #[serde(skip)]
    pub ready:bool,
}
