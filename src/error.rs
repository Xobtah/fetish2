use std::io;

use tdlib::types::Error;
use tokio::{sync::broadcast::error::SendError, task::JoinError};

pub type FetishResult<T> = Result<T, FetishError>;

#[derive(Debug)]
pub enum FetishError {
    Td(Error),
    Join(JoinError),
    Io(io::Error),
    TokioSend(SendError<()>),
    MessageHandle,
}

impl From<Error> for FetishError {
    fn from(error: Error) -> Self {
        FetishError::Td(error)
    }
}

impl From<JoinError> for FetishError {
    fn from(error: JoinError) -> Self {
        FetishError::Join(error)
    }
}

impl From<io::Error> for FetishError {
    fn from(error: io::Error) -> Self {
        FetishError::Io(error)
    }
}

impl From<SendError<()>> for FetishError {
    fn from(error: SendError<()>) -> Self {
        FetishError::TokioSend(error)
    }
}
