use std::marker::PhantomData;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::actor::{Actor, Handler};
use crate::message::{Message, Envelope};
use crate::error::SendError;
use crate::system::ActorSystem;

/// Actor 引用，用于发送消息
pub struct ActorRef<A: Actor> {
    // 内部消息发送器
    sender: mpsc::UnboundedSender<Envelope>,
    // Actor 唯一标识符
    id: ActorId,
    // Actor 路径
    path: ActorPath,
    // 幽灵数据
    _phantom: PhantomData<A>,
}

impl<A: Actor> ActorRef<A> {
    /// 发送消息并等待结果
    pub async fn send<M>(&self, msg: M) -> Result<M::Result, SendError>
    where
        M: Message,
        A: Handler<M>,
    {
        let (tx, rx) = oneshot::channel();

        let envelope = Envelope::new(msg, Some(tx));

        self.sender.send(envelope).map_err(|_| SendError::Closed)?;

        rx.await.map_err(|_| SendError::Canceled)
    }

    /// 发送消息但不等待结果
    pub fn do_send<M>(&self, msg: M) -> Result<(), SendError>
    where
        M: Message,
        A: Handler<M>,
    {
        let envelope = Envelope::new(msg, None);

        self.sender.send(envelope).map_err(|_| SendError::Closed)
    }
}

/// Actor 路径
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActorPath {
    // 路径段
    segments: Vec<String>,
    // 完整路径字符串
    full_path: String,
}

impl ActorPath {
    /// 创建根路径
    pub fn root(name: &str) -> Self {
        let segments = vec![name.to_string()];
        let full_path = format!("/{}", name);

        Self {
            segments,
            full_path,
        }
    }

    /// 创建子路径
    pub fn child(&self, name: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(name.to_string());

        let full_path = format!("{}/{}", self.full_path, name);

        Self {
            segments,
            full_path,
        }
    }
}

/// Actor 选择器，用于通过路径查找 Actor
pub struct ActorSelection {
    // Actor 路径
    path: ActorPath,
    // Actor 系统
    system: ActorSystem,
}

impl ActorSelection {
    /// 创建新的 Actor 选择器
    pub fn new(path: ActorPath, system: ActorSystem) -> Self {
        Self { path, system }
    }

    /// 解析为特定类型的 Actor 引用
    pub fn resolve<A: Actor>(&self) -> Option<ActorRef<A>> {
        self.system
            .actor_by_path(&self.path)?
            .downcast_ref::<ActorRef<A>>()
            .cloned()
    }

    /// 发送消息且不等待结果
    pub fn tell<M: Message>(&self, msg: M) -> Result<(), SendError>
    where
        M: Clone,
    {
        // 查找匹配路径的所有 Actor，并发送消息
        if let Some(actor_ref) = self.system.actor_by_path(&self.path) {
            // 尝试发送消息
            if let Some(handler) = actor_ref.message_handler::<M>() {
                return handler.do_send(msg);
            }
        }

        Err(SendError::NoHandler)
    }

    /// 发送消息并等待结果
    pub async fn ask<M: Message, R>(&self, msg: M) -> Result<R, SendError>
    where
        M: Clone,
        M::Result: TryInto<R, Error = SendError>,
    {
        // 查找匹配路径的 Actor，并发送消息
        if let Some(actor_ref) = self.system.actor_by_path(&self.path) {
            // 尝试发送消息
            if let Some(handler) = actor_ref.message_handler::<M>() {
                let result = handler.send(msg).await?;
                return result.try_into();
            }
        }

        Err(SendError::NoHandler)
    }
} 