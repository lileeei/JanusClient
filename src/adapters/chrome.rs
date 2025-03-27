use async_trait::async_trait;
use serde_json::{json, Value};
use crate::error::DebuggerError;
use super::{ProtocolAdapter, Message, Connection};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Chrome DevTools Protocol adapter
#[derive(Clone)]
pub struct ChromeAdapter {
    next_id: Arc<Mutex<i64>>,
}

impl ChromeAdapter {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    fn next_command_id(&self) -> i64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }
}

#[async_trait]
impl ProtocolAdapter for ChromeAdapter {
    fn convert_command(&self, method: &str, params: Option<Value>) -> Result<String, DebuggerError> {
        let command = json!({
            "id": self.next_command_id(),
            "method": method,
            "params": params.unwrap_or(json!({}))
        });
        
        serde_json::to_string(&command)
            .map_err(|e| DebuggerError::SerializationError(e))
    }

    fn parse_response(&self, response: &str) -> Result<Value, DebuggerError> {
        let value: Value = serde_json::from_str(response)
            .map_err(|e| DebuggerError::SerializationError(e))?;
            
        if let Some(error) = value.get("error") {
            return Err(DebuggerError::ProtocolError(error.to_string()));
        }
        
        value.get("result")
            .cloned()
            .ok_or_else(|| DebuggerError::ProtocolError("No result field in response".to_string()))
    }

    fn convert_event(&self, event: &str) -> Result<(String, Value), DebuggerError> {
        let value: Value = serde_json::from_str(event)
            .map_err(|e| DebuggerError::SerializationError(e))?;
            
        let method = value.get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| DebuggerError::ProtocolError("No method field in event".to_string()))?;
            
        let params = value.get("params")
            .cloned()
            .unwrap_or(json!({}));
            
        Ok((method.to_string(), params))
    }
}

/// Chrome WebSocket connection
#[derive(Clone)]
pub struct ChromeConnection {
    ws_stream: Arc<Mutex<Option<(
        futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Message>,
        futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
    )>>>,
    adapter: ChromeAdapter,
}

impl ChromeConnection {
    pub fn new() -> Self {
        Self {
            ws_stream: Arc::new(Mutex::new(None)),
            adapter: ChromeAdapter::new(),
        }
    }
}

#[async_trait]
impl Connection for ChromeConnection {
    async fn connect(&mut self, endpoint: &str) -> Result<(), DebuggerError> {
        let url = url::Url::parse(endpoint)
            .map_err(|e| DebuggerError::InvalidArgument(e.to_string()))?;
            
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| DebuggerError::ConnectionError(e.to_string()))?;
            
        let (write, read) = ws_stream.split();
        *self.ws_stream.lock().unwrap() = Some((write, read));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), DebuggerError> {
        if let Some((mut write, _)) = self.ws_stream.lock().unwrap().take() {
            write.close().await
                .map_err(|e| DebuggerError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn send_message(&self, message: Message) -> Result<(), DebuggerError> {
        if let Some((write, _)) = &*self.ws_stream.lock().unwrap() {
            let msg = match message {
                Message::Command { method, params, .. } => {
                    self.adapter.convert_command(&method, params)?
                },
                _ => return Err(DebuggerError::InvalidArgument("Only commands can be sent".to_string())),
            };
            
            let mut write = write.clone();
            write.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await
                .map_err(|e| DebuggerError::ConnectionError(e.to_string()))?;
        } else {
            return Err(DebuggerError::NotConnected);
        }
        Ok(())
    }

    async fn receive_message(&self) -> Result<Message, DebuggerError> {
        if let Some((_, read)) = &*self.ws_stream.lock().unwrap() {
            let mut read = read.clone();
            
            match read.next().await {
                Some(Ok(msg)) => {
                    match msg {
                        tokio_tungstenite::tungstenite::Message::Text(text) => {
                            let value: Value = serde_json::from_str(&text)
                                .map_err(|e| DebuggerError::SerializationError(e))?;
                                
                            if value.get("id").is_some() {
                                Ok(Message::Response {
                                    id: value["id"].as_i64().unwrap(),
                                    result: value.get("result").cloned(),
                                    error: value.get("error").cloned(),
                                })
                            } else if value.get("method").is_some() {
                                Ok(Message::Event {
                                    method: value["method"].as_str().unwrap().to_string(),
                                    params: value.get("params").cloned().unwrap_or(json!({})),
                                })
                            } else {
                                Err(DebuggerError::ProtocolError("Invalid message format".to_string()))
                            }
                        },
                        _ => Err(DebuggerError::ProtocolError("Unexpected message type".to_string())),
                    }
                },
                Some(Err(e)) => Err(DebuggerError::ConnectionError(e.to_string())),
                None => Err(DebuggerError::ConnectionError("Connection closed".to_string())),
            }
        } else {
            Err(DebuggerError::NotConnected)
        }
    }

    fn is_connected(&self) -> bool {
        self.ws_stream.lock().unwrap().is_some()
    }
} 