use std::time::Duration;
use crate::context::ActorContext;
use crate::message::Message;

/// Actor 基础特质，定义 Actor 的基本行为
pub trait Actor: Send + 'static {
    /// 关联的上下文类型
    type Context: ActorContext;

    /// Actor 启动时调用
    fn started(&mut self, ctx: &mut Self::Context) {}

    /// Actor 停止前调用
    fn stopping(&mut self, ctx: &mut Self::Context) {}

    /// Actor 彻底停止后调用
    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

/// 消息处理器特质
pub trait Handler<M: Message>: Actor {
    /// 处理消息的结果类型
    type Result: Into<M::Result>;

    /// 处理消息
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Result;
}

/// Actor 单元，包含 Actor 实例及其上下文
pub(crate) struct ActorCell<A: Actor> {
    // Actor 实例
    pub actor: Option<A>,
    // Actor 上下文
    pub context: A::Context,
    // 生命周期状态
    pub state: ActorState,
    // 失败计数
    pub failure_count: u32,
}

/// Actor 状态
#[derive(PartialEq)]
pub enum ActorState {
    // 初始化中
    Initializing,
    // 运行中
    Running,
    // 停止中
    Stopping,
    // 已停止
    Stopped,
    // 失败
    Failed,
    // 重启中
    Restarting,
} 