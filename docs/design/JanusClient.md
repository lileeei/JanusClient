## 1. 核心Actor模型

### 1.1 基础Actor特质

```rust
/// Actor基础特质，定义Actor的基本行为
pub trait Actor: Send + 'static {
    /// 关联的上下文类型
    type Context: ActorContext;

    /// Actor启动时调用
    fn started(&mut self, ctx: &mut Self::Context) {}

    /// Actor停止时调用
    fn stopping(&mut self, ctx: &mut Self::Context) {}

    /// Actor彻底停止后调用
    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

/// Actor上下文，管理Actor的生命周期和消息处理
pub trait ActorContext {
    /// 关联的Actor类型
    type Actor: Actor<Context = Self>;

    /// 停止Actor
    fn stop(&mut self);

    /// 获取Actor地址
    fn address(&self) -> ActorRef<Self::Actor>;

    /// 安排一个将来执行的操作
    fn schedule<F>(&mut self, duration: Duration, f: F)
    where
        F: FnOnce(&mut Self::Actor, &mut Self) + 'static;

    /// 创建子Actor
    fn spawn<A, F>(&mut self, name: &str, f: F) -> ActorRef<A>
    where
        A: Actor,
        F: FnOnce() -> A + 'static;
}
```

### 1.2 消息和处理器

```rust
/// 消息特质
pub trait Message: Send + 'static {
    /// 消息处理的结果类型
    type Result: Send;
}

/// 消息处理器特质
pub trait Handler<M: Message>: Actor {
    /// 处理消息的结果类型
    type Result: MessageResponse<M>;

    /// 处理消息
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Result;
}

/// 异步消息处理器特质
pub trait AsyncHandler<M: Message>: Actor {
    /// 异步处理消息
    fn handle_async(&mut self, msg: M, ctx: &mut Self::Context) -> BoxFuture<'static, M::Result>;
}
```

## 2. Actor引用和路径

```rust
/// Actor引用，用于发送消息
pub struct ActorRef<A: Actor> {
    // 内部消息发送器
    sender: mpsc::UnboundedSender<Envelope>,
    // Actor唯一标识符
    id: ActorId,
    // Actor路径
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

        self.sender.send(envelope)
            .map_err(|_| SendError::Closed)?;

        rx.await.map_err(|_| SendError::Canceled)
    }

    /// 发送消息但不等待结果
    pub fn do_send<M>(&self, msg: M) -> Result<(), SendError>
    where
        M: Message,
        A: Handler<M>,
    {
        let envelope = Envelope::new(msg, None);

        self.sender.send(envelope)
            .map_err(|_| SendError::Closed)
    }

    /// 获取Actor路径
    pub fn path(&self) -> &ActorPath {
        &self.path
    }
}

/// Actor路径
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

    /// 检查是否是某个路径的祖先
    pub fn is_ancestor_of(&self, other: &ActorPath) -> bool {
        if other.segments.len() <= self.segments.len() {
            return false;
        }

        self.segments.iter().zip(other.segments.iter())
            .all(|(a, b)| a == b)
    }

    /// 获取路径深度
    pub fn depth(&self) -> usize {
        self.segments.len()
    }
}
```

## 3. Actor系统

```rust
/// Actor系统，管理所有Actor
pub struct ActorSystem {
    // 系统名称
    name: String,
    // 根Actor引用
    root: ActorRef<RootActor>,
    // 执行上下文
    execution_context: ExecutionContext,
    // Actor路径到ActorRef的映射
    actors_by_path: DashMap<ActorPath, Box<dyn AnyActorRef>>,
    // 配置
    config: ActorSystemConfig,
}

impl ActorSystem {
    /// 创建新的Actor系统
    pub fn new(name: &str, config: ActorSystemConfig) -> Self {
        let execution_context = ExecutionContext::new(config.thread_pool_size);

        // 创建根Actor
        let root_path = ActorPath::root(name);
        let (root, _) = Self::create_root_actor(&root_path, execution_context.clone());

        let actors_by_path = DashMap::new();

        Self {
            name: name.to_string(),
            root,
            execution_context,
            actors_by_path,
            config,
        }
    }

    /// 在顶层创建Actor
    pub fn create_actor<A, F>(&self, name: &str, factory: F) -> ActorRef<A>
    where
        A: Actor,
        F: FnOnce() -> A + 'static,
    {
        // 顶层Actor是根Actor的直接子Actor
        let path = self.root.path().child(name);

        // 创建Actor
        self.spawn_actor_at_path(path, factory)
    }

    /// 获取Actor选择器
    pub fn actor_selection(&self, path: &str) -> Option<ActorSelection> {
        // 解析路径
        let path_obj = self.parse_path(path)?;

        Some(ActorSelection::new(path_obj, self.clone()))
    }

    // 其他内部方法...
}
```

## 4. 通过组织结构实现分层

### 4.1 通过组织实现L1-L3层级结构

```rust
// 组织结构:
// RootActor
// └── BrowserManagerActor (L1层)
//     ├── ChromeBrowserActor (L2层)
//     │   ├── ChromeConnectionActor (L3层)
//     │   ├── ChromeTabActor (L3层)
//     │   └── ChromeNetworkActor (L3层)
//     └── FirefoxBrowserActor (L2层)
//         ├── FirefoxConnectionActor (L3层)
//         ├── FirefoxTabActor (L3层)
//         └── FirefoxNetworkActor (L3层)

/// Actor系统构建器，用于便捷创建分层Actor结构
pub struct JanusActorSystemBuilder {
    system: ActorSystem,
}

impl JanusActorSystemBuilder {
    /// 创建新的构建器
    pub fn new(name: &str) -> Self {
        let system = ActorSystem::new(name, ActorSystemConfig::default());
        Self { system }
    }

    /// 构建完整的Actor系统
    pub fn build(self) -> ActorSystem {
        // 创建顶层L1组件: BrowserManagerActor
        let browser_manager = self.system.create_actor("browser_manager", || {
            BrowserManagerActor::new()
        });

        // 现在可以返回系统
        self.system
    }

    /// 添加Chrome浏览器支持
    pub fn with_chrome_support(self) -> Self {
        // 获取浏览器管理器
        if let Some(browser_manager) = self.system.actor_selection("/browser_manager") {
            // 添加Chrome浏览器
            let _ = browser_manager.tell(CreateBrowserActor {
                browser_type: BrowserType::Chrome,
                name: "chrome".to_string(),
            });
        }

        self
    }

    /// 添加Firefox浏览器支持
    pub fn with_firefox_support(self) -> Self {
        // 获取浏览器管理器
        if let Some(browser_manager) = self.system.actor_selection("/browser_manager") {
            // 添加Firefox浏览器
            let _ = browser_manager.tell(CreateBrowserActor {
                browser_type: BrowserType::Firefox,
                name: "firefox".to_string(),
            });
        }

        self
    }
}

// 用法示例
fn create_janus_system() -> ActorSystem {
    JanusActorSystemBuilder::new("janus")
        .with_chrome_support()
        .with_firefox_support()
        .build()
}
```

### 4.2 L1层Actor实现

```rust
/// 浏览器管理器Actor (L1层)
pub struct BrowserManagerActor {
    // 管理的浏览器
    browsers: HashMap<String, ActorRef<dyn BrowserActor>>,
    // 配置
    config: BrowserManagerConfig,
}

impl Actor for BrowserManagerActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("BrowserManagerActor started");
    }
}

impl Handler<CreateBrowserActor> for BrowserManagerActor {
    type Result = Result<ActorRef<dyn BrowserActor>, BrowserError>;

    fn handle(&mut self, msg: CreateBrowserActor, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Creating browser actor: {:?} - {}", msg.browser_type, msg.name);

        // 检查是否已存在同名浏览器
        if self.browsers.contains_key(&msg.name) {
            return Err(BrowserError::BrowserAlreadyExists(msg.name));
        }

        // 根据浏览器类型创建具体实现
        let browser_ref = match msg.browser_type {
            BrowserType::Chrome => {
                // 创建Chrome浏览器Actor（L2层）
                let chrome_ref = ctx.spawn(&msg.name, || {
                    ChromeBrowserActor::new(ChromeBrowserConfig::default())
                });

                // 安全地转换为通用BrowserActor引用
                chrome_ref.upcast()
            },
            BrowserType::Firefox => {
                // 创建Firefox浏览器Actor（L2层）
                let firefox_ref = ctx.spawn(&msg.name, || {
                    FirefoxBrowserActor::new(FirefoxBrowserConfig::default())
                });

                // 安全地转换为通用BrowserActor引用
                firefox_ref.upcast()
            },
            _ => return Err(BrowserError::UnsupportedBrowser(msg.browser_type)),
        };

        // 保存浏览器引用
        self.browsers.insert(msg.name.clone(), browser_ref.clone());

        Ok(browser_ref)
    }
}

impl Handler<GetBrowserActor> for BrowserManagerActor {
    type Result = Option<ActorRef<dyn BrowserActor>>;

    fn handle(&mut self, msg: GetBrowserActor, _ctx: &mut Self::Context) -> Self::Result {
        self.browsers.get(&msg.name).cloned()
    }
}

impl Handler<ListBrowsers> for BrowserManagerActor {
    type Result = Vec<BrowserInfo>;

    fn handle(&mut self, _msg: ListBrowsers, _ctx: &mut Self::Context) -> Self::Result {
        self.browsers.iter().map(|(name, _)| {
            BrowserInfo {
                name: name.clone(),
                // 其他信息可以通过向浏览器Actor发送消息获取
            }
        }).collect()
    }
}
```

### 4.3 L2层Actor实现

```rust
/// 浏览器Actor特质
pub trait BrowserActor: Actor {
    /// 获取浏览器类型
    fn browser_type(&self) -> BrowserType;

    /// 获取浏览器能力
    fn capabilities(&self) -> BrowserCapabilities;
}

/// Chrome浏览器Actor (L2层)
pub struct ChromeBrowserActor {
    // 浏览器配置
    config: ChromeBrowserConfig,
    // 浏览器状态
    state: BrowserState,
    // 会话管理
    sessions: HashMap<SessionId, ActorRef<ChromeSessionActor>>,
    // 连接Actor的引用
    connection: Option<ActorRef<ChromeConnectionActor>>,
    // 标签页Actor的引用
    tabs: HashMap<TabId, ActorRef<ChromeTabActor>>,
    // 网络Actor的引用
    network: Option<ActorRef<ChromeNetworkActor>>,
}

impl Actor for ChromeBrowserActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ChromeBrowserActor started");

        // 创建L3层连接Actor
        let connection_actor = ctx.spawn("connection", || {
            ChromeConnectionActor::new(self.config.connection.clone())
        });

        // 保存连接引用
        self.connection = Some(connection_actor);

        // 创建L3层网络Actor
        let network_actor = ctx.spawn("network", || {
            ChromeNetworkActor::new()
        });

        // 保存网络Actor引用
        self.network = Some(network_actor);
    }
}

impl BrowserActor for ChromeBrowserActor {
    fn browser_type(&self) -> BrowserType {
        BrowserType::Chrome
    }

    fn capabilities(&self) -> BrowserCapabilities {
        BrowserCapabilities {
            can_take_screenshot: true,
            can_intercept_network: true,
            supports_headless: true,
            // 其他能力...
        }
    }
}

impl Handler<CreateSession> for ChromeBrowserActor {
    type Result = Result<SessionId, BrowserError>;

    fn handle(&mut self, msg: CreateSession, ctx: &mut Self::Context) -> Self::Result {
        // 生成会话ID
        let session_id = SessionId::new();

        // 创建会话Actor
        let session_actor = ctx.spawn(&format!("session_{}", session_id), || {
            ChromeSessionActor::new(session_id, msg.options)
        });

        // 保存会话引用
        self.sessions.insert(session_id, session_actor);

        Ok(session_id)
    }
}

impl Handler<Navigate> for ChromeBrowserActor {
    type Result = Result<(), BrowserError>;

    fn handle(&mut self, msg: Navigate, _ctx: &mut Self::Context) -> Self::Result {
        // 检查是否有连接
        let connection = self.connection.as_ref()
            .ok_or(BrowserError::NotConnected)?;

        // 将导航命令转换为Chrome特定命令
        let chrome_command = NavigateCommand {
            url: msg.url,
            referrer: msg.referrer,
            transition_type: msg.transition_type,
            // 其他Chrome特定参数...
        };

        // 发送到连接Actor
        // 注意：这里简化为同步，实际应该是异步的
        connection.do_send(chrome_command)
            .map_err(|_| BrowserError::CommandFailed("Navigate failed".into()))?;

        Ok(())
    }
}
```

### 4.4 L3层Actor实现

```rust
/// Chrome连接Actor (L3层)
pub struct ChromeConnectionActor {
    // 连接配置
    config: ChromeConnectionConfig,
    // 连接状态
    state: ConnectionState,
    // WebSocket客户端
    ws_client: Option<WebSocketClient>,
    // 等待响应的命令
    pending_commands: HashMap<CommandId, oneshot::Sender<Result<Value, ConnectionError>>>,
}

impl Actor for ChromeConnectionActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ChromeConnectionActor started");

        // 如果配置了自动连接，则连接
        if self.config.auto_connect {
            ctx.schedule(Duration::from_millis(0), |actor, ctx| {
                // 发送连接消息给自己
                ctx.address().do_send(Connect).ok();
            });
        }
    }
}

impl Handler<Connect> for ChromeConnectionActor {
    type Result = ResponseFuture<Result<(), ConnectionError>>;

    fn handle(&mut self, _msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        // 检查是否已连接
        if self.state == ConnectionState::Connected {
            return Box::pin(future::ok(()));
        }

        // 更新状态
        self.state = ConnectionState::Connecting;

        // 构建WebSocket URL
        let ws_url = format!("ws://{}:{}/devtools/browser",
                           self.config.host, self.config.port);

        // 创建WebSocket客户端
        let ws_future = WebSocketClient::connect(&ws_url);

        // 克隆地址，用于回调
        let addr = ctx.address();

        Box::pin(async move {
            match ws_future.await {
                Ok(ws_client) => {
                    // 连接成功，更新客户端
                    addr.do_send(SetWebSocketClient {
                        client: ws_client
                    }).map_err(|_| ConnectionError::SendFailed)?;

                    // 启动消息接收循环
                    addr.do_send(StartReceiving)
                        .map_err(|_| ConnectionError::SendFailed)?;

                    Ok(())
                },
                Err(e) => {
                    // 连接失败
                    addr.do_send(ConnectionFailed {
                        error: e.to_string()
                    }).ok();

                    Err(ConnectionError::ConnectFailed(e.to_string()))
                },
            }
        })
    }
}

impl Handler<SendCommand> for ChromeConnectionActor {
    type Result = ResponseFuture<Result<Value, ConnectionError>>;

    fn handle(&mut self, msg: SendCommand, _ctx: &mut Self::Context) -> Self::Result {
        // 检查是否已连接
        if self.state != ConnectionState::Connected {
            return Box::pin(future::err(ConnectionError::NotConnected));
        }

        // 生成命令ID
        let command_id = CommandId::new();

        // 创建响应通道
        let (tx, rx) = oneshot::channel();

        // 保存待处理命令
        self.pending_commands.insert(command_id, tx);

        // 序列化命令
        let command_json = match serde_json::to_string(&msg.command) {
            Ok(json) => json,
            Err(e) => {
                // 序列化失败，移除待处理命令
                self.pending_commands.remove(&command_id);
                return Box::pin(future::err(ConnectionError::SerializationFailed(e.to_string())));
            }
        };

        // 发送WebSocket消息
        if let Some(ws) = &mut self.ws_client {
            if let Err(e) = ws.send(command_json) {
                // 发送失败，移除待处理命令
                self.pending_commands.remove(&command_id);
                return Box::pin(future::err(ConnectionError::SendFailed(e.to_string())));
            }
        } else {
            // 未连接
            self.pending_commands.remove(&command_id);
            return Box::pin(future::err(ConnectionError::NotConnected));
        }

        // 返回等待响应的Future
        Box::pin(async move {
            match rx.await {
                Ok(result) => result,
                Err(_) => Err(ConnectionError::ResponseChannelClosed),
            }
        })
    }
}

/// Chrome标签页Actor (L3层)
pub struct ChromeTabActor {
    // 标签页ID
    id: TabId,
    // 标签页状态
    state: TabState,
    // 父浏览器引用
    browser: ActorRef<ChromeBrowserActor>,
}

impl Actor for ChromeTabActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ChromeTabActor started: {}", self.id);
    }
}

/// Chrome网络Actor (L3层)
pub struct ChromeNetworkActor {
    // 网络请求监控状态
    monitoring: bool,
    // 网络请求拦截规则
    interception_rules: Vec<InterceptionRule>,
    // 父浏览器引用
    browser: ActorRef<ChromeBrowserActor>,
}

impl Actor for ChromeNetworkActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ChromeNetworkActor started");
    }
}
```

## 5. 消息与状态

### 5.1 消息定义

```rust
// 浏览器管理器消息
#[derive(Debug)]
pub struct CreateBrowserActor {
    pub browser_type: BrowserType,
    pub name: String,
}

impl Message for CreateBrowserActor {
    type Result = Result<ActorRef<dyn BrowserActor>, BrowserError>;
}

#[derive(Debug)]
pub struct GetBrowserActor {
    pub name: String,
}

impl Message for GetBrowserActor {
    type Result = Option<ActorRef<dyn BrowserActor>>;
}

#[derive(Debug)]
pub struct ListBrowsers;

impl Message for ListBrowsers {
    type Result = Vec<BrowserInfo>;
}

// 浏览器Actor消息
#[derive(Debug)]
pub struct CreateSession {
    pub options: SessionOptions,
}

impl Message for CreateSession {
    type Result = Result<SessionId, BrowserError>;
}

#[derive(Debug)]
pub struct Navigate {
    pub url: String,
    pub referrer: Option<String>,
    pub transition_type: Option<String>,
}

impl Message for Navigate {
    type Result = Result<(), BrowserError>;
}

// 连接Actor消息
#[derive(Debug)]
pub struct Connect;

impl Message for Connect {
    type Result = Result<(), ConnectionError>;
}

#[derive(Debug)]
pub struct SendCommand {
    pub command: ChromeCommand,
}

impl Message for SendCommand {
    type Result = Result<Value, ConnectionError>;
}

#[derive(Debug)]
pub struct SetWebSocketClient {
    pub client: WebSocketClient,
}

impl Message for SetWebSocketClient {
    type Result = ();
}
```

### 5.2 状态定义

```rust
// 浏览器状态
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserState {
    Initializing,
    Ready,
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    Error(String),
}

// 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error(String),
}

// 标签页状态
#[derive(Debug, Clone, PartialEq)]
pub enum TabState {
    Initializing,
    Loading,
    Ready,
    Closing,
    Closed,
    Error(String),
}
```

## 6. 监督与恢复

```rust
/// 监督策略
pub enum SupervisionStrategy {
    /// 停止Actor
    Stop,
    /// 重启Actor
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
    fn strategy(&self, error: &ActorError, child: &dyn std::any::Any) -> SupervisionStrategy;

    /// 处理子Actor错误
    fn handle_failure(&mut self, error: ActorError, child: ActorRef<dyn Actor>, ctx: &mut Self::Context) {
        let strategy = self.strategy(&error, &child);

        match strategy {
            SupervisionStrategy::Stop => {
                // 停止子Actor
                child.tell(Stop);
            },
            SupervisionStrategy::Restart { max_retries, reset_window } => {
                // 重启子Actor
                child.tell(Restart { max_retries, reset_window });
            },
            SupervisionStrategy::Resume => {
                // 不做任何事，让子Actor继续运行
            },
            SupervisionStrategy::Escalate => {
                // 将错误传递给父Actor
                if let Some(parent) = ctx.parent() {
                    parent.tell(ChildFailed {
                        child: child.clone(),
                        error: error.clone(),
                    });
                } else {
                    // 没有父Actor，作为根Actor处理
                    log::error!("Root actor received escalated error: {:?}", error);
                    // 默认重启子Actor
                    child.tell(Restart {
                        max_retries: Some(3),
                        reset_window: Some(Duration::from_secs(60)),
                    });
                }
            },
        }
    }
}

// 浏览器管理器作为L2层Actor的监督者
impl Supervisor for BrowserManagerActor {
    fn strategy(&self, error: &ActorError, child: &dyn std::any::Any) -> SupervisionStrategy {
        match error {
            ActorError::Connection(_) => SupervisionStrategy::Restart {
                max_retries: Some(3),
                reset_window: Some(Duration::from_secs(60)),
            },
            ActorError::Protocol(_) => SupervisionStrategy::Resume,
            ActorError::Fatal(_) => SupervisionStrategy::Stop,
            _ => SupervisionStrategy::Escalate,
        }
    }
}

// Chrome浏览器Actor作为L3层Actor的监督者
impl Supervisor for ChromeBrowserActor {
    fn strategy(&self, error: &ActorError, child: &dyn std::any::Any) -> SupervisionStrategy {
        match error {
            ActorError::Connection(_) => SupervisionStrategy::Restart {
                max_retries: Some(5),
                reset_window: Some(Duration::from_secs(120)),
            },
            ActorError::Session(_) => SupervisionStrategy::Resume,
            ActorError::Fatal(_) => SupervisionStrategy::Stop,
            _ => SupervisionStrategy::Escalate,
        }
    }
}
```

## 7. 使用示例

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建Actor系统
    let system = JanusActorSystemBuilder::new("janus")
        .with_chrome_support()
        .build();

    // 获取浏览器管理器
    let browser_manager = system.actor_selection("/browser_manager")
        .expect("Failed to find browser manager");

    // 获取Chrome浏览器Actor
    let chrome_browser = browser_manager.ask::<GetBrowserActor, _>(GetBrowserActor {
        name: "chrome".to_string(),
    }).await?.expect("Chrome browser not found");

    // 创建会话
    let session_id = chrome_browser.ask::<CreateSession, _>(CreateSession {
        options: SessionOptions::default(),
    }).await?;

    // 导航到URL
    chrome_browser.ask::<Navigate, _>(Navigate {
        url: "https://www.rust-lang.org".to_string(),
        referrer: None,
        transition_type: None,
    }).await?;

    // 使用完毕后关闭系统
    system.shutdown().await;

    Ok(())
}
```

## 8. 总结

通过这种设计，我们实现了一个干净的分层Actor系统：

2. **清晰的职责划分**：
   - L1层（BrowserManagerActor）：统一接口，管理所有浏览器类型
   - L2层（ChromeBrowserActor/FirefoxBrowserActor）：特定浏览器实现
   - L3层（ConnectionActor/TabActor/NetworkActor）：具体功能实现

3. **灵活的消息路由**：消息可以在不同层级间传递，而不需要显式地转换层级概念。

4. **自然的监督树**：每一层的Actor自然地成为其子Actor的监督者，提供错误隔离和恢复机制。

5. **代码组织清晰**：代码结构直观地反映系统的逻辑结构，便于理解和维护。

## 9. 高级扩展特性

### 9.1 消息路由与转发

```rust
/// 实现消息路由能力
pub trait MessageRouter: Actor {
    /// 路由消息到目标Actor
    fn route<M: Message>(&mut self, target: &str, msg: M, ctx: &mut Self::Context) -> Result<(), RoutingError>
    where
        Self: Handler<M>,
    {
        // 查找目标Actor
        let selection = ctx.system().actor_selection(target)
            .ok_or(RoutingError::ActorNotFound(target.to_string()))?;

        // 发送消息
        selection.tell(msg);

        Ok(())
    }

    /// 转发消息到多个目标Actor
    fn broadcast<M: Message + Clone>(&mut self, targets: &[&str], msg: M, ctx: &mut Self::Context) -> Result<(), RoutingError>
    where
        Self: Handler<M>,
    {
        for target in targets {
            self.route(target, msg.clone(), ctx)?;
        }

        Ok(())
    }
}

// 应用到浏览器管理器
impl MessageRouter for BrowserManagerActor {}

// 使用示例
impl Handler<BroadcastCommand> for BrowserManagerActor {
    type Result = Result<(), RoutingError>;

    fn handle(&mut self, msg: BroadcastCommand, ctx: &mut Self::Context) -> Self::Result {
        // 获取所有浏览器Actor的路径
        let browser_paths: Vec<String> = self.browsers.keys()
            .map(|name| format!("/browser_manager/{}", name))
            .collect();

        // 转发命令到所有浏览器
        self.broadcast(
            &browser_paths.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
            msg.command,
            ctx
        )
    }
}
```

### 9.2 Actor生命周期观察者

```rust
/// Actor生命周期事件
pub enum LifecycleEvent {
    Started(ActorRef<dyn Actor>),
    Stopping(ActorRef<dyn Actor>),
    Restarting(ActorRef<dyn Actor>),
    Stopped(ActorPath),
}

/// 生命周期观察者特质
pub trait LifecycleObserver: Actor {
    /// 处理生命周期事件
    fn on_lifecycle_event(&mut self, event: LifecycleEvent, ctx: &mut Self::Context);
}

/// 观察者Actor
pub struct ObserverActor {
    observed_paths: HashSet<ActorPath>,
    handlers: HashMap<ActorPath, Box<dyn Fn(&mut Self, LifecycleEvent, &mut BasicContext<Self>) + Send>>,
}

impl Actor for ObserverActor {
    type Context = BasicContext<Self>;
}

impl LifecycleObserver for ObserverActor {
    fn on_lifecycle_event(&mut self, event: LifecycleEvent, ctx: &mut Self::Context) {
        match &event {
            LifecycleEvent::Started(actor_ref) |
            LifecycleEvent::Stopping(actor_ref) |
            LifecycleEvent::Restarting(actor_ref) => {
                if self.observed_paths.contains(actor_ref.path()) {
                    if let Some(handler) = self.handlers.get(actor_ref.path()) {
                        handler(self, event.clone(), ctx);
                    }
                }
            },
            LifecycleEvent::Stopped(path) => {
                if self.observed_paths.contains(path) {
                    if let Some(handler) = self.handlers.get(path) {
                        handler(self, event.clone(), ctx);
                    }
                }
            }
        }
    }
}

impl ObserverActor {
    /// 观察指定Actor路径
    pub fn observe<F>(&mut self, path: ActorPath, handler: F)
    where
        F: Fn(&mut Self, LifecycleEvent, &mut BasicContext<Self>) + Send + 'static,
    {
        self.observed_paths.insert(path.clone());
        self.handlers.insert(path, Box::new(handler));
    }
}

impl Handler<Subscribe> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) -> Self::Result {
        self.observe(msg.path.clone(), msg.handler);
    }
}
```

### 9.3 消息拦截与中间件

```rust
/// 消息中间件特质
pub trait MessageMiddleware: Send + Sync {
    /// 处理入站消息
    fn on_incoming<M: Message>(&self, msg: &mut M, ctx: &dyn ActorContext) -> Result<(), MiddlewareError>;

    /// 处理出站消息
    fn on_outgoing<M: Message>(&self, result: &mut M::Result, ctx: &dyn ActorContext) -> Result<(), MiddlewareError>;
}

/// 日志中间件
pub struct LoggingMiddleware {
    log_level: log::Level,
}

impl MessageMiddleware for LoggingMiddleware {
    fn on_incoming<M: Message>(&self, msg: &mut M, ctx: &dyn ActorContext) -> Result<(), MiddlewareError> {
        log::log!(self.log_level, "Incoming message to {}: {:?}", ctx.path(), msg);
        Ok(())
    }

    fn on_outgoing<M: Message>(&self, result: &mut M::Result, ctx: &dyn ActorContext) -> Result<(), MiddlewareError> {
        log::log!(self.log_level, "Outgoing result from {}: {:?}", ctx.path(), result);
        Ok(())
    }
}

/// 性能跟踪中间件
pub struct TracingMiddleware {
    min_duration_to_log: Duration,
}

impl MessageMiddleware for TracingMiddleware {
    fn on_incoming<M: Message>(&self, msg: &mut M, ctx: &dyn ActorContext) -> Result<(), MiddlewareError> {
        // 在消息上附加时间戳
        ctx.extensions_mut().insert(MessageStartTime(Instant::now()));
        Ok(())
    }

    fn on_outgoing<M: Message>(&self, _result: &mut M::Result, ctx: &dyn ActorContext) -> Result<(), MiddlewareError> {
        if let Some(start_time) = ctx.extensions().get::<MessageStartTime>() {
            let duration = start_time.0.elapsed();
            if duration >= self.min_duration_to_log {
                log::warn!("Message processing in {} took {:?}", ctx.path(), duration);
            }
        }
        Ok(())
    }
}

// 在上下文中应用中间件
impl<A: Actor> BasicContext<A> {
    /// 添加消息中间件
    pub fn add_middleware<M: MessageMiddleware + 'static>(&mut self, middleware: M) {
        self.middlewares.push(Box::new(middleware));
    }

    /// 应用入站中间件
    fn apply_incoming_middleware<M: Message>(&self, msg: &mut M) -> Result<(), MiddlewareError> {
        for middleware in &self.middlewares {
            middleware.on_incoming(msg, self)?;
        }
        Ok(())
    }

    /// 应用出站中间件
    fn apply_outgoing_middleware<M: Message>(&self, result: &mut M::Result) -> Result<(), MiddlewareError> {
        for middleware in &self.middlewares {
            middleware.on_outgoing(result, self)?;
        }
        Ok(())
    }
}
```

### 9.4 异步消息流处理

```rust
/// 消息流处理特质
pub trait MessageStream<T: Message>: Actor {
    /// 处理消息流元素
    fn handle_element(&mut self, element: T, ctx: &mut Self::Context) -> Result<(), StreamError>;

    /// 处理流结束
    fn handle_completion(&mut self, ctx: &mut Self::Context);

    /// 处理流错误
    fn handle_error(&mut self, error: StreamError, ctx: &mut Self::Context);
}

/// 启动消息流处理
pub fn process_stream<A, T, S>(actor: &ActorRef<A>, stream: S)
where
    A: MessageStream<T>,
    T: Message,
    S: Stream<Item = T> + Send + 'static,
{
    let addr = actor.clone();

    tokio::spawn(async move {
        let mut stream = stream;

        while let Some(element) = stream.next().await {
            match addr.send(StreamElement(element)).await {
                Ok(result) => {
                    if let Err(e) = result {
                        addr.do_send(StreamError(e)).ok();
                        break;
                    }
                },
                Err(_) => break, // Actor可能已停止
            }
        }

        // 流结束
        addr.do_send(StreamCompleted).ok();
    });
}

// 应用到事件流处理
impl MessageStream<BrowserEvent> for EventProcessorActor {
    fn handle_element(&mut self, event: BrowserEvent, ctx: &mut Self::Context) -> Result<(), StreamError> {
        // 处理事件
        match event {
            BrowserEvent::PageLoaded { url } => {
                // 通知所有监听加载事件的Actor
                for listener in self.listeners.get("page_loaded").unwrap_or(&Vec::new()) {
                    listener.do_send(PageLoadedEvent { url: url.clone() }).ok();
                }
            },
            BrowserEvent::NetworkRequest { request_id, url, method } => {
                // 通知所有监听网络请求的Actor
                for listener in self.listeners.get("network_request").unwrap_or(&Vec::new()) {
                    listener.do_send(NetworkRequestEvent {
                        request_id,
                        url: url.clone(),
                        method: method.clone()
                    }).ok();
                }
            },
            // 处理其他事件类型...
        }

        Ok(())
    }

    fn handle_completion(&mut self, ctx: &mut Self::Context) {
        log::info!("Event stream completed");
        // 通知所有监听器流已结束
        self.notify_all_listeners(StreamEndedEvent);
    }

    fn handle_error(&mut self, error: StreamError, ctx: &mut Self::Context) {
        log::error!("Event stream error: {:?}", error);
        // 通知所有监听器发生错误
        self.notify_all_listeners(StreamErrorEvent { error });
    }
}
```

### 9.5 Actor系统指标与监控

```rust
/// Actor系统指标收集器
pub struct MetricsCollector {
    metrics: HashMap<String, Metric>,
    report_interval: Duration,
    prometheus_registry: Option<prometheus::Registry>,
}

impl Actor for MetricsCollector {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // 定期收集指标
        ctx.schedule(self.report_interval, |actor, ctx| {
            actor.collect_metrics(ctx);
            // 重新调度
            ctx.schedule(actor.report_interval, Self::started);
        });
    }
}

impl MetricsCollector {
    /// 收集系统中所有Actor的指标
    fn collect_metrics(&mut self, ctx: &mut BasicContext<Self>) {
        // 获取系统中所有Actor
        let actors = ctx.system().list_all_actors();

        // 更新Actor计数指标
        self.update_metric("actor_count", actors.len() as f64);

        // 按类型统计Actor
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for actor in &actors {
            let type_name = actor.type_name().to_string();
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        // 更新类型计数指标
        for (type_name, count) in type_counts {
            self.update_metric(&format!("actor_count_{}", type_name), count as f64);
        }

        // 更新消息指标
        self.update_metric("messages_processed", ctx.system().messages_processed() as f64);
        self.update_metric("messages_dropped", ctx.system().messages_dropped() as f64);

        // 如果启用了Prometheus，更新Prometheus指标
        if let Some(registry) = &self.prometheus_registry {
            // ...更新Prometheus指标
        }

        // 记录指标
        log::info!("Actor system metrics: {:?}", self.metrics);
    }

    /// 更新指标
    fn update_metric(&mut self, name: &str, value: f64) {
        self.metrics.entry(name.to_string())
            .or_insert_with(|| Metric::new(name))
            .update(value);
    }
}
```

## 10. 完整的例子：创建Chrome浏览器自动化系统

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    // 创建Actor系统
    let system = JanusActorSystemBuilder::new("janus")
        .with_chrome_support()
        .build();

    // 获取浏览器管理器
    let browser_manager = system.actor_selection("/browser_manager")
        .expect("Failed to find browser manager");

    // 获取Chrome浏览器Actor
    let chrome_browser = browser_manager.ask::<GetBrowserActor, _>(GetBrowserActor {
        name: "chrome".to_string(),
    }).await?.expect("Chrome browser not found");

    // 连接到浏览器
    chrome_browser.ask::<Connect, _>(Connect).await?;

    // 创建会话
    let session_id = chrome_browser.ask::<CreateSession, _>(CreateSession {
        options: SessionOptions {
            headless: true,
            ..Default::default()
        },
    }).await?;

    // 获取会话Actor
    let session = chrome_browser.ask::<GetSession, _>(GetSession {
        session_id,
    }).await?.expect("Session not found");

    // 创建新标签页
    let tab_id = session.ask::<CreateTab, _>(CreateTab).await?;

    // 获取标签页Actor
    let tab = session.ask::<GetTab, _>(GetTab {
        tab_id,
    }).await?.expect("Tab not found");

    // 导航到URL
    tab.ask::<Navigate, _>(Navigate {
        url: "https://www.rust-lang.org".to_string(),
        referrer: None,
        transition_type: None,
    }).await?;

    // 等待页面加载完成
    tab.ask::<WaitForLoad, _>(WaitForLoad {
        timeout: Some(Duration::from_secs(30)),
    }).await?;

    // 截图
    let screenshot = tab.ask::<TakeScreenshot, _>(TakeScreenshot {
        format: "png".to_string(),
        full_page: true,
    }).await?;

    // 保存截图
    std::fs::write("screenshot.png", screenshot)?;
    println!("Screenshot saved to screenshot.png");

    // 执行JavaScript
    let result = tab.ask::<ExecuteScript, _>(ExecuteScript {
        script: r#"
            document.title + " - " +
            document.querySelector('h1').textContent
        "#.to_string(),
    }).await?;

    println!("Page info: {}", result);

    // 关闭标签页
    tab.ask::<CloseTab, _>(CloseTab).await?;

    // 关闭会话
    session.ask::<CloseSession, _>(CloseSession).await?;

    // 断开连接
    chrome_browser.ask::<Disconnect, _>(Disconnect).await?;

    // 关闭系统
    system.shutdown().await;

    Ok(())
}
```

## 11. 结论

通过这种基于组织结构的分层Actor设计，我们实现了：

1. **自然的层级划分**：通过Actor的父子关系自然地表达了L1-L3的层级结构，而不需要在Actor内部硬编码层级概念。

2. **关注点分离**：每一层的Actor都有明确的职责：
   - L1层关注统一的浏览器抽象
   - L2层关注特定浏览器的实现逻辑
   - L3层关注具体功能和连接细节

3. **消息驱动的架构**：整个系统完全基于消息传递，使代码更加模块化，更容易测试。

4. **强大的错误隔离**：通过监督树，可以在适当的层级处理错误，防止故障蔓延。

5. **灵活的可扩展性**：可以轻松添加新的浏览器类型、新的功能模块，而不影响现有代码。
