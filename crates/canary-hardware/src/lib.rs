//! # canary-hardware
//!
//! Hardware adapter abstraction for CAN bus interfaces.
//!
//! Provides a unified `CanAdapter` trait that abstracts over different
//! hardware backends:
//!
//! - **SocketCAN** - Linux kernel CAN interface (USB adapters, vcan)
//! - **Virtual** - In-memory adapter for testing (no hardware needed)
//! - **WiFi** - TCP/UDP over WiFi (ESP32, OBDLink MX WiFi)
//! - **Bluetooth** - BLE/Classic (ELM327, OBDLink LX)
//!
//! ## Quick Start
//!
//! ```rust
//! use canary_hardware::{VirtualAdapter, CanAdapter, CanFrame};
//!
//! # async fn example() -> Result<(), canary_hardware::CanError> {
//! let mut adapter = VirtualAdapter::new("vcan0");
//! adapter.connect().await?;
//!
//! // Send a UDS DiagnosticSessionControl request
//! let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
//! adapter.send_frame(&frame).await?;
//! # Ok(())
//! # }
//! ```

pub mod adapter_trait;
pub mod bluetooth_adapter;
pub mod error;
pub mod network_protocol;
pub mod obd_vendor;
pub mod socketcan_adapter;
pub mod virtual_adapter;
pub mod wifi_adapter;

// Re-exports for convenience
pub use adapter_trait::{AdapterInfo, AdapterType, CanAdapter, CanFrame};
pub use bluetooth_adapter::{BluetoothAdapter, BluetoothDevice};
pub use error::CanError;
pub use network_protocol::{NetworkProtocol, ProtocolType};
pub use obd_vendor::ObdVendor;
pub use socketcan_adapter::SocketCanAdapter;
pub use virtual_adapter::VirtualAdapter;
pub use wifi_adapter::WiFiAdapter;

/// List all available adapter types
pub fn list_adapter_types() -> Vec<AdapterInfo> {
    vec![
        AdapterInfo {
            name: "socketcan".to_string(),
            adapter_type: AdapterType::SocketCan,
            description: "Linux SocketCAN interface (USB adapters, vcan)".to_string(),
            connected: false,
        },
        AdapterInfo {
            name: "virtual".to_string(),
            adapter_type: AdapterType::Virtual,
            description: "Virtual CAN for testing (no hardware)".to_string(),
            connected: false,
        },
        AdapterInfo {
            name: "wifi".to_string(),
            adapter_type: AdapterType::WiFi,
            description: "WiFi adapter (ESP32, OBDLink MX WiFi)".to_string(),
            connected: false,
        },
        AdapterInfo {
            name: "bluetooth".to_string(),
            adapter_type: AdapterType::Bluetooth,
            description: "Bluetooth adapter (ELM327, OBDLink LX)".to_string(),
            connected: false,
        },
    ]
}

/// Create an adapter by type and connection string
///
/// # Arguments
/// * `adapter_type` - Type of adapter to create
/// * `connection` - Connection string (interface name, IP:port, device name)
pub fn create_adapter(adapter_type: AdapterType, connection: &str) -> Box<dyn CanAdapter> {
    match adapter_type {
        AdapterType::SocketCan => Box::new(SocketCanAdapter::new(connection)),
        AdapterType::Virtual => Box::new(VirtualAdapter::new(connection)),
        AdapterType::WiFi => {
            let parts: Vec<&str> = connection.splitn(2, ':').collect();
            let host = parts[0];
            let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(35000);
            Box::new(WiFiAdapter::new(host, port))
        }
        AdapterType::Bluetooth => Box::new(BluetoothAdapter::new(connection)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_adapter_types() {
        let types = list_adapter_types();
        assert_eq!(types.len(), 4);
        assert!(types.iter().any(|t| t.adapter_type == AdapterType::SocketCan));
        assert!(types.iter().any(|t| t.adapter_type == AdapterType::Virtual));
        assert!(types.iter().any(|t| t.adapter_type == AdapterType::WiFi));
        assert!(types.iter().any(|t| t.adapter_type == AdapterType::Bluetooth));
    }

    #[test]
    fn test_create_virtual_adapter() {
        let adapter = create_adapter(AdapterType::Virtual, "vcan0");
        assert_eq!(adapter.adapter_type(), AdapterType::Virtual);
        assert_eq!(adapter.adapter_name(), "vcan0");
        assert!(!adapter.is_connected());
    }

    #[test]
    fn test_create_socketcan_adapter() {
        let adapter = create_adapter(AdapterType::SocketCan, "can0");
        assert_eq!(adapter.adapter_type(), AdapterType::SocketCan);
    }

    #[test]
    fn test_create_wifi_adapter() {
        let adapter = create_adapter(AdapterType::WiFi, "192.168.4.1:35000");
        assert_eq!(adapter.adapter_type(), AdapterType::WiFi);
    }

    #[test]
    fn test_create_bluetooth_adapter() {
        let adapter = create_adapter(AdapterType::Bluetooth, "OBDLink LX");
        assert_eq!(adapter.adapter_type(), AdapterType::Bluetooth);
    }

    #[tokio::test]
    async fn test_virtual_adapter_full_workflow() {
        let mut adapter = VirtualAdapter::new("test_vcan");
        adapter.connect().await.unwrap();

        // Send a UDS request
        let request = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
        adapter.send_frame(&request).await.unwrap();

        // Receive loopback
        let response = adapter.recv_frame(1000).await.unwrap();
        assert_eq!(response.id, 0x7E0);

        // Test connection
        let test_result = adapter.test_connection().await.unwrap();
        assert!(test_result);

        adapter.disconnect().await.unwrap();
        assert!(!adapter.is_connected());
    }
}
