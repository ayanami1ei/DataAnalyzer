use crate::{data::data::Data, error::Error};

pub trait DataSink: Send {
    fn sink(&mut self, data: &mut Data) -> Result<(), Error>;
    fn get_next_sink(&self) -> Result<Option<Box<dyn DataSink>>, Error>;
    fn forward(&self, data: &mut Data) -> Result<(), Error> {
        let next_sink_ptr = self.get_next_sink()?;

        if let Some(mut next_sink) = next_sink_ptr {
            next_sink.sink(data)?;
        }
        Ok(())
    }
}
