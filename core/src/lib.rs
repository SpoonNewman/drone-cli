use thiserror::Error;

pub mod transport;
pub mod drone;
pub mod api;
pub mod http_api;
pub mod dtos;

// #[derive(Debug, Error)]
// pub enum DroneError {
//     #[error("connection failed: {0}")]
//     Connect(String),
//     #[error("I/O error: {0}")]
//     Io(String),
//     #[error("protocol error: {0}")]
//     Protocol(String),
//     #[error("invalid state: {0}")]
//     State(String),
// }

// pub type Result<T> = std::result::Result<T, DroneError>;
