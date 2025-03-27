use async_trait::async_trait;
use serde_json::Value;
use crate::core::{Dom, Element};
use crate::error::DebuggerError;
use crate::adapters::chrome::ChromeConnection;
use crate::adapters::Message;

#[derive(Clone)]
pub struct ChromeDom {
    connection: ChromeConnection,
}

impl ChromeDom {
    pub fn new(connection: ChromeConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl Dom for ChromeDom {
    async fn query_selector(&self, selector: &str) -> Result<Vec<Element>, DebuggerError> {
        let response = self.connection.send_message(Message::Command {
            id: 1,
            method: "DOM.querySelector".to_string(),
            params: Some(serde_json::json!({
                "selector": selector,
            })),
        }).await?;
        
        if let Message::Response { result, .. } = response {
            if let Some(node_id) = result.and_then(|v| v.get("nodeId")).and_then(|v| v.as_i64()) {
                // Get node details
                let details = self.connection.send_message(Message::Command {
                    id: 2,
                    method: "DOM.describeNode".to_string(),
                    params: Some(serde_json::json!({
                        "nodeId": node_id,
                    })),
                }).await?;
                
                if let Message::Response { result: Some(details), .. } = details {
                    let node = details.get("node").unwrap_or(&Value::Null);
                    let tag_name = node.get("nodeName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                        
                    let mut attributes = Vec::new();
                    if let Some(attrs) = node.get("attributes").and_then(|v| v.as_array()) {
                        for chunk in attrs.chunks(2) {
                            if let (Some(name), Some(value)) = (chunk.get(0), chunk.get(1)) {
                                if let (Some(name), Some(value)) = (name.as_str(), value.as_str()) {
                                    attributes.push((name.to_string(), value.to_string()));
                                }
                            }
                        }
                    }
                    
                    Ok(vec![Element {
                        node_id: node_id as i32,
                        tag_name,
                        attributes,
                    }])
                } else {
                    Ok(Vec::new())
                }
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
    
    async fn get_computed_style(&self, element: &Element) -> Result<Value, DebuggerError> {
        let response = self.connection.send_message(Message::Command {
            id: 1,
            method: "CSS.getComputedStyleForNode".to_string(),
            params: Some(serde_json::json!({
                "nodeId": element.node_id,
            })),
        }).await?;
        
        if let Message::Response { result, .. } = response {
            Ok(result.unwrap_or_default())
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
    
    async fn set_style_text(&self, element: &Element, style: &str) -> Result<(), DebuggerError> {
        // First get the current styles
        let response = self.connection.send_message(Message::Command {
            id: 1,
            method: "CSS.getMatchedStylesForNode".to_string(),
            params: Some(serde_json::json!({
                "nodeId": element.node_id,
            })),
        }).await?;
        
        if let Message::Response { result, .. } = response {
            if let Some(styles) = result.and_then(|v| v.get("inlineStyle")) {
                // Update the style text
                self.connection.send_message(Message::Command {
                    id: 2,
                    method: "CSS.setStyleTexts".to_string(),
                    params: Some(serde_json::json!({
                        "edits": [{
                            "styleSheetId": styles.get("styleSheetId").unwrap_or(&Value::Null),
                            "range": styles.get("range").unwrap_or(&Value::Null),
                            "text": style,
                        }],
                    })),
                }).await?;
                
                Ok(())
            } else {
                Err(DebuggerError::DomError("No inline style found".to_string()))
            }
        } else {
            Err(DebuggerError::ProtocolError("Invalid response type".to_string()))
        }
    }
} 