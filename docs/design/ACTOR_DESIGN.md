# Actor 模式设计

## 1. 概述

引入 Actor 模式可以帮助我们更好地处理：
- 并发操作
- 状态管理
- 消息传递
- 错误隔离
- 资源管理

## 2. Actor 系统架构

```
┌─────────────────────────────────────────────────────┐
│                  Supervisor Actor                    │
├─────────────────────────────────────────────────────┤
│   ┌─────────────┐    ┌─────────────┐    ┌────────┐  │
│   │Browser Actor│    │Session Actor│    │Plugin  │  │
│   │            │    │            │    │Actor   │  │
│   └─────────────┘    └─────────────┘    └────────┘  │
├─────────────────────────────────────────────────────┤
│   ┌─────────────┐    ┌─────────────┐    ┌────────┐  │
│   │Event Actor  │    │Command Actor│    │Monitor │  │
│   │            │    │            │    │Actor   │  │
│   └─────────────┘    └─────────────┘    └────────┘  │
└─────────────────────────────────────────────────────┘
```

## 3. 核心 Actor 定义

### 3.1 Supervisor Actor
```rust
pub struct SupervisorActor {
    browser_actors: HashMap<BrowserId, Addr<BrowserActor>>,
    session_actors: HashMap<SessionId, Addr<SessionActor>>,
    plugin_actors: HashMap<PluginId, Addr<PluginActor>>,
    event_actor: Addr<EventActor>,
    command_actor: Addr<CommandActor>,
    monitor_actor: Addr<MonitorActor>,
}

impl Actor for SupervisorActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // 初始化子 Actor
        // 设置监控策略
        // 启动必要的服务
    }
}
```

### 3.2 Browser Actor
```rust
pub struct BrowserActor {
    browser_type: BrowserType,
    connection: Box<dyn Connection>,
    state: BrowserState,
    event_handler: Addr<EventActor>,
}

impl Actor for BrowserActor {
    type Context = Context<Self>;
}

// 消息定义
pub enum BrowserMessage {
    Connect(String),
    Disconnect,
    ExecuteCommand(Command),
    HandleEvent(Event),
}

impl Handler<BrowserMessage> for BrowserActor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: BrowserMessage, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            BrowserMessage::Connect(url) => self.handle_connect(url),
            BrowserMessage::Disconnect => self.handle_disconnect(),
            BrowserMessage::ExecuteCommand(cmd) => self.handle_command(cmd),
            BrowserMessage::HandleEvent(event) => self.handle_event(event),
        }
    }
}
```

### 3.3 Session Actor
```rust
pub struct SessionActor {
    session_id: SessionId,
    browser_actor: Addr<BrowserActor>,
    state: SessionState,
}

impl Actor for SessionActor {
    type Context = Context<Self>;
}

// 消息定义
pub enum SessionMessage {
    Create,
    Destroy,
    Execute(Command),
    UpdateState(SessionState),
}
```

### 3.4 Event Actor
```rust
pub struct EventActor {
    subscribers: HashMap<String, Vec<Recipient<EventMessage>>>,
    event_buffer: VecDeque<Event>,
}

impl Actor for EventActor {
    type Context = Context<Self>;
}

// 消息定义
pub enum EventMessage {
    Subscribe(String, Recipient<EventMessage>),
    Unsubscribe(String, Recipient<EventMessage>),
    Publish(Event),
}
```

## 4. 层级整合

### 4.1 L1 统一接口层
```rust
pub trait BrowserInterface {
    fn get_browser_actor(&self) -> Addr<BrowserActor>;
    fn get_session_actor(&self) -> Addr<SessionActor>;
    fn get_event_actor(&self) -> Addr<EventActor>;
}
```

### 4.2 L2 浏览器实现层
```rust
pub struct ChromeActorSystem {
    supervisor: Addr<SupervisorActor>,
    chrome_actor: Addr<BrowserActor>,
    session_actors: HashMap<SessionId, Addr<SessionActor>>,
}

impl BrowserInterface for ChromeActorSystem {
    // 实现接口方法
}
```

### 4.3 L3 连接层
```rust
pub struct ConnectionActor {
    connection: Box<dyn Connection>,
    state: ConnectionState,
}

impl Actor for ConnectionActor {
    type Context = Context<Self>;
}
```

## 5. 消息流

### 5.1 命令执行流程
```
Client -> BrowserActor -> CommandActor -> ConnectionActor -> Browser
Browser -> ConnectionActor -> EventActor -> Subscribers
```

### 5.2 事件处理流程
```
Browser -> ConnectionActor -> EventActor -> BrowserActor/SessionActor/PluginActor
```

## 6. 错误处理

### 6.1 Actor 级别错误处理
```rust
impl SupervisorStrategy for SupervisorActor {
    fn strategy(&mut self, child: &mut Addr<impl Actor>, error: &Error) -> SupervisorAction {
        match error {
            Error::Connection(_) => SupervisorAction::Restart,
            Error::Protocol(_) => SupervisorAction::Resume,
            Error::Fatal(_) => SupervisorAction::Stop,
            _ => SupervisorAction::Escalate,
        }
    }
}
```

### 6.2 消息错误处理
```rust
impl Handler<BrowserMessage> for BrowserActor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: BrowserMessage, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            BrowserMessage::Execute(cmd) => {
                if let Err(e) = self.execute_command(cmd) {
                    // 错误处理策略
                    ctx.notify(SupervisorMessage::HandleError(e));
                    return Err(e);
                }
                Ok(())
            }
            // ... 其他消息处理
        }
    }
}
```

## 7. 状态管理

### 7.1 Actor 状态
```rust
#[derive(Debug, Clone)]
pub enum ActorState {
    Starting,
    Running,
    Stopping,
    Failed(Error),
}
```

### 7.2 状态转换
```rust
impl BrowserActor {
    fn transition_state(&mut self, new_state: ActorState) {
        self.state = new_state;
        self.notify_state_change();
    }
}
```

## 8. 实现计划

### 8.1 Phase 1: Actor 框架
- [ ] 实现基本的 Actor 特质
- [ ] 实现 Supervisor
- [ ] 实现基础消息类型

### 8.2 Phase 2: 核心 Actor
- [ ] 实现 BrowserActor
- [ ] 实现 SessionActor
- [ ] 实现 EventActor

### 8.3 Phase 3: 功能 Actor
- [ ] 实现 CommandActor
- [ ] 实现 PluginActor
- [ ] 实现 MonitorActor

### 8.4 Phase 4: 集成测试
- [ ] Actor 通信测试
- [ ] 错误恢复测试
- [ ] 性能测试 