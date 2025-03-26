use JanusClient::FdpClient;
use base64;
use std::env;
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    pretty_env_logger::init();

    // 默认连接到 localhost:9222，这是Chrome的默认端口
    let url = env::var("FDP_URL").unwrap_or_else(|_| {
        "ws://127.0.0.1:9222/devtools/browser/dc066b9a-de64-4dbd-bbce-3de6446e3cbc".to_string()
    });

    println!("连接到: {}", url);
    println!("注意：Firefox需要特殊配置才能支持DevTools协议");
    println!("如果您使用的是Firefox，请参考README.md中的配置指南");

    // 创建带有事件通道的客户端
    let (client, _event_rx) = FdpClient::with_event_channel(url, 32);

    // 连接到浏览器
    match client.connect().await {
        Ok(_) => println!("成功连接到浏览器"),
        Err(e) => {
            eprintln!("连接失败: {}", e);
            return Err(e.into());
        }
    }

    // 获取浏览器版本信息
    match client.get_browser_version().await {
        Ok(version) => {
            println!("\n浏览器版本信息:");
            if let Some(product) = version.get("product").and_then(|v| v.as_str()) {
                println!("产品: {}", product);
            }
            if let Some(revision) = version.get("revision").and_then(|v| v.as_str()) {
                println!("修订版本: {}", revision);
            }
            if let Some(user_agent) = version.get("userAgent").and_then(|v| v.as_str()) {
                println!("用户代理: {}", user_agent);
            }
        }
        Err(e) => {
            eprintln!("\n获取浏览器版本失败: {}", e);
            return Err(e.into());
        }
    }

    // 获取可用目标
    match client.get_targets().await {
        Ok(targets_value) => {
            println!("\n可用页面/标签页:");

            let empty_vec = Vec::new();
            let targets = targets_value
                .get("targetInfos")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty_vec);

            if targets.is_empty() {
                println!("没有找到可用的页面");
            } else {
                for (i, target) in targets.iter().enumerate() {
                    let id = target
                        .get("targetId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知");
                    let title = target
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知");
                    let url = target.get("url").and_then(|v| v.as_str()).unwrap_or("未知");
                    let type_ = target
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知");

                    println!("{}. {} ({})", i + 1, title, type_);
                    println!("   ID: {}", id);
                    println!("   URL: {}", url);
                }

                // 创建新标签页
                println!("\n创建新标签页...");
                match client.create_target("https://example.com").await {
                    Ok(target_result) => {
                        if let Some(target_id) =
                            target_result.get("targetId").and_then(|v| v.as_str())
                        {
                            println!("新标签页已创建，ID: {}", target_id);

                            // 附加到新标签页
                            println!("\n尝试附加到新标签页...");
                            match client.attach_to_target(&target_id).await {
                                Ok(session_info) => {
                                    let session_id = session_info
                                        .get("sessionId")
                                        .and_then(|v| v.as_str())
                                        .ok_or_else(|| {
                                            Box::new(std::io::Error::new(
                                                std::io::ErrorKind::Other,
                                                "No session ID in response",
                                            ))
                                        })?;

                                    println!("附加成功，会话ID: {}", session_id);
                                    println!("等待页面加载...");
                                    tokio::time::sleep(Duration::from_secs(5)).await;

                                    println!("\n激活目标页面...");
                                    if let Err(e) = client.activate_target(&target_id).await {
                                        println!("激活目标页面失败: {}", e);
                                        return Ok(());
                                    }
                                    println!("目标页面已激活");

                                    println!("\n获取窗口信息...");
                                    match client.get_window_for_target(&target_id).await {
                                        Ok(window_info) => {
                                            if let Some(window_id) =
                                                window_info.get("windowId").and_then(|v| v.as_i64())
                                            {
                                                println!("获取到窗口ID: {}", window_id);
                                                match client.get_window_bounds(window_id).await {
                                                    Ok(bounds) => println!("窗口边界: {}", bounds),
                                                    Err(e) => println!("获取窗口边界失败: {}", e),
                                                }
                                            }
                                        }
                                        Err(e) => println!("获取窗口信息失败: {}", e),
                                    }

                                    println!("\n正在截取屏幕截图...");
                                    match client.capture_screenshot(session_id).await {
                                        Ok(base64_data) => {
                                            println!("截图成功！");
                                            // 将 base64 数据解码为二进制
                                            let bytes = base64::decode(base64_data).unwrap();
                                            // 保存为文件
                                            std::fs::write("screenshot.png", bytes).unwrap();
                                            println!("截图已保存为 screenshot.png");
                                        }
                                        Err(e) => {
                                            println!(" ERROR JanusClient > 截图失败: {}", e);
                                        }
                                    }

                                    println!("\n从页面分离...");
                                    if let Err(e) = client.detach_from_target(session_id).await {
                                        println!("分离失败: {}", e);
                                    } else {
                                        println!("已成功分离");
                                    }
                                }
                                Err(e) => println!("附加失败: {}", e),
                            }
                        }
                    }
                    Err(e) => println!("创建新标签页失败: {}", e),
                }
            }
        }
        Err(e) => {
            println!("\n获取目标列表失败: {}", e);
        }
    }

    // 断开连接
    println!("\n断开连接...");
    client.disconnect().await?;
    println!("已断开连接");

    Ok(())
}
