//! Actor system implementation in Rust
//! 
//! This crate provides an actor system implementation inspired by Akka.
//! It supports hierarchical actor supervision, message passing, and scheduling.

mod actor;
mod address;
mod common;
mod context;
mod error;
mod execution;
mod message;
mod supervision;
mod system;

pub use actor::{Actor, Handler};
pub use address::{ActorPath, ActorRef, ActorSelection};
pub use common::{ActorId, AnyActorRef, MessageHandler, MessageMiddleware};
pub use context::{ActorContext, BasicContext};
pub use error::{ActorError, SendError};
pub use execution::{ExecutionContext, ExecutionContextConfig};
pub use message::{Message, SystemMessage, SupervisionEvent};
pub use supervision::{Supervisor, SupervisionStrategy};
pub use system::{ActorSystem, ActorSystemConfig};
