//! Error types and `Result` for the Pixli engine.
//!
//! All fallible public APIs return `Result<T, Error>` so that applications
//! can handle failures (e.g. missing GPU, unsupported backend) without panicking.

/// Result type for Pixli operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during engine initialization or runtime.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("event loop: {0}")]
    EventLoop(String),
    #[error("window: {0}")]
    Window(String),
    #[error("no compatible GPU adapter found")]
    NoAdapter,
    #[error("GPU device: {0}")]
    DeviceRequest(String),
    #[error("surface: {0}")]
    Surface(String),
    #[error("run loop: {0}")]
    RunLoop(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}
