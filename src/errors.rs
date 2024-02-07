use failure::Fail;
use std::io;

#[derive(Fail, Debug)]
pub enum KvStoreError {
    #[fail(display = "IO error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Serialization error: {}", _0)]
    SerdeError(#[cause] serde_json::Error),
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "Invalid command")]
    InvalidCommand,
    #[fail(display = "Invalid File Header")]
    InvalidFileHeader,
}

pub type BackendResult<T> = std::result::Result<T, KvStoreError>;
