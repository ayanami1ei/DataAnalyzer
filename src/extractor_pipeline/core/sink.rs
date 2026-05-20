// Sink 子模块：接收构造好的 Data 对象并异步写入持久化存储

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::warn;

use crate::{data::data::Data, error::Error, sink::data_sink::DataSink, traits::{DataCreater, DataReader}};

use super::ExtractorPipeline;

impl<ReaderType: DataReader, CreaterType: DataCreater> ExtractorPipeline<ReaderType, CreaterType> {
    /// Sink 调度器：从通道中接收 Data，为每个 Data 启动一个异步落库任务
    ///
    /// 当通道关闭后，等待所有落库任务完成。
    ///
    /// # 参数
    /// - `sink`: 数据落库目标
    /// - `data_rx`: 接收就绪 Data 的通道接收端
    pub(super) async fn sink_spawner(
        sink: Box<dyn DataSink>,
        mut data_rx: mpsc::Receiver<Arc<Mutex<Data>>>,
    ) -> Result<(), Error> {
        // 将 sink 包装为共享引用，供多个任务并发使用
        let shared_sink: Arc<dyn DataSink> = Arc::from(sink);
        let mut tasks = JoinSet::new();

        // 从通道中不断接收数据，每收到一个就启动一个落库任务
        while let Some(data) = data_rx.recv().await {
            let sink = Arc::clone(&shared_sink);
            tasks.spawn(async move {
                if let Err(e) = sink.sink(data).await {
                    warn!("sink failed, skip one data: {}", e);
                }
            });
        }

        // 等待所有落库任务完成，记录 panic 信息
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                warn!("sink task panicked: {}", e);
            }
        }

        Ok(())
    }
}
