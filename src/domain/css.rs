use serde::{Deserialize, Serialize};
// 暂时移除actor模块导入，因为找不到该模块
// use crate::actor::{Actor, ActorMessage, ActorHandle};
use crate::error::{FdpError, Result};
use crate::message::{Request, Response, Event};
use tokio::sync::{mpsc, oneshot};
use serde_json::json;
// 由于不再使用async_trait，移除它
// use async_trait::async_trait;
use std::collections::HashMap;

// Types for CSS domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSheetId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(pub i32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    #[serde(rename = "styleSheetId")]
    pub style_sheet_id: Option<StyleSheetId>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub important: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedStyle {
    pub properties: Vec<ComputedProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedProperty {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleEdit {
    #[serde(rename = "styleSheetId")]
    pub style_sheet_id: StyleSheetId,
    #[serde(rename = "styleText")]
    pub style_text: String,
}

pub struct CssActor {
    name: String,
    system: mpsc::Sender<Request>,
    event_handlers: HashMap<String, Vec<mpsc::Sender<Event>>>,
}

impl CssActor {
    pub fn new(system: mpsc::Sender<Request>) -> Self {
        Self {
            name: "css".to_string(),
            system,
            event_handlers: HashMap::new(),
        }
    }
    
    pub async fn get_computed_style_for_node(&self, node_id: i32) -> Result<ComputedStyle> {
        log::debug!("获取节点计算样式, node_id={}", node_id);
        
        let _request = Request {
            id: 0,  // 连接Actor会分配ID
            method: "CSS.getComputedStyleForNode".to_string(),
            params: Some(json!({
                "nodeId": node_id
            })),
        };
        
        // 此处存在问题，我们创建了通道但没有实际使用它，可能需要完整的Actor模型实现
        // 由于简化版本不使用Actor模型，这里直接返回模拟数据进行测试
        log::warn!("CssActor.get_computed_style_for_node(): 由于简化版本不使用Actor模型，返回模拟数据");
        
        // 返回一些模拟的CSS属性作为测试
        let properties = vec![
            ComputedProperty {
                name: "color".to_string(),
                value: "rgb(0, 0, 0)".to_string(),
            },
            ComputedProperty {
                name: "background-color".to_string(),
                value: "rgb(255, 255, 255)".to_string(),
            },
            ComputedProperty {
                name: "font-family".to_string(),
                value: "Arial, sans-serif".to_string(),
            },
            ComputedProperty {
                name: "font-size".to_string(),
                value: "16px".to_string(),
            },
            ComputedProperty {
                name: "margin".to_string(),
                value: "8px".to_string(),
            },
        ];
        
        Ok(ComputedStyle { properties })
    }
    
    pub async fn on_stylesheet_added(&self) -> Result<mpsc::Receiver<Event>> {
        log::debug!("注册样式表添加事件监听器");
        
        let (tx, rx) = mpsc::channel(32);
        
        let mut handlers = self.event_handlers.clone();
        let event_name = "CSS.styleSheetAdded".to_string();
        
        let handlers_for_event = handlers.entry(event_name).or_insert_with(Vec::new);
        handlers_for_event.push(tx);
        
        Ok(rx)
    }
}

// 由于缺少Actor trait，我们暂时注释掉这部分
/*
#[async_trait]
impl Actor for CssActor {
    type Message = Request;
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn handle_message(&mut self, msg: ActorMessage<Self::Message>) -> Result<()> {
        match msg {
            ActorMessage::Request { request, response_tx } => {
                log::debug!("CSS Actor 收到请求: {}", request.method);
                
                // 将请求转发到系统Actor，由系统Actor处理
                let new_msg = ActorMessage::Request {
                    request: request.clone(),
                    response_tx,
                };
                
                self.system.send(new_msg).await
                    .map_err(|e| FdpError::ActorError(format!("Failed to forward request: {}", e)))?;
            }
            ActorMessage::Event(event) => {
                log::debug!("CSS Actor 收到事件: {}", event.method);
                
                // 如果有监听这个事件的处理器，则通知它们
                if let Some(handlers) = self.event_handlers.get(&event.method) {
                    for handler in handlers {
                        // 克隆事件，以便每个处理器都能获得自己的副本
                        let event_clone = event.clone();
                        if let Err(e) = handler.send(event_clone).await {
                            log::error!("Failed to send event to handler: {}", e);
                        }
                    }
                }
            }
            ActorMessage::Custom(_) => {
                log::warn!("CSS Actor 收到意外消息类型");
            }
        }
        
        Ok(())
    }
}
*/ 