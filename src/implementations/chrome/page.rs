use async_trait::async_trait;
use serde_json::Value;
use crate::core::{Page, Dom, Network};
use crate::error::DebuggerError;
use crate::adapters::chrome::ChromeConnection;
use crate::adapters::Message;
use super::dom::ChromeDom;
use super::network::ChromeNetwork;

pub struct ChromePage {
    id: String,
    url: String,
    title: String,
    connection: ChromeConnection,
    dom: ChromeDom,
    network: ChromeNetwork,
}

impl ChromePage {
    pub fn new(id: String, connection: ChromeConnection) -> Self {
        Self {
            id,
            url: String::new(),
            title: String::new(),
            connection: connection.clone(),
            dom: ChromeDom::new(connection.clone()),
            network: ChromeNetwork::new(connection),
        }
    }
}

#[async_trait]
impl Page for ChromePage {
    fn get_id(&self) -> &str {
        &self.id
    }
    
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    async fn navigate(&mut self, url: &str) -> Result<(), DebuggerError> {
        let response = self.connection.send_message(Message::Command {
            id: 1,
            method: "Page.navigate".to_string(),
            params: Some(serde_json::json!({
                "url": url,
                "targetId": self.id,
            })),
        }).await?;
        
        if let Message::Response { result, error } = response {
            if error.is_some() {
                return Err(DebuggerError::PageError("Navigation failed".to_string()));
            }
            self.url = url.to_string();
            
            // Wait for page load
            // TODO: Implement proper page load waiting mechanism
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Ok(())
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
    
    async fn reload(&mut self, ignore_cache: bool) -> Result<(), DebuggerError> {
        self.connection.send_message(Message::Command {
            id: 1,
            method: "Page.reload".to_string(),
            params: Some(serde_json::json!({
                "ignoreCache": ignore_cache,
                "targetId": self.id,
            })),
        }).await?;
        
        // Wait for page load
        // TODO: Implement proper page load waiting mechanism
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }
    
    fn get_dom(&self) -> Box<dyn Dom> {
        Box::new(self.dom.clone())
    }
    
    fn get_network(&self) -> Box<dyn Network> {
        Box::new(self.network.clone())
    }
    
    async fn take_screenshot(&self, format: &str) -> Result<Vec<u8>, DebuggerError> {
        let response = self.connection.send_message(Message::Command {
            id: 1,
            method: "Page.captureScreenshot".to_string(),
            params: Some(serde_json::json!({
                "format": format,
                "targetId": self.id,
            })),
        }).await?;
        
        if let Message::Response { result, .. } = response {
            if let Some(data) = result.and_then(|v| v.get("data")).and_then(|v| v.as_str()) {
                base64::decode(data)
                    .map_err(|e| DebuggerError::ProtocolError(format!("Invalid base64 data: {}", e)))
            } else {
                Err(DebuggerError::ProtocolError("No screenshot data in response".to_string()))
            }
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
} 