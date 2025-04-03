use std::time::Duration;
use std::time::Instant;
use crate::actor::ActorId;

/// 消息特质
pub trait Message: Send + 'static {
    /// 消息处理的结果类型
    type Result: Send;
}

/// 信封，包含消息及可选的响应发送器
pub(crate) struct Envelope {
    // 类型擦除的消息
    pub message: Box<dyn EnvelopeMessage>,
    // 创建时间
    pub created_at: Instant,
}

/// 信封消息接口（类型擦除）
pub(crate) trait EnvelopeMessage: Send {
    fn handle(&mut self, actor: &mut dyn ActorHandler, ctx: &mut dyn ActorContextHandler);
}

/// 系统消息
pub enum SystemMessage {
    /// 停止 Actor
    Stop,
    /// 重启 Actor
    Restart {
        max_retries: Option<u32>,
        reset_window: Option<Duration>,
    },
    /// 子 Actor 终止
    ChildTerminated(ActorId),
    /// 监督事件
    Supervision(SupervisionEvent),
}

/// 监督事件
pub enum SupervisionEvent {
    /// 子 Actor 失败
    ChildFailure {
        child_id: ActorId,
        error: ActorError,
    },
    /// 子 Actor 重启
    ChildRestarted { child_id: ActorId },
    /// 子 Actor 已停止
    ChildStopped { child_id: ActorId },
}

/// 定时任务消息包装
pub(crate) struct ScheduledTask<F>(pub F); 