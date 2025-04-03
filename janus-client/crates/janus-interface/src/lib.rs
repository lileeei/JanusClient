use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;
use janus_core::error::{CoreError, ProtocolError, TransportError}; // Import internal errors

// --- Placeholder Types (Define properly or remove if not needed yet) ---
#[derive(Debug, Clone)]
pub struct ElementHandle { /* Opaque handle representation */ pub internal_id: String }
#[derive(Debug, Clone)]
pub struct ConsoleMessage { /* Details of console message */ pub text: String }
#[derive(Debug, Clone)]
pub enum ScreenshotFormat { Jpeg, Png, Webp }
#[derive(Debug, Clone, Default)]
pub struct ScreenshotOptions { /* Quality, clip rect etc. */ pub quality: Option<u8> }
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64); // Example simple subscription ID

// --- L1 API Error Type ---
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String), // Can wrap TransportError details
    #[error("Operation timed out")]
    Timeout, // Simplified from core::ProtocolError::Timeout
    #[error("Protocol error: {0}")]
    ProtocolError(String), // Can wrap core::ProtocolError details
    #[error("Browser process crashed or closed unexpectedly")]
    BrowserCrashed,
    #[error("Invalid parameters provided: {0}")]
    InvalidParameters(String),
    #[error("Target (e.g., Page) not found or closed")]
    TargetNotFound,
    #[error("Navigation failed: {0}")]
    NavigationError(String),
    #[error("Script execution failed: {0}")]
    ScriptError(String),
    #[error("DOM operation failed: {0}")]
    DomError(String),
    #[error("Feature not supported by this browser/protocol")]
    NotSupported,
    #[error("Internal client error: {0}")]
    InternalError(String), // For unexpected issues within the client library
    // Add more specific high-level errors as needed
}

// --- Conversion from internal errors to public API error ---
impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            // Transport Errors
            CoreError::Transport(t_err) => match t_err {
                TransportError::ConnectionFailed(reason) |
                TransportError::TlsError(reason) |
                TransportError::InvalidUrl(reason) => ApiError::ConnectionFailed(reason),
                TransportError::Timeout(reason) => ApiError::ConnectionFailed(format!("Timeout during connection: {}", reason)), // Or specific ApiError::Timeout? Depends on context.
                TransportError::ConnectionClosed { .. } => ApiError::BrowserCrashed, // Assuming unexpected close means crash
                TransportError::NotConnected => ApiError::ConnectionFailed("Not connected".to_string()), // Or maybe InternalError?
                TransportError::WebSocket(reason) | // Treat WS protocol errors as general protocol errors
                TransportError::Serde(reason) => ApiError::ProtocolError(reason),
                TransportError::UnsupportedScheme(scheme) => ApiError::ConnectionFailed(format!("Unsupported protocol scheme: {}", scheme)),
                TransportError::SendFailed(reason) |
                TransportError::ReceiveFailed(reason) => ApiError::ProtocolError(format!("Message transport failed: {}", reason)), // Map to ProtocolError as it affects commands/events
                TransportError::Io(reason) => ApiError::ConnectionFailed(format!("Network I/O error: {}", reason)),
                TransportError::Internal(reason) => ApiError::InternalError(format!("Transport layer internal error: {}", reason)),
            },

            // Protocol Errors
            CoreError::Protocol(p_err) => match p_err {
                ProtocolError::Timeout => ApiError::Timeout,
                ProtocolError::InvalidRequest(reason) => ApiError::InvalidParameters(reason),
                ProtocolError::BrowserError { message, .. } => ApiError::ProtocolError(message), // Use browser's message
                ProtocolError::ResponseParseError { reason, .. } |
                ProtocolError::EventParseError { reason, .. } |
                ProtocolError::SerializationError(reason) => ApiError::ProtocolError(format!("Protocol serialization/parsing error: {}", reason)),
                ProtocolError::TargetOrSessionNotFound(id) => ApiError::TargetNotFound, // Specific target not found error
                ProtocolError::Internal(reason) => ApiError::InternalError(format!("Protocol layer internal error: {}", reason)),
            },

            // Core Errors
            CoreError::Config(cfg_err) => ApiError::InternalError(format!("Configuration error: {}", cfg_err)),
            CoreError::ActorSystem(reason) |
            CoreError::Internal(reason) => ApiError::InternalError(reason),
            CoreError::ActorMailbox(mb_err) => ApiError::InternalError(format!("Internal communication error: {}", mb_err)),
            CoreError::ResourceInitialization(reason) => ApiError::ConnectionFailed(format!("Failed to initialize browser resource: {}", reason)),
        }
    }
}

// --- L1 Browser Trait ---
#[async_trait]
pub trait Browser: Send + Sync { // Ensure Send + Sync for async usage
    // Lifecycle & Connection
    // async fn connect(&mut self) -> Result<(), ApiError>; // Connect might be implicit in creation
    async fn disconnect(&mut self) -> Result<(), ApiError>;
    async fn close(&mut self) -> Result<(), ApiError>; // Close the browser process

    // Page Management
    async fn new_page(&self) -> Result<Box<dyn Page>, ApiError>;
    async fn pages(&self) -> Result<Vec<Box<dyn Page>>, ApiError>; // Get handles to existing pages

    // Browser-level operations
    async fn version(&self) -> Result<String, ApiError>;

    // Event Subscription (Example - requires more design for handler lifetime/sync)
    // async fn on_target_created(&self, handler: Box<dyn Fn(Box<dyn Page>) + Send + Sync + 'static>) -> Result<SubscriptionId, ApiError>;
    // async fn unsubscribe(&self, id: SubscriptionId) -> Result<(), ApiError>;

    // Add methods for other browser-level features: Contexts, Permissions, Cookies etc.
}

// --- L1 Page Trait ---
#[async_trait]
pub trait Page: Send + Sync { // Ensure Send + Sync for async usage
    // Navigation
    async fn navigate(&self, url: &str) -> Result<(), ApiError>;
    async fn reload(&self) -> Result<(), ApiError>;
    async fn go_back(&self) -> Result<(), ApiError>;
    async fn go_forward(&self) -> Result<(), ApiError>;

    // Lifecycle
    async fn close(&self) -> Result<(), ApiError>;
    fn id(&self) -> String; // Get the page/target identifier (sync)

    // Content & Scripting
    async fn content(&self) -> Result<String, ApiError>; // Get HTML content
    async fn evaluate_script(&self, script: &str) -> Result<Value, ApiError>;
    async fn call_function(&self, function_declaration: &str, args: Vec<Value>) -> Result<Value, ApiError>;

    // DOM Interaction (Simplified)
    async fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>, ApiError>;
    async fn wait_for_selector(&self, selector: &str, timeout_ms: Option<u64>) -> Result<ElementHandle, ApiError>;

    // Input (Simplified)
    // async fn click(&self, selector: &str) -> Result<(), ApiError>;
    // async fn type_text(&self, selector: &str, text: &str) -> Result<(), ApiError>;

    // Information
    async fn url(&self) -> Result<String, ApiError>;
    async fn title(&self) -> Result<String, ApiError>;

    // Screenshot
    async fn take_screenshot(&self, format: ScreenshotFormat, options: Option<ScreenshotOptions>) -> Result<Vec<u8>, ApiError>;

    // Event Subscription (Example - requires more design)
    // async fn on_load(&self, handler: Box<dyn Fn() + Send + Sync + 'static>) -> Result<SubscriptionId, ApiError>;
    // async fn on_console_message(&self, handler: Box<dyn Fn(ConsoleMessage) + Send + Sync + 'static>) -> Result<SubscriptionId, ApiError>;
}

// --- L1 ElementHandle Trait (Example) ---
// #[async_trait]
// pub trait Element: Send + Sync {
//     async fn click(&self) -> Result<(), ApiError>;
//     async fn type_text(&self, text: &str) -> Result<(), ApiError>;
//     async fn inner_text(&self) -> Result<String, ApiError>;
//     // ... other element operations
// }
