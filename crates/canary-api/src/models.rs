//! API request/response models with OpenAPI documentation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to start a new diagnostic session
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    /// Target ECU CAN ID (e.g., 0x7E0)
    pub ecu_id: u32,
    /// Session type: "default", "extended", "programming"
    #[serde(default = "default_session_type")]
    pub session_type: String,
    /// Optional adapter identifier
    pub adapter: Option<String>,
}

fn default_session_type() -> String {
    "default".to_string()
}

/// Response after creating a diagnostic session
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionResponse {
    /// Session UUID
    pub session_id: String,
    /// Target ECU ID
    pub ecu_id: u32,
    /// Session type
    pub session_type: String,
    /// Session status
    pub status: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
}

/// DTC (Diagnostic Trouble Code) from a read operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DtcResponse {
    /// DTC code (e.g., "P0301")
    pub code: String,
    /// Human-readable description
    pub description: String,
    /// Status byte
    pub status: u8,
    /// Severity level
    pub severity: String,
    /// ECU that reported this DTC
    pub ecu_id: u32,
    /// System category (Powertrain, Body, Chassis, Network)
    pub system: String,
}

/// Request to read DTCs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReadDtcRequest {
    /// Target ECU CAN ID (optional, reads all if not specified)
    pub ecu_id: Option<u32>,
    /// DTC status mask (0xFF for all)
    #[serde(default = "default_status_mask")]
    pub status_mask: u8,
}

fn default_status_mask() -> u8 {
    0xFF
}

/// Request to clear DTCs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClearDtcRequest {
    /// Target ECU CAN ID
    pub ecu_id: u32,
    /// Specific DTC codes to clear (empty = clear all)
    #[serde(default)]
    pub codes: Vec<String>,
}

/// Response after clearing DTCs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClearDtcResponse {
    /// Whether the clear was successful
    pub success: bool,
    /// Number of DTCs cleared
    pub cleared_count: u32,
    /// Message
    pub message: String,
}

/// ECU information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EcuInfo {
    /// ECU identifier
    pub id: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Model
    pub model: String,
    /// Year range
    pub year_range: String,
    /// ECU type (ECM, TCM, ABS, etc.)
    pub ecu_type: String,
    /// CAN ID for diagnostics
    pub can_id: u32,
    /// Supported protocols
    pub protocols: Vec<String>,
}

/// Live data point for WebSocket streaming
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LiveDataPoint {
    /// Timestamp in milliseconds
    pub timestamp_ms: u64,
    /// ECU source ID
    pub ecu_id: u32,
    /// Parameter ID (PID)
    pub pid: u16,
    /// Parameter name
    pub name: String,
    /// Current value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Minimum expected value
    pub min: Option<f64>,
    /// Maximum expected value
    pub max: Option<f64>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Subscribe to live data from specific ECU/PIDs
    Subscribe {
        ecu_id: u32,
        pids: Vec<u16>,
    },
    /// Unsubscribe from live data
    Unsubscribe {
        ecu_id: u32,
    },
    /// Live data update (server to client)
    LiveData(LiveDataPoint),
    /// CAN frame (server to client)
    CanFrame {
        id: u32,
        data: Vec<u8>,
        timestamp_us: u64,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Heartbeat/ping
    Ping,
    /// Heartbeat/pong response
    Pong,
}

/// API health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// API version
    pub version: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Paginated list response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    /// Items in this page
    pub items: Vec<T>,
    /// Total number of items
    pub total: u64,
    /// Current page (1-indexed)
    pub page: u32,
    /// Items per page
    pub per_page: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_request_defaults() {
        let json = r#"{"ecu_id": 2016}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ecu_id, 2016);
        assert_eq!(req.session_type, "default");
        assert!(req.adapter.is_none());
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Subscribe {
            ecu_id: 0x7E0,
            pids: vec![0x0C, 0x0D],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Subscribe"));
        assert!(json.contains("2016")); // 0x7E0 = 2016
    }

    #[test]
    fn test_dtc_response() {
        let dtc = DtcResponse {
            code: "P0301".to_string(),
            description: "Cylinder 1 Misfire".to_string(),
            status: 0x08,
            severity: "high".to_string(),
            ecu_id: 0x7E0,
            system: "Powertrain".to_string(),
        };
        let json = serde_json::to_string(&dtc).unwrap();
        assert!(json.contains("P0301"));
    }

    #[test]
    fn test_live_data_point() {
        let point = LiveDataPoint {
            timestamp_ms: 1234567890,
            ecu_id: 0x7E0,
            pid: 0x0C,
            name: "Engine RPM".to_string(),
            value: 3500.0,
            unit: "rpm".to_string(),
            min: Some(0.0),
            max: Some(8000.0),
        };
        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("3500"));
    }
}
