use crate::connection::{ConnectParams, Transport};
use janus_core::error::TransportError; // Use the core error type
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async_with_config, MaybeTlsStream, WebSocketStream,
    tungstenite::protocol::Message as WsMessage,
    tungstenite::protocol::WebSocketConfig,
    tungstenite::error::Error as WsError,
};
use tokio::time::timeout;
use url::Url;


#[derive(Debug)] // WebSocketStream doesn't impl Clone easily
pub struct WebSocketTransport {
    // Inner stream is wrapped in Option to allow taking ownership on disconnect
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    url: Url, // Store parsed URL
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn connect(params: ConnectParams) -> Result<Self, TransportError> {
        log::debug!("Connecting WebSocket to: {}", params.url);

        let url = Url::parse(&params.url)
            .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        let ws_config: Option<WebSocketConfig> = params.ws_config; // From ConnectParams

        let connect_future = connect_async_with_config(url.clone(), ws_config, false); // `false` = disable_nagle

        match timeout(params.connect_timeout, connect_future).await {
            Ok(Ok((stream, response))) => {
                log::info!("WebSocket connected successfully to {}. Response status: {}", params.url, response.status());
                // Optional: Log response headers if needed (response.headers())
                Ok(WebSocketTransport { stream: Some(stream), url })
            }
            Ok(Err(e)) => {
                log::error!("WebSocket connection error to {}: {}", params.url, e);
                Err(map_ws_error(e))
            }
            Err(_) => {
                log::error!("WebSocket connection timed out after {:?} to {}", params.connect_timeout, params.url);
                Err(TransportError::Timeout(format!(
                    "Connection timed out after {:?}",
                    params.connect_timeout
                )))
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        log::debug!("Disconnecting WebSocket from: {}", self.url);
        if let Some(mut stream) = self.stream.take() { // Take ownership
            match stream.close(None).await {
                Ok(_) => {
                     log::info!("WebSocket closed gracefully for {}", self.url);
                    Ok(())
                },
                Err(e) => {
                    log::warn!("Error during WebSocket close for {}: {}", self.url, e);
                    // Still consider it disconnected, but report potential issue
                    Err(map_ws_error(e))
                }
            }
        } else {
            // Already disconnected or never connected
             log::debug!("WebSocket already disconnected for {}", self.url);
            Ok(())
        }
    }

    async fn send(&mut self, message: String) -> Result<(), TransportError> {
         log::trace!("Sending WebSocket message: {}", message); // Use trace for potentially verbose logs
        if let Some(stream) = self.stream.as_mut() {
            match stream.send(WsMessage::Text(message)).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    log::error!("WebSocket send error: {}", e);
                    Err(map_ws_error(e))
                }
            }
        } else {
            Err(TransportError::NotConnected)
        }
    }

    async fn receive(&mut self) -> Option<Result<String, TransportError>> {
        if let Some(stream) = self.stream.as_mut() {
             match stream.next().await {
                Some(Ok(msg)) => {
                    log::trace!("Received WebSocket message: {:?}", msg);
                    match msg {
                        WsMessage::Text(text) => Some(Ok(text)),
                        WsMessage::Binary(bin) => {
                            // Decide how to handle binary data - error for now?
                            log::warn!("Received unexpected binary WebSocket message ({} bytes)", bin.len());
                            Some(Err(TransportError::ReceiveFailed("Received unexpected binary message".to_string())))
                        },
                        WsMessage::Ping(data) => {
                            // Tungstenite handles pongs automatically by default
                            log::trace!("Received WebSocket Ping: {:?}", data);
                            // Need to recurse or loop to get the *next* actual message
                            // This simple implementation might miss messages if ping/pong flood
                            self.receive().await // Tail recursion might be okay here
                        },
                        WsMessage::Pong(data) => {
                            log::trace!("Received WebSocket Pong: {:?}", data);
                            // Recurse or loop
                            self.receive().await
                        },
                        WsMessage::Close(close_frame) => {
                            log::info!("Received WebSocket Close frame: {:?}", close_frame);
                            None // Signal stream closure
                        },
                        WsMessage::Frame(_) => {
                            // Raw frame, likely shouldn't happen with default config
                            log::warn!("Received unexpected raw WebSocket frame");
                            Some(Err(TransportError::ReceiveFailed("Received unexpected raw frame".to_string())))
                        }
                    }
                },
                Some(Err(e)) => {
                    log::error!("WebSocket receive error: {}", e);
                     // Check if it's a "ConnectionClosed" error vs other IO error
                     if matches!(e, WsError::ConnectionClosed | WsError::AlreadyClosed) {
                         None // Treat as clean closure
                     } else {
                         Some(Err(map_ws_error(e))) // Report other errors
                     }
                },
                None => {
                     // Stream ended without a Close frame (unexpected EOF)
                     log::warn!("WebSocket stream ended unexpectedly (EOF)");
                    None // Signal stream closure
                },
            }
        } else {
            // Not connected, stream is None
            None
        }
    }
}

// Helper to map tungstenite errors to our TransportError
fn map_ws_error(e: WsError) -> TransportError {
    match e {
        WsError::ConnectionClosed | WsError::AlreadyClosed => {
            TransportError::ConnectionClosed { reason: Some("Connection closed by peer or locally".to_string()) }
        }
        WsError::Io(io_err) => TransportError::Io(io_err.to_string()),
        WsError::Tls(tls_err) => TransportError::TlsError(tls_err.to_string()),
        WsError::Capacity(cap_err) => TransportError::SendFailed(format!("Capacity error: {}", cap_err)), // Or specific capacity error type
        WsError::Protocol(proto_err) => TransportError::WebSocket(proto_err.to_string()),
        WsError::SendQueueFull(_) => TransportError::SendFailed("Send queue full".to_string()),
        WsError::Utf8 => TransportError::Serde("Invalid UTF-8 received".to_string()),
        WsError::Url(url_err) => TransportError::InvalidUrl(url_err.to_string()),
        WsError::Http(http_err) => TransportError::ConnectionFailed(format!("HTTP error during handshake: {}", http_err.status())),
        WsError::HttpFormat(http_fmt_err) => TransportError::ConnectionFailed(format!("HTTP format error: {}", http_fmt_err)),
        // Add other specific mappings as needed
         _ => TransportError::WebSocket(e.to_string()), // Catch-all for other WsError variants
    }
}
