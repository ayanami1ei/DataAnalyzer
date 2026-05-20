use crate::sink::data_sink::DataSink;

pub struct RegistInfo {
    pub(crate) name: &'static str,
    pub(crate) constructor: fn(&str) -> Box<dyn DataSink>,
}
