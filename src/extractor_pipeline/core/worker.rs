// Worker 子模块：从 reader 通道接收原始行，通过 Creater 组装为 Data 并转发至 sink

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use tokio::sync::mpsc;
use tracing::warn;

use crate::{data::data::Data, error::Error, traits::{DataCreater, DataReader}};

use super::ExtractorPipeline;

impl<ReaderType: DataReader, CreaterType: DataCreater> ExtractorPipeline<ReaderType, CreaterType> {
    /// 尝试用 Creater 将一行原始数据转换为一批 Data
    ///
    /// 如果转换失败，跳过该行并记录警告。
    ///
    /// # 参数
    /// - `creater`: 共享的数据构造器（需加锁访问）
    /// - `datas`: 一行原始数据（按列分割的字符串数组）
    ///
    /// # 返回
    /// - `Ok(Some(vec))`: 转换成功，得到一组 Data
    /// - `Ok(None)`: 转换失败，已跳过
    fn try_create_batch(
        creater: &Arc<Mutex<CreaterType>>,
        datas: Vec<String>,
    ) -> Result<Option<Vec<Data>>, Error> {
        let mut guard = creater
            .lock()
            .map_err(|_| Error::PipelineError("creater mutex poisoned".into()))?;
        match guard.create_by_batch(datas) {
            Ok(data) => Ok(Some(data)),
            Err(e) => {
                warn!("create_by_batch failed, skip one row: {}", e);
                Ok(None)
            }
        }
    }

    /// 将一组就绪的 Data 逐个发送到 sink 通道
    ///
    /// # 参数
    /// - `data_to_sink_tx`: 发送到 sink 的通道发送端
    /// - `ready_data`: 已构造完成的 Data 列表
    async fn send_batch_to_sink(
        data_to_sink_tx: &mpsc::Sender<Arc<Mutex<Data>>>,
        ready_data: Vec<Data>,
    ) -> Result<(), Error> {
        for data in ready_data {
            data_to_sink_tx
                .send(Arc::new(Mutex::new(data)))
                .await
                .map_err(|_| Error::PipelineError("cannot send ready data to sink buffer".into()))?;
        }
        Ok(())
    }

    /// Worker 主循环：从 data_rx 接收原始行，转换后发送到 sink 通道
    ///
    /// 通道关闭后自动退出。
    ///
    /// # 参数
    /// - `creater`: 共享的数据构造器
    /// - `data_rx`: 接收原始行数据的通道
    /// - `data_to_sink_tx`: 发送就绪 Data 到 sink 的通道
    /// - `created_counter`: 已处理批次计数器
    pub(super) async fn pipeline_worker_task(
        creater: Arc<Mutex<CreaterType>>,
        mut data_rx: mpsc::Receiver<Vec<String>>,
        data_to_sink_tx: mpsc::Sender<Arc<Mutex<Data>>>,
        created_counter: Arc<AtomicUsize>,
    ) -> Result<(), Error> {
        while let Some(datas) = data_rx.recv().await {
            if let Some(ready_data) = Self::try_create_batch(&creater, datas)? {
                Self::send_batch_to_sink(&data_to_sink_tx, ready_data).await?;
                created_counter.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// 读取完成后，刷出 Creater 中所有尚未发出的剩余数据
    ///
    /// 这些数据在源文件结束时仍未完成，会被标记为无效。
    ///
    /// # 参数
    /// - `creater`: 共享的数据构造器
    /// - `data_to_sink_tx`: 发送就绪 Data 到 sink 的通道发送端
    pub(super) async fn flush_after_read_complete(
        creater: Arc<Mutex<CreaterType>>,
        data_to_sink_tx: &mpsc::Sender<Arc<Mutex<Data>>>,
    ) -> Result<(), Error> {
        // 加锁并取出所有残余数据
        let all_data = {
            let mut guard = creater
                .lock()
                .map_err(|_| Error::PipelineError("creater mutex poisoned".into()))?;
            guard.drain_all_data()?
        };
        // 逐条发送到 sink
        for data in all_data {
            data_to_sink_tx
                .send(Arc::new(Mutex::new(data)))
                .await
                .map_err(|_| {
                    Error::PipelineError("cannot send parsed data to sink buffer".into())
                })?;
        }
        Ok(())
    }
}
