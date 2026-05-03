use crate::data::data::Data;
use crate::error::Error;
use crate::sink::{data_sink::DataSink, end_sink::EndSinkType};
use crate::test::extractor_pipeline::leaf_sink::LeafSink;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ForwardSink {
    pub sink_count: Arc<AtomicUsize>,
    pub next_sink: LeafSink,
}

impl DataSink for ForwardSink {
    type NextType = EndSinkType;

    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        self.sink_count.fetch_add(1, Ordering::Relaxed);
        self.next_sink.sink(data)
    }

    fn get_next_sink(&self) -> Result<Option<Self::NextType>, Error> {
        Ok(None)
    }
}
