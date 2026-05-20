// 错误类型定义模块：集中管理应用中所有可能的错误类型，统一实现 std::error::Error
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error{
    // JSON 解析/序列化错误
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),

    // 文件 I/O 操作错误
    #[error("file io error: {0}")]
    FileError(#[from] std::io::Error),

    // 流水线执行过程中的通用错误
    #[error("pipeline error: {0}")]
    PipelineError(String),

    // 源数据文件相关错误（如格式异常、缺少列等）
    #[error("source data file error: {0}")]
    SourceDataFileError(String),
    // 源数据后端引擎（calamine）错误
    #[error("source data backend error: {0}")]
    SourceDataBackendError(#[from] calamine::Error),

    // 数据处理过程中的逻辑错误
    #[error("data error: {0}")]
    DataError(String),
    // SQLite 数据库操作错误
    #[error("sqlite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    // 数据尚未准备就绪时被访问
    #[error("data not ready: {0}")]
    DataNotReady(String),

    // 其他未分类错误
    #[error("other error: {0}")]
    Other(String),
}
