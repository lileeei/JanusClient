# Firefox DevTools Protocol Client 使用说明

## 准备工作

1. 启动 Firefox 并启用远程调试

   Firefox 可以通过命令行参数启动远程调试：

   ```bash
   firefox --remote-debugging-port=9222
   ```

   或者在已运行的 Firefox 中设置：
   - 打开 `about:config`
   - 设置 `devtools.debugger.remote-enabled` 为 `true`
   - 设置 `devtools.debugger.remote-port` 为 `9222`
   - 重启 Firefox

2. 确保你的应用有正确的依赖

   ```toml
   [dependencies]
   JanusClient = { path = "../path/to/JanusClient" }
   tokio = { version = "1.36", features = ["full"] }
   ```

## 基本使用

### 连接到 Firefox

```rust
use JanusClient::FdpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到 Firefox DevTools WebSocket 端点
    let client = FdpClient::connect("ws://localhost:9222").await?;

    // 使用 client 对象执行操作...

    Ok(())
}
```

### 获取浏览器信息

```rust
// 获取浏览器版本信息
let version = client.get_browser_version().await?;
println!("浏览器: {}", version.product);
println!("版本: {}", version.revision);
```

## 运行示例

项目包含一个示例程序，可以使用以下命令运行：

```bash
RUST_LOG=debug cargo run --example simple_client
```

## 扩展

如需添加更多浏览器域功能，可以按照以下步骤操作：

1. 在 `src/domain/` 目录下创建新的域实现文件
2. 实现对应的 Actor 结构体和方法
3. 在 `src/lib.rs` 中添加新域的 API 方法

例如，添加 DOM 域支持：

```rust
// src/domain/dom.rs
pub struct DomActor {
    // 实现...
}

impl DomActor {
    pub async fn get_document(&mut self) -> Result<Document> {
        // 实现...
    }
}

// src/lib.rs
pub async fn get_document(&self) -> Result<domain::dom::Document> {
    // 实现...
}
```
