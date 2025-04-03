use std::any::Any;
use std::fmt;

/// Actor 唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId(pub(crate) u64);

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 任意 Actor 引用特质
pub trait AnyActorRef: Send + Sync {
    /// 停止 Actor
    fn stop(&self);

    /// 发送系统消息
    fn send_system_message(&self, msg: crate::message::SystemMessage);

    /// 获取消息处理器
    fn message_handler<M: crate::message::Message>(&self) -> Option<Box<dyn MessageHandler<M>>>;

    /// 转换为任意类型
    fn as_any(&self) -> &dyn Any;
}

/// 消息处理器特质
pub trait MessageHandler<M: crate::message::Message>: Send + Sync {
    /// 发送消息并等待结果
    fn send(&self, msg: M) -> impl std::future::Future<Output = Result<M::Result, crate::error::SendError>> + Send;

    /// 发送消息但不等待结果
    fn do_send(&self, msg: M) -> Result<(), crate::error::SendError>;
}

/// 消息中间件特质
pub trait MessageMiddleware: Send + Sync {
    /// 处理传入消息
    fn handle_incoming<M>(&self, msg: &M, ctx: &mut dyn Any) -> bool
    where
        M: crate::message::Message;

    /// 处理传出消息
    fn handle_outgoing<M>(&self, msg: &M, ctx: &mut dyn Any) -> bool
    where
        M: crate::message::Message;
}

/// 根 Actor
pub(crate) struct RootActor {
    id: ActorId,
} 