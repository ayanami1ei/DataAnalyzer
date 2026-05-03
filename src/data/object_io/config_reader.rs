use std::{io::Read};

use crate::{data::data::Data, error::Error};

pub struct ConfigReader{
    path:String,
    object:Data,
}

impl ConfigReader{
    pub fn new(path:&String)->Self{
        Self { path: path.clone(), object: Data::default() }
    }

    pub fn load_config(&mut self)->Result<(), Error>{
        let mut file=std::fs::File::open(&self.path)?;
        let mut json=String::new();
        file.read_to_string(&mut json)?;

        self.object=serde_json::from_str(&json)?;
        self.object.reset_runtime_state();

        Ok(())
    }

    pub fn get_object(&self)->Data{
        self.object.clone()
    }
}