mod page;
mod dom;
mod network;

use async_trait::async_trait;
use serde_json::Value;
use crate::core::{BrowserDebugger, Page};
use crate::error::DebuggerError;
use crate::adapters::chrome::{ChromeAdapter, ChromeConnection};
use page::ChromePage;

pub struct ChromeDebugger {
    connection: ChromeConnection,
    adapter: ChromeAdapter,
}

impl ChromeDebugger {
    pub fn new() -> Self {
        Self {
            connection: ChromeConnection::new(),
            adapter: ChromeAdapter::new(),
        }
    }
}

#[async_trait]
impl BrowserDebugger for ChromeDebugger {
    async fn connect(&mut self, endpoint: &str) -> Result<(), DebuggerError> {
        self.connection.connect(endpoint).await?;
        
        // Enable necessary domains
        self.connection.send_message(crate::adapters::Message::Command {
            id: 1,
            method: "Target.setDiscoverTargets".to_string(),
            params: Some(serde_json::json!({ "discover": true })),
        }).await?;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), DebuggerError> {
        self.connection.disconnect().await
    }
    
    async fn get_pages(&self) -> Result<Vec<Box<dyn Page>>, DebuggerError> {
        let response = self.connection.send_message(crate::adapters::Message::Command {
            id: 1,
            method: "Target.getTargets".to_string(),
            params: None,
        }).await?;
        
        // Parse response and create ChromePage instances
        let mut pages = Vec::new();
        if let crate::adapters::Message::Response { result, .. } = response {
            if let Some(targets) = result.and_then(|v| v.get("targetInfos")).and_then(|v| v.as_array()) {
                for target in targets {
                    if let Some(target_id) = target.get("targetId").and_then(|v| v.as_str()) {
                        pages.push(Box::new(ChromePage::new(
                            target_id.to_string(),
                            self.connection.clone(),
                        )) as Box<dyn Page>);
                    }
                }
            }
        }
        
        Ok(pages)
    }
    
    async fn execute_script(&self, page_id: &str, script: &str) -> Result<Value, DebuggerError> {
        let response = self.connection.send_message(crate::adapters::Message::Command {
            id: 1,
            method: "Runtime.evaluate".to_string(),
            params: Some(serde_json::json!({
                "expression": script,
                "targetId": page_id,
            })),
        }).await?;
        
        if let crate::adapters::Message::Response { result, .. } = response {
            Ok(result.unwrap_or_default())
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
    
    async fn create_page(&mut self, url: Option<&str>) -> Result<Box<dyn Page>, DebuggerError> {
        let params = if let Some(url) = url {
            serde_json::json!({ "url": url })
        } else {
            serde_json::json!({ "url": "about:blank" })
        };
        
        let response = self.connection.send_message(crate::adapters::Message::Command {
            id: 1,
            method: "Target.createTarget".to_string(),
            params: Some(params),
        }).await?;
        
        if let crate::adapters::Message::Response { result, .. } = response {
            if let Some(target_id) = result.and_then(|v| v.get("targetId")).and_then(|v| v.as_str()) {
                Ok(Box::new(ChromePage::new(
                    target_id.to_string(),
                    self.connection.clone(),
                )) as Box<dyn Page>)
            } else {
                Err(DebuggerError::ProtocolError("No target ID in response".to_string()))
            }
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
    
    async fn close_page(&mut self, page_id: &str) -> Result<(), DebuggerError> {
        self.connection.send_message(crate::adapters::Message::Command {
            id: 1,
            method: "Target.closeTarget".to_string(),
            params: Some(serde_json::json!({
                "targetId": page_id,
            })),
        }).await?;
        
        Ok(())
    }
    
    async fn get_browser_version(&self) -> Result<String, DebuggerError> {
        let response = self.connection.send_message(crate::adapters::Message::Command {
            id: 1,
            method: "Browser.getVersion".to_string(),
            params: None,
        }).await?;
        
        if let crate::adapters::Message::Response { result, .. } = response {
            result
                .and_then(|v| v.get("product"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| DebuggerError::ProtocolError("No version information".to_string()))
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
} 