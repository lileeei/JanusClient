use std::time::Duration;
use crate::actor::Actor;
use crate::address::ActorRef;
use crate::error::ActorError;
use crate::message::SystemMessage;

/// 监督策略
pub enum SupervisionStrategy {
    /// 停止 Actor
    Stop,
    /// 重启 Actor
    Restart {
        max_retries: Option<u32>,
        reset_window: Option<Duration>,
    },
    /// 忽略错误，继续运行
    Resume,
    /// 升级错误到父级
    Escalate,
}

/// 监督者特质
pub trait Supervisor: Actor {
    /// 获取监督策略
    fn strategy(&self, error: &ActorError, child: ActorRef<dyn Actor>) -> SupervisionStrategy;

    /// 处理子 Actor 错误
    fn handle_failure(
        &mut self,
        error: ActorError,
        child: ActorRef<dyn Actor>,
        ctx: &mut Self::Context,
    ) {
        // 获取策略
        let strategy = self.strategy(&error, child.clone());

        match strategy {
            SupervisionStrategy::Stop => {
                // 停止子 Actor
                child.do_send(SystemMessage::Stop).ok();
            }
            SupervisionStrategy::Restart {
                max_retries,
                reset_window,
            } => {
                // 重启子 Actor
                child
                    .do_send(SystemMessage::Restart {
                        max_retries,
                        reset_window,
                    })
                    .ok();
            }
            SupervisionStrategy::Resume => {
                // 不做任何事，让子 Actor 继续运行
            }
            SupervisionStrategy::Escalate => {
                // 将错误传递给父 Actor
                if let Some(parent) = ctx.parent() {
                    parent
                        .do_send(SystemMessage::Supervision(SupervisionEvent::ChildFailure {
                            child: child.clone(),
                            error: error.clone(),
                        }))
                        .ok();
                }
            }
        }
    }
} 