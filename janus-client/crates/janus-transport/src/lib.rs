use actix::prelude::*;
use crate::connection::{ConnectParams, ConnectionActor, ConnectionState, ConnectionStatusUpdate, Transport, ConnectionId};
use crate::websocket::WebSocketTransport; // Assuming WebSocketTransport is the primary one for now
use janus_core::actor::IncomingRawMessage;
use janus_core::error::TransportError;


// Make connection types public
pub mod connection;
// Make specific transport implementations public if needed directly,
// otherwise they might just be used internally via ConnectionActor setup.
pub mod websocket;
// pub mod tcp; // Future support
// pub mod ipc; // Future support

// Re-export key types from connection module
pub use connection::{ConnectParams, ConnectionActor, ConnectionState, ConnectionStatusUpdate, Transport, ConnectionId};
// Re-export specific transport types if they need to be instantiated directly by users
pub use websocket::WebSocketTransport;


/// Creates and starts the appropriate ConnectionActor based on the URL scheme.
///
/// This function acts as a factory for transport connection actors.
///
/// # Arguments
///
/// * `id` - Unique identifier for the connection.
/// * `params` - Connection parameters including the URL and timeouts.
/// * `message_handler` - The actor recipient designated to receive `IncomingRawMessage`s.
/// * `supervisor` - An optional actor recipient to receive `ConnectionStatusUpdate` messages.
///
/// # Returns
///
/// A `Result` containing the `Addr` of the started `ConnectionActor` specialized for the
/// determined transport protocol (e.g., `WebSocketTransport`), or a `TransportError`
/// if the URL scheme is unsupported or invalid.
///
/// # Example
///
/// ```no_run
/// # use actix::prelude::*;
/// # use std::time::Duration;
/// # use janus_transport::{create_transport_actor, ConnectParams, ConnectionId};
/// # use janus_core::actor::IncomingRawMessage;
/// #
/// # #[derive(Message)]
/// # #[rtype(result = "()")]
/// # struct MyMessage(String);
/// # impl Handler<IncomingRawMessage> for MyActor {
/// #     type Result = ();
/// #     fn handle(&mut self, msg: IncomingRawMessage, ctx: &mut Context<Self>) -> Self::Result {
/// #         println!("Received: {}", msg.0);
/// #     }
/// # }
/// # struct MyActor;
/// # impl Actor for MyActor { type Context = Context<Self>; }
/// #
/// # async fn setup() -> anyhow::Result<()> {
/// let system = System::new();
/// let my_actor_addr = MyActor.start(); // Actor to handle incoming messages
/// let msg_handler: Recipient<IncomingRawMessage> = my_actor_addr.recipient();
/// let connection_id: ConnectionId = 123; // Example ID
///
/// let params = ConnectParams {
///     url: "ws://127.0.0.1:9222/devtools/browser/some-uuid".to_string(),
///     connect_timeout: Duration::from_secs(10),
///     request_timeout: Duration::from_secs(30),
///     ws_config: None, // Use default tungstenite config
/// };
///
/// let connection_actor_addr = create_transport_actor(connection_id, params, msg_handler, None)?;
/// // Now you can send SendRawMessage to connection_actor_addr
/// # Ok(())
/// # }
/// ```
pub fn create_transport_actor(
    id: ConnectionId,
    params: ConnectParams,
    message_handler: Recipient<IncomingRawMessage>,
    supervisor: Option<Recipient<ConnectionStatusUpdate>>,
) -> Result<Addr<ConnectionActor<WebSocketTransport>>, TransportError> { // Return concrete Addr type
    let url_scheme = url::Url::parse(&params.url)
        .map_err(|e| TransportError::InvalidUrl(e.to_string()))?
        .scheme()
        .to_lowercase();

    match url_scheme.as_str() {
        "ws" | "wss" => {
            // Start the WebSocket specific connection actor
            let actor = ConnectionActor::<WebSocketTransport>::new(
                id, // Pass connection ID
                params,
                message_handler,
                supervisor,
            );
            let addr = actor.start();
            Ok(addr) // Return concrete Addr
        }
        // Example for future TCP transport:
        // "tcp" => {
        //     #[cfg(feature = "tcp")]
        //     {
        //         Ok(ConnectionActor::<TcpTransport>::new(params, message_handler, supervisor).start())
        //     }
        //     #[cfg(not(feature = "tcp"))]
        //     {
        //         Err(TransportError::UnsupportedScheme("tcp (feature not enabled)".to_string()))
        //     }
        // }
        _ => Err(TransportError::UnsupportedScheme(url_scheme)),
    }
}
