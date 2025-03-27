# 分层架构设计

## 1. 架构概述

系统采用三层架构设计，每层都有其特定的职责和抽象级别：

```
┌─────────────────────────────────────┐
│           统一接口层 (L1)            │
│    Unified Interface Layer          │
├─────────────────────────────────────┤
│           浏览器实现层 (L2)          │
│    Browser Implementation Layer     │
├─────────────────────────────────────┤
│           连接传输层 (L3)            │
│    Connection Transport Layer       │
└─────────────────────────────────────┘
```

## 2. 层级详解

### 2.1 统一接口层 (L1)

#### 职责
- 提供统一的跨浏览器API
- 定义通用的操作接口
- 处理通用的错误类型
- 提供基础的类型系统

#### 核心接口
```rust
pub trait Browser {
    async fn navigate(&self, url: &str) -> Result<(), Error>;
    async fn evaluate_script(&self, script: &str) -> Result<Value, Error>;
    async fn take_screenshot(&self, format: &str) -> Result<Vec<u8>, Error>;
    // ... 其他通用操作
}

pub trait Page {
    async fn close(&self) -> Result<(), Error>;
    async fn reload(&self) -> Result<(), Error>;
    // ... 通用页面操作
}

pub trait Dom {
    async fn query_selector(&self, selector: &str) -> Result<Element, Error>;
    // ... 通用DOM操作
}
```

### 2.2 浏览器实现层 (L2)

#### 职责
- 实现特定浏览器的功能
- 处理浏览器特有的命令和事件
- 提供浏览器特有的扩展功能
- 错误转换和处理

#### 示例实现
```rust
// Chrome实现
pub struct ChromeBrowser {
    connection: ChromeConnection,
    // Chrome特有字段
}

impl ChromeBrowser {
    // Chrome特有方法
    pub async fn launch_with_debugging(&self, port: u16) -> Result<(), ChromeError> {
        // Chrome特有实现
    }
    
    pub async fn get_performance_metrics(&self) -> Result<ChromeMetrics, ChromeError> {
        // Chrome特有性能指标
    }
}

// Firefox实现
pub struct FirefoxBrowser {
    connection: FirefoxConnection,
    // Firefox特有字段
}

impl FirefoxBrowser {
    // Firefox特有方法
    pub async fn enable_remote_debugging(&self) -> Result<(), FirefoxError> {
        // Firefox特有实现
    }
}
```

### 2.3 连接传输层 (L3)

#### 职责
- 建立和维护与浏览器的连接
- 处理消息的发送和接收
- 实现不同的传输协议
- 处理连接状态管理

#### 核心接口
```rust
pub trait Connection {
    async fn connect(&mut self) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
    async fn send_message(&self, message: &str) -> Result<(), ConnectionError>;
    async fn receive_message(&self) -> Result<String, ConnectionError>;
}

// WebSocket实现
pub struct WebSocketConnection {
    url: String,
    socket: Option<WebSocket>,
}

// TCP实现
pub struct TcpConnection {
    address: SocketAddr,
    stream: Option<TcpStream>,
}
```

## 3. 使用模式

### 3.1 统一接口使用
```rust
// 通过统一接口使用
let browser: Box<dyn Browser> = BrowserFactory::create("chrome");
browser.navigate("https://www.rust-lang.org").await?;
```

### 3.2 直接使用特定浏览器
```rust
// 直接使用Chrome特有功能
let chrome = ChromeBrowser::new();
chrome.launch_with_debugging(9222).await?;
let metrics = chrome.get_performance_metrics().await?;
```

## 4. 扩展性设计

### 4.1 新增浏览器支持
- 实现L2层的浏览器特定实现
- 可选实现L1层的统一接口
- 选择或实现适当的L3层连接方式

### 4.2 新增传输协议
- 在L3层实现新的传输协议
- 确保与现有的L2层实现兼容

## 5. 错误处理

### 5.1 错误分层
- L1: 通用错误类型
- L2: 浏览器特定错误
- L3: 连接和传输错误

### 5.2 错误转换
```rust
impl From<ChromeError> for BrowserError {
    fn from(error: ChromeError) -> Self {
        // 错误转换逻辑
    }
}
```

## 6. 配置系统

### 6.1 分层配置
- 全局配置：影响所有层
- 浏览器配置：特定浏览器的配置
- 连接配置：传输层配置

### 6.2 配置示例
```rust
pub struct Config {
    global: GlobalConfig,
    browser: BrowserConfig,
    connection: ConnectionConfig,
}
```

## 7. 实现计划

### 7.1 Phase 1: 基础框架
- [ ] 定义核心接口
- [ ] 实现基本的错误类型
- [ ] 建立配置系统

### 7.2 Phase 2: Chrome实现
- [ ] 实现Chrome特定功能
- [ ] 实现WebSocket连接
- [ ] 添加Chrome特有扩展

### 7.3 Phase 3: Firefox实现
- [ ] 实现Firefox特定功能
- [ ] 实现必要的连接层
- [ ] 添加Firefox特有扩展

### 7.4 Phase 4: 统一接口
- [ ] 实现通用接口适配
- [ ] 完善错误处理
- [ ] 添加跨浏览器功能 