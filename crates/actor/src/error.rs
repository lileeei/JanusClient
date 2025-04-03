use std::error::Error;

/// Actor 错误类型
#[derive(Debug, Clone)]
pub enum ActorError {
    /// 消息处理错误
    Handler(String),
    /// 监督错误
    Supervision(String),
    /// 系统错误
    System(String),
    /// Actor 恐慌
    Panic(String),
    /// 用户自定义错误
    User(Box<dyn Error + Send + Sync>),
}

/// 消息发送错误
#[derive(Debug, Clone)]
pub enum SendError {
    /// 接收方已关闭
    Closed,
    /// 发送已取消
    Canceled,
    /// 没有消息处理器
    NoHandler,
} 