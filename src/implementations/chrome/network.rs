use async_trait::async_trait;
use crate::core::{Network, NetworkRequest};
use crate::error::DebuggerError;
use crate::adapters::chrome::ChromeConnection;
use crate::adapters::Message;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ChromeNetwork {
    connection: ChromeConnection,
    requests: Arc<Mutex<HashMap<String, NetworkRequest>>>,
}

impl ChromeNetwork {
    pub fn new(connection: ChromeConnection) -> Self {
        Self {
            connection,
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Network for ChromeNetwork {
    async fn enable(&mut self) -> Result<(), DebuggerError> {
        self.connection.send_message(Message::Command {
            id: 1,
            method: "Network.enable".to_string(),
            params: None,
        }).await?;
        
        // Set up event listeners for network events
        // TODO: Implement proper event handling
        Ok(())
    }
    
    async fn disable(&mut self) -> Result<(), DebuggerError> {
        self.connection.send_message(Message::Command {
            id: 1,
            method: "Network.disable".to_string(),
            params: None,
        }).await?;
        Ok(())
    }
    
    async fn get_requests(&self) -> Result<Vec<NetworkRequest>, DebuggerError> {
        let requests = self.requests.lock()
            .map_err(|_| DebuggerError::Unknown("Failed to lock requests".to_string()))?;
            
        Ok(requests.values().cloned().collect())
    }
    
    async fn clear(&mut self) -> Result<(), DebuggerError> {
        let mut requests = self.requests.lock()
            .map_err(|_| DebuggerError::Unknown("Failed to lock requests".to_string()))?;
            
        requests.clear();
        
        self.connection.send_message(Message::Command {
            id: 1,
            method: "Network.clearBrowserCache".to_string(),
            params: None,
        }).await?;
        
        Ok(())
    }
} 