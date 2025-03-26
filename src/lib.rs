use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    WebSocketStream,
    MaybeTlsStream,
};
use tokio::net::TcpStream;
use url::Url;
use serde_json::{json, Value, Map};
use tokio::task::JoinHandle;
use std::time::Duration;
use log::{error, info, debug};

pub mod error;
pub mod message;
pub mod domain;

use error::{Result, FdpError};
use message::{Event, Request, Response};

/// Firefox DevTools Protocol Client
pub struct FdpClient {
    url: String,
    next_id: Arc<Mutex<i64>>,
    websocket_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    response_channels: Arc<Mutex<HashMap<i64, oneshot::Sender<String>>>>,
    event_sender: Option<mpsc::Sender<Event>>,
    ws_sender: Arc<Mutex<Option<mpsc::Sender<String>>>>,
}

impl FdpClient {
    /// 创建新的客户端实例
    pub fn new(url: impl Into<String>) -> Self {
        FdpClient {
            url: url.into(),
            next_id: Arc::new(Mutex::new(0)),
            websocket_task: Arc::new(Mutex::new(None)),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
            event_sender: None,
            ws_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// 创建带有事件通道的客户端实例
    pub fn with_event_channel(url: impl Into<String>, buffer: usize) -> (Self, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel(buffer);
        let client = FdpClient {
            url: url.into(),
            next_id: Arc::new(Mutex::new(0)),
            websocket_task: Arc::new(Mutex::new(None)),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
            event_sender: Some(tx),
            ws_sender: Arc::new(Mutex::new(None)),
        };
        (client, rx)
    }
    
    /// Connect to the browser
    pub async fn connect(&self) -> Result<()> {
        let mut websocket_task = self.websocket_task.lock().unwrap();
        if websocket_task.is_some() {
            return Ok(());
        }

        info!("正在连接到 {}", self.url);
        let url = Url::parse(&self.url)?;

        let (ws_stream, _) = match tokio::time::timeout(Duration::from_secs(5), connect_async(&url)).await {
            Ok(result) => result.map_err(|e| FdpError::NetworkError(format!("连接失败: {}", e)))?,
            Err(_) => return Err(FdpError::NetworkError("连接超时".to_string())),
        };

        info!("已连接到 {}", self.url);

        let (write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<String>(32);
        
        // 存储发送器
        *self.ws_sender.lock().unwrap() = Some(tx);

        let response_channels = Arc::clone(&self.response_channels);
        let event_sender = self.event_sender.clone().ok_or_else(|| FdpError::ProtocolError("事件发送器未初始化".to_string()))?;

        let write_task = tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(Message::Text(msg)).await {
                    error!("发送消息失败: {}", e);
                    break;
                }
            }
        });

        let read_task = tokio::spawn(async move {
            let mut read = read;
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let value: Value = match serde_json::from_str(&text) {
                            Ok(v) => v,
                            Err(e) => {
                                error!("解析消息失败: {}", e);
                                continue;
                            }
                        };
                        
                        if let Some(id) = value.get("id").and_then(Value::as_i64) {
                            // 这是一个响应
                            if let Some(sender) = response_channels.lock().unwrap().remove(&id) {
                                if sender.send(text).is_err() {
                                    error!("无法发送响应，接收端可能已关闭");
                                }
                            }
                        } else if value.get("method").is_some() {
                            // 这是一个事件
                            match serde_json::from_value::<Event>(value.clone()) {
                                Ok(event) => {
                                    if let Err(e) = event_sender.send(event).await {
                                        error!("发送事件失败: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("解析事件失败: {}", e);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        info!("WebSocket连接关闭: {:?}", frame);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket错误: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        *websocket_task = Some(write_task);
        Ok(())
    }
    
    /// Disconnect from the browser
    pub async fn disconnect(&self) -> Result<()> {
        if let Some(task) = {
            let mut ws_task = self.websocket_task.lock().unwrap();
            ws_task.take()
        } {
            task.abort();
            log::info!("Disconnected from {}", self.url);
        }
        
        Ok(())
    }
    
    /// 获取下一个请求ID
    fn next_id(&self) -> i64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }
    
    /// 发送请求并等待响应
    pub async fn send_request<T: serde::de::DeserializeOwned>(
        &self,
        request: Request,
        session_id: Option<String>,
    ) -> Result<T> {
        let request_id = request.id;
        let request_json = serde_json::to_string(&request)?;
        debug!("准备发送请求: {}", request_json);

        let ws_sender = self.ws_sender.lock().unwrap();
        if let Some(sender) = &*ws_sender {
            sender.send(request_json).await?;
            debug!("请求已发送");

            let response = self.wait_for_response(request_id).await?;
            if let Some(error) = response.error {
                error!("请求返回错误: {}", error);
                return Err(FdpError::ProtocolError(format!("请求返回错误: {}", error)));
            }

            if let Some(result) = response.result {
                let result: T = serde_json::from_value(result)?;
                Ok(result)
            } else {
                Err(FdpError::ProtocolError("响应中没有结果".to_string()))
            }
        } else {
            Err(FdpError::NotConnected)
        }
    }
    
    /// 发送原始请求
    pub async fn send_raw_request(&self, request: &str) -> Result<()> {
        // 检查当前是否已有连接
        let task_guard = self.websocket_task.lock().unwrap();
        let already_connected = task_guard.is_some();
        drop(task_guard);
        
        if !already_connected {
            log::info!("没有活跃的WebSocket连接，尝试连接...");
            self.connect().await?;
        }

        // 获取发送器并发送请求
        if let Some(sender) = self.ws_sender.lock().unwrap().as_ref() {
            sender.send(request.to_string())
                .await
                .map_err(|e| FdpError::NetworkError(format!("发送请求失败: {}", e)))?;
            log::debug!("请求已发送");
            Ok(())
        } else {
            Err(FdpError::NetworkError("WebSocket发送器未初始化".to_string()))
        }
    }
    
    /// 注册事件处理器
    pub fn on<F>(&self, method: impl Into<String>, _handler: F) -> Result<()>
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        let handlers = self.event_sender.as_ref()
            .ok_or_else(|| FdpError::ProtocolError("未启用事件处理".to_string()))?;
        
        // 添加处理器
        handlers.try_send(Event {
            method: method.into(),
            params: None,
            session_id: None,
        }).map_err(|e| FdpError::ProtocolError(format!("注册事件处理器失败: {}", e)))
    }
    
    /// 获取浏览器版本信息
    pub async fn get_browser_version(&self) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "Browser.getVersion".to_string(),
                params: Some(json!({})),
            },
            None
        ).await
    }
    
    /// 关闭浏览器
    pub async fn close_browser(&self) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "Browser.close".to_string(),
                params: Some(json!({})),
            },
            None
        ).await
    }
    
    /// 获取目标页面列表
    pub async fn get_targets(&self) -> Result<Value> {
        self.send_command("Target.getTargets", None, None).await
    }
    
    /// 创建新标签页
    pub async fn create_target(&self, url: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("url".to_string(), Value::String(url.to_string()));
        self.send_command("Target.createTarget", Some(params), None).await
    }
    
    /// 附加到目标
    pub async fn attach_to_target(&self, target_id: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("targetId".to_string(), Value::String(target_id.to_string()));
        params.insert("flatten".to_string(), Value::Bool(true));
        self.send_command("Target.attachToTarget", Some(params), None).await
    }
    
    /// 从目标分离
    pub async fn detach_from_target(&self, session_id: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("sessionId".to_string(), Value::String(session_id.to_string()));
        self.send_command("Target.detachFromTarget", Some(params), None).await
    }
    
    /// 导航到指定页面
    pub async fn navigate_to_page(&self, url: &str) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "Target.activateTarget".to_string(),
                params: Some(json!({
                    "url": url
                })),
            },
            None
        ).await
    }
    
    /// 发送命令到浏览器
    async fn send_command(&self, method: &str, params: Option<Map<String, Value>>, session_id: Option<&str>) -> Result<Value> {
        let request = Request {
            id: self.next_id(),
            method: method.to_string(),
            params: params.map(Value::Object),
        };
        self.send_request::<Value>(request, session_id.map(|s| s.to_string())).await
    }
    
    /// 启用 Target 域
    pub async fn enable_target_domain(&self) -> Result<Value> {
        self.send_command("Target.enable", None, None).await
    }
    
    /// 启用 Page 域
    pub async fn enable_page_domain(&self, session_id: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("enabled".to_string(), Value::Bool(true));
        self.send_command("Page.enable", Some(params), Some(session_id)).await
    }
    
    /// 在指定会话中导航到页面
    pub async fn navigate_in_session(&self, session_id: &str, url: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("url".to_string(), Value::String(url.to_string()));
        self.send_command("Page.navigate", Some(params), Some(session_id)).await
    }
    
    /// 重新加载页面
    pub async fn reload(&self, ignore_cache: bool) -> Result<()> {
        self.send_request::<()>(
            Request {
                id: self.next_id(),
                method: "Page.reload".to_string(),
                params: Some(json!({
                    "ignoreCache": ignore_cache
                })),
            },
            None
        ).await
    }
    
    /// 激活目标页面
    pub async fn activate_target(&self, target_id: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("targetId".to_string(), Value::String(target_id.to_string()));
        self.send_command("Target.activateTarget", Some(params), None).await
    }
    
    /// 截取屏幕截图
    pub async fn capture_screenshot(&self, session_id: &str) -> Result<String> {
        // 获取目标页面列表
        let targets = self.get_targets().await?;
        println!("目标页面列表: {}", targets);

        let mut params = Map::new();
        params.insert("format".to_string(), Value::String("png".to_string()));
        params.insert("quality".to_string(), Value::Number(100.into()));

        let response = self.send_command("Page.captureScreenshot", Some(params), Some(session_id)).await?;
        match response {
            Value::Object(obj) => {
                if let Some(data) = obj.get("data") {
                    if let Some(base64_str) = data.as_str() {
                        Ok(base64_str.to_string())
                    } else {
                        Err(FdpError::ProtocolError("Screenshot data is not a string".into()))
                    }
                } else {
                    Err(FdpError::ProtocolError("No data field in screenshot response".into()))
                }
            }
            _ => Err(FdpError::ProtocolError("Invalid response format".into())),
        }
    }
    
    /// 获取计算后的样式
    pub async fn get_computed_style(&self, node_id: i32) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "CSS.getComputedStyleForNode".to_string(),
                params: Some(json!({
                    "nodeId": node_id
                })),
            },
            None
        ).await
    }
    
    /// 获取匹配的样式
    pub async fn get_matched_styles(&self, node_id: i32) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "CSS.getMatchedStylesForNode".to_string(),
                params: Some(json!({
                    "nodeId": node_id
                })),
            },
            None
        ).await
    }
    
    /// 设置样式文本
    pub async fn set_style_texts(&self, edits: Vec<Value>) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "CSS.setStyleTexts".to_string(),
                params: Some(json!({
                    "edits": edits
                })),
            },
            None
        ).await
    }
    
    /// 获取文档节点
    pub async fn get_document(&self) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "DOM.getDocument".to_string(),
                params: Some(json!({})),
            },
            None
        ).await
    }
    
    /// 查询节点
    pub async fn query_selector(&self, node_id: i32, selector: &str) -> Result<Value> {
        self.send_request::<Value>(
            Request {
                id: self.next_id(),
                method: "DOM.querySelector".to_string(),
                params: Some(json!({
                    "nodeId": node_id,
                    "selector": selector
                })),
            },
            None
        ).await
    }

    /// 获取目标的窗口信息
    pub async fn get_window_for_target(&self, target_id: &str) -> Result<Value> {
        let mut params = Map::new();
        params.insert("targetId".to_string(), Value::String(target_id.to_string()));
        self.send_command("Browser.getWindowForTarget", Some(params), None).await
    }

    /// 获取窗口边界
    pub async fn get_window_bounds(&self, window_id: i64) -> Result<Value> {
        let mut params = Map::new();
        params.insert("windowId".to_string(), Value::Number(window_id.into()));
        self.send_command("Browser.getWindowBounds", Some(params), None).await
    }

    async fn wait_for_response(&self, request_id: i64) -> Result<Response> {
        let (tx, rx) = oneshot::channel();
        {
            let mut channels = self.response_channels.lock().unwrap();
            channels.insert(request_id, tx);
        }

        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(result) => match result {
                Ok(response_text) => {
                    debug!("收到响应: {}", response_text);
                    let response: Response = serde_json::from_str(&response_text)?;
                    Ok(response)
                }
                Err(_) => Err(FdpError::NetworkError("响应通道已关闭".to_string())),
            },
            Err(_) => Err(FdpError::NetworkError("等待响应超时".to_string())),
        }
    }
}

impl Drop for FdpClient {
    fn drop(&mut self) {
        if let Some(task) = self.websocket_task.lock().unwrap().take() {
            task.abort();
        }
        // 清理 WebSocket 发送器
        self.ws_sender.lock().unwrap().take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connect() {
        let client = FdpClient::new("ws://localhost:9222/devtools/browser");
        let result = client.get_browser_version().await;
        assert!(result.is_ok());
    }
} 