use crate::{data::data::Data, error::Error};

pub trait DataReader {
    fn read_line(&mut self, index: usize) -> Result<Vec<String>, Error>;
    fn max_line(&mut self) -> Result<usize, Error>;
}

pub trait DataCreater {
    fn set_row_elements(&mut self, indexing_elements: Vec<String>) -> Result<(), Error>;
    fn create_by_batch(&mut self, batch: Vec<String>) -> Result<(), Error>;
    fn get_data(&self) -> Result<Data, Error>;
    fn drain_all_data(&mut self) -> Result<Vec<Data>, Error>;
}
