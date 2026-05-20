// Reader 子模块：在阻塞线程中逐行读取原始数据并发送到通道

use tokio::sync::mpsc;

use crate::{error::Error, traits::{DataCreater, DataReader}};

use super::ExtractorPipeline;

impl<ReaderType: DataReader, CreaterType: DataCreater> ExtractorPipeline<ReaderType, CreaterType> {
    /// 阻塞式逐行读取数据源
    ///
    /// 从 data_start_row 开始到最大行号结束，
    /// 每读取一行即通过 data_tx 阻塞发送给 worker。
    ///
    /// # 参数
    /// - `reader`: 数据源读取器
    /// - `data_tx`: 原始行数据发送端
    /// - `data_start_row`: 起始行号（含）
    pub(super) fn read_data_blocking(
        mut reader: ReaderType,
        data_tx: mpsc::Sender<Vec<String>>,
        data_start_row: usize,
    ) -> Result<(), Error> {
        // 获取数据源总行数
        let max_line = reader.max_line()?;
        // 逐行遍历并发送
        for i in data_start_row..max_line {
            let line = reader.read_line(i)?;
            data_tx
                .blocking_send(line)
                .map_err(|_| Error::PipelineError("cannot send data to buffer".into()))?;
        }
        Ok(())
    }
}
