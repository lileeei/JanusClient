use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use dashmap::DashMap;

use crate::actor::Actor;
use crate::address::{ActorPath, ActorRef};
use crate::execution::ExecutionContext;

/// Actor 系统配置
#[derive(Default)]
pub struct ActorSystemConfig {
    pub thread_pool_size: usize,
}

/// Actor 系统，管理所有 Actor
pub struct ActorSystem {
    // 系统名称
    name: String,
    // 根 Actor 引用
    root: ActorRef<RootActor>,
    // 执行上下文
    execution_context: ExecutionContext,
    // Actor 路径到 ActorRef 的映射
    actors_by_path: DashMap<ActorPath, Box<dyn AnyActorRef>>,
    // 系统关闭标志
    shutdown_flag: Arc<AtomicBool>,
    // 系统守护 Actor 注册表
    guardian_actors: DashMap<String, Box<dyn AnyActorRef>>,
    // 配置
    config: ActorSystemConfig,
}

impl ActorSystem {
    /// 创建新的 Actor 系统
    pub fn new(name: &str, config: ActorSystemConfig) -> Self {
        // 初始化执行上下文
        let execution_context = ExecutionContext::new(config.thread_pool_size);

        // 创建根 Actor
        let root_path = ActorPath::root(name);
        let (root, _) = Self::create_root_actor(&root_path, execution_context.clone());

        let actors_by_path = DashMap::new();
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let guardian_actors = DashMap::new();

        Self {
            name: name.to_string(),
            root,
            execution_context,
            actors_by_path,
            shutdown_flag,
            guardian_actors,
            config,
        }
    }

    /// 创建顶层 Actor
    pub fn create_actor<A, F>(&self, name: &str, factory: F) -> ActorRef<A>
    where
        A: Actor,
        F: FnOnce() -> A + 'static,
    {
        // 顶层 Actor 是根 Actor 的直接子 Actor
        let path = self.root.path.child(name);

        // 创建 Actor
        self.spawn_actor_at_path(path, factory)
    }

    /// 在指定路径创建 Actor
    pub(crate) fn spawn_actor_at_path<A, F>(&self, path: ActorPath, factory: F) -> ActorRef<A>
    where
        A: Actor,
        F: FnOnce() -> A + 'static,
    {
        // 创建 Actor 单元
        let (actor_ref, _) = self.create_actor_cell(path, factory);

        actor_ref
    }

    /// 关闭 Actor 系统
    pub async fn shutdown(&self) {
        // 设置关闭标志
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // 停止所有 Actor
        self.stop_all_actors().await;

        // 等待执行上下文关闭
        self.execution_context.shutdown().await;
    }

    /// 通过路径查找 Actor
    pub(crate) fn actor_by_path(&self, path: &ActorPath) -> Option<Box<dyn AnyActorRef>> {
        self.actors_by_path.get(path).map(|r| r.clone())
    }

    // 其他内部方法...
} 