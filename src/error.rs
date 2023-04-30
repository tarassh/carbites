use thiserror::Error;

#[derive(Debug, Error)]
pub enum CarSplitterError {
    #[error("invalid file error: {0}")]
    InvalidFile(String),

    #[error("parsing the file error: {0}")]
    Parsing(String),

    #[error("invalid section error: {0}")]
    InvalidSection(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("too large section error: {0}")]
    TooLargeSection(usize),

    #[error("not found {0}")]
    NotFound(String),
}
