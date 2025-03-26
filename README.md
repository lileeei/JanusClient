# Firefox DevTools Protocol Client (FDP Client)

使用 Rust 实现的 Firefox DevTools Protocol 客户端。这个库允许开发者连接到 Firefox 或 Chrome 的远程调试接口，获取信息并控制浏览器。

## 简化版本

注意：当前版本是简化实现，提供了基本的远程调试功能。它使用简单的WebSocket连接来发送请求和接收响应，而不是Actor模型。

## 项目结构

```
JanusClient/
├── src/
│   ├── actor/          # Actor 模型核心实现
│   ├── connection/     # WebSocket 连接管理
│   ├── domain/         # 不同协议域的实现
│   │   ├── browser.rs  # 浏览器域
│   │   └── ...         # 其他域实现
│   ├── error/          # 错误类型定义
│   ├── message/        # 协议消息格式
│   ├── utils/          # 工具函数
│   └── lib.rs          # 库入口点
├── knowledges/         # 项目文档和知识库
└── ...
```

## 架构设计

FDP Client 使用 Actor 模型实现，主要组件包括：

1. **系统 Actor (SystemActor)**：负责整体消息路由和 Actor 管理
2. **连接 Actor (ConnectionActor)**：处理 WebSocket 通信
3. **域 Actor (DomainActor)**：对应不同的协议域，如浏览器、DOM、网络等

## 使用方法

```rust
use JanusClient::FdpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端实例
    let client = FdpClient::new("ws://localhost:9222/devtools/browser");

    // 获取浏览器版本信息
    let (product, revision, user_agent) = client.get_browser_version().await?;
    println!("浏览器: {}", product);
    println!("版本: {}", revision);
    println!("用户代理: {}", user_agent);

    Ok(())
}
```

## 浏览器配置

### Chrome/Edge

1. 启动Chrome或Edge时启用远程调试：
   ```
   chrome --remote-debugging-port=9222
   ```

2. 验证远程调试是否工作：
   ```
   curl http://localhost:9222/json/version
   ```

   应该返回包含浏览器版本信息的JSON。

### Firefox

Firefox的远程调试配置更为复杂：

1. 在地址栏输入 `about:config` 并确认风险警告

2. 设置以下配置项：
   - `devtools.debugger.remote-enabled` = true
   - `devtools.chrome.enabled` = true
   - `devtools.debugger.prompt-connection` = false (可选，避免每次连接时提示)
   - `devtools.debugger.remote-port` = 6000 (或其他端口)

3. 重启Firefox

4. 使用环境变量设置正确的连接URL：
   ```
   FDP_URL=ws://localhost:6000/devtools-remote
   ```

## 注意事项

- WebSocket连接是短暂的，每个请求都会建立新连接
- 错误处理是基本的，实际应用中可能需要更复杂的重试和恢复机制
- 当前仅实现了Browser.getVersion方法，可以根据需要添加更多方法

## 扩展

可以通过添加更多的域 Actor 来扩展功能，例如：

- DOM 域 - 操作页面 DOM 结构
- 网络域 - 监控网络请求
- 控制台域 - 获取控制台消息
- 调试器域 - 设置断点和调试 JavaScript

每个域都应该实现 Actor trait，并遵循相同的消息处理模式。
