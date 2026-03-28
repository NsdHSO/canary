//! WebSocket handler for live ECU data streaming.
//!
//! Supports subscribing to real-time CAN frames and PID data
//! from connected ECUs with <100ms latency target.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::models::{LiveDataPoint, WsMessage};

/// Shared state for WebSocket connections
#[derive(Clone)]
pub struct WsState {
    /// Broadcast channel for live data updates
    pub tx: Arc<broadcast::Sender<WsMessage>>,
}

impl WsState {
    /// Create a new WebSocket state with the specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx: Arc::new(tx) }
    }

    /// Broadcast a live data point to all connected WebSocket clients
    pub fn broadcast_live_data(&self, data: LiveDataPoint) {
        let _ = self.tx.send(WsMessage::LiveData(data));
    }

    /// Broadcast a CAN frame to all connected clients
    pub fn broadcast_can_frame(&self, id: u32, data: Vec<u8>, timestamp_us: u64) {
        let _ = self.tx.send(WsMessage::CanFrame {
            id,
            data,
            timestamp_us,
        });
    }
}

/// WebSocket upgrade handler
#[utoipa::path(
    get,
    path = "/api/v1/stream/live",
    responses(
        (status = 101, description = "WebSocket connection established"),
    ),
    tag = "streaming"
)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    state: axum::extract::State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state.0))
}

/// Handle an individual WebSocket connection
async fn handle_ws_connection(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Spawn task to forward broadcast messages to this client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(_) => continue,
            };
            if sender.send(Message::Text(json.into())).await.is_err() {
                break; // Client disconnected
            }
        }
    });

    // Process incoming messages from this client
    let state_clone = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    handle_client_message(&text, &state_clone).await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// Process incoming WebSocket messages from a client
async fn handle_client_message(text: &str, state: &WsState) {
    match serde_json::from_str::<WsMessage>(text) {
        Ok(WsMessage::Subscribe { ecu_id, pids }) => {
            tracing::info!(
                "Client subscribed to ECU 0x{:03X}, PIDs: {:?}",
                ecu_id,
                pids
            );
            // In production, start polling the UDS session for these PIDs
            // and broadcasting results via state.broadcast_live_data()
        }
        Ok(WsMessage::Unsubscribe { ecu_id }) => {
            tracing::info!("Client unsubscribed from ECU 0x{:03X}", ecu_id);
        }
        Ok(WsMessage::Ping) => {
            let _ = state.tx.send(WsMessage::Pong);
        }
        Ok(_) => {
            // Client-only messages, ignore
        }
        Err(e) => {
            tracing::warn!("Invalid WebSocket message: {}", e);
            let _ = state.tx.send(WsMessage::Error {
                message: format!("Invalid message format: {}", e),
            });
        }
    }
}

/// Start a simulated live data stream for testing
pub async fn start_simulated_stream(state: WsState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
    let mut rpm = 800.0_f64;
    let mut speed = 0.0_f64;
    let mut tick: u64 = 0;

    loop {
        interval.tick().await;
        tick += 1;

        // Simulate engine RPM oscillation
        rpm = 2500.0 + 500.0 * (tick as f64 * 0.05).sin();

        // Simulate vehicle speed
        speed = (rpm / 50.0).min(120.0);

        state.broadcast_live_data(LiveDataPoint {
            timestamp_ms: tick * 50,
            ecu_id: 0x7E0,
            pid: 0x0C,
            name: "Engine RPM".to_string(),
            value: rpm,
            unit: "rpm".to_string(),
            min: Some(0.0),
            max: Some(8000.0),
        });

        state.broadcast_live_data(LiveDataPoint {
            timestamp_ms: tick * 50,
            ecu_id: 0x7E0,
            pid: 0x0D,
            name: "Vehicle Speed".to_string(),
            value: speed,
            unit: "km/h".to_string(),
            min: Some(0.0),
            max: Some(250.0),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_state_creation() {
        let state = WsState::new(100);
        // Should be able to subscribe
        let _rx = state.tx.subscribe();
    }

    #[test]
    fn test_broadcast_live_data() {
        let state = WsState::new(100);
        let mut rx = state.tx.subscribe();

        state.broadcast_live_data(LiveDataPoint {
            timestamp_ms: 1000,
            ecu_id: 0x7E0,
            pid: 0x0C,
            name: "RPM".to_string(),
            value: 3500.0,
            unit: "rpm".to_string(),
            min: None,
            max: None,
        });

        let msg = rx.try_recv().unwrap();
        match msg {
            WsMessage::LiveData(data) => {
                assert_eq!(data.pid, 0x0C);
                assert_eq!(data.value, 3500.0);
            }
            _ => panic!("Expected LiveData message"),
        }
    }

    #[test]
    fn test_broadcast_can_frame() {
        let state = WsState::new(100);
        let mut rx = state.tx.subscribe();

        state.broadcast_can_frame(0x7E0, vec![0x02, 0x10, 0x01], 12345);

        let msg = rx.try_recv().unwrap();
        match msg {
            WsMessage::CanFrame {
                id,
                data,
                timestamp_us,
            } => {
                assert_eq!(id, 0x7E0);
                assert_eq!(data, vec![0x02, 0x10, 0x01]);
                assert_eq!(timestamp_us, 12345);
            }
            _ => panic!("Expected CanFrame message"),
        }
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Subscribe {
            ecu_id: 0x7E0,
            pids: vec![0x0C, 0x0D],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsMessage::Subscribe { ecu_id, pids } => {
                assert_eq!(ecu_id, 0x7E0);
                assert_eq!(pids, vec![0x0C, 0x0D]);
            }
            _ => panic!("Wrong variant"),
        }
    }
}
