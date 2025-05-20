use std::str::Utf8Error;
use thiserror::Error;
use uuid::Error as UuidError;

#[derive(Error, Debug)]
pub enum MinestomError {
    #[error("JNI error: {0}")]
    Jni(#[from] jni::errors::Error),

    #[error("JVM initialization error: {0}")]
    JvmInit(String),

    #[error("JVM error: {0}")]
    JvmError(String),

    #[error("Invalid path: could not convert path to string")]
    InvalidPath,

    #[error("World loading error: {0}")]
    WorldLoading(String),

    #[error("Invalid instance: {0}")]
    InvalidInstance(String),

    #[error("Invalid player: {0}")]
    InvalidPlayer(String),

    #[error("Event error: {0}")]
    EventError(String),

    #[error("Command error: {0}")]
    CommandError(String),

    #[error("Coordinate error: {0}")]
    CoordinateError(String),

    #[error("Text component error: {0}")]
    TextError(String),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("UUID error: {0}")]
    UuidError(#[from] UuidError),
}
