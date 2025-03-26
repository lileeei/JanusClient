use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use crate::error::{FdpError, Result};
use crate::message::{Request, Response, Event};

pub type ActorHandle<T> = mpsc::Sender<ActorMessage<T>>;
pub type ResponseChannel = oneshot::Sender<Result<Response>>;

#[derive(Debug)]
pub enum ActorMessage<T: Clone> {
    Request {
        request: Request,
        response_tx: ResponseChannel,
    },
    Event(Event),
    Custom(T),
}

#[async_trait]
pub trait Actor: Sized + Send + 'static {
    type Message: Send + Clone + 'static;
    
    fn name(&self) -> &str;
    
    async fn handle_message(&mut self, msg: ActorMessage<Self::Message>) -> Result<()>;
}

pub struct SystemActor {
    name: String,
    connection: Arc<Mutex<Option<ActorHandle<Request>>>>,
    domain_actors: Arc<Mutex<Vec<ActorHandle<Request>>>>,
}

impl SystemActor {
    pub fn new() -> Self {
        Self {
            name: "system".to_string(),
            connection: Arc::new(Mutex::new(None)),
            domain_actors: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn register_connection(&self, connection: ActorHandle<Request>) {
        log::debug!("注册连接 Actor");
        let mut lock = self.connection.lock().unwrap();
        *lock = Some(connection);
    }
    
    pub fn register_domain_actor(&self, actor: ActorHandle<Request>) {
        log::debug!("注册域 Actor");
        let mut lock = self.domain_actors.lock().unwrap();
        lock.push(actor);
    }
    
    pub fn start(&self) -> ActorHandle<Request> {
        let (tx, mut rx) = mpsc::channel(32);
        let tx_clone = tx.clone();
        
        let connection = self.connection.clone();
        let domain_actors = self.domain_actors.clone();
        
        tokio::spawn(async move {
            log::debug!("系统 Actor 任务启动");
            while let Some(msg) = rx.recv().await {
                log::debug!("系统 Actor 收到消息");
                
                match msg {
                    ActorMessage::Request { request, response_tx } => {
                        log::debug!("处理请求: id={}, method={}", request.id, request.method);
                        let conn_option = {
                            let conn_lock = connection.lock().unwrap();
                            conn_lock.clone()
                        };
                        
                        if let Some(conn) = conn_option {
                            log::debug!("转发请求到连接 Actor");
                            let new_msg = ActorMessage::Request {
                                request: request.clone(),
                                response_tx,
                            };
                            
                            if let Err(e) = conn.send(new_msg).await {
                                log::error!("转发请求失败: {}", e);
                            }
                        } else {
                            log::error!("没有可用的连接");
                            let _ = response_tx.send(Err(FdpError::ActorError("No connection available".to_string())));
                        }
                    }
                    ActorMessage::Event(event) => {
                        log::debug!("处理事件: {}", event.method);
                        let actors = {
                            let actors_lock = domain_actors.lock().unwrap();
                            actors_lock.clone()
                        };
                        
                        for actor in &actors {
                            let new_msg = ActorMessage::Event(event.clone());
                            if let Err(e) = actor.send(new_msg).await {
                                log::error!("转发事件失败: {}", e);
                            }
                        }
                    }
                    ActorMessage::Custom(_) => {
                        log::warn!("系统 Actor 收到意外消息类型");
                    }
                }
            }
            log::debug!("系统 Actor 任务结束");
        });
        
        tx_clone
    }
}

#[async_trait]
impl Actor for SystemActor {
    type Message = Request;
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn handle_message(&mut self, msg: ActorMessage<Self::Message>) -> Result<()> {
        match msg {
            ActorMessage::Request { request, response_tx } => {
                let conn_option = {
                    let conn_lock = self.connection.lock().unwrap();
                    conn_lock.clone()
                };
                
                if let Some(connection) = conn_option {
                    let new_msg = ActorMessage::Request {
                        request: request.clone(),
                        response_tx,
                    };
                    
                    connection.send(new_msg).await.map_err(|e| {
                        FdpError::ActorError(format!("Failed to forward request: {}", e))
                    })?;
                } else {
                    return Err(FdpError::ActorError("No connection available".to_string()));
                }
            }
            ActorMessage::Event(event) => {
                let actors = {
                    let actors_lock = self.domain_actors.lock().unwrap();
                    actors_lock.clone()
                };
                
                for actor in &actors {
                    let new_msg = ActorMessage::Event(event.clone());
                    if let Err(e) = actor.send(new_msg).await {
                        log::error!("Failed to forward event to domain actor: {}", e);
                    }
                }
            }
            ActorMessage::Custom(_) => {
                log::warn!("Unexpected custom message received by system actor");
            }
        }
        Ok(())
    }
} 