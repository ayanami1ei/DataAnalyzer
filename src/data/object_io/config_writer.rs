use crate::{data::data::Data, error::Error};

pub struct ConfigWriter {
    path: String,
    obejct: Data,
}

impl ConfigWriter {
    pub fn new(object: Data) -> Self {
        Self {
            path: format!("{}.json", object.name),
            obejct: object,
        }
    }

    pub fn new_with_path(path: &str, object: Data) -> Self {
        Self {
            path: path.to_string(),
            obejct: object,
        }
    }

    pub fn save_config(&self) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(&self.obejct)?;
        std::fs::write(&self.path, json)?;

        Ok(())
    }
}
