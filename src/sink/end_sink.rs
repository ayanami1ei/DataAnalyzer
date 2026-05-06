use crate::{data::data::Data, error::Error, sink::data_sink::DataSink};

pub struct EndSinkType {}

impl DataSink for EndSinkType {
    fn sink(&mut self, _data: &mut Data) -> Result<(), Error> {
        Ok(())
    }

    fn get_next_sink(&self) -> Result<Option<Box<dyn DataSink>>, Error> {
        Ok(None)
    }
}
