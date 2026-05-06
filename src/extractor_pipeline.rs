use std::{
    collections::HashMap, marker::PhantomData, sync::{Arc, Mutex, RwLock, atomic::{AtomicUsize, Ordering}}, thread
};

use crossbeam_channel::{Receiver, Sender, bounded};

use crate::log::LogSender;
use crate::{
    data::data::Data,
    error::Error,
    sink::data_sink::DataSink,
    traits::{DataCreater, DataReader},
};

const RAW_LINE_BUFFER_SIZE: usize = 3000;
const DATA_TO_SINK_BUFFER_SIZE: usize = 3000;
const READ_PROGRESS_EVERY: usize = 5000;
const CREATE_PROGRESS_EVERY: usize = 5000;

//static mut SINK_REGISTER: Arc<RwLock<HashMap<String, Box<dyn DataSink>>>> =
//    Arc::new(RwLock::new(HashMap::new()));

struct PipelineLogger;
impl LogSender for PipelineLogger {}

pub struct ExtractorPipeline<ReaderType: DataReader, CreaterType: DataCreater> {
    reader: ReaderType,
    creater: CreaterType,
    data_buffer_front: Receiver<Vec<String>>,
    data_buffer_tail: Sender<Vec<String>>,
    sink: Box<dyn DataSink>,
    data_start_row: usize,

    _marker: PhantomData<Data>,
}

impl<ReaderType: DataReader, CreaterType: DataCreater> ExtractorPipeline<ReaderType, CreaterType>
{
    fn creater_worker_count() -> usize {
        // 当前 creater 是共享 Mutex，过多 worker 会严重锁竞争
        // 保守取 1，读线程与写库线程仍并发运行
        1
    }

    pub fn new(
        mut reader: ReaderType,
        mut creater: CreaterType,
        sink: Box<dyn DataSink>,
        indexing_row: usize,
    ) -> Result<ExtractorPipeline<ReaderType, CreaterType>, Error> {
        let indexing_elements = reader.read_line(indexing_row)?;
        creater.set_row_elements(indexing_elements.clone())?;
        let (data_buffer_tail, data_buffer_front) = bounded::<Vec<String>>(RAW_LINE_BUFFER_SIZE);
        Ok(ExtractorPipeline {
            reader,
            creater,
            sink,
            data_buffer_front,
            data_buffer_tail,
            data_start_row: indexing_row + 1,

            _marker: PhantomData,
        })
    }

    fn read_data_thread(
        mut reader: ReaderType,
        data_buffer_tail: Sender<Vec<String>>,
        data_start_row: usize,
    ) -> Result<(), Error> {
        let max_line = reader.max_line()?;
        let mut sent = 0usize;
        for i in data_start_row..max_line {
            let line = reader.read_line(i)?;
            data_buffer_tail
                .send(line)
                .map_err(|_| Error::PipelineError("cannot send data to buffer".into()))?;
            sent += 1;
            let _ = sent % READ_PROGRESS_EVERY == 0;
        }

        Ok(())
    }

    fn pipeline_worker_thread(
        creater: Arc<Mutex<CreaterType>>,
        data_buffer_front: Receiver<Vec<String>>,
        created_counter: Arc<AtomicUsize>,
    ) -> Result<(), Error> {
        while let Ok(datas) = data_buffer_front.recv() {
            let mut guard = creater
                .lock()
                .map_err(|_| Error::PipelineError("creater mutex poisoned".into()))?;
            if let Err(e) = guard.create_by_batch(datas) {
                PipelineLogger.send_log(&format!("create_by_batch failed, skip one row: {}", e));
                continue;
            }

            let created = created_counter.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = created % CREATE_PROGRESS_EVERY == 0;
        }

        Ok(())
    }

    fn flush_after_read_complete(
        creater: Arc<Mutex<CreaterType>>,
        data_to_sink_tail: &Sender<Data>,
    ) -> Result<(), Error> {
        let all_data = {
            let mut guard = creater
                .lock()
                .map_err(|_| Error::PipelineError("creater mutex poisoned".into()))?;
            guard.drain_all_data()?
        };

        // 将 creater 聚合出的全部对象交给 sink 链处理。
        // 是否可写入/是否继续转发由具体 sink 决定，pipeline 仅负责传递。
        for data in all_data {
            data_to_sink_tail.send(data).map_err(|_| {
                Error::PipelineError("cannot send parsed data to sink buffer".into())
            })?;
        }

        Ok(())
    }

    fn sink_thread(
        mut sink: Box<dyn DataSink>,
        data_to_sink_front: Receiver<Data>,
    ) -> Result<(), Error> {
        while let Ok(mut data) = data_to_sink_front.recv() {
            // 单条数据失败只记录日志，不终止整个消费线程。
            if let Err(e) = sink.sink(&mut data) {
                PipelineLogger.send_log(&format!("sink failed, skip one data: {}", e));
            }
        }

        Ok(())
    }

    pub fn run(self) -> Result<(), Error>
    where
        Data: Send + 'static,
        ReaderType: Send + 'static,
        CreaterType: Send + 'static,
    {
        let ExtractorPipeline {
            reader,
            creater,
            data_buffer_front,
            data_buffer_tail,
            sink,
            data_start_row,
            _marker: _,
        } = self;

        let reader_handle =
            thread::spawn(move || Self::read_data_thread(reader, data_buffer_tail, data_start_row));

        let (data_to_sink_tail, data_to_sink_front) = bounded::<Data>(DATA_TO_SINK_BUFFER_SIZE);

        let sink_handle = thread::spawn(move || Self::sink_thread(sink, data_to_sink_front));

        let shared_creater = Arc::new(Mutex::new(creater));
        let created_counter = Arc::new(AtomicUsize::new(0));
        let worker_count = Self::creater_worker_count();
        let mut worker_handles = Vec::with_capacity(worker_count);

        for _ in 0..worker_count {
            let creater_clone = Arc::clone(&shared_creater);
            let rx_clone = data_buffer_front.clone();
            let counter_clone = Arc::clone(&created_counter);
            worker_handles.push(thread::spawn(move || {
                Self::pipeline_worker_thread(creater_clone, rx_clone, counter_clone)
            }));
        }

        // 关键：释放 run 作用域中的原始 receiver，避免 worker 永远等不到通道关闭信号
        drop(data_buffer_front);

        let read_result = reader_handle
            .join()
            .map_err(|_| Error::PipelineError("reader thread panicked".into()))?;
        read_result?;

        for handle in worker_handles {
            let worker_result = handle
                .join()
                .map_err(|_| Error::PipelineError("pipeline worker thread panicked".into()))?;
            worker_result?;
        }

        Self::flush_after_read_complete(shared_creater, &data_to_sink_tail)?;
        drop(data_to_sink_tail);

        let sink_result = sink_handle
            .join()
            .map_err(|_| Error::PipelineError("sink thread panicked".into()))?;
        sink_result?;

        Ok(())
    }
}
