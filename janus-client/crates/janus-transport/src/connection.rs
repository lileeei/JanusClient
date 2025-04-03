use actix::io::{ FramedWrite, WriteHandler }; // Add FramedWrite, WriteHandler
use actix::prelude::*;
use async_trait::async_trait;
use futures_util::stream::StreamExt; // Add StreamExt for stream handling
use std::time::Duration;
use janus_core::error::{TransportError, CoreError};
use janus_core::actor::{SendRawMessage, IncomingRawMessage};
use janus_core::error::ProtocolError; // Import ProtocolError if needed for SendRawMessage error mapping
use tokio::time::timeout;
use tokio::io::split; // For splitting the stream

// Use a specific ConnectionId type alias from janus-core or define locally
pub type ConnectionId = u64;

#[derive(Debug, Clone)]
pub struct ConnectParams {
    pub url: String,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    #[cfg(feature = "websocket")]
    pub ws_config: Option<tokio_tungstenite::tungstenite::protocol::WebSocketConfig>,
}

#[async_trait]
pub trait Transport: Send + Unpin + StreamExt<Item = Result<String, TransportError>> + 'static { // Require StreamExt for actix stream handling
    // Type alias for the underlying Write half if splitting is required (common for TCP/TLS)
    // For WebSocket, the stream itself might implement Sink. Adjust if necessary.
    type Sink: futures_util::sink::Sink<String, Error = TransportError> + Send + Unpin + 'static;

    async fn connect(params: ConnectParams) -> Result<(Self, Self::Sink), TransportError> where Self: Sized; // Return Read/Write halves or combined stream/sink
    async fn disconnect(sink: Self::Sink) -> Result<(), TransportError>; // Disconnect needs the sink/writer
    // Send/Receive are handled via Sink/Stream traits now
    // async fn send(&mut self, message: String) -> Result<(), TransportError>;
    // async fn receive(&mut self) -> Option<Result<String, TransportError>>;
}

// --- Connection Actor ---

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Idle,
    Connecting,
    Connected,
    Disconnecting,
    Disconnected(Option<TransportError>),
}

/// Public message to report connection status changes (sent to supervisor).
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct ConnectionStatusUpdate {
    pub id: ConnectionId, // Add the ID
    pub state: ConnectionState,
}


/// Actor responsible for managing a single underlying transport connection.
// Derive WriteHandler for FramedWrite if using it
pub struct ConnectionActor<T: Transport>
    where <T as Transport>::Sink: ActorFrame // Require ActorFrame for FramedWrite sink
{
    id: ConnectionId, // Add ID field
    // Store the write half (Sink) for sending messages
    writer: Option<actix::io::FramedWrite<<T as Transport>::Sink, ConnectionCodec>>, // Store FramedWrite
    params: ConnectParams,
    state: ConnectionState,
    message_handler: Recipient<IncomingRawMessage>,
    supervisor: Option<Recipient<ConnectionStatusUpdate>>,
    // reader_handle is removed, stream handling is integrated
}

impl<T: Transport> ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
{
    pub fn new(
        id: ConnectionId, // Receive ID
        params: ConnectParams,
        message_handler: Recipient<IncomingRawMessage>,
        supervisor: Option<Recipient<ConnectionStatusUpdate>>
    ) -> Self {
        Self {
            id, // Store ID
            writer: None, // Initialize writer as None
            params,
            state: ConnectionState::Idle,
            message_handler,
            supervisor,
        }
    }

    fn update_state(&mut self, new_state: ConnectionState, ctx: &mut Context<Self>) {
         if self.state != new_state {
            log::info!("({}) Connection state (ID: {}) changing: {:?} -> {:?}", self.params.url, self.id, self.state, new_state);
            self.state = new_state.clone();
            if let Some(supervisor) = &self.supervisor {
                let update_msg = ConnectionStatusUpdate { id: self.id, state: self.state.clone() };
                if let Err(e) = supervisor.try_send(update_msg) {
                    log::error!("({}) Failed to send connection status update (ID: {}) to supervisor: {}", self.params.url, self.id, e);
                }
            }

            // If disconnected, ensure the writer is cleared and context might stop
            if matches!(self.state, ConnectionState::Disconnected(_)) {
                log::debug!("({}) Clearing writer due to disconnection (ID: {}).", self.params.url, self.id);
                self.writer = None;
                // Optionally stop the actor context if disconnection is fatal
                // ctx.stop();
            }
        }
    }
}


impl<T: Transport> Actor for ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("({}) ConnectionActor (ID: {}) starting connection process", self.params.url, self.id);
        if self.state != ConnectionState::Idle {
             log::warn!("({}) ConnectionActor (ID: {}) started in unexpected state: {:?}", self.params.url, self.id, self.state);
             self.state = ConnectionState::Idle; // Reset state
        }
        self.update_state(ConnectionState::Connecting, ctx);

        let params = self.params.clone();
        let actor_addr = ctx.address();

        // Spawn the connection attempt task
        let connect_future = async move {
             log::debug!("({}) Connect future starting (ID: {})", params.url, actor_addr.recipient::<ConnectionEstablished<T>>().actor_id()); // Can't easily get ID here, log URL
             match T::connect(params.clone()).await {
                Ok((stream_reader, stream_writer)) => { // Expecting read/write halves
                    log::debug!("({}) Connection successful, sending ConnectionEstablished to actor", params.url);
                    // Send the established stream and sink back to the actor's context
                    if actor_addr.try_send(ConnectionEstablished(stream_reader, stream_writer)).is_err() {
                         log::error!("({}) Actor context closed before connection established message could be sent.", params.url);
                         // Attempt to disconnect the dangling writer half
                         Arbiter::current().spawn(async move {
                             let _ = T::disconnect(stream_writer).await;
                         });
                    }
                },
                Err(e) => {
                    log::error!("({}) Transport connect error: {}", params.url, e);
                    // Report failure back to the actor context.
                    let _ = actor_addr.try_send(ConnectionLost(Some(e))); // Use try_send as actor might be stopping
                }
            }
        };
        // Spawn the connection attempt. Completion sends message back to actor.
        ctx.spawn(connect_future.into_actor(self));
    }

    fn stopping(&mut self, ctx: &mut Context<Self>) -> Running {
        log::info!("({}) ConnectionActor (ID: {}) stopping...", self.params.url, self.id);
        self.update_state(ConnectionState::Disconnecting, ctx); // Update state first

         // Attempt graceful disconnect using the stored writer (if any)
         if let Some(writer) = self.writer.take() {
             log::debug!("({}) Initiating graceful disconnect of transport sink (ID: {})...", self.params.url, self.id);
             // Get the inner Sink and disconnect it
             let sink = writer.into_inner();
             Arbiter::current().spawn(async move {
                 match T::disconnect(sink).await {
                     Ok(_) => log::debug!("Transport disconnected successfully."),
                     Err(e) => log::warn!("Error during transport disconnect: {}", e),
                 }
             });
         } else {
              log::debug!("({}) No transport writer present to disconnect.", self.params.url);
         }

        // Final state update before stopping
        self.update_state(ConnectionState::Disconnected(None), ctx);
        log::info!("({}) ConnectionActor (ID: {}) fully stopped.", self.params.url, self.id);
        Running::Stop
    }
}

// --- Actor Messages specific to ConnectionActor state ---

/// Internal message sent when the transport connection is successfully established.
#[derive(Message)]
#[rtype(result = "()")]
struct ConnectionEstablished<T: Transport>(T, T::Sink); // Contains reader (Stream) and writer (Sink)

/// Internal message sent when the transport connection is lost or fails to connect.
#[derive(Message)]
#[rtype(result = "()")]
struct ConnectionLost(Option<TransportError>);

// StartReadLoop message is removed

// --- Message Handlers ---

impl<T: Transport> Handler<ConnectionEstablished<T>> for ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
{
    type Result = ();

    fn handle(&mut self, msg: ConnectionEstablished<T>, ctx: &mut Context<Self>) {
         log::info!("({}) Connection established successfully (ID: {})", self.params.url, self.id);
         if self.state == ConnectionState::Connecting {
            let (stream_reader, stream_writer) = (msg.0, msg.1);

            // Store the writer half using FramedWrite for Sink integration
            self.writer = Some(FramedWrite::new(stream_writer, ConnectionCodec, ctx));

            // Add the reader stream to the actor context
            // This starts processing incoming messages using the StreamHandler trait implementation
            Self::add_stream(stream_reader, ctx);

            self.update_state(ConnectionState::Connected, ctx);
            log::info!("({}) ConnectionActor (ID: {}) is now Connected and handling stream.", self.params.url, self.id);

         } else {
              log::warn!("({}) Received ConnectionEstablished (ID: {}) but state was not Connecting ({:?}). Discarding.", self.params.url, self.id, self.state);
              // Disconnect the newly received transport as we won't use it
              let sink_to_drop = msg.1;
              Arbiter::current().spawn(async move { let _ = T::disconnect(sink_to_drop).await; });
         }
    }
}


// Implement StreamHandler to process messages received from the transport stream
impl<T: Transport> StreamHandler<Result<String, TransportError>> for ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
{
    fn handle(&mut self, item: Result<String, TransportError>, ctx: &mut Context<Self>) {
        match item {
            Ok(msg) => {
                // Forward successfully received message to the designated handler
                log::trace!("({}) Received raw message (ID: {}), forwarding to handler.", self.params.url, self.id);
                if let Err(e) = self.message_handler.try_send(IncomingRawMessage(msg)) {
                    log::error!("({}) Failed to send incoming message to handler (ID: {}): {}. Dropping message.", self.params.url, self.id, e);
                    // Handle backpressure or error if necessary
                }
            }
            Err(e) => {
                log::error!("({}) Transport receive error (ID: {}): {}", self.params.url, self.id, e);
                // Connection is considered lost on stream error
                self.update_state(ConnectionState::Disconnected(Some(e)), ctx);
                ctx.stop(); // Stop the actor on receive error
            }
        }
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        log::info!("({}) Transport stream finished (ID: {}). Connection closed by peer.", self.params.url, self.id);
        // Stream finished means the peer closed gracefully (or unexpectedly EOF)
        self.update_state(ConnectionState::Disconnected(None), ctx);
        ctx.stop(); // Stop the actor when the stream finishes
    }
}

// Implement WriteHandler for FramedWrite based communication
impl<T: Transport> WriteHandler<TransportError> for ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
{
    fn error(&mut self, err: TransportError, _ctx: &mut Context<Self>) -> Running {
        log::error!("({}) Transport sink (write) error (ID: {}): {}", self.params.url, self.id, err);
        // Treat sink errors as fatal disconnection
        self.update_state(ConnectionState::Disconnected(Some(err)), _ctx);
        Running::Stop // Stop actor on write error
    }

    fn finished(&mut self, _ctx: &mut Context<Self>) -> Running {
         log::debug!("({}) Transport sink finished (ID: {}), likely due to disconnect.", self.params.url, self.id);
         // Sink finishing often means connection is closed/closing. Ensure state reflects this.
         if !matches!(self.state, ConnectionState::Disconnected(_) | ConnectionState::Disconnecting) {
              self.update_state(ConnectionState::Disconnected(None), _ctx);
         }
        Running::Stop // Stop actor if sink finishes unexpectedly
    }
}


 impl<T: Transport> Handler<ConnectionLost> for ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
 {
    type Result = ();

    fn handle(&mut self, msg: ConnectionLost, ctx: &mut Context<Self>) {
         log::warn!("({}) Handling ConnectionLost signal (ID: {}). Reason: {:?}", self.params.url, self.id, msg.0);

         self.writer = None; // Ensure writer is cleared

         if !matches!(self.state, ConnectionState::Disconnecting | ConnectionState::Disconnected(_)) {
              self.update_state(ConnectionState::Disconnected(msg.0), ctx);
         } else {
              log::debug!("({}) Connection (ID: {}) already in state {:?}, not changing state.", self.params.url, self.id, self.state);
         }
         ctx.stop(); // Stop the actor when connection is lost externally (e.g., connect failed)
    }
}


// Handler for SendRawMessage (inherited from janus-core)
impl<T: Transport> Handler<SendRawMessage> for ConnectionActor<T>
    where <T as Transport>::Sink: ActorFrame
{
    // Use MessageResult for synchronous handling within actor context
    type Result = Result<(), TransportError>;

    fn handle(&mut self, msg: SendRawMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if self.state != ConnectionState::Connected {
            log::warn!("({}) Attempted to send message (ID: {}) while not connected (State: {:?})", self.params.url, self.id, self.state);
            return Err(TransportError::NotConnected);
        }

        // --- Using FramedWrite ---
        if let Some(writer) = &mut self.writer {
             log::trace!("({}) Sending raw message (ID: {}) via FramedWrite.", self.params.url, self.id);
             writer.write(msg.0); // Send the message using the FramedWrite
             Ok(())
        } else {
             log::error!("({}) Internal state inconsistency (ID: {}): State is Connected but writer is None.", self.params.url, self.id);
             Err(TransportError::Internal("Transport writer missing in Connected state".to_string()))
        }
    }
}

// --- Codec for FramedWrite ---
// This assumes text-based protocols like WebSocket JSON messages.
// Adjust if binary framing is needed.
use bytes::{BytesMut, BufMut};
use tokio_util::codec::{Encoder, Decoder};

#[derive(Default)]
pub struct ConnectionCodec;

// Implement Decoder to handle incoming byte streams -> String messages
impl Decoder for ConnectionCodec {
    type Item = String; // Decode into String messages
    type Error = TransportError; // Use our TransportError

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // This is a simple example assuming UTF-8 strings delimited somehow
        // For WebSocket, tungstenite handles framing; this codec might be simpler
        // or even unnecessary if the Sink/Stream directly handle String.
        // Assuming the Transport::Sink/Stream deals with String directly, this might not be needed.
        // If the underlying transport gives raw bytes, implement framing logic here.

        // Simplistic example: Treat entire buffer as one message (adjust!)
        if src.is_empty() {
            Ok(None)
        } else {
            match std::str::from_utf8(src) {
                 Ok(s) => {
                     let s_owned = s.to_owned();
                     src.clear(); // Consume buffer
                     Ok(Some(s_owned))
                 },
                 Err(e) => {
                      log::error!("Codec UTF-8 decoding error: {}", e);
                      Err(TransportError::Serde(format!("Invalid UTF-8 sequence: {}", e)))
                 }
            }
        }
    }
}

// Implement Encoder to handle outgoing String messages -> byte streams
impl Encoder<String> for ConnectionCodec {
    type Error = TransportError;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Reserve space and write the string bytes
        dst.reserve(item.len());
        dst.put(item.as_bytes());
        Ok(())
    }
}

// --- ActorFrame for Sink ---
// We need to tell actix how to handle frames with our chosen Sink type.
// This might require a wrapper around the Sink if it doesn't implement ActorFrame directly.
// If Transport::Sink is e.g., SplitSink<WebSocketStream<...>, WsMessage>, we need to handle that.

// Placeholder/Example: This might need significant adjustment based on the actual Sink type
// returned by the Transport implementation (e.g., WebSocketTransport).
use actix::io::ActorFrame;
impl<S> ActorFrame for S where S: futures_util::sink::Sink<String, Error = TransportError> + Send + Unpin + 'static {}

// Ensure WebSocketTransport::Sink implements ActorFrame if FramedWrite is used directly on it.
// This likely involves changes in websocket.rs to return a Sink that works with FramedWrite,
// or using a different approach than FramedWrite if the Sink isn't suitable.
// For WebSockets, often you don't need FramedWrite because the Sink directly takes WsMessage::Text(String).
// Consider simplifying by NOT using FramedWrite if the Sink already handles String messages.

// --- Simplified SendRawMessage without FramedWrite ---
/*
impl<T: Transport> Handler<SendRawMessage> for ConnectionActor<T> {
    type Result = ResponseFuture<Result<(), TransportError>>;

    fn handle(&mut self, msg: SendRawMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if self.state != ConnectionState::Connected {
             log::warn!("({}) Attempted to send message (ID: {}) while not connected (State: {:?})", self.params.url, self.id, self.state);
             return Box::pin(async { Err(TransportError::NotConnected) });
        }

        if let Some(writer) = &mut self.writer { // Assuming self.writer stores the Sink directly
             log::trace!("({}) Sending raw message (ID: {}) via Sink.", self.params.url, self.id);
             // Need mutable access to writer, might require taking it or careful borrowing
             // This requires the handler return type to be ResponseFuture again.
             // Let's assume we store writer: Arc<Mutex<T::Sink>> or similar shared approach,
             // or pass send capability differently.

             // --- Alternative: Store Addr<WriteActor> ---
             // If sending is complex, delegate to a separate WriteActor that owns the Sink.
             // self.writer.do_send(SendMessage(msg.0)) ...

             // --- Direct Sink usage (requires careful state management / ResponseFuture) ---
             // This example assumes we can get mutable access temporarily.
             // It's often easier to use FramedWrite or a dedicated writer actor.
             let writer_ref = self.writer.as_mut().expect("Writer missing in connected state"); // Needs mutable borrow
             Box::pin(async move {
                 use futures_util::sink::SinkExt;
                 writer_ref.send(msg.0).await // Use SinkExt::send
             })

        } else {
             log::error!("({}) Internal state inconsistency (ID: {}): State is Connected but writer is None.", self.params.url, self.id);
             Box::pin(async { Err(TransportError::Internal("Transport writer missing in Connected state".to_string())) })
        }
    }
}
*/
