use std::future::Future;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// 执行上下文配置
#[derive(Default)]
pub struct ExecutionContextConfig {
    pub thread_pool_size: usize,
    pub scheduler_tick_duration: Duration,
}

/// 执行上下文，管理任务执行
pub struct ExecutionContext {
    // 线程池
    runtime: tokio::runtime::Runtime,
    // 调度器
    scheduler: Scheduler,
    // 关闭通知通道
    shutdown_tx: Option<mpsc::Sender<()>>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
    // 配置
    config: ExecutionContextConfig,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(thread_pool_size: usize) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(thread_pool_size)
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        Self {
            runtime,
            scheduler: Scheduler::new(),
            shutdown_tx: Some(shutdown_tx),
            shutdown_rx: Some(shutdown_rx),
            config: ExecutionContextConfig {
                thread_pool_size,
                scheduler_tick_duration: Duration::from_millis(100),
            },
        }
    }

    /// 派生任务
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// 调度定时任务
    pub fn schedule<F>(&self, duration: Duration, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.scheduler.schedule(duration, f);
    }

    /// 关闭执行上下文
    pub async fn shutdown(&mut self) {
        // 通知关闭
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // 等待所有任务完成
        self.runtime.shutdown_timeout(Duration::from_secs(10));
    }
}

/// 调度器
struct Scheduler {
    // 调度队列
    tasks: Vec<ScheduledTask>,
}

impl Scheduler {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    fn schedule<F>(&mut self, duration: Duration, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = ScheduledTask {
            duration,
            task: Box::new(f),
        };
        self.tasks.push(task);
    }
}

struct ScheduledTask {
    duration: Duration,
    task: Box<dyn FnOnce() + Send>,
} 