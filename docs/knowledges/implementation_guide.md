# Firefox DevTools Protocol Implementation Guide

## 1. Implementation Overview

This guide details the implementation approach for Firefox DevTools Protocol (FDP) in our RDP client using the Actor model.

### 1.1 Core Implementation Components

- **Connection Manager**: Handles WebSocket connections
- **Message Processor**: Serializes/deserializes JSON messages
- **Domain Handlers**: Implements domain-specific functionality
- **Event System**: Manages event subscriptions and notifications
- **Type System**: Provides type-safe interfaces to protocol objects

## 2. Actor Model Implementation

### 2.1 Actor Hierarchy

```
RdpSystemActor (root)
├── ConnectionActor
├── DomainManagerActor
│   ├── BrowserActor
│   ├── TargetActor
│   ├── PageActor
│   ├── DomActor
│   ├── CssActor
│   ├── NetworkActor
│   ├── ConsoleActor
│   ├── DebuggerActor
│   ├── RuntimeActor
│   └── PerformanceActor
└── EventDispatcherActor
```

### 2.2 Message Flow

1. Client sends message via public API
2. Message converted to internal format
3. ConnectionActor forwards to appropriate domain actor
4. Domain actor processes message
5. Response sent back through the chain
6. Client receives response via callback/future

## 3. Connection Management

### 3.1 Connection Establishment

```rust
impl ConnectionActor {
    async fn connect(&mut self, url: &str) -> Result<(), ConnectionError> {
        let (ws_stream, _) = connect_async(url).await?;
        let (write, read) = ws_stream.split();
        
        self.writer = Some(write);
        self.spawn_reader(read);
        
        // Notify system about successful connection
        self.send_to_parent(ConnectionEstablished { url: url.to_string() }).await?;
        
        Ok(())
    }
}
```

### 3.2 Message Processing

```rust
impl ConnectionActor {
    fn spawn_reader(&mut self, mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
        let actor_ref = self.self_ref.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse and forward message
                        if let Ok(protocol_msg) = serde_json::from_str(&text) {
                            let _ = actor_ref.send(MessageReceived { message: protocol_msg }).await;
                        }
                    },
                    Ok(Message::Close(_)) => {
                        let _ = actor_ref.send(ConnectionClosed {}).await;
                        break;
                    },
                    Err(e) => {
                        let _ = actor_ref.send(ConnectionError { error: e.to_string() }).await;
                        break;
                    },
                    _ => {}
                }
            }
        });
    }
}
```

## 4. Domain Implementations

### 4.1 Core Domain Actor Structure

```rust
struct DomainActor<T> {
    name: String,
    pending_requests: HashMap<u64, oneshot::Sender<Result<T, DomainError>>>,
    event_subscribers: HashMap<String, Vec<mpsc::Sender<DomainEvent>>>,
}

impl<T> Actor for DomainActor<T> {
    type Mailbox = BoundedMailbox<Self>;
    type Error = DomainError;
    
    fn name() -> &'static str {
        "DomainActor"
    }
}
```

### 4.2 Browser Domain Example

```rust
#[messages]
impl BrowserActor {
    #[message]
    async fn get_version(&mut self) -> Result<BrowserVersion, DomainError> {
        let request = RdpMessage {
            id: self.next_id(),
            method: "Browser.getVersion".to_string(),
            params: None,
        };
        
        self.connection.send(request).await?;
        self.wait_for_response().await
    }
    
    #[message]
    async fn close(&mut self) -> Result<(), DomainError> {
        let request = RdpMessage {
            id: self.next_id(),
            method: "Browser.close".to_string(),
            params: None,
        };
        
        self.connection.send(request).await?;
        self.wait_for_response().await
    }
}
```

## 5. Type System

### 5.1 Message Types

```rust
#[derive(Debug, Serialize, Deserialize)]
struct RdpMessage {
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RdpResponse {
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RdpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RdpEvent {
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct RdpError {
    code: i32,
    message: String,
}
```

### 5.2 Domain-Specific Types

```rust
// Browser domain types
#[derive(Debug, Serialize, Deserialize)]
struct BrowserVersion {
    protocol_version: String,
    product: String,
    revision: String,
    user_agent: String,
    js_version: String,
}

// DOM domain types
#[derive(Debug, Serialize, Deserialize)]
struct Node {
    node_id: NodeId,
    node_type: i32,
    node_name: String,
    local_name: String,
    node_value: String,
    child_node_count: Option<i32>,
    attributes: Option<Vec<String>>,
    children: Option<Vec<Node>>,
    document_url: Option<String>,
}

type NodeId = i32;
```

## 6. Event Handling

### 6.1 Event Subscription

```rust
impl EventDispatcherActor {
    async fn subscribe(&mut self, event_type: String, subscriber: ActorRef) -> Result<(), EventError> {
        let subscribers = self.subscribers.entry(event_type.clone()).or_insert_with(Vec::new);
        subscribers.push(subscriber);
        Ok(())
    }
    
    async fn dispatch_event(&mut self, event: RdpEvent) -> Result<(), EventError> {
        if let Some(subscribers) = self.subscribers.get(&event.method) {
            for subscriber in subscribers {
                subscriber.send(EventReceived { event: event.clone() }).await?;
            }
        }
        Ok(())
    }
}
```

### 6.2 Domain Event Handling

```rust
#[messages]
impl DomActor {
    #[message]
    async fn on_document_updated(&mut self, event: DomEvent) -> Result<(), DomainError> {
        // Process DOM update event
        if let Some(callback) = self.document_callbacks.take() {
            let _ = callback.send(Ok(()));
        }
        
        // Notify subscribers
        for subscriber in &self.subscribers {
            let _ = subscriber.tell(DocumentUpdated {}).await;
        }
        
        Ok(())
    }
}
```

## 7. Error Handling

### 7.1 Error Types

```rust
#[derive(Debug, Error)]
enum ProtocolError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Message error: {0}")]
    Message(String),
    
    #[error("Domain error: {code} {message}")]
    Domain {
        code: i32,
        message: String,
    },
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 7.2 Error Handling Strategy

```rust
impl DomainActor {
    async fn handle_response(&mut self, response: RdpResponse) -> Result<(), ProtocolError> {
        if let Some(sender) = self.pending_requests.remove(&response.id) {
            let result = if let Some(error) = response.error {
                Err(ProtocolError::Domain {
                    code: error.code,
                    message: error.message,
                })
            } else {
                Ok(response.result.unwrap_or(Value::Null))
            };
            
            if sender.send(result).is_err() {
                // Recipient dropped, log the error
                log::warn!("Response recipient dropped");
            }
        }
        
        Ok(())
    }
}
```

## 8. Testing Strategy

### 8.1 Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_browser_get_version() {
        let mut mock_connection = MockConnection::new();
        
        mock_connection.expect_send()
            .withf(|msg| msg.method == "Browser.getVersion")
            .returning(|_| {
                Ok(())
            });
            
        mock_connection.expect_receive()
            .returning(|| {
                Ok(RdpResponse {
                    id: 1,
                    result: Some(json!({
                        "protocol_version": "1.0",
                        "product": "Firefox",
                        "revision": "abc123",
                        "user_agent": "Mozilla/5.0",
                        "js_version": "1.8.5"
                    })),
                    error: None,
                })
            });
            
        let browser = BrowserActor::new(mock_connection);
        let result = browser.get_version().await;
        
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.product, "Firefox");
    }
}
```

### 8.2 Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_connection_flow() {
        // Setup test Firefox instance
        let test_server = TestServer::start().await;
        
        // Create client system
        let system = RdpSystem::new();
        let result = system.connect(&test_server.url()).await;
        
        assert!(result.is_ok());
        
        // Test browser operations
        let browser = system.browser_domain();
        let version = browser.get_version().await.unwrap();
        
        assert_eq!(version.product, "Firefox");
        
        // Cleanup
        system.disconnect().await.unwrap();
        test_server.stop().await;
    }
}
```

## 9. Performance Considerations

### 9.1 Message Batching

When performing multiple related operations:

```rust
impl DomActor {
    async fn get_document_with_styles(&mut self, depth: i32) -> Result<EnhancedNode, DomainError> {
        // Get document in one call
        let document = self.get_document(depth).await?;
        
        // Then batch style requests
        let style_requests = document.node_ids().map(|id| {
            self.get_computed_style_for_node(id)
        });
        
        let styles = futures::future::join_all(style_requests)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            
        // Combine results
        Ok(EnhancedNode::new(document, styles))
    }
}
```

### 9.2 Connection Pooling

```rust
impl ConnectionPool {
    fn get_connection(&mut self) -> ActorRef<ConnectionActor> {
        // Return existing connection or create new one
        if let Some(conn) = self.available_connections.pop() {
            conn
        } else {
            let conn = self.create_connection();
            self.active_connections.push(conn.clone());
            conn
        }
    }
    
    fn release_connection(&mut self, conn: ActorRef<ConnectionActor>) {
        // Return connection to pool
        self.available_connections.push(conn);
    }
}
```

## 10. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial implementation guide | AI Assistant |
```
