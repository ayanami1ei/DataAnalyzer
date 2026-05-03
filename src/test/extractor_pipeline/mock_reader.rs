use crate::error::Error;
use crate::traits::DataReader;

pub struct MockReader {
    pub rows: Vec<Vec<String>>,
}

impl DataReader for MockReader {
    fn read_line(&mut self, index: usize) -> Result<Vec<String>, Error> {
        self.rows
            .get(index)
            .cloned()
            .ok_or_else(|| Error::SourceDataFileError("row out of range".to_string()))
    }

    fn max_line(&mut self) -> Result<usize, Error> {
        Ok(self.rows.len())
    }
}
