use crate::{data::data::Data, error::Error, sink::data_sink::DataSink};

pub struct EndSinkType {}

impl DataSink for EndSinkType {
    type NextType = Self;

    fn sink(&mut self, _data: &mut Data) -> Result<(), Error> {
        Ok(())
    }

    fn get_next_sink(&self) -> Result<Option<Self::NextType>, Error> {
        Ok(None)
    }
}
