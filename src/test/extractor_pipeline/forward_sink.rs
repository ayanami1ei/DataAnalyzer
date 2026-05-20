// ForwardSink：管道测试用的转发 Sink 模拟
// 先锁定数据再递增计数，模拟实际 Sink 的加锁行为

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sink_macro::DataSink;

use crate::data::data::Data;
use crate::error::Error;
use crate::sink::data_sink::DataSink;
use std::sync::atomic::{AtomicUsize, Ordering};

// 转发 Sink：先获取数据锁，再递增计数
#[derive(DataSink)]
pub struct ForwardSink {
    // 收到的数据计数
    pub sink_count: Arc<AtomicUsize>,
}

impl ForwardSink {
    pub fn new(_config_path: &str) -> Self {
        Self {
            sink_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl DataSink for ForwardSink {
    fn new(_config_path: &str) -> Self {
        Self::new(_config_path)
    }

    async fn sink(&self, data: Arc<Mutex<Data>>) -> Result<(), Error> {
        let _guard = data
            .lock()
            .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
        self.sink_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
