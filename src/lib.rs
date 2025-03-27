pub mod core;
pub mod adapters;
pub mod implementations;
pub mod utils;
pub mod error;

// Re-export commonly used items
pub use crate::core::{BrowserDebugger, Page, Dom, Network};
pub use crate::error::DebuggerError;
pub use crate::implementations::chrome::ChromeDebugger;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
} 