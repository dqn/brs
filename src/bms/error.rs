use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum BmsError {
    #[error("Failed to read BMS file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse BMS file: {0}")]
    Parse(String),

    #[error("Invalid timing data: {0}")]
    InvalidTiming(String),

    #[error("Keysound not found: {id}")]
    KeysoundNotFound { id: u32 },

    #[error("Unsupported play mode: {mode}")]
    UnsupportedPlayMode { mode: String },
}
