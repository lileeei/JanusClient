# Actor Model for Firefox DevTools Protocol

## 1. Overview

This document describes how the Actor model is applied to implement the Firefox DevTools Protocol (FDP) client. The Actor model provides a robust concurrency abstraction that aligns well with the event-driven nature of debugging protocols.

## 2. Actor Model Fundamentals

### 2.1 Actor Model Concepts

The Actor model is a mathematical model of concurrent computation where "actors" are the universal primitives of computation:

1. **Actors** are the fundamental unit of computation
2. **Messaging** is the fundamental mode of communication
3. **State Encapsulation** isolates actors from each other
4. **Asynchronous Processing** ensures non-blocking operations

### 2.2 Actor Model Benefits for FDP

The Actor model offers several advantages for implementing FDP clients:

1. **Concurrency Safety**: Each domain operates independently without shared mutable state
2. **Isolation**: Domain failures are isolated and don't affect the entire system
3. **Asynchronous Communication**: Natural fit for WebSocket-based protocols
4. **Message Passing**: Aligns with the request-response nature of the protocol
5. **Supervision**: Enables robust error handling and recovery strategies

## 3. Actor Hierarchy for FDP

### 3.1 Core Actor Structure

```
RdpClient (root)
├── ConnectionActor
│   └── WebSocketActor
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

### 3.2 Actor Responsibilities

#### 3.2.1 RdpClient (Root Actor)

- Entry point for client API
- Manages connection lifecycle
- Coordinates between domains
- Handles global state

```rust
#[derive(Actor)]
pub struct RdpClient {
    connection: ActorRef<ConnectionActor>,
    domain_manager: ActorRef<DomainManagerActor>,
    event_dispatcher: ActorRef<EventDispatcherActor>,
    config: ClientConfig,
}
```

#### 3.2.2 ConnectionActor

- Manages WebSocket connection
- Handles message serialization/deserialization
- Routes responses to request originators
- Routes events to EventDispatcherActor

```rust
#[derive(Actor)]
struct ConnectionActor {
    websocket: Option<WebSocketStream>,
    request_map: HashMap<u64, oneshot::Sender<RdpResponse>>,
    event_dispatcher: ActorRef<EventDispatcherActor>,
    next_id: AtomicU64,
}
```

#### 3.2.3 DomainManagerActor

- Creates and manages domain-specific actors
- Routes messages to appropriate domain actors
- Maintains domain actor registry

```rust
#[derive(Actor)]
struct DomainManagerActor {
    connection: ActorRef<ConnectionActor>,
    domains: HashMap<DomainType, ActorRef<dyn DomainActor>>,
}
```

#### 3.2.4 Domain Actors

- Implement domain-specific protocol logic
- Maintain domain state
- Process domain-specific events
- Provide type-safe API

```rust
#[derive(Actor)]
struct DomActor {
    connection: ActorRef<ConnectionActor>,
    document_root: Option<Node>,
    node_map: HashMap<NodeId, Node>,
    event_subscribers: HashMap<String, Vec<mpsc::Sender<DomEvent>>>,
}
```

#### 3.2.5 EventDispatcherActor

- Routes events to appropriate subscribers
- Manages event subscriptions
- Handles event filtering

```rust
#[derive(Actor)]
struct EventDispatcherActor {
    domain_handlers: HashMap<String, ActorRef<dyn EventHandler>>,
    global_subscribers: HashMap<String, Vec<mpsc::Sender<RdpEvent>>>,
}
```

## 4. Message Flow Patterns

### 4.1 Request-Response Flow

```
Client API -> Domain API -> Domain Actor -> ConnectionActor -> WebSocket
                                                             /
Response   <- Domain API <- Domain Actor <- ConnectionActor <
```

1. **Client Request**: Client calls domain API method
2. **Domain Processing**: Domain API sends message to domain actor
3. **Protocol Request**: Domain actor creates protocol message and forwards to connection actor
4. **WebSocket Communication**: Connection actor sends message over WebSocket
5. **Response Routing**: Response is routed back through the same path

```rust
// Client API
impl PageDomain {
    pub async fn navigate(&self, url: String) -> Result<NavigationResult, DomainError> {
        self.actor_ref.ask(Navigate { url }).await
    }
}

// Domain Actor
#[messages]
impl PageActor {
    #[message]
    async fn navigate(&mut self, url: String) -> Result<NavigationResult, DomainError> {
        // Create protocol message
        let request = RdpMessage {
            id: self.next_id(),
            method: "Page.navigate".to_string(),
            params: Some(json!({
                "url": url
            })),
        };
        
        // Send request and await response
        let response = self.connection.ask(SendRequest { request }).await?;
        
        // Parse response
        let result = response.result.ok_or(DomainError::EmptyResponse)?;
        serde_json::from_value(result)
            .map_err(|e| DomainError::ParseError(e.to_string()))
    }
}
```

### 4.2 Event Flow

```
WebSocket -> ConnectionActor -> EventDispatcherActor -> Domain Actor -> Event Subscribers
```

1. **Event Reception**: Connection actor receives event from WebSocket
2. **Event Dispatching**: Event is forwarded to event dispatcher
3. **Domain Processing**: Event dispatcher routes event to appropriate domain actor
4. **Subscriber Notification**: Domain actor processes event and notifies subscribers

```rust
// Connection actor receives event
impl ConnectionActor {
    fn process_incoming_message(&mut self, text: String) -> Result<(), ConnectionError> {
        let value: serde_json::Value = serde_json::from_str(&text)?;
        
        // Check if it's an event (no 'id' field but has 'method')
        if value.get("id").is_none() && value.get("method").is_some() {
            if let Ok(event) = serde_json::from_value::<RdpEvent>(value) {
                self.event_dispatcher.tell(DispatchEvent { event })?;
            }
        }
        // ... handle responses ...
        
        Ok(())
    }
}

// Event dispatcher routes event
#[messages]
impl EventDispatcherActor {
    #[message]
    async fn dispatch_event(&mut self, event: RdpEvent) -> Result<(), EventError> {
        // Extract domain from method (e.g., "DOM.documentUpdated" -> "DOM")
        let domain = event.method.split('.').next().unwrap_or_default();
        
        // Forward to domain handler
        if let Some(handler) = self.domain_handlers.get(domain) {
            handler.tell(HandleEvent { event: event.clone() }).await?;
        }
        
        // Notify global subscribers
        if let Some(subscribers) = self.global_subscribers.get(&event.method) {
            for subscriber in subscribers {
                let _ = subscriber.send(event.clone()).await;
            }
        }
        
        Ok(())
    }
}

// Domain actor processes event
impl DomActor {
    async fn handle_event(&mut self, event: RdpEvent) -> Result<(), EventError> {
        match event.method.as_str() {
            "DOM.documentUpdated" => self.handle_document_updated().await?,
            "DOM.attributeModified" => {
                let params: AttributeModifiedParams = serde_json::from_value(event.params)?;
                self.handle_attribute_modified(params).await?;
            }
            // ... other events ...
            _ => {} // Ignore unknown events
        }
        
        Ok(())
    }
    
    async fn handle_document_updated(&mut self) -> Result<(), EventError> {
        // Clear cached state
        self.document_root = None;
        self.node_map.clear();
        
        // Notify subscribers
        if let Some(subscribers) = self.event_subscribers.get("documentUpdated") {
            for subscriber in subscribers {
                let _ = subscriber.send(DomEvent::DocumentUpdated).await;
            }
        }
        
        Ok(())
    }
}
```

## 5. Actor System Integration

### 5.1 Actor Creation

The actor system creates the actor hierarchy during initialization:

```rust
impl RdpClient {
    pub async fn new(config: ClientConfig) -> Result<Self, ClientError> {
        // Create actor system
        let system = ActorSystem::new();
        
        // Create event dispatcher actor
        let event_dispatcher = system.create_actor(
            EventDispatcherActor::new()
        ).await?;
        
        // Create connection actor
        let connection = system.create_actor(
            ConnectionActor::new(event_dispatcher.clone(), config.connection_timeout)
        ).await?;
        
        // Create domain manager
        let domain_manager = system.create_actor(
            DomainManagerActor::new(connection.clone(), config.request_timeout)
        ).await?;
        
        // Initialize domains
        let domains = domain_manager.ask(InitializeDomains {}).await?;
        
        // Register domain event handlers
        for (domain_type, actor_ref) in &domains {
            event_dispatcher.tell(RegisterDomainHandler {
                domain: domain_type.to_string(),
                handler: actor_ref.clone(),
            }).await?;
        }
        
        Ok(Self {
            connection,
            domain_manager,
            event_dispatcher,
            config,
        })
    }
}
```

### 5.2 Actor Supervision

The actor system implements supervision to handle failures:

```rust
impl RdpClient {
    async fn supervise(&mut self) -> Result<(), ClientError> {
        // Define supervision strategy
        let strategy = SupervisionStrategy::OneForOne {
            max_retries: 3,
            within: Duration::from_secs(60),
            backoff: ExponentialBackoff::new(
                Duration::from_millis(100),
                2.0,
                Duration::from_secs(5),
            ),
        };
        
        // Apply supervision to connection actor
        self.actor_system.supervise(
            self.connection.clone(),
            strategy.clone(),
            |err, ctx| async move {
                log::error!("Connection actor failed: {}", err);
                
                // Create new connection actor
                let new_connection = ctx.create_actor(
                    ConnectionActor::new(ctx.event_dispatcher.clone(), ctx.config.connection_timeout)
                ).await?;
                
                // Update references
                ctx.connection = new_connection.clone();
                ctx.domain_manager.tell(UpdateConnection { connection: new_connection }).await?;
                
                Ok(())
            },
        ).await?;
        
        // Similar supervision for other actors...
        
        Ok(())
    }
}
```

## 6. Actor Lifecycle Management

### 6.1 Connection Lifecycle

The connection actor manages WebSocket connection lifecycle:

```rust
impl ConnectionActor {
    async fn on_start(&mut self) -> Result<(), ConnectionError> {
        // Initialize resources
        self.request_map = HashMap::new();
        self.next_id = AtomicU64::new(1);
        
        Ok(())
    }
    
    async fn on_stop(&mut self) -> Result<(), ConnectionError> {
        // Close WebSocket connection
        if let Some(socket) = &mut self.websocket {
            let _ = socket.close(None).await;
        }
        
        // Notify pending requests
        for (_, sender) in self.request_map.drain() {
            let _ = sender.send(RdpResponse {
                id: 0,
                result: None,
                error: Some(RdpError {
                    code: -1,
                    message: "Connection closed".to_string(),
                }),
            });
        }
        
        Ok(())
    }
    
    async fn connect(&mut self, url: String) -> Result<(), ConnectionError> {
        // Close existing connection
        if let Some(socket) = &mut self.websocket {
            let _ = socket.close(None).await;
        }
        
        // Establish new connection
        let (socket, _) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| ConnectionError::WebSocket(e.to_string()))?;
            
        self.websocket = Some(socket);
        
        // Spawn receiver task
        self.spawn_receiver();
        
        Ok(())
    }
}
```

### 6.2 Domain Actor Lifecycle

Domain actors maintain state and handle domain-specific lifecycle events:

```rust
impl DomActor {
    async fn on_start(&mut self) -> Result<(), DomainError> {
        // Initialize state
        self.document_root = None;
        self.node_map = HashMap::new();
        self.event_subscribers = HashMap::new();
        
        Ok(())
    }
    
    async fn on_stop(&mut self) -> Result<(), DomainError> {
        // Notify subscribers about shutdown
        for (_, subscribers) in self.event_subscribers.drain() {
            for subscriber in subscribers {
                let _ = subscriber.send(DomEvent::ActorStopped).await;
            }
        }
        
        Ok(())
    }
    
    async fn on_domain_enabled(&mut self) -> Result<(), DomainError> {
        // Send "DOM.enable" request
        let request = RdpMessage {
            id: self.next_id(),
            method: "DOM.enable".to_string(),
            params: None,
        };
        
        let _ = self.connection.ask(SendRequest { request }).await?;
        
        Ok(())
    }
    
    async fn on_domain_disabled(&mut self) -> Result<(), DomainError> {
        // Send "DOM.disable" request
        let request = RdpMessage {
            id: self.next_id(),
            method: "DOM.disable".to_string(),
            params: None,
        };
        
        let _ = self.connection.ask(SendRequest { request }).await?;
        
        // Clear state
        self.document_root = None;
        self.node_map.clear();
        
        Ok(())
    }
}
```

## 7. Messaging Patterns

### 7.1 Tell Pattern (Fire and Forget)

Used for one-way messaging without response:

```rust
impl EventDispatcherActor {
    async fn broadcast_event(&mut self, event: RdpEvent) -> Result<(), EventError> {
        // Find all subscribers for this event type
        if let Some(subscribers) = self.global_subscribers.get(&event.method) {
            for subscriber in subscribers {
                // Tell pattern - fire and forget
                let _ = subscriber.send(event.clone()).await;
            }
        }
        
        Ok(())
    }
}

// Usage
event_dispatcher.tell(BroadcastEvent { event }).await?;
```

### 7.2 Ask Pattern (Request-Response)

Used for two-way communication with response:

```rust
impl DomActor {
    async fn get_document(&mut self, depth: Option<i32>) -> Result<Node, DomainError> {
        // Create request
        let request = RdpMessage {
            id: self.next_id(),
            method: "DOM.getDocument".to_string(),
            params: Some(json!({
                "depth": depth.unwrap_or(1)
            })),
        };
        
        // Ask pattern - wait for response
        let response = self.connection.ask(SendRequest { request }).await?;
        
        // Process response
        if let Some(result) = response.result {
            let document = serde_json::from_value::<GetDocumentResponse>(result)
                .map_err(|e| DomainError::ParseError(e.to_string()))?;
                
            // Cache document nodes
            self.document_root = Some(document.root.clone());
            self.cache_node(&document.root);
            
            Ok(document.root)
        } else if let Some(error) = response.error {
            Err(DomainError::MethodError {
                code: error.code,
                message: error.message,
            })
        } else {
            Err(DomainError::EmptyResponse)
        }
    }
}

// Usage
let document = dom_actor.ask(GetDocument { depth: Some(2) }).await?;
```

### 7.3 Stream Pattern (Event Subscription)

Used for continuous event streams:

```rust
impl DomDomain {
    pub async fn on_document_updated(&self) -> EventStream<DomEvent> {
        // Create channel for events
        let (sender, receiver) = mpsc::channel(32);
        
        // Subscribe to events
        self.actor_ref.ask(SubscribeToEvent {
            event_type: "documentUpdated".to_string(),
            sender: sender.clone(),
        }).await?;
        
        // Return stream with unsubscribe capability
        EventStream::new(receiver, EventUnsubscriber {
            actor_ref: self.actor_ref.clone(),
            event_type: "documentUpdated".to_string(),
            sender,
        })
    }
}

// Usage
let mut document_events = dom.on_document_updated().await;

while let Some(event) = document_events.next().await {
    println!("Document updated!");
}
```

## 8. State Management Techniques

### 8.1 Caching Strategy

Domain actors implement caching to reduce protocol overhead:

```rust
impl DomActor {
    async fn get_node_by_id(&mut self, node_id: NodeId) -> Result<Node, DomainError> {
        // Check cache first
        if let Some(node) = self.node_map.get(&node_id) {
            return Ok(node.clone());
        }
        
        // Fetch from protocol if not cached
        let request = RdpMessage {
            id: self.next_id(),
            method: "DOM.getNode".to_string(),
            params: Some(json!({
                "nodeId": node_id
            })),
        };
        
        let response = self.connection.ask(SendRequest { request }).await?;
        
        if let Some(result) = response.result {
            let node_response = serde_json::from_value::<GetNodeResponse>(result)
                .map_err(|e| DomainError::ParseError(e.to_string()))?;
                
            // Update cache
            self.cache_node(&node_response.node);
            
            Ok(node_response.node)
        } else if let Some(error) = response.error {
            Err(DomainError::MethodError {
                code: error.code,
                message: error.message,
            })
        } else {
            Err(DomainError::EmptyResponse)
        }
    }
    
    fn cache_node(&mut self, node: &Node) {
        // Add to cache
        self.node_map.insert(node.node_id, node.clone());
        
        // Also cache children recursively
        if let Some(children) = &node.children {
            for child in children {
                self.cache_node(child);
            }
        }
    }
    
    fn invalidate_cache(&mut self) {
        self.document_root = None;
        self.node_map.clear();
    }
}
```

### 8.2 State Synchronization

Domain actors keep state synchronized with browser:

```rust
impl DomActor {
    async fn handle_child_node_inserted(&mut self, params: ChildNodeInsertedParams) -> Result<(), DomainError> {
        // Update internal state
        if let Some(parent) = self.node_map.get_mut(&params.parent_node_id) {
            // Create children array if needed
            if parent.children.is_none() {
                parent.children = Some(Vec::new());
            }
            
            // Determine insert position
            let position = if params.previous_node_id != 0 {
                parent.children.as_ref().unwrap().iter()
                    .position(|node| node.node_id == params.previous_node_id)
                    .map(|pos| pos + 1)
                    .unwrap_or(0)
            } else {
                0
            };
            
            // Insert node
            if let Some(children) = parent.children.as_mut() {
                if position < children.len() {
                    children.insert(position, params.node.clone());
                } else {
                    children.push(params.node.clone());
                }
            }
            
            // Update child count
            parent.child_node_count = Some(parent.children.as_ref().map_or(0, |c| c.len() as i32));
        }
        
        // Cache the new node
        self.cache_node(&params.node);
        
        // Notify subscribers
        if let Some(subscribers) = self.event_subscribers.get("childNodeInserted") {
            for subscriber in subscribers {
                let _ = subscriber.send(DomEvent::ChildNodeInserted(params.clone())).await;
            }
        }
        
        Ok(())
    }
}
```

## 9. Error Handling Strategies

### 9.1 Error Propagation

Errors are propagated through the actor hierarchy:

```rust
impl DomainActor for DomActor {
    async fn handle_message(&mut self, message: Message) -> Result<Option<Message>, DomainError> {
        match message {
            Message::GetDocument(params, respond_to) => {
                let result = self.get_document(params.depth).await;
                
                match result {
                    Ok(document) => {
                        let _ = respond_to.send(Ok(document));
                    }
                    Err(err) => {
                        // Log error
                        log::error!("Failed to get document: {}", err);
                        
                        // Propagate error
                        let _ = respond_to.send(Err(err));
                    }
                }
                
                Ok(None)
            }
            // Other message handlers...
        }
    }
}
```

### 9.2 Recovery Strategies

Actors implement recovery strategies for different error scenarios:

```rust
impl ConnectionActor {
    async fn recover_from_disconnect(&mut self) -> Result<(), ConnectionError> {
        // Check if we have a URL to reconnect
        if let Some(url) = &self.url {
            // Attempt reconnection with exponential backoff
            let mut delay = Duration::from_millis(100);
            let max_delay = Duration::from_secs(5);
            let max_attempts = 5;
            
            for attempt in 0..max_attempts {
                match self.connect(url.clone()).await {
                    Ok(_) => {
                        log::info!("Reconnected successfully after {} attempts", attempt + 1);
                        return Ok(());
                    }
                    Err(e) => {
                        log::warn!("Reconnection attempt {} failed: {}", attempt + 1, e);
                        
                        if attempt < max_attempts - 1 {
                            tokio::time::sleep(delay).await;
                            delay = std::cmp::min(delay * 2, max_delay);
                        }
                    }
                }
            }
            
            Err(ConnectionError::ReconnectionFailed)
        } else {
            Err(ConnectionError::NoConnectionUrl)
        }
    }
}
```

### 9.3 Supervision Hierarchy

Error handling is also managed through supervision:

```rust
impl ActorSystem {
    async fn handle_actor_failure<A: Actor>(&mut self, actor_ref: ActorRef<A>, error: ActorError) {
        // Get supervision strategy for this actor
        let strategy = self.supervision_strategies.get(&actor_ref.id())
            .cloned()
            .unwrap_or(SupervisionStrategy::Stop);
            
        match strategy {
            SupervisionStrategy::Resume => {
                // Just resume the actor without restarting
                log::info!("Resuming actor {} after error: {}", actor_ref.name(), error);
                self.resume_actor(&actor_ref).await;
            }
            SupervisionStrategy::Restart { max_retries, within } => {
                // Check if we've exceeded max retries
                let restart_count = self.failure_counts.entry(actor_ref.id())
                    .or_insert_with(|| (0, Instant::now()));
                    
                // Reset count if outside time window
                if restart_count.1.elapsed() > within {
                    restart_count.0 = 0;
                    restart_count.1 = Instant::now();
                }
                
                restart_count.0 += 1;
                
                if restart_count.0 <= max_retries {
                    log::info!("Restarting actor {} after error: {} (attempt {}/{})",
                              actor_ref.name(), error, restart_count.0, max_retries);
                    self.restart_actor(&actor_ref).await;
                } else {
                    log::error!("Actor {} failed too many times, stopping: {}",
                               actor_ref.name(), error);
                    self.stop_actor(&actor_ref).await;
                    
                    // Escalate to parent
                    if let Some(parent) = self.actor_parents.get(&actor_ref.id()) {
                        self.escalate_failure(parent.clone(), actor_ref.clone(), error).await;
                    }
                }
            }
            SupervisionStrategy::Stop => {
                log::error!("Stopping actor {} due to error: {}", actor_ref.name(), error);
                self.stop_actor(&actor_ref).await;
                
                // Escalate to parent
                if let Some(parent) = self.actor_parents.get(&actor_ref.id()) {
                    self.escalate_failure(parent.clone(), actor_ref.clone(), error).await;
                }
            }
            SupervisionStrategy::Escalate => {
                // Directly escalate to parent
                if let Some(parent) = self.actor_parents.get(&actor_ref.id()) {
                    self.escalate_failure(parent.clone(), actor_ref.clone(), error).await;
                } else {
                    log::error!("Cannot escalate failure for root actor {}: {}",
                               actor_ref.name(), error);
                    self.stop_actor(&actor_ref).await;
                }
            }
        }
    }
}
```

## 10. Performance Considerations

### 10.1 Message Batching

Domain actors batch related operations to improve performance:

```rust
impl DomActor {
    async fn find_elements_with_styles(&mut self, selector: String) -> Result<Vec<ElementWithStyles>, DomainError> {
        // First, find all matching elements
        let nodes = self.query_selector_all(self.document_root_id()?, selector).await?;
        
        if nodes.is_empty() {
            return Ok(Vec::new());
        }
        
        // Create batched request for styles
        let mut futures = Vec::with_capacity(nodes.len());
        for node_id in &nodes {
            futures.push(self.css.ask(GetComputedStyleForNode { node_id: *node_id }));
        }
        
        // Execute all requests concurrently
        let styles_results = futures::future::join_all(futures).await;
        
        // Combine nodes with styles
        let mut elements = Vec::with_capacity(nodes.len());
        for (i, node_id) in nodes.iter().enumerate() {
            let node = self.get_node_by_id(*node_id).await?;
            let styles = styles_results.get(i)
                .and_then(|r| r.as_ref().ok())
                .cloned()
                .unwrap_or_default();
                
            elements.push(ElementWithStyles {
                element: node.into(),
                styles,
            });
        }
        
        Ok(elements)
    }
}
```

### 10.2 Connection Pooling

For handling multiple connections efficiently:

```rust
impl ConnectionPool {
    async fn get_connection(&mut self) -> Result<ConnectionHandle, ConnectionError> {
        // Return idle connection if available
        if let Some(conn) = self.idle_connections.pop() {
            let handle = ConnectionHandle {
                id: self.next_handle_id(),
                pool_ref: self.self_ref.clone(),
            };
            
            self.active_connections.insert(handle.id, conn);
            return Ok(handle);
        }
        
        // Create new connection if below limit
        if self.active_connections.len() < self.max_connections {
            let handle = ConnectionHandle {
                id: self.next_handle_id(),
                pool_ref: self.self_ref.clone(),
            };
            
            let conn = WebSocketActor::new().await?;
            self.active_connections.insert(handle.id, conn);
            
            return Ok(handle);
        }
        
        // Wait for connection if at limit
        let (sender, receiver) = oneshot::channel();
        self.waiting_requests.push_back(sender);
        
        // Wait for connection with timeout
        match tokio::time::timeout(self.wait_timeout, receiver).await {
            Ok(Ok(handle)) => Ok(handle),
            Ok(Err(_)) => Err(ConnectionError::PoolClosed),
            Err(_) => Err(ConnectionError::PoolTimeout),
        }
    }
}

// Connection handle with automatic return
impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        let pool_ref = self.pool_ref.clone();
        let id = self.id;
        
        tokio::spawn(async move {
            let _ = pool_ref.tell(ReturnConnection { id }).await;
        });
    }
}
```

## 11. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial Actor model documentation | AI Assistant |
