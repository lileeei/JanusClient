pub mod chrome;

use async_trait::async_trait;
use serde_json::Value;
use crate::error::DebuggerError;

/// Protocol adapter trait for converting between different browser debugging protocols
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Convert a command to the target protocol format
    fn convert_command(&self, command: &str, params: Option<Value>) -> Result<String, DebuggerError>;
    
    /// Parse a response from the target protocol format
    fn parse_response(&self, response: &str) -> Result<Value, DebuggerError>;
    
    /// Convert an event from the target protocol format
    fn convert_event(&self, event: &str) -> Result<(String, Value), DebuggerError>;
}

/// Message types that can be sent/received
#[derive(Debug, Clone)]
pub enum Message {
    Command {
        id: i64,
        method: String,
        params: Option<Value>,
    },
    Response {
        id: i64,
        result: Option<Value>,
        error: Option<Value>,
    },
    Event {
        method: String,
        params: Value,
    },
}

/// Connection interface for browser debugging protocols
#[async_trait]
pub trait Connection: Send + Sync {
    /// Connect to the debugging endpoint
    async fn connect(&mut self, endpoint: &str) -> Result<(), DebuggerError>;
    
    /// Disconnect from the endpoint
    async fn disconnect(&mut self) -> Result<(), DebuggerError>;
    
    /// Send a message
    async fn send_message(&self, message: Message) -> Result<(), DebuggerError>;
    
    /// Receive a message
    async fn receive_message(&self) -> Result<Message, DebuggerError>;
    
    /// Check if the connection is active
    fn is_connected(&self) -> bool;
} 