// 数据校验规则 trait：定义所有校验器必须实现的统一接口

use crate::{data::data::Data, error::Error};

// 校验规则 trait，所有具体校验器需实现 check 方法
// 要求 Send + Sync，支持跨线程安全共享
pub trait DataCheckRule: Send + Sync {
    // 对一条数据进行校验，通过返回 Ok(())，不通过返回 Error
    fn check(&self, data: &Data) -> Result<(), Error>;
}
