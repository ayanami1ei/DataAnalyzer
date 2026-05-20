// 提取管线模块：协调数据读取、数据创建与数据落库三个阶段的异步执行

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};

use tokio::sync::mpsc;

use crate::{
    config, data::data::Data,
    error::Error,
    sink::data_sink::DataSink,
    traits::{DataCreater, DataReader},
};

mod reader;
mod sink;
mod worker;

/// 提取管线：将 Reader（数据源）、Creater（数据构造器）和 Sink（落库）串联为一条流水线
pub struct ExtractorPipeline<ReaderType: DataReader, CreaterType: DataCreater> {
    // 数据源读取器
    reader: ReaderType,
    // 数据构造器，负责将原始行转换为结构化 Data
    creater: CreaterType,
    // 原始行数据接收端（worker 消费）
    data_rx: mpsc::Receiver<Vec<String>>,
    // 原始行数据发送端（reader 生产）
    data_tx: mpsc::Sender<Vec<String>>,
    // 数据落库目标
    sink: Box<dyn DataSink>,
    // 数据起始行号（索引行之后的第一行）
    data_start_row: usize,
    // 类型占位符，不实际存储数据
    _marker: PhantomData<Data>,
}

impl<ReaderType: DataReader, CreaterType: DataCreater> ExtractorPipeline<ReaderType, CreaterType> {
    /// 新建提取管线
    ///
    /// - 读取第 indexing_row 行作为列索引行
    /// - 用索引行初始化 Creater 的列映射
    /// - 创建原始行传输通道
    ///
    /// # 参数
    /// - `reader`: 数据源读取器
    /// - `creater`: 数据构造器
    /// - `sink`: 数据落库目标
    /// - `indexing_row`: 列索引所在的行号（从 0 开始）
    pub fn new(
        mut reader: ReaderType,
        mut creater: CreaterType,
        sink: Box<dyn DataSink>,
        indexing_row: usize,
    ) -> Result<ExtractorPipeline<ReaderType, CreaterType>, Error> {
        // 读取索引行，获取所有列名
        let indexing_elements = reader.read_line(indexing_row)?;
        // 将列名映射写入 creater
        creater.set_row_elements(indexing_elements.clone())?;
        let pc = config::pipeline_constants();
        let (data_tx, data_rx) = mpsc::channel::<Vec<String>>(pc.raw_line_buffer_size);
        Ok(ExtractorPipeline {
            reader, creater, data_rx, data_tx, sink,
            // 数据从索引行的下一行开始
            data_start_row: indexing_row + 1,
            _marker: PhantomData,
        })
    }

    /// 启动管线，异步执行以下三个阶段：
    /// 1. 读取线程阻塞地逐行读取原始数据
    /// 2. Worker 任务将原始行组装为结构化 Data
    /// 3. Sink 任务将就绪 Data 写入持久化存储
    pub async fn run(self) -> Result<(), Error>
    where
        Data: Send + 'static,
        ReaderType: Send + 'static,
        CreaterType: Send + 'static,
    {
        let ExtractorPipeline { reader, creater, data_rx, data_tx, sink, data_start_row, .. } = self;

        // 阶段一：在阻塞线程中逐行读取数据，通过 data_tx 发送给 worker
        let reader_handle = tokio::task::spawn_blocking(move || {
            Self::read_data_blocking(reader, data_tx, data_start_row)
        });

        let pc = config::pipeline_constants();
        let (data_to_sink_tx, data_to_sink_rx) =
            mpsc::channel::<Arc<Mutex<Data>>>(pc.data_to_sink_buffer_size);

        // 阶段三：Sink 异步任务，不断从 data_to_sink_rx 接收数据并落库
        let sink_handle = tokio::spawn(async move {
            Self::sink_spawner(sink, data_to_sink_rx).await
        });

        // 共享的 creater：reader 和 worker 需要同时访问
        let shared_creater = Arc::new(Mutex::new(creater));
        // 已处理批次计数器（仅用于统计）
        let created_counter = Arc::new(AtomicUsize::new(0));

        // 阶段二：Worker 异步任务，消费原始行并生产结构化 Data
        let sc_for_worker = shared_creater.clone();
        let tx_for_worker = data_to_sink_tx.clone();
        let worker_handle = tokio::spawn(async move {
            Self::pipeline_worker_task(
                sc_for_worker, data_rx, tx_for_worker, created_counter,
            ).await
        });

        // 等待 reader 完成
        let read_result = reader_handle
            .await.map_err(|_| Error::PipelineError("reader thread panicked".into()))?;
        read_result?;

        // 等待 worker 处理完所有已读取的数据
        let worker_result = worker_handle
            .await.map_err(|_| Error::PipelineError("pipeline worker task panicked".into()))?;
        worker_result?;

        // 刷出 creater 中剩余未发出的数据
        Self::flush_after_read_complete(shared_creater, &data_to_sink_tx).await?;
        drop(data_to_sink_tx);

        // 等待 sink 将所有数据落库完毕
        let sink_result = sink_handle
            .await.map_err(|_| Error::PipelineError("sink spawner panicked".into()))?;
        sink_result?;

        Ok(())
    }
}
