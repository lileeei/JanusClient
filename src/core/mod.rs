use async_trait::async_trait;
use serde_json::Value;
use crate::error::DebuggerError;

/// Browser debugger trait that defines the core functionality
#[async_trait]
pub trait BrowserDebugger {
    /// Connect to a browser debugging endpoint
    async fn connect(&mut self, endpoint: &str) -> Result<(), DebuggerError>;
    
    /// Disconnect from the browser
    async fn disconnect(&mut self) -> Result<(), DebuggerError>;
    
    /// Get a list of available pages
    async fn get_pages(&self) -> Result<Vec<Box<dyn Page>>, DebuggerError>;
    
    /// Execute JavaScript in the context of a page
    async fn execute_script(&self, page_id: &str, script: &str) -> Result<Value, DebuggerError>;
    
    /// Create a new page/tab
    async fn create_page(&mut self, url: Option<&str>) -> Result<Box<dyn Page>, DebuggerError>;
    
    /// Close a page/tab
    async fn close_page(&mut self, page_id: &str) -> Result<(), DebuggerError>;
    
    /// Get browser version information
    async fn get_browser_version(&self) -> Result<String, DebuggerError>;
}

/// Page interface for interacting with browser pages/tabs
#[async_trait]
pub trait Page: Send + Sync {
    /// Get the unique identifier for this page
    fn get_id(&self) -> &str;
    
    /// Get the current URL of the page
    fn get_url(&self) -> &str;
    
    /// Get the page title
    fn get_title(&self) -> &str;
    
    /// Navigate to a URL
    async fn navigate(&mut self, url: &str) -> Result<(), DebuggerError>;
    
    /// Reload the page
    async fn reload(&mut self, ignore_cache: bool) -> Result<(), DebuggerError>;
    
    /// Get the DOM interface for this page
    fn get_dom(&self) -> Box<dyn Dom>;
    
    /// Get the Network interface for this page
    fn get_network(&self) -> Box<dyn Network>;
    
    /// Take a screenshot of the page
    async fn take_screenshot(&self, format: &str) -> Result<Vec<u8>, DebuggerError>;
}

/// DOM manipulation interface
#[async_trait]
pub trait Dom: Send + Sync {
    /// Query for elements using a CSS selector
    async fn query_selector(&self, selector: &str) -> Result<Vec<Element>, DebuggerError>;
    
    /// Get the computed style for an element
    async fn get_computed_style(&self, element: &Element) -> Result<Value, DebuggerError>;
    
    /// Set style text for an element
    async fn set_style_text(&self, element: &Element, style: &str) -> Result<(), DebuggerError>;
}

/// Network monitoring interface
#[async_trait]
pub trait Network: Send + Sync {
    /// Enable network monitoring
    async fn enable(&mut self) -> Result<(), DebuggerError>;
    
    /// Disable network monitoring
    async fn disable(&mut self) -> Result<(), DebuggerError>;
    
    /// Get network requests
    async fn get_requests(&self) -> Result<Vec<NetworkRequest>, DebuggerError>;
    
    /// Clear network requests
    async fn clear(&mut self) -> Result<(), DebuggerError>;
}

/// Element representation
#[derive(Debug, Clone)]
pub struct Element {
    pub node_id: i32,
    pub tag_name: String,
    pub attributes: Vec<(String, String)>,
}

/// Network request representation
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub status: Option<i32>,
    pub status_text: Option<String>,
} 