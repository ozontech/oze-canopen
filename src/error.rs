use crate::{canopen::RxMessage, transmitter::TxPacket};
use tokio::{
    sync::{broadcast, mpsc},
    time::error::Elapsed,
};

/// Custom error type for CANopen operations.
#[derive(Debug)]
pub enum CoError {
    /// I/O error.
    Io(std::io::Error),
    /// Timeout error.
    Timeout(Elapsed),
    /// SDO retry error with retry count.
    SdoRetryError(usize),
    /// Error from binrw crate.
    Binrw(binrw::Error),
    /// SDO received an unexpected answer.
    SdoWrongAnswer(String),
    /// Incorrect ID error.
    WrongId(String),
    /// Transmit packet timeout.
    TxPackerTimeout,
    /// Receive packet timeout.
    RxPackerTimeout,
    /// Error from the socketcan crate.
    SocketCan(socketcan::Error),
    /// Frame error with a detailed message.
    FrameError(String),
    /// Interface error with a detailed message.
    InterfaceError(String),
    /// Error indicating closure.
    Close,
}

impl From<binrw::Error> for CoError {
    fn from(error: binrw::Error) -> Self {
        CoError::Binrw(error)
    }
}

impl From<Elapsed> for CoError {
    fn from(error: Elapsed) -> Self {
        CoError::Timeout(error)
    }
}

impl From<mpsc::error::SendTimeoutError<TxPacket>> for CoError {
    fn from(_error: mpsc::error::SendTimeoutError<TxPacket>) -> Self {
        CoError::TxPackerTimeout
    }
}

impl From<broadcast::error::SendError<RxMessage>> for CoError {
    fn from(_error: broadcast::error::SendError<RxMessage>) -> Self {
        CoError::TxPackerTimeout
    }
}

impl From<std::io::Error> for CoError {
    fn from(error: std::io::Error) -> Self {
        CoError::Io(error)
    }
}

impl From<socketcan::Error> for CoError {
    fn from(error: socketcan::Error) -> Self {
        CoError::SocketCan(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coerror_conversions() {
        use std::io;

        let io_error: CoError = io::Error::new(io::ErrorKind::Other, "io error").into();
        assert!(matches!(io_error, CoError::Io(_)));
    }
}
