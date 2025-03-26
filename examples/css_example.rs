use JanusClient::FdpClient;
use serde_json::json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 设置日志级别
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    // 创建客户端实例
    let client = FdpClient::new("ws://127.0.0.1:9222/devtools/browser");

    // 连接到浏览器
    client.connect().await?;

    // 获取浏览器版本信息
    let (product, revision, user_agent) = client.get_browser_version().await?;
    println!("Browser version: {}", product);
    println!("Browser revision: {}", revision);
    println!("User Agent: {}", user_agent);

    // 获取可用的目标列表
    let targets = client.get_targets().await?;
    println!("Found {} targets", targets.len());

    // 选择第一个目标
    let target = targets.first().ok_or("No targets available")?;

    let target_id = target
        .get("targetId")
        .and_then(|v| v.as_str())
        .ok_or("Invalid target ID")?;

    // 附加到目标
    let session_id = client.attach_to_target(target_id).await?;
    println!("Attached to target with session ID: {}", session_id);

    // 导航到网页
    let frame_id = client.navigate_to_page("https://www.example.com").await?;
    println!("Navigated to frame: {}", frame_id);

    // 等待页面加载完成
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 获取文档根节点
    let document = client.get_document().await?;
    let root_node_id = document
        .get("root")
        .and_then(|v| v.get("nodeId"))
        .and_then(|v| v.as_i64())
        .ok_or("Invalid root node ID")?;

    // 获取计算后的样式
    let computed_properties = client.get_computed_style(root_node_id as i32).await?;
    println!(
        "Found {} computed style properties",
        computed_properties.len()
    );

    // 打印一些样式属性
    for property in computed_properties {
        if let (Some(name), Some(value)) = (
            property.get("name").and_then(|v| v.as_str()),
            property.get("value").and_then(|v| v.as_str()),
        ) {
            println!("{}: {}", name, value);
        }
    }

    // 关闭浏览器
    client.close_browser().await?;

    Ok(())
}
