use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use serde_json::json;

use crate::actor::{Actor, ActorMessage, ActorHandle, SystemActor};
use crate::error::{FdpError, Result};
use crate::message::{Request, Response, Event};

// 我们需要添加 futures 依赖到 Cargo.toml
// Cargo.toml: futures = "0.3"
type WebSocketSink = tokio::sync::mpsc::Sender<Message>;
type ResponseMap = HashMap<u32, oneshot::Sender<Result<Response>>>;

pub struct ConnectionActor {
    name: String,
    next_id: AtomicU32,
    sink: Option<WebSocketSink>,
    system_actor: ActorHandle<Request>,
    response_channels: Arc<Mutex<ResponseMap>>,
}

impl ConnectionActor {
    pub fn new(system_actor: ActorHandle<Request>) -> Self {
        Self {
            name: "connection".to_string(),
            next_id: AtomicU32::new(1),
            sink: None,
            system_actor,
            response_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self, url: &str) -> Result<()> {
        log::debug!("连接到: {}", url);
        
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| FdpError::ConnectionError(format!("Failed to connect: {}", e)))?;
            
        self.sink = Some(ws_stream);
        
        // 我们需要重新设计消息处理方式，不使用分割的sink和source
        // TODO: 实现正确的WebSocket消息处理
        
        Ok(())
    }
    
    async fn send_request(&mut self, request: Request, response_tx: oneshot::Sender<Result<Response>>) -> Result<()> {
        if self.sink.is_none() {
            return Err(FdpError::ConnectionError("Not connected".to_string()));
        }
        
        // 确保请求有ID
        let request_id = if request.id == 0 {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let mut req = request;
            req.id = id;
            
            {
                let mut channels = self.response_channels.lock().unwrap();
                channels.insert(id, response_tx);
            }
            
            req
        } else {
            let id = request.id;
            {
                let mut channels = self.response_channels.lock().unwrap();
                channels.insert(id, response_tx);
            }
            request
        };
        
        // 发送请求
        let json = serde_json::to_string(&request_id)
            .map_err(|e| FdpError::JsonError(e))?;
            
        log::debug!("发送请求: {}", json);
        
        // TODO: 实现正确的WebSocket消息发送
        // 暂时返回未实现错误
        Err(FdpError::InternalError("WebSocket communication not fully implemented".to_string()))
    }
}

#[async_trait]
impl Actor for ConnectionActor {
    type Message = Request;
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn handle_message(&mut self, msg: ActorMessage<Self::Message>) -> Result<()> {
        match msg {
            ActorMessage::Request { request, response_tx } => {
                self.send_request(request, response_tx).await?;
            }
            ActorMessage::Event(event) => {
                log::warn!("Connection actor received an event: {}", event.method);
            }
            ActorMessage::Custom(custom_msg) => {
                log::debug!("Connection actor received custom message");
                // 目前不实现自定义消息处理
                Err(FdpError::InternalError("Custom message handling not implemented".to_string()))?;
            }
        }
        
        Ok(())
    }
} 