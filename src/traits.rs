// 核心抽象接口模块：定义数据读取与数据创建两个 trait，实现读取器与创建器的解耦
use crate::{data::data::Data, error::Error};

// 数据读取器接口：从源数据中按行读取原始字符串数据
pub trait DataReader {
    // 读取指定索引行的数据，返回该行各列的字符串值列表
    fn read_line(&mut self, index: usize) -> Result<Vec<String>, Error>;
    // 返回源数据的最大行数
    fn max_line(&mut self) -> Result<usize, Error>;
}

// 数据创建器接口：将原始行数据转换为结构化的 Data 对象
pub trait DataCreater {
    // 设置行中各列对应的索引元素（即表头映射关系）
    fn set_row_elements(&mut self, indexing_elements: Vec<String>) -> Result<(), Error>;
    // 按批次处理原始数据行，返回生成的 Data 对象列表
    fn create_by_batch(&mut self, batch: Vec<String>) -> Result<Vec<Data>, Error>;
    // 获取当前创建器内部已生成的 Data 对象
    fn get_data(&self) -> Result<Data, Error>;
    // 取出创建器中所有 Data 对象并清空内部缓冲区
    fn drain_all_data(&mut self) -> Result<Vec<Data>, Error>;
}
