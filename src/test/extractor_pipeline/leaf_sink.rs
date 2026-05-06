use crate::data::data::Data;
use crate::error::Error;
use crate::sink::data_sink::DataSink;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct LeafSink {
    pub sink_count: Arc<AtomicUsize>,
}

impl DataSink for LeafSink {
    fn sink(&mut self, _data: &mut Data) -> Result<(), Error> {
        self.sink_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn get_next_sink(&self) -> Result<Option<Box<dyn DataSink>>, Error> {
        Ok(None)
    }
}
