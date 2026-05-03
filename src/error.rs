use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error{
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("file io error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("pipeline error: {0}")]
    PipelineError(String),

    #[error("source data file error: {0}")]
    SourceDataFileError(String),
    #[error("source data backend error: {0}")]
    SourceDataBackendError(#[from] calamine::Error),

    #[error("data error: {0}")]
    DataError(String),
    #[error("database error: {0}")]
    DataBaseError(#[from] mysql::Error),
    #[error("sqlite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("data not ready: {0}")]
    DataNotReady(String),

    #[error("other error: {0}")]
    Other(String),
} 

