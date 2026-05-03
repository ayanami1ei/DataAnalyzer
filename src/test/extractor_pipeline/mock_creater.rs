use crate::data::data::Data;
use crate::error::Error;
use crate::traits::DataCreater;

pub struct MockCreater {
    pub all_data: Vec<Data>,
}

impl MockCreater {
    pub fn new() -> Self {
        Self { all_data: vec![] }
    }
}

impl DataCreater for MockCreater {
    fn set_row_elements(&mut self, _indexing_elements: Vec<String>) -> Result<(), Error> {
        Ok(())
    }

    fn create_by_batch(&mut self, batch: Vec<String>) -> Result<(), Error> {
        let value = batch.first().cloned().unwrap_or_default();
        let is_ready = batch.get(1).is_some_and(|x| x == "1");

        let mut data = Data::new("mock_data".to_string());
        let key = "k".to_string();
        data.add_pair(key.clone(), value);
        data.set_pair_ready(&key, is_ready);
        self.all_data.push(data);
        Ok(())
    }

    fn get_data(&self) -> Result<Data, Error> {
        self.all_data
            .last()
            .cloned()
            .ok_or_else(|| Error::SourceDataFileError("no data".to_string()))
    }

    fn drain_all_data(&mut self) -> Result<Vec<Data>, Error> {
        Ok(std::mem::take(&mut self.all_data))
    }
}
