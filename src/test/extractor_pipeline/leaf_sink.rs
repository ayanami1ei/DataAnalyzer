// LeafSink：管道测试用的叶子节点 Sink 模拟
// 只递增计数，不处理数据内容

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sink_macro::DataSink;

use crate::data::data::Data;
use crate::error::Error;
use crate::sink::data_sink::DataSink;
use std::sync::atomic::{AtomicUsize, Ordering};

// 叶子 Sink：仅统计接收到的数据条数
#[derive(DataSink)]
pub struct LeafSink {
    // 收到的数据计数
    pub sink_count: Arc<AtomicUsize>,
}

impl LeafSink {
    pub fn new(_config_path: &str) -> Self {
        Self {
            sink_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl DataSink for LeafSink {
    fn new(_config_path: &str) -> Self {
        Self::new(_config_path)
    }

    async fn sink(&self, _data: Arc<Mutex<Data>>) -> Result<(), Error> {
        self.sink_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
