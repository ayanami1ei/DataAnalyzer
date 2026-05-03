use crate::{checker::data_check_rule::DataCheckRule, error::Error, sink::data_sink::DataSink};

pub struct DataChecker<SinkType: DataSink> {
    next_sink: SinkType,
    rules: Vec<Box<dyn DataCheckRule>>,
}

impl<SinkType: DataSink> DataChecker<SinkType> {
    pub fn new(next_sink: SinkType, rules: Vec<Box<dyn DataCheckRule>>) -> Self {
        Self { next_sink, rules }
    }

    fn key_identity(data: &crate::data::data::Data) -> String {
        data.object_key_str().unwrap_or_default()
    }
}

impl<SinkType: DataSink> DataSink for DataChecker<SinkType> {
    type NextType = SinkType;

    fn sink(&mut self, data: &mut crate::data::data::Data) -> Result<(), Error> {
        data.reset_validation_state();

        if !data.is_ready() {
            let error_msg = Error::DataNotReady(data.get_key_str()?).to_string();
            crate::log::append_log_line(&format!(
                "data checker warning: key[{}], {}",
                Self::key_identity(data),
                error_msg
            ));
            data.mark_invalid(error_msg);
        }

        for rule in &self.rules {
            if let Err(e) = rule.check(data) {
                let error_msg = e.to_string();
                crate::log::append_log_line(&format!(
                    "data checker warning: key[{}], {}",
                    Self::key_identity(data),
                    error_msg
                ));
                data.mark_invalid(error_msg);
            }
        }

        self.next_sink.sink(data)
    }

    fn get_next_sink(&self) -> Result<Option<Self::NextType>, crate::error::Error> {
        Ok(None)
    }
}
