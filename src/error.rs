use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

/// 错误结果类型
pub type Result<T> = std::result::Result<T, FdpError>;

/// Firefox DevTools Protocol 客户端错误
#[derive(Error, Debug)]
pub enum FdpError {
    /// URL 解析错误
    #[error("URL解析错误: {0}")]
    InvalidUrl(String),
    
    /// 网络连接错误
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    /// JSON 解析/序列化错误
    #[error("协议错误: {0}")]
    ProtocolError(String),
    
    /// 无效的响应
    #[error("无效的响应: {0}")]
    InvalidResponse(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),

    /// 未连接错误
    #[error("未连接到浏览器")]
    NotConnected,
}

impl From<std::io::Error> for FdpError {
    fn from(err: std::io::Error) -> Self {
        FdpError::NetworkError(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for FdpError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        FdpError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for FdpError {
    fn from(err: serde_json::Error) -> Self {
        FdpError::ProtocolError(err.to_string())
    }
}

impl From<url::ParseError> for FdpError {
    fn from(err: url::ParseError) -> Self {
        FdpError::InvalidUrl(format!("URL解析错误: {}", err))
    }
}

impl<T> From<SendError<T>> for FdpError {
    fn from(err: SendError<T>) -> Self {
        FdpError::NetworkError(err.to_string())
    }
} 