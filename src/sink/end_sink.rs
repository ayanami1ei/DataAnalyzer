use sink_macro::DataSink;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use crate::{data::data::Data, error::Error, sink::data_sink::DataSink};

#[derive(DataSink)]
pub struct EndSinkType {}

impl EndSinkType {
    pub fn new(_config_path: &str) -> Self {
        Self {}
    }
}

#[async_trait]
impl DataSink for EndSinkType {
    fn new(_config_path: &str) -> Self {
        Self {}
    }

    async fn sink(&self, _data: Arc<Mutex<Data>>) -> Result<(), Error> {
        Ok(())
    }
}
