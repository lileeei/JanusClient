# Janus Client Architecture Design

## 1. Overview

Janus Client is designed as a unified browser debugging protocol client that supports multiple browser debugging protocols through a combination of layered architecture and actor model. The system is designed to be extensible, maintainable, and performant.

## 2. Design Goals

### 2.1 Primary Goals
- Protocol Agnostic: Support multiple browser debugging protocols through a unified interface
- Layered Architecture: Clear separation of concerns with three main layers
- Actor-Based: Leverage actor model for concurrent operations and state management
- Type Safety: Leverage Rust's type system for compile-time guarantees
- Async First: Built with async/await for optimal performance
- Error Handling: Comprehensive error handling and recovery mechanisms

### 2.2 Secondary Goals
- Hot Reload: Support plugin hot-reloading where possible
- Configuration: Flexible configuration system
- Monitoring: Built-in monitoring and debugging capabilities
- Documentation: Comprehensive documentation and examples

## 3. System Architecture

### 3.1 Layered Architecture
```
┌─────────────────────────────────────┐
│           统一接口层 (L1)            │
│    - 提供统一的跨浏览器API           │
│    - 定义通用操作接口                │
│    - 处理通用错误类型                │
├─────────────────────────────────────┤
│           浏览器实现层 (L2)          │
│    - 实现特定浏览器功能              │
│    - 处理浏览器特有命令和事件         │
│    - 提供浏览器特有扩展              │
├─────────────────────────────────────┤
│           连接传输层 (L3)            │
│    - 建立和维护浏览器连接            │
│    - 处理消息收发                    │
│    - 实现不同传输协议                │
└─────────────────────────────────────┘
```

### 3.2 Actor System Architecture
```
┌─────────────────────────────────────────────────────┐
│                  Supervisor Actor                    │
├─────────────────────────────────────────────────────┤
│   Browser Actor     Session Actor      Plugin Actor  │
├─────────────────────────────────────────────────────┤
│   Event Actor       Command Actor      Monitor Actor │
└─────────────────────────────────────────────────────┘
```

### 3.3 Directory Structure
```
src/
├── core/           # Core Layer (L1)
│   ├── interface/     # Unified interfaces
│   ├── actor/         # Actor system core
│   ├── error/         # Error handling
│   └── config/        # Configuration
├── browsers/       # Browser Layer (L2)
│   ├── chrome/        # Chrome implementation
│   ├── firefox/       # Firefox implementation
│   └── edge/          # Edge implementation
└── transport/      # Transport Layer (L3)
    ├── websocket/     # WebSocket implementation
    ├── tcp/           # TCP implementation
    └── ipc/           # IPC implementation
```

## 4. Core Components

### 4.1 Actor System
- Supervisor Actor: Top-level management and error handling
- Browser Actor: Browser-specific operations
- Session Actor: Session management
- Event Actor: Event distribution
- Command Actor: Command execution
- Plugin Actor: Plugin management
- Monitor Actor: System monitoring

### 4.2 Layer Components

#### 4.2.1 L1 - Unified Interface Layer
```rust
pub trait Browser {
    async fn navigate(&self, url: &str) -> Result<(), Error>;
    async fn evaluate_script(&self, script: &str) -> Result<Value, Error>;
    async fn take_screenshot(&self, format: &str) -> Result<Vec<u8>, Error>;
}

pub trait Page {
    async fn close(&self) -> Result<(), Error>;
    async fn reload(&self) -> Result<(), Error>;
}
```

#### 4.2.2 L2 - Browser Implementation Layer
```rust
pub struct ChromeBrowser {
    actor_system: ChromeActorSystem,
    // Browser-specific fields
}

pub struct FirefoxBrowser {
    actor_system: FirefoxActorSystem,
    // Browser-specific fields
}
```

#### 4.2.3 L3 - Connection Layer
```rust
pub trait Connection {
    async fn connect(&mut self) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
    async fn send_message(&self, message: &str) -> Result<(), ConnectionError>;
    async fn receive_message(&self) -> Result<String, ConnectionError>;
}
```

### 4.3 Message Flow
```
Command Flow:
Client -> BrowserActor -> CommandActor -> ConnectionActor -> Browser

Event Flow:
Browser -> ConnectionActor -> EventActor -> Subscribers
```

### 4.4 Error Handling
- Layer-specific error types
- Actor supervision strategies
- Error recovery mechanisms
- Logging and monitoring

### 4.5 Configuration System
```rust
pub struct Config {
    // Global configuration
    global: GlobalConfig,
    
    // Layer-specific configuration
    l1_config: UnifiedInterfaceConfig,
    l2_config: BrowserConfig,
    l3_config: ConnectionConfig,
    
    // Actor system configuration
    actor_config: ActorSystemConfig,
}
```

## 5. Implementation Plan

### 5.1 Phase 1: Foundation
- [ ] Layer infrastructure setup
- [ ] Basic actor system implementation
- [ ] Core trait definitions

### 5.2 Phase 2: Chrome Support
- [ ] Chrome actor implementation
- [ ] WebSocket transport
- [ ] Basic browser operations

### 5.3 Phase 3: Features
- [ ] Event system
- [ ] Session management
- [ ] Plugin system

### 5.4 Phase 4: Additional Browsers
- [ ] Firefox support
- [ ] Edge support
- [ ] Protocol abstractions

## 6. Testing Strategy

### 6.1 Unit Testing
- Actor tests
- Layer-specific tests
- Mock implementations

### 6.2 Integration Testing
- Cross-layer integration
- Actor system integration
- Browser-specific integration

### 6.3 End-to-End Testing
- Full system tests
- Performance benchmarks
- Stress testing

## 7. Documentation

### 7.1 API Documentation
- Layer interfaces
- Actor messages
- Configuration options

### 7.2 User Documentation
- Getting started
- Browser-specific guides
- Best practices

### 7.3 Developer Documentation
- Architecture details
- Implementation guides
- Contributing guidelines

## 8. Future Considerations

### 8.1 Performance
- Actor pool optimization
- Message batching
- Connection pooling

### 8.2 Security
- Authentication
- Encryption
- Access control

### 8.3 Extensibility
- New browser support
- Plugin ecosystem
- Custom protocols

## 9. Version Roadmap

### v0.1.0 - Foundation
- Basic layer structure
- Actor system core
- Chrome support

### v0.2.0 - Enhancement
- Complete Chrome implementation
- Event system
- Session management

### v0.3.0 - Expansion
- Firefox support
- Plugin system
- Advanced features

### v1.0.0 - Production
- Edge support
- Full test coverage
- Production optimizations 