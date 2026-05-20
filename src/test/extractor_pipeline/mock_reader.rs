// MockDataReader：用于提取器管道测试的模拟数据读取器
// 从预定义的二维字符串列表中按行读取数据

use crate::error::Error;
use crate::traits::DataReader;

// 模拟数据读取器
pub struct MockReader {
    // 预定义的测试数据行，每行为一组字符串
    pub rows: Vec<Vec<String>>,
}

impl DataReader for MockReader {
    // 读取指定索引的行数据
    // index: 行号 → 返回该行字符串向量，越界返回错误
    fn read_line(&mut self, index: usize) -> Result<Vec<String>, Error> {
        self.rows
            .get(index)
            .cloned()
            .ok_or_else(|| Error::SourceDataFileError("row out of range".to_string()))
    }

    // 返回数据总行数
    fn max_line(&mut self) -> Result<usize, Error> {
        Ok(self.rows.len())
    }
}
