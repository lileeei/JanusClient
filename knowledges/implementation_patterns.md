# Firefox DevTools Protocol Implementation Patterns

## 1. Overview

This document outlines recommended patterns for implementing Firefox DevTools Protocol (FDP) client in Rust using the Actor model. These patterns provide guidance on code organization, message handling, error management, and state synchronization.

## 2. Core Architectural Patterns

### 2.1 Actor-Based Architecture

The FDP client uses a hierarchy of actors, each responsible for a specific aspect of the debugging protocol:

```
RdpClient (root)
├── ConnectionActor
├── DomainManagerActor
│   ├── BrowserActor
│   ├── TargetActor
│   ├── PageActor
│   ├── DomActor
│   ├── ...other domain actors
└── EventDispatcherActor
```

**Implementation Pattern:**
```rust
#[derive(Actor)]
struct RdpClient {
    connection: ActorRef<ConnectionActor>,
    domain_manager: ActorRef<DomainManagerActor>,
    event_dispatcher: ActorRef<EventDispatcherActor>,
}

#[messages]
impl RdpClient {
    #[message]
    async fn connect(&mut self, url: String) -> Result<(), ClientError> {
        self.connection.ask(Connect { url }).await?;
        Ok(())
    }
    
    #[message]
    async fn get_browser_domain(&self) -> BrowserDomain {
        let actor_ref = self.domain_manager.ask(GetDomainActor { 
            domain_type: DomainType::Browser 
        }).await?;
        
        BrowserDomain::new(actor_ref)
    }
}
```

### 2.2 Message Flow Pattern

The message flow follows a consistent pattern:

1. Public API call → Domain API
2. Domain API → Actor message
3. Actor → Protocol message
4. Response → Actor state update
5. Result → Public API result

**Implementation Pattern:**
```rust
// Public API
impl BrowserDomain {
    pub async fn get_version(&self) -> Result<BrowserVersion, DomainError> {
        self.actor_ref.ask(GetVersion {}).await
    }
}

// Actor implementation
#[messages]
impl BrowserActor {
    #[message]
    async fn get_version(&mut self) -> Result<BrowserVersion, DomainError> {
        // Create protocol message
        let request = RdpMessage {
            id: self.next_id(),
            method: "Browser.getVersion".to_string(),
            params: None,
        };
        
        // Send to connection
        let response = self.connection.ask(SendRequest { 
            request, 
            timeout: self.request_timeout 
        }).await?;
        
        // Parse response
        self.parse_version_response(response)
    }
    
    fn parse_version_response(&self, response: RdpResponse) -> Result<BrowserVersion, DomainError> {
        // Parse JSON response into typed struct
        match response.result {
            Some(value) => {
                serde_json::from_value(value)
                    .map_err(|e| DomainError::ParseError(e.to_string()))
            },
            None => {
                if let Some(error) = response.error {
                    Err(DomainError::MethodError { 
                        code: error.code, 
                        message: error.message 
                    })
                } else {
                    Err(DomainError::EmptyResponse)
                }
            }
        }
    }
}
```

## 3. State Management Patterns

### 3.1 Domain State Synchronization

Domain actors maintain state that needs to be synchronized with the browser:

**Implementation Pattern:**
```rust
#[derive(Actor)]
struct DomActor {
    connection: ActorRef<ConnectionActor>,
    document_root: Option<Node>,
    node_map: HashMap<NodeId, Node>,
    event_listeners: HashMap<String, Vec<mpsc::Sender<DomEvent>>>,
}

impl DomActor {
    async fn handle_document_updated(&mut self, _event: DocumentUpdated) -> Result<(), DomainError> {
        // Clear cached state
        self.document_root = None;
        self.node_map.clear();
        
        // Notify listeners
        for listeners in self.event_listeners.get("documentUpdated").unwrap_or(&vec![]) {
            let _ = listeners.send(DomEvent::DocumentUpdated).await;
        }
        
        Ok(())
    }
    
    async fn refresh_document(&mut self) -> Result<(), DomainError> {
        // Re-fetch document if needed
        if self.document_root.is_none() {
            let document = self.get_document(Some(1)).await?;
            self.document_root = Some(document);
        }
        
        Ok(())
    }
}
```

### 3.2 Connection State Management

Connection actor manages WebSocket state and reconnection logic:

**Implementation Pattern:**
```rust
#[derive(Actor)]
struct ConnectionActor {
    url: Option<String>,
    socket: Option<WebSocketStream>,
    request_map: HashMap<u64, oneshot::Sender<RdpResponse>>,
    next_id: u64,
    reconnect_attempts: u32,
}

#[messages]
impl ConnectionActor {
    #[message]
    async fn connect(&mut self, url: String) -> Result<(), ConnectionError> {
        self.url = Some(url.clone());
        self.socket = Some(self.establish_connection(&url).await?);
        self.spawn_receiver();
        Ok(())
    }
    
    #[message]
    async fn send_request(&mut self, request: RdpMessage, timeout: Duration) -> Result<RdpResponse, ConnectionError> {
        // Ensure connection
        if self.socket.is_none() {
            if let Some(url) = &self.url {
                self.socket = Some(self.establish_connection(url).await?);
                self.spawn_receiver();
            } else {
                return Err(ConnectionError::NotConnected);
            }
        }
        
        // Create response channel
        let (sender, receiver) = oneshot::channel();
        self.request_map.insert(request.id, sender);
        
        // Send request
        let message = serde_json::to_string(&request)?;
        self.socket.as_mut().unwrap().send(Message::Text(message)).await?;
        
        // Wait for response with timeout
        match tokio::time::timeout(timeout, receiver).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(ConnectionError::ChannelClosed),
            Err(_) => {
                self.request_map.remove(&request.id);
                Err(ConnectionError::Timeout)
            }
        }
    }
    
    fn spawn_receiver(&mut self) {
        let mut socket = self.socket.take().unwrap();
        let request_map = self.request_map.clone();
        let event_dispatcher = self.event_dispatcher.clone();
        
        tokio::spawn(async move {
            while let Some(message) = socket.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(message) = serde_json::from_str::<JsonValue>(&text) {
                            if message.get("id").is_some() {
                                // Handle response
                                if let Ok(response) = serde_json::from_value::<RdpResponse>(message) {
                                    if let Some(sender) = request_map.remove(&response.id) {
                                        let _ = sender.send(response);
                                    }
                                }
                            } else if message.get("method").is_some() {
                                // Handle event
                                if let Ok(event) = serde_json::from_value::<RdpEvent>(message) {
                                    let _ = event_dispatcher.tell(DispatchEvent { event }).await;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        });
    }
}
```

## 4. Event Handling Patterns

### 4.1 Event Subscription Pattern

Domain-specific events require subscription management:

**Implementation Pattern:**
```rust
// Public API
impl DomDomain {
    pub async fn on_document_updated(&self) -> EventStream<DomEvent> {
        let (sender, receiver) = mpsc::channel(32);
        
        self.actor_ref.tell(SubscribeToEvent {
            event_type: "documentUpdated".to_string(),
            sender: sender.clone(),
        }).await?;
        
        EventStream::new(receiver, EventUnsubscriber {
            actor_ref: self.actor_ref.clone(),
            event_type: "documentUpdated".to_string(),
            sender,
        })
    }
}

// Helper structures
struct EventUnsubscriber {
    actor_ref: ActorRef<DomActor>,
    event_type: String,
    sender: mpsc::Sender<DomEvent>,
}

impl Drop for EventUnsubscriber {
    fn drop(&mut self) {
        let actor_ref = self.actor_ref.clone();
        let event_type = self.event_type.clone();
        let sender = self.sender.clone();
        
        tokio::spawn(async move {
            let _ = actor_ref.tell(UnsubscribeFromEvent {
                event_type,
                sender,
            }).await;
        });
    }
}

// EventStream implementation
pub struct EventStream<T> {
    receiver: mpsc::Receiver<T>,
    _unsubscriber: EventUnsubscriber,
}

impl<T> Stream for EventStream<T> {
    type Item = T;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}
```

### 4.2 Event Dispatcher Pattern

The event dispatcher routes events to appropriate domain actors:

**Implementation Pattern:**
```rust
#[derive(Actor)]
struct EventDispatcherActor {
    domain_map: HashMap<String, ActorRef<dyn EventHandler>>,
}

#[messages]
impl EventDispatcherActor {
    #[message]
    async fn register_domain(&mut self, domain: String, handler: ActorRef<dyn EventHandler>) {
        self.domain_map.insert(domain, handler);
    }
    
    #[message]
    async fn dispatch_event(&mut self, event: RdpEvent) -> Result<(), EventError> {
        let domain = event.method.split('.').next().unwrap_or_default();
        
        if let Some(handler) = self.domain_map.get(domain) {
            handler.tell(HandleEvent { event }).await?;
        }
        
        Ok(())
    }
}

// EventHandler trait
#[async_trait]
trait EventHandler: Actor {
    async fn handle_event(&mut self, event: RdpEvent) -> Result<(), EventError>;
}

// Implementation for domain actors
#[async_trait]
impl EventHandler for DomActor {
    async fn handle_event(&mut self, event: RdpEvent) -> Result<(), EventError> {
        match event.method.as_str() {
            "DOM.documentUpdated" => {
                self.handle_document_updated(DocumentUpdated {}).await?;
            },
            "DOM.attributeModified" => {
                let params = serde_json::from_value::<AttributeModifiedParams>(event.params)?;
                self.handle_attribute_modified(AttributeModified { params }).await?;
            },
            // Other event types...
            _ => {
                // Unknown event
            }
        }
        
        Ok(())
    }
}
```

## 5. Error Handling Patterns

### 5.1 Error Propagation Pattern

Consistent error handling across layers:

**Implementation Pattern:**
```rust
// Error definitions
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Actor system error: {0}")]
    ActorSystem(String),
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Not connected")]
    NotConnected,
    
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Response channel closed")]
    ChannelClosed,
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Method error: {code} - {message}")]
    MethodError { code: i32, message: String },
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Empty response")]
    EmptyResponse,
    
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    
    #[error("Actor unavailable")]
    ActorUnavailable,
    
    #[error("Response channel closed")]
    ResponseChannelClosed,
}

// Error handling in actor methods
#[messages]
impl DomActor {
    #[message]
    async fn get_node_by_id(&mut self, node_id: NodeId) -> Result<Node, DomainError> {
        // Check cache first
        if let Some(node) = self.node_map.get(&node_id) {
            return Ok(node.clone());
        }
        
        // Create request
        let request = RdpMessage {
            id: self.next_id(),
            method: "DOM.getNode".to_string(),
            params: Some(json!({
                "nodeId": node_id
            })),
        };
        
        // Send request with error handling
        let response = self.connection.ask(SendRequest { 
            request, 
            timeout: self.request_timeout 
        })
        .await
        .map_err(|e| {
            // Log error
            log::error!("Failed to send request: {}", e);
            // Convert to domain error
            DomainError::Connection(e)
        })?;
        
        // Parse response with detailed error
        match response.result {
            Some(value) => {
                let node = serde_json::from_value::<Node>(value)
                    .map_err(|e| DomainError::ParseError(format!("Failed to parse node: {}", e)))?;
                
                // Update cache
                self.node_map.insert(node_id, node.clone());
                
                Ok(node)
            },
            None => {
                if let Some(error) = response.error {
                    Err(DomainError::MethodError { 
                        code: error.code, 
                        message: error.message 
                    })
                } else {
                    Err(DomainError::EmptyResponse)
                }
            }
        }
    }
}
```

### 5.2 Retry Pattern

For handling transient errors:

**Implementation Pattern:**
```rust
async fn with_retry<F, Fut, T, E>(
    operation: F,
    max_attempts: u32,
    retry_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    let mut last_error = None;
    
    while attempts < max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempts += 1;
                last_error = Some(error);
                
                if attempts < max_attempts {
                    tokio::time::sleep(retry_delay * attempts as u32).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

// Usage in connection actor
impl ConnectionActor {
    async fn connect_with_retry(&mut self, url: &str) -> Result<WebSocketStream, ConnectionError> {
        with_retry(
            || async {
                self.establish_connection(url).await
            },
            3,
            Duration::from_secs(1),
        ).await
    }
}
```

## 6. Type Conversion Patterns

### 6.1 Domain-Specific Type Conversion

Converting between protocol JSON and domain-specific types:

**Implementation Pattern:**
```rust
// Type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_id: NodeId,
    pub node_type: i32,
    pub node_name: String,
    pub local_name: String,
    pub node_value: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_node_count: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Node>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_url: Option<String>,
}

// With custom conversion functions
impl Node {
    pub fn get_attributes_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        if let Some(attributes) = &self.attributes {
            for chunk in attributes.chunks(2) {
                if chunk.len() == 2 {
                    map.insert(chunk[0].clone(), chunk[1].clone());
                }
            }
        }
        
        map
    }
    
    pub fn from_response(value: Value) -> Result<Self, serde_json::Error> {
        // Handle specific transformations if needed
        serde_json::from_value(value)
    }
}
```

### 6.2 Public API Type Conversion

Converting between internal actor types and public API types:

**Implementation Pattern:**
```rust
// Internal types (used in actors)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalBreakpoint {
    id: String,
    location: InternalLocation,
    condition: Option<String>,
    hit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalLocation {
    script_id: String,
    line_number: u32,
    column_number: Option<u32>,
}

// Public API types
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: String,
    pub location: SourceLocation,
    pub condition: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub script_id: String,
    pub line: u32,
    pub column: Option<u32>,
}

// Conversion functions
impl From<InternalBreakpoint> for Breakpoint {
    fn from(bp: InternalBreakpoint) -> Self {
        Breakpoint {
            id: bp.id,
            location: SourceLocation {
                script_id: bp.location.script_id,
                line: bp.location.line_number,
                column: bp.location.column_number,
            },
            condition: bp.condition,
        }
    }
}

impl From<Breakpoint> for InternalBreakpoint {
    fn from(bp: Breakpoint) -> Self {
        InternalBreakpoint {
            id: bp.id,
            location: InternalLocation {
                script_id: bp.location.script_id,
                line_number: bp.location.line,
                column_number: bp.location.column,
            },
            condition: bp.condition,
            hit_count: 0,
        }
    }
}

// Usage in actors
impl DebuggerActor {
    async fn set_breakpoint(&mut self, location: SourceLocation, condition: Option<String>) -> Result<Breakpoint, DomainError> {
        // Convert to internal format
        let internal_location = InternalLocation {
            script_id: location.script_id,
            line_number: location.line,
            column_number: location.column,
        };
        
        // Create request
        let request = RdpMessage {
            id: self.next_id(),
            method: "Debugger.setBreakpoint".to_string(),
            params: Some(json!({
                "location": internal_location,
                "condition": condition
            })),
        };
        
        // Send request
        let response = self.connection.ask(SendRequest { request, timeout: self.request_timeout }).await?;
        
        // Parse response
        let result = response.result.ok_or(DomainError::EmptyResponse)?;
        let internal_bp = serde_json::from_value::<InternalBreakpoint>(result)
            .map_err(|e| DomainError::ParseError(e.to_string()))?;
        
        // Convert to public API type
        Ok(Breakpoint::from(internal_bp))
    }
}
```

## 7. Resource Management Patterns

### 7.1 Resource Cleanup Pattern

Proper cleanup of resources:

**Implementation Pattern:**
```rust
impl Drop for RdpClient {
    fn drop(&mut self) {
        // Spawn cleanup task to avoid blocking in drop
        let connection = self.connection.clone();
        tokio::spawn(async move {
            // Attempt graceful disconnect
            let _ = connection.ask(Disconnect {}).await;
        });
    }
}

// Actor lifecycle hooks
#[messages]
impl ConnectionActor {
    async fn on_start(&mut self) -> Result<(), ConnectionError> {
        // Initialize resources
        Ok(())
    }
    
    async fn on_stop(&mut self) -> Result<(), ConnectionError> {
        // Clean up connections
        if let Some(socket) = &mut self.socket {
            let _ = socket.close(None).await;
        }
        
        // Clear request map and notify failures
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
}
```

### 7.2 Timeout Management Pattern

Consistent timeout handling:

**Implementation Pattern:**
```rust
// Client configuration
pub struct ClientConfig {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub event_buffer_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(10),
            event_buffer_size: 32,
        }
    }
}

// Builder pattern for client
pub struct RdpClientBuilder {
    config: ClientConfig,
}

impl RdpClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }
    
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.config.connection_timeout = timeout;
        self
    }
    
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.config.request_timeout = timeout;
        self
    }
    
    pub fn event_buffer_size(mut self, size: usize) -> Self {
        self.config.event_buffer_size = size;
        self
    }
    
    pub async fn build(self) -> Result<RdpClient, ClientError> {
        // Create actor system with configuration
        let system = ActorSystem::new();
        
        // Create actors with timeout configuration
        let connection = system.create_actor(ConnectionActor::new(
            self.config.connection_timeout,
        )).await?;
        
        let domain_manager = system.create_actor(DomainManagerActor::new(
            connection.clone(),
            self.config.request_timeout,
        )).await?;
        
        let event_dispatcher = system.create_actor(EventDispatcherActor::new(
            self.config.event_buffer_size,
        )).await?;
        
        Ok(RdpClient {
            connection,
            domain_manager,
            event_dispatcher,
        })
    }
}

// Usage
let client = RdpClientBuilder::new()
    .connection_timeout(Duration::from_secs(60))
    .request_timeout(Duration::from_secs(30))
    .build()
    .await?;
```

## 8. Testing Patterns

### 8.1 Mock Connection Pattern

Testing with mock connections:

**Implementation Pattern:**
```rust
// Mock connection for testing
#[derive(Clone)]
struct MockConnection {
    expected_requests: Arc<Mutex<Vec<(String, Value)>>>,
    responses: Arc<Mutex<HashMap<String, Value>>>,
    events: Arc<Mutex<Vec<RdpEvent>>>,
}

impl MockConnection {
    fn new() -> Self {
        Self {
            expected_requests: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn expect_request(&self, method: &str, response: Value) {
        let mut requests = self.expected_requests.lock().unwrap();
        let mut responses = self.responses.lock().unwrap();

        requests.push((method.to_string(), response.clone()));
        responses.insert(method.to_string(), response);
    }

    fn add_event(&self, event: RdpEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    async fn process_request(&self, request: RdpMessage) -> Result<RdpResponse, ConnectionError> {
        let method = request.method.clone();
        let responses = self.responses.lock().unwrap();

        if let Some(result) = responses.get(&method) {
            Ok(RdpResponse {
                id: request.id,
                result: Some(result.clone()),
                error: None,
            })
        } else {
            Ok(RdpResponse {
                id: request.id,
                result: None,
                error: Some(RdpError {
                    code: -32601,
                    message: format!("Method not found: {}", method),
                }),
            })
        }
    }
}

// Mock actor implementation
#[messages]
impl MockConnectionActor {
    #[message]
    async fn send_request(&mut self, request: RdpMessage, _timeout: Duration) -> Result<RdpResponse, ConnectionError> {
        self.mock.process_request(request).await
    }

    #[message]
    async fn connect(&mut self, _url: String) -> Result<(), ConnectionError> {
        // Emit queued events
        let events = self.mock.events.lock().unwrap().clone();
        for event in events {
            self.event_dispatcher.tell(DispatchEvent { event }).await?;
        }

        Ok(())
    }
}

// Test code
#[tokio::test]
async fn test_dom_get_document() {
    // Create mock
    let mock = MockConnection::new();

    // Set up expectations
    mock.expect_request(
        "DOM.getDocument",
        json!({
            "root": {
                "nodeId": 1,
                "nodeType": 9,
                "nodeName": "#document",
                "childNodeCount": 1,
                "children": [
                    {
                        "nodeId": 2,
                        "nodeType": 1,
                        "nodeName": "HTML",
                        "attributes": ["lang", "en"]
                    }
                ]
            }
        }),
    );

    // Create actor system with mocks
    let system = ActorSystem::new();
    let connection = system.create_actor(MockConnectionActor::new(mock)).await.unwrap();
    let dom_actor = system.create_actor(DomActor::new(connection)).await.unwrap();

    // Test the method
    let result = dom_actor.ask(GetDocument { depth: None }).await.unwrap();

    // Verify result
    assert_eq!(result.node_id, 1);
    assert_eq!(result.node_name, "#document");
    assert_eq!(result.children.as_ref().unwrap().len(), 1);
    assert_eq!(result.children.as_ref().unwrap()[0].node_id, 2);
}
```

### 8.2 Integration Test Pattern

End-to-end testing with a real Firefox instance:

**Implementation Pattern:**
```rust
struct FirefoxTestContext {
    firefox_process: Child,
    debug_port: u16,
    url: String,
}

impl FirefoxTestContext {
    async fn new() -> Result<Self, anyhow::Error> {
        // Find free port
        let debug_port = find_free_port()?;

        // Start Firefox with remote debugging
        let firefox_process = Command::new("firefox")
            .args(&[
                "--headless",
                "--remote-debugging-port",
                &debug_port.to_string(),
                "--no-remote",
                "--profile",
                "test_profile",
            ])
            .spawn()?;

        // Wait for Firefox to start
        tokio::time::sleep(Duration::from_secs(2)).await;

        Ok(Self {
            firefox_process,
            debug_port,
            url: format!("http://localhost:{}", debug_port),
        })
    }

    fn debug_url(&self) -> String {
        format!("ws://localhost:{}/devtools/browser", self.debug_port)
    }

    async fn get_target_id(&self) -> Result<String, anyhow::Error> {
        let client = reqwest::Client::new();
        let response = client.get(&format!("{}/json/list", self.url))
            .send()
            .await?
            .json::<Vec<serde_json::Value>>()
            .await?;

        let target_id = response.get(0)
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No targets found"))?
            .to_string();

        Ok(target_id)
    }
}

impl Drop for FirefoxTestContext {
    fn drop(&mut self) {
        // Kill Firefox process
        let _ = self.firefox_process.kill();
    }
}

// Integration test
#[tokio::test]
async fn integration_test_navigate() -> Result<(), anyhow::Error> {
    // Start Firefox
    let context = FirefoxTestContext::new().await?;

    // Create client
    let client = RdpClientBuilder::new()
        .connection_timeout(Duration::from_secs(60))
        .build()
        .await?;

    // Connect to Firefox
    client.connect(&context.debug_url()).await?;

    // Find target and attach
    let target_id = context.get_target_id().await?;
    let session = client.target().attach_to_target(target_id).await?;

    // Navigate page
    let navigation_result = client.page()
        .navigate("https://example.com")
        .await?;

    // Verify navigation succeeded
    assert!(navigation_result.frame_id.is_some());

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get page title
    let result = client.runtime()
        .evaluate("document.title", None)
        .await?;

    // Verify title
    assert_eq!(result.value.as_str().unwrap_or(""), "Example Domain");

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

## 9. Performance Optimization Patterns

### 9.1 DOM Query Optimization

Efficient DOM traversal and querying:

**Implementation Pattern:**
```rust
impl DomActor {
    // Get elements with efficient batching
    async fn get_elements_by_selector(&mut self, selector: String) -> Result<Vec<Element>, DomainError> {
        // Get document root (cached if possible)
        self.ensure_document().await?;

        let root_id = self.document_root.as_ref().unwrap().node_id;

        // Query all elements in one call
        let node_ids = self.query_selector_all(root_id, selector).await?;

        // Batch fetch element details
        let mut elements = Vec::with_capacity(node_ids.len());

        // Process in batches of 10 to avoid overloading
        for chunk in node_ids.chunks(10) {
            let mut futures = Vec::with_capacity(chunk.len());

            for &node_id in chunk {
                futures.push(self.get_element_details(node_id));
            }

            let results = futures::future::join_all(futures).await;

            for result in results {
                if let Ok(element) = result {
                    elements.push(element);
                }
            }
        }

        Ok(elements)
    }

    // Cache nodes to reduce requests
    fn cache_node(&mut self, node: Node) {
        self.node_map.insert(node.node_id, node.clone());

        // Also cache children if available
        if let Some(children) = &node.children {
            for child in children {
                self.cache_node(child.clone());
            }
        }
    }

    // Efficiently process only needed nodes
    async fn get_element_details(&mut self, node_id: NodeId) -> Result<Element, DomainError> {
        // Get node (from cache if possible)
        let node = self.get_node_by_id(node_id).await?;

        // Get computed styles only if needed (expensive operation)
        let computed_style = if self.include_styles {
            Some(self.css.get_computed_style_for_node(node_id).await?)
        } else {
            None
        };

        // Transform into user-friendly Element type
        Ok(Element {
            id: node.node_id,
            tag_name: node.node_name,
            attributes: node.get_attributes_map(),
            computed_style,
        })
    }
}
```

### 9.2 Event Debouncing Pattern

Debounce frequent events:

**Implementation Pattern:**
```rust
struct DebouncedEvent<T> {
    last_value: Option<T>,
    timer: Option<tokio::task::JoinHandle<()>>,
    delay: Duration,
    sender: mpsc::Sender<T>,
}

impl<T: Clone + Send + 'static> DebouncedEvent<T> {
    fn new(delay: Duration, sender: mpsc::Sender<T>) -> Self {
        Self {
            last_value: None,
            timer: None,
            delay,
            sender,
        }
    }

    fn update(&mut self, value: T) {
        // Store the latest value
        self.last_value = Some(value);

        // Cancel existing timer
        if let Some(timer) = self.timer.take() {
            timer.abort();
        }

        // Create new timer
        let delay = self.delay;
        let value = self.last_value.clone().unwrap();
        let sender = self.sender.clone();

        self.timer = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = sender.send(value).await;
        }));
    }
}

// Usage in domain actor
impl DomActor {
    async fn handle_attribute_modified(&mut self, event: AttributeModified) -> Result<(), DomainError> {
        // Update internal state
        if let Some(node) = self.node_map.get_mut(&event.params.node_id) {
            // Update attributes in cache
            let mut attributes = node.get_attributes_map();
            attributes.insert(event.params.name, event.params.value);

            // Update node attributes
            let mut attr_vec = Vec::new();
            for (name, value) in attributes {
                attr_vec.push(name);
                attr_vec.push(value);
            }
            node.attributes = Some(attr_vec);
        }

        // Debounce notifications
        if let Some(debounced) = self.debounced_events.get_mut("attributeModified") {
            debounced.update(DomEvent::AttributeModified(event.params));
        } else {
            // Directly notify if not debounced
            for sender in self.event_listeners.get("attributeModified").unwrap_or(&Vec::new()) {
                let _ = sender.send(DomEvent::AttributeModified(event.params.clone())).await;
            }
        }

        Ok(())
    }

    fn setup_debounced_events(&mut self) {
        // Setup debounced event handlers
        let attr_mod_sender = self.create_event_broadcaster("attributeModified");
        self.debounced_events.insert(
            "attributeModified".to_string(),
            DebouncedEvent::new(Duration::from_millis(100), attr_mod_sender)
        );
    }

    fn create_event_broadcaster(&self, event_type: &str) -> mpsc::Sender<DomEvent> {
        let (sender, mut receiver) = mpsc::channel(16);
        let listeners = self.event_listeners.get(event_type).cloned().unwrap_or_default();
        let listeners = Arc::new(Mutex::new(listeners));

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let listeners = listeners.lock().await;
                for listener in listeners.iter() {
                    let _ = listener.send(event.clone()).await;
                }
            }
        });

        sender
    }
}
```

### 9.3 Resource Pooling Pattern

Connection and resource pooling:

**Implementation Pattern:**
```rust
#[derive(Actor)]
struct ConnectionPool {
    idle_connections: Vec<WebSocketStream>,
    active_connections: HashMap<usize, WebSocketStream>,
    next_id: usize,
    max_connections: usize,
    connect_timeout: Duration,
}

#[messages]
impl ConnectionPool {
    #[message]
    async fn get_connection(&mut self, url: String) -> Result<ConnectionHandle, ConnectionError> {
        // Reuse idle connection if available
        if let Some(connection) = self.idle_connections.pop() {
            let id = self.next_id;
            self.next_id += 1;
            self.active_connections.insert(id, connection);
            return Ok(ConnectionHandle { id });
        }

        // Check if we can create a new connection
        if self.active_connections.len() >= self.max_connections {
            return Err(ConnectionError::PoolExhausted);
        }

        // Create new connection
        let connection = self.establish_connection(&url).await?;
        let id = self.next_id;
        self.next_id += 1;
        self.active_connections.insert(id, connection);

        Ok(ConnectionHandle { id })
    }

    #[message]
    async fn release_connection(&mut self, handle: ConnectionHandle) -> Result<(), ConnectionError> {
        if let Some(connection) = self.active_connections.remove(&handle.id) {
            self.idle_connections.push(connection);
        }

        Ok(())
    }

    #[message]
    async fn send_request(&mut self, handle: ConnectionHandle, request: RdpMessage) -> Result<RdpResponse, ConnectionError> {
        let connection = self.active_connections.get_mut(&handle.id)
            .ok_or(ConnectionError::InvalidHandle)?;

        // Send request and get response
        // ...implementation...

        Ok(response)
    }
}

// ConnectionHandle for client code
#[derive(Clone, Copy)]
pub struct ConnectionHandle {
    id: usize,
}

// Usage
impl ConnectionActor {
    async fn with_pooled_connection<F, T>(&self, url: &str, operation: F) -> Result<T, ConnectionError>
    where
        F: FnOnce(ConnectionHandle) -> Future<Output = Result<T, ConnectionError>>,
    {
        // Get connection from pool
        let handle = self.pool.ask(GetConnection { url: url.to_string() }).await?;

        // Execute operation
        let result = operation(handle).await;

        // Always release connection
        let _ = self.pool.tell(ReleaseConnection { handle }).await;

        result
    }
}
```

## 10. Concurrency Management Patterns

### 10.1 Request Batching Pattern

Batch related requests for efficiency:

**Implementation Pattern:**
```rust
struct BatchedRequester {
    connection: ActorRef<ConnectionActor>,
    batch_size: usize,
    request_timeout: Duration,
}

impl BatchedRequester {
    async fn execute_batch<T, F, Fut>(
        &self,
        items: Vec<T>,
        create_request: F
    ) -> Result<Vec<RdpResponse>, ConnectionError>
    where
        F: Fn(T) -> RdpMessage,
        T: Clone,
    {
        let mut results = Vec::with_capacity(items.len());

        // Process in batches
        for chunk in items.chunks(self.batch_size) {
            let mut futures = Vec::with_capacity(chunk.len());

            // Create futures for each request in batch
            for item in chunk {
                let request = create_request(item.clone());
                futures.push(self.connection.ask(SendRequest {
                    request,
                    timeout: self.request_timeout,
                }));
            }

            // Execute batch concurrently
            let batch_results = futures::future::join_all(futures).await;
            results.extend(batch_results.into_iter().collect::<Result<Vec<_>, _>>()?);
        }

        Ok(results)
    }
}

// Usage
impl DomActor {
    async fn get_attributes_batch(&mut self, node_ids: Vec<NodeId>) -> Result<HashMap<NodeId, HashMap<String, String>>, DomainError> {
        let batch_requester = BatchedRequester {
            connection: self.connection.clone(),
            batch_size: 10,
            request_timeout: self.request_timeout,
        };

        // Create requests for each node ID
        let responses = batch_requester.execute_batch(
            node_ids,
            |node_id| RdpMessage {
                id: self.next_id(),
                method: "DOM.getAttributes".to_string(),
                params: Some(json!({ "nodeId": node_id })),
            }
        ).await?;

        // Process responses
        let mut results = HashMap::new();
        for (i, response) in responses.into_iter().enumerate() {
            let node_id = node_ids.get(i)
                .ok_or_else(|| DomainError::ParseError("Node ID index out of bounds".to_string()))?;

            if let Some(result) = response.result {
                let attributes: Vec<String> = serde_json::from_value(result)
                    .map_err(|e| DomainError::ParseError(e.to_string()))?;

                // Convert attribute list to map
                let mut attr_map = HashMap::new();
                for chunk in attributes.chunks(2) {
                    if chunk.len() == 2 {
                        attr_map.insert(chunk[0].clone(), chunk[1].clone());
                    }
                }

                results.insert(*node_id, attr_map);
            }
        }

        Ok(results)
    }
}
```

### 10.2 Cancellation Pattern

Support for cancellable operations:

**Implementation Pattern:**
```rust
// Cancellable operation handle
pub struct CancellableOperation<T> {
    result_receiver: oneshot::Receiver<Result<T, DomainError>>,
    cancel_sender: Option<oneshot::Sender<()>>,
}

impl<T> CancellableOperation<T> {
    fn new(result_receiver: oneshot::Receiver<Result<T, DomainError>>, cancel_sender: oneshot::Sender<()>) -> Self {
        Self {
            result_receiver,
            cancel_sender: Some(cancel_sender),
        }
    }

    pub async fn cancel(&mut self) {
        if let Some(sender) = self.cancel_sender.take() {
            let _ = sender.send(());
        }
    }

    pub async fn wait(self) -> Result<T, DomainError> {
        self.result_receiver.await
            .map_err(|_| DomainError::OperationCancelled)?
    }
}

// Implementation in domain actor
impl DomActor {
    async fn find_elements_cancellable(&mut self, selector: String) -> CancellableOperation<Vec<Element>> {
        let (result_sender, result_receiver) = oneshot::channel();
        let (cancel_sender, mut cancel_receiver) = oneshot::channel();

        let actor_ref = self.self_ref.clone();
        let selector_clone = selector.clone();

        // Spawn task for operation
        tokio::spawn(async move {
            // Create a future for the actual operation
            let operation_future = actor_ref.ask(FindElements { selector: selector_clone });

            // Run operation with cancellation
            let result = tokio::select! {
                result = operation_future => result,
                _ = &mut cancel_receiver => Err(DomainError::OperationCancelled),
            };

            // Send result back
            let _ = result_sender.send(result);
        });

        CancellableOperation::new(result_receiver, cancel_sender)
    }
}

// Usage example
async fn find_with_timeout(dom: &DomDomain, selector: &str, timeout: Duration) -> Result<Vec<Element>, DomainError> {
    let mut operation = dom.find_elements_cancellable(selector.to_string()).await;

    // Create timeout future
    let timeout_future = tokio::time::sleep(timeout);

    // Wait for result or timeout
    tokio::select! {
        result = operation.wait() => result,
        _ = timeout_future => {
            operation.cancel().await;
            Err(DomainError::Timeout)
        }
    }
}
```

## 11. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial implementation patterns | AI Assistant |
