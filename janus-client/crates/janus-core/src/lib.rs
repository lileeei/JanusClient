pub mod actor;
pub mod config;
pub mod error; // Ensure this line exists and is public

// Re-export key types for convenience
pub use error::{CoreError, TransportError, ProtocolError, ConfigError, MailboxError}; // Export new types
pub use config::Config;
// Potentially re-export common actor messages if used widely
// pub use actor::{SendRawMessage, IncomingRawMessage, ExecuteCommand, ProtocolEvent};
