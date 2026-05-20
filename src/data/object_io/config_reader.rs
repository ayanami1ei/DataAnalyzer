use std::{io::Read};

use crate::{data::data::Data, error::Error};

// ConfigReader：从 JSON 配置文件中读取并解析 Data 对象的读取器
pub struct ConfigReader{
    // 配置文件的路径
    path:String,
    // 从文件解析后持有的 Data 对象
    object:Data,
}

impl ConfigReader{
    // 创建一个新的 ConfigReader，指向指定路径，对象初始化为默认值
    pub fn new(path:&String)->Self{
        Self { path: path.clone(), object: Data::default() }
    }

    // 打开配置文件，读取全部内容并反序列化为 Data 对象，然后重置运行时状态
    pub fn load_config(&mut self)->Result<(), Error>{
        let mut file=std::fs::File::open(&self.path)?;
        let mut json=String::new();
        file.read_to_string(&mut json)?;

        self.object=serde_json::from_str(&json)?;
        self.object.reset_runtime_state();

        Ok(())
    }

    // 返回当前持有的 Data 对象的克隆副本
    pub fn get_object(&self)->Data{
        self.object.clone()
    }
}
