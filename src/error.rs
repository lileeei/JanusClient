use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

/// Debugger errors that can occur during browser automation
#[derive(Error, Debug)]
pub enum DebuggerError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Page error: {0}")]
    PageError(String),

    #[error("DOM error: {0}")]
    DomError(String),

    #[error("JavaScript error: {0}")]
    JavaScriptError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 错误结果类型
pub type Result<T> = std::result::Result<T, DebuggerError>;

impl From<std::io::Error> for DebuggerError {
    fn from(err: std::io::Error) -> Self {
        DebuggerError::NetworkError(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for DebuggerError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        DebuggerError::NetworkError(err.to_string())
    }
}

impl From<url::ParseError> for DebuggerError {
    fn from(err: url::ParseError) -> Self {
        DebuggerError::InvalidArgument(format!("URL解析错误: {}", err))
    }
}

impl<T> From<SendError<T>> for DebuggerError {
    fn from(err: SendError<T>) -> Self {
        DebuggerError::NetworkError(err.to_string())
    }
} 