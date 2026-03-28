use async_trait::async_trait;
use std::fmt;

use crate::error::CanError;

/// Represents a single CAN bus frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanFrame {
    /// CAN arbitration ID (11-bit or 29-bit)
    pub id: u32,
    /// Frame data payload (0-8 bytes for CAN 2.0, up to 64 for CAN FD)
    pub data: Vec<u8>,
    /// Whether this is an extended frame (29-bit ID)
    pub extended: bool,
    /// Timestamp in microseconds (relative)
    pub timestamp_us: u64,
}

impl CanFrame {
    /// Create a new standard CAN frame
    pub fn new(id: u32, data: Vec<u8>) -> Self {
        Self {
            id,
            data,
            extended: false,
            timestamp_us: 0,
        }
    }

    /// Create a new extended CAN frame (29-bit ID)
    pub fn new_extended(id: u32, data: Vec<u8>) -> Self {
        Self {
            id,
            data,
            extended: true,
            timestamp_us: 0,
        }
    }

    /// Serialize frame to bytes for network transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + 1 + self.data.len());
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.push(self.data.len() as u8);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Deserialize frame from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CanError> {
        if bytes.len() < 5 {
            return Err(CanError::InvalidFrame("Frame too short".into()));
        }
        let id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let data_len = bytes[4] as usize;
        if bytes.len() < 5 + data_len {
            return Err(CanError::InvalidFrame("Data length mismatch".into()));
        }
        Ok(Self {
            id,
            data: bytes[5..5 + data_len].to_vec(),
            extended: false,
            timestamp_us: 0,
        })
    }
}

impl fmt::Display for CanFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data_hex: Vec<String> = self.data.iter().map(|b| format!("{:02X}", b)).collect();
        write!(
            f,
            "CAN Frame [0x{:03X}] {}",
            self.id,
            data_hex.join(" ")
        )
    }
}

/// Type of hardware adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterType {
    /// Linux SocketCAN (USB adapters like Peak PCAN, Kvaser)
    SocketCan,
    /// Virtual CAN interface (vcan0 for testing)
    Virtual,
    /// WiFi adapter (ESP32, OBDLink MX WiFi)
    WiFi,
    /// Bluetooth adapter (ELM327, OBDLink LX)
    Bluetooth,
}

impl fmt::Display for AdapterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterType::SocketCan => write!(f, "SocketCAN"),
            AdapterType::Virtual => write!(f, "Virtual CAN"),
            AdapterType::WiFi => write!(f, "WiFi"),
            AdapterType::Bluetooth => write!(f, "Bluetooth"),
        }
    }
}

/// Information about an available adapter
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    /// Adapter name/identifier
    pub name: String,
    /// Type of adapter
    pub adapter_type: AdapterType,
    /// Description
    pub description: String,
    /// Whether the adapter is currently connected
    pub connected: bool,
}

/// Common interface for all CAN bus adapters.
///
/// Uses the Strategy pattern to allow swapping adapters at runtime
/// without changing application code. Supports SocketCAN, Virtual,
/// WiFi, and Bluetooth adapters.
#[async_trait]
pub trait CanAdapter: Send + Sync {
    /// Connect to the CAN bus interface
    async fn connect(&mut self) -> Result<(), CanError>;

    /// Disconnect from the CAN bus interface
    async fn disconnect(&mut self) -> Result<(), CanError>;

    /// Send a single CAN frame
    async fn send_frame(&self, frame: &CanFrame) -> Result<(), CanError>;

    /// Receive a single CAN frame (blocks until available or timeout)
    async fn recv_frame(&self, timeout_ms: u64) -> Result<CanFrame, CanError>;

    /// Send ISO-TP segmented data (for UDS messages >8 bytes)
    async fn send_isotp(&self, target_id: u32, data: &[u8]) -> Result<(), CanError>;

    /// Receive ISO-TP segmented data
    async fn recv_isotp(&self, timeout_ms: u64) -> Result<Vec<u8>, CanError>;

    /// Check if adapter is currently connected
    fn is_connected(&self) -> bool;

    /// Get adapter type
    fn adapter_type(&self) -> AdapterType;

    /// Get adapter name/identifier
    fn adapter_name(&self) -> &str;

    /// Test the connection by sending a diagnostic ping
    async fn test_connection(&self) -> Result<bool, CanError> {
        if !self.is_connected() {
            return Ok(false);
        }
        // Default: send OBD-II mode 01 PID 00 (supported PIDs)
        let test_frame = CanFrame::new(0x7DF, vec![0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        match self.send_frame(&test_frame).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_frame_new() {
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        assert_eq!(frame.id, 0x7E0);
        assert_eq!(frame.data, vec![0x02, 0x10, 0x01]);
        assert!(!frame.extended);
    }

    #[test]
    fn test_can_frame_extended() {
        let frame = CanFrame::new_extended(0x18DA00F1, vec![0x02, 0x10, 0x01]);
        assert!(frame.extended);
        assert_eq!(frame.id, 0x18DA00F1);
    }

    #[test]
    fn test_can_frame_serialization() {
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        let bytes = frame.to_bytes();
        let recovered = CanFrame::from_bytes(&bytes).unwrap();
        assert_eq!(frame.id, recovered.id);
        assert_eq!(frame.data, recovered.data);
    }

    #[test]
    fn test_can_frame_display() {
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        let display = format!("{}", frame);
        assert!(display.contains("7E0"));
        assert!(display.contains("02 10 01"));
    }

    #[test]
    fn test_adapter_type_display() {
        assert_eq!(format!("{}", AdapterType::SocketCan), "SocketCAN");
        assert_eq!(format!("{}", AdapterType::Virtual), "Virtual CAN");
        assert_eq!(format!("{}", AdapterType::WiFi), "WiFi");
        assert_eq!(format!("{}", AdapterType::Bluetooth), "Bluetooth");
    }

    #[test]
    fn test_can_frame_from_bytes_too_short() {
        let result = CanFrame::from_bytes(&[0x00, 0x01]);
        assert!(result.is_err());
    }
}
