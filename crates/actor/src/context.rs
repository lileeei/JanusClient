use std::collections::HashMap;
use std::time::Duration;
use dashmap::DashMap;

use crate::actor::Actor;
use crate::address::ActorRef;
use crate::supervision::SupervisionStrategy;
use crate::system::ActorSystem;

/// Actor 上下文，管理 Actor 的生命周期和消息处理
pub trait ActorContext {
    /// 关联的 Actor 类型
    type Actor: Actor<Context = Self>;

    /// 停止 Actor
    fn stop(&mut self);

    /// 获取 Actor 地址
    fn address(&self) -> ActorRef<Self::Actor>;

    /// 创建子 Actor
    fn spawn<A, F>(&mut self, name: &str, f: F) -> ActorRef<A>
    where
        A: Actor,
        F: FnOnce() -> A + 'static;

    /// 安排一个将来执行的操作
    fn schedule<F>(&mut self, duration: Duration, f: F)
    where
        F: FnOnce(&mut Self::Actor, &mut Self) + 'static;
}

/// 基础 Actor 上下文实现
pub struct BasicContext<A: Actor> {
    // Actor ID
    actor_id: ActorId,
    // Actor 引用
    actor_ref: ActorRef<A>,
    // Actor 系统
    system: ActorSystem,
    // 父 Actor 引用
    parent: Option<Box<dyn AnyActorRef>>,
    // 子 Actor 引用
    children: HashMap<String, Box<dyn AnyActorRef>>,
    // 监督策略
    supervision_strategy: SupervisionStrategy,
    // 消息中间件
    middlewares: Vec<Box<dyn MessageMiddleware>>,
    // 扩展存储
    extensions: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl<A: Actor<Context = Self>> ActorContext for BasicContext<A> {
    type Actor = A;

    fn stop(&mut self) {
        // 标记 Actor 为停止状态
        // 停止所有子 Actor
        for (_, child) in self.children.drain() {
            child.stop();
        }

        // 通知父 Actor（如果有）
        if let Some(parent) = &self.parent {
            parent.send_system_message(SystemMessage::ChildTerminated(self.actor_id));
        }
    }

    fn address(&self) -> ActorRef<A> {
        self.actor_ref.clone()
    }

    fn spawn<C, F>(&mut self, name: &str, factory: F) -> ActorRef<C>
    where
        C: Actor,
        F: FnOnce() -> C + 'static,
    {
        // 创建子 Actor 路径
        let child_path = self.actor_ref.path.child(name);

        // 使用系统创建 Actor
        let child_ref = self.system.spawn_actor_at_path(child_path, factory);

        // 保存子 Actor 引用
        self.children
            .insert(name.to_string(), Box::new(child_ref.clone()));

        child_ref
    }

    fn schedule<F>(&mut self, duration: Duration, f: F)
    where
        F: FnOnce(&mut A, &mut Self) + 'static,
    {
        let actor_ref = self.actor_ref.clone();

        // 创建一个定时器任务
        self.system.execution_context.schedule(duration, move || {
            actor_ref.do_send(ScheduledTask(Box::new(f))).ok();
        });
    }
} 