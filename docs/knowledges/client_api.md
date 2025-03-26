# Firefox DevTools Protocol Client API

## 1. API Overview

This document describes the client API for interacting with the Firefox DevTools Protocol (FDP). The API provides a type-safe, ergonomic interface for remote browser debugging.

### 1.1 Design Principles

- **Type Safety**: Leverage Rust's type system
- **Async-First**: Built on async/await
- **Error Handling**: Comprehensive error handling
- **Resource Management**: Automatic cleanup
- **Domain Organization**: Matches protocol domains

## 2. Core API Components

### 2.1 Client Interface

```rust
/// Main entry point for Firefox DevTools Protocol
pub struct FirefoxDevToolsClient {
    actor_system: ActorSystem,
    root_actor: ActorRef<RootActor>,
}

impl FirefoxDevToolsClient {
    /// Create a new client instance
    pub async fn new() -> Result<Self, ClientError> {
        // Initialize actor system and create root actor
    }

    /// Connect to Firefox instance
    pub async fn connect(&self, url: &str) -> Result<(), ClientError> {
        // Connect to remote Firefox instance
    }

    /// Disconnect from Firefox instance
    pub async fn disconnect(&self) -> Result<(), ClientError> {
        // Disconnect from remote Firefox instance
    }

    /// Access Browser domain
    pub fn browser(&self) -> BrowserDomain {
        BrowserDomain::new(self.root_actor.clone())
    }

    /// Access Target domain
    pub fn target(&self) -> TargetDomain {
        TargetDomain::new(self.root_actor.clone())
    }

    // Other domain accessors...
}
```

### 2.2 Domain API Pattern

Each domain follows a consistent pattern:

```rust
/// Browser domain API
pub struct BrowserDomain {
    actor_ref: ActorRef<BrowserActor>,
}

impl BrowserDomain {
    pub fn new(root_actor: ActorRef<RootActor>) -> Self {
        // Initialize domain
    }

    /// Get browser version information
    pub async fn get_version(&self) -> Result<BrowserVersion, DomainError> {
        // Send message to actor and await response
    }

    /// Close browser
    pub async fn close(&self) -> Result<(), DomainError> {
        // Send message to actor and await response
    }

    // Other browser methods...
}
```

## 3. Domain APIs

### 3.1 Browser Domain

```rust
impl BrowserDomain {
    /// Get browser version information
    pub async fn get_version(&self) -> Result<BrowserVersion, DomainError>;

    /// Get window for target
    pub async fn get_window_for_target(&self, target_id: String) -> Result<WindowInfo, DomainError>;

    /// Close browser
    pub async fn close(&self) -> Result<(), DomainError>;
}
```

### 3.2 Target Domain

```rust
impl TargetDomain {
    /// Get all available targets
    pub async fn get_targets(&self) -> Result<Vec<TargetInfo>, DomainError>;

    /// Attach to target
    pub async fn attach_to_target(&self, target_id: String) -> Result<SessionInfo, DomainError>;

    /// Detach from target
    pub async fn detach_from_target(&self, session_id: String) -> Result<(), DomainError>;

    /// Create new target
    pub async fn create_target(&self, url: String) -> Result<TargetInfo, DomainError>;

    /// Close target
    pub async fn close_target(&self, target_id: String) -> Result<bool, DomainError>;

    /// Subscribe to target created events
    pub async fn on_target_created(&self) -> EventStream<TargetInfo>;

    /// Subscribe to target destroyed events
    pub async fn on_target_destroyed(&self) -> EventStream<String>;
}
```

### 3.3 Page Domain

```rust
impl PageDomain {
    /// Navigate to URL
    pub async fn navigate(&self, url: String) -> Result<NavigationResult, DomainError>;

    /// Reload page
    pub async fn reload(&self, ignore_cache: bool) -> Result<(), DomainError>;

    /// Capture screenshot
    pub async fn capture_screenshot(&self, format: Option<String>) -> Result<Vec<u8>, DomainError>;

    /// Print to PDF
    pub async fn print_to_pdf(&self, options: PdfOptions) -> Result<Vec<u8>, DomainError>;

    /// Subscribe to frame navigated events
    pub async fn on_frame_navigated(&self) -> EventStream<FrameInfo>;

    /// Subscribe to load event
    pub async fn on_load_event_fired(&self) -> EventStream<Timestamp>;
}
```

### 3.4 DOM Domain

```rust
impl DomDomain {
    /// Get document
    pub async fn get_document(&self, depth: Option<i32>) -> Result<Node, DomainError>;

    /// Query selector
    pub async fn query_selector(&self, node_id: NodeId, selector: String) -> Result<NodeId, DomainError>;

    /// Query selector all
    pub async fn query_selector_all(&self, node_id: NodeId, selector: String) -> Result<Vec<NodeId>, DomainError>;

    /// Set attribute value
    pub async fn set_attribute_value(&self, node_id: NodeId, name: String, value: String) -> Result<(), DomainError>;

    /// Remove node
    pub async fn remove_node(&self, node_id: NodeId) -> Result<(), DomainError>;

    /// Subscribe to document updated events
    pub async fn on_document_updated(&self) -> EventStream<()>;

    /// Subscribe to attribute modified events
    pub async fn on_attribute_modified(&self) -> EventStream<AttributeModifiedEvent>;

    /// Subscribe to child node inserted events
    pub async fn on_child_node_inserted(&self) -> EventStream<ChildNodeInsertedEvent>;
}
```

### 3.5 CSS Domain

```rust
impl CssDomain {
    /// Get matched styles for node
    pub async fn get_matched_styles_for_node(&self, node_id: NodeId) -> Result<MatchedStyles, DomainError>;

    /// Get computed style for node
    pub async fn get_computed_style_for_node(&self, node_id: NodeId) -> Result<ComputedStyle, DomainError>;

    /// Set style texts
    pub async fn set_style_texts(&self, edits: Vec<StyleEdit>) -> Result<Vec<Style>, DomainError>;

    /// Subscribe to stylesheet added events
    pub async fn on_stylesheet_added(&self) -> EventStream<StyleSheet>;

    /// Subscribe to stylesheet removed events
    pub async fn on_stylesheet_removed(&self) -> EventStream<StyleSheetId>;
}
```

### 3.6 Network Domain

```rust
impl NetworkDomain {
    /// Enable network monitoring
    pub async fn enable(&self) -> Result<(), DomainError>;

    /// Disable network monitoring
    pub async fn disable(&self) -> Result<(), DomainError>;

    /// Get cookies
    pub async fn get_cookies(&self, urls: Option<Vec<String>>) -> Result<Vec<Cookie>, DomainError>;

    /// Delete cookies
    pub async fn delete_cookies(&self, cookie_info: CookieDeleteInfo) -> Result<(), DomainError>;

    /// Subscribe to request will be sent events
    pub async fn on_request_will_be_sent(&self) -> EventStream<RequestInfo>;

    /// Subscribe to response received events
    pub async fn on_response_received(&self) -> EventStream<ResponseInfo>;

    /// Subscribe to loading finished events
    pub async fn on_loading_finished(&self) -> EventStream<LoadingFinishedInfo>;
}
```

### 3.7 Console Domain

```rust
impl ConsoleDomain {
    /// Enable console monitoring
    pub async fn enable(&self) -> Result<(), DomainError>;

    /// Disable console monitoring
    pub async fn disable(&self) -> Result<(), DomainError>;

    /// Clear console messages
    pub async fn clear_messages(&self) -> Result<(), DomainError>;

    /// Subscribe to message added events
    pub async fn on_message_added(&self) -> EventStream<ConsoleMessage>;
}
```

### 3.8 Debugger Domain

```rust
impl DebuggerDomain {
    /// Enable debugger
    pub async fn enable(&self) -> Result<(), DomainError>;

    /// Disable debugger
    pub async fn disable(&self) -> Result<(), DomainError>;

    /// Set breakpoint
    pub async fn set_breakpoint(&self, location: BreakpointLocation) -> Result<BreakpointId, DomainError>;

    /// Remove breakpoint
    pub async fn remove_breakpoint(&self, breakpoint_id: BreakpointId) -> Result<(), DomainError>;

    /// Pause execution
    pub async fn pause(&self) -> Result<(), DomainError>;

    /// Resume execution
    pub async fn resume(&self) -> Result<(), DomainError>;

    /// Step over
    pub async fn step_over(&self) -> Result<(), DomainError>;

    /// Step into
    pub async fn step_into(&self) -> Result<(), DomainError>;

    /// Step out
    pub async fn step_out(&self) -> Result<(), DomainError>;

    /// Evaluate on call frame
    pub async fn evaluate_on_call_frame(&self, frame_id: CallFrameId, expression: String) -> Result<RemoteObject, DomainError>;

    /// Subscribe to script parsed events
    pub async fn on_script_parsed(&self) -> EventStream<ScriptInfo>;

    /// Subscribe to paused events
    pub async fn on_paused(&self) -> EventStream<PausedInfo>;

    /// Subscribe to resumed events
    pub async fn on_resumed(&self) -> EventStream<()>;
}
```

### 3.9 Runtime Domain

```rust
impl RuntimeDomain {
    /// Evaluate expression
    pub async fn evaluate(&self, expression: String, options: EvaluateOptions) -> Result<EvaluationResult, DomainError>;

    /// Call function on object
    pub async fn call_function_on(&self, object_id: RemoteObjectId, function: String, args: Vec<CallArgument>) -> Result<RemoteObject, DomainError>;

    /// Get properties
    pub async fn get_properties(&self, object_id: RemoteObjectId, options: PropertyOptions) -> Result<Vec<PropertyDescriptor>, DomainError>;

    /// Subscribe to execution context created events
    pub async fn on_execution_context_created(&self) -> EventStream<ExecutionContextDescription>;

    /// Subscribe to execution context destroyed events
    pub async fn on_execution_context_destroyed(&self) -> EventStream<ExecutionContextId>;
}
```

### 3.10 Performance Domain

```rust
impl PerformanceDomain {
    /// Enable performance monitoring
    pub async fn enable(&self) -> Result<(), DomainError>;

    /// Disable performance monitoring
    pub async fn disable(&self) -> Result<(), DomainError>;

    /// Get metrics
    pub async fn get_metrics(&self) -> Result<Vec<Metric>, DomainError>;

    /// Subscribe to metrics events
    pub async fn on_metrics(&self) -> EventStream<MetricsEvent>;
}
```

## 4. Event Handling

### 4.1 Event Stream API

```rust
/// Stream of events from a specific domain
pub struct EventStream<T> {
    receiver: mpsc::Receiver<T>,
    _unsubscribe: Arc<dyn Drop>,
}

impl<T> EventStream<T> {
    /// Create a new event stream
    fn new(receiver: mpsc::Receiver<T>, unsubscribe: impl Drop + 'static) -> Self {
        Self {
            receiver,
            _unsubscribe: Arc::new(unsubscribe),
        }
    }
}

impl<T> Stream for EventStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}
```

### 4.2 Event Subscription Example

```rust
// Subscribe to console messages
let mut console_messages = client.console().on_message_added().await;

// Process events using Stream API
while let Some(message) = console_messages.next().await {
    println!("Console message: {:?}", message);
}

// Alternative pattern with for_each
client.console().on_message_added().await
    .for_each(|message| {
        println!("Console message: {:?}", message);
        future::ready(())
    })
    .await;
```

## 5. Error Handling

### 5.1 Error Types

```rust
/// Client error types
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Timeout: operation took longer than {0:?}")]
    Timeout(Duration),

    #[error("Actor system error: {0}")]
    ActorSystem(String),
}

/// Domain-specific error
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Method error: {code} - {message}")]
    Method { code: i32, message: String },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Target closed")]
    TargetClosed,
}
```

### 5.2 Error Handling Examples

```rust
// Basic error handling
match client.dom().get_document(None).await {
    Ok(document) => {
        // Process document
    },
    Err(err) => {
        eprintln!("Error getting document: {}", err);
    }
}

// With anyhow
let document = client.dom().get_document(None)
    .await
    .context("Failed to get document")?;

// With error conversion
let result = client.runtime().evaluate("document.title", Default::default())
    .await
    .map_err(|e| format!("Evaluation failed: {}", e))?;
```

## 6. Advanced Usage

### 6.1 Concurrent Operations

```rust
// Execute multiple operations concurrently
let (version, targets) = tokio::join!(
    client.browser().get_version(),
    client.target().get_targets()
);

println!("Browser version: {:?}", version?);
println!("Available targets: {:?}", targets?);
```

### 6.2 Timeouts

```rust
// Set global client timeout
let client = FirefoxDevToolsClient::builder()
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// Set per-request timeout
let result = tokio::time::timeout(
    Duration::from_secs(5),
    client.runtime().evaluate("slow_operation()", Default::default())
).await??;
```

### 6.3 Bulk Operations

```rust
// Batch DOM operations
async fn get_element_details(client: &FirefoxDevToolsClient, selector: &str) -> Result<ElementDetails, anyhow::Error> {
    let dom = client.dom();
    let document = dom.get_document(None).await?;

    let element_id = dom.query_selector(document.node_id, selector.to_string()).await?;

    let (attributes, styles) = tokio::join!(
        dom.get_attributes(element_id),
        client.css().get_computed_style_for_node(element_id)
    );

    Ok(ElementDetails {
        node_id: element_id,
        attributes: attributes?,
        styles: styles?,
    })
}
```

## 7. Implementation Details

### 7.1 Actor Message Flow

1. Client API calls a method on a domain
2. Domain creates a message and sends it to its actor
3. Actor processes the message and sends a protocol request
4. Actor waits for response and processes it
5. Result is sent back to the client

```rust
// Inside BrowserDomain implementation
pub async fn get_version(&self) -> Result<BrowserVersion, DomainError> {
    // Create a one-shot channel for the response
    let (sender, receiver) = oneshot::channel();

    // Send message to actor
    self.actor_ref.send(GetVersion { response_channel: sender }).await
        .map_err(|_| DomainError::ActorUnavailable)?;

    // Await response
    receiver.await
        .map_err(|_| DomainError::ResponseChannelClosed)?
}

// Inside BrowserActor implementation
#[message]
async fn get_version(&mut self, response_channel: oneshot::Sender<Result<BrowserVersion, DomainError>>) {
    let result = self.send_request("Browser.getVersion", None).await
        .and_then(|value| {
            // Parse response into BrowserVersion
            serde_json::from_value(value)
                .map_err(|e| DomainError::InvalidResponse(e.to_string()))
        });

    // Send result back through channel
    let _ = response_channel.send(result);
}
```

### 7.2 Event Dispatching

```rust
// Inside ConnectionActor implementation
fn handle_event(&mut self, event: RdpEvent) {
    // Determine event type and target
    let parts: Vec<&str> = event.method.splitn(2, '.').collect();
    if parts.len() != 2 {
        return;
    }

    let domain = parts[0];
    let event_type = parts[1];

    // Dispatch to appropriate domain actor
    match domain {
        "Browser" => self.dispatch_to_domain(self.browser_actor.clone(), event),
        "DOM" => self.dispatch_to_domain(self.dom_actor.clone(), event),
        // Other domains...
        _ => log::warn!("Unknown domain in event: {}", domain),
    }
}
```

## 8. Best Practices

### 8.1 Resource Management

1. Always disconnect the client when done
2. Unsubscribe from events when no longer needed (done automatically when EventStream is dropped)
3. Set appropriate timeouts for operations
4. Use bounded channels for event streams

### 8.2 Error Handling

1. Handle domain-specific errors appropriately
2. Implement retry logic for transient failures
3. Add context to errors with anyhow or similar
4. Log detailed error information

### 8.3 Performance

1. Reuse client for multiple operations
2. Use concurrent operations where applicable
3. Keep event subscriptions focused
4. Batch related operations

## 9. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial client API documentation | AI Assistant |
