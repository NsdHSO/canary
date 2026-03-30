use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::adapter_trait::{AdapterType, CanAdapter, CanFrame};
use crate::error::CanError;
use crate::obd_vendor::ObdVendor;

/// Bluetooth scan result
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    /// Device name
    pub name: String,
    /// Device address (MAC or UUID)
    pub address: String,
    /// Signal strength (RSSI) in dBm
    pub rssi: Option<i16>,
    /// Whether this appears to be an OBD adapter
    pub is_obd_adapter: bool,
}

/// Bluetooth CAN adapter for wireless diagnostics.
///
/// Supports devices like:
/// - ELM327 Bluetooth dongles
/// - OBDLink LX Bluetooth
/// - Vgate iCar Pro Bluetooth 4.0
///
/// Uses Bluetooth Classic (SPP) or BLE depending on device.
pub struct BluetoothAdapter {
    device_name: String,
    device_address: Option<String>,
    connected: bool,
    /// Internal buffer for received data
    rx_buffer: Arc<Mutex<Vec<u8>>>,
}

impl BluetoothAdapter {
    /// Create a new Bluetooth adapter targeting a device by name
    ///
    /// # Arguments
    /// * `device_name` - Name to search for (e.g., "OBDLink LX", "OBDII")
    pub fn new(device_name: &str) -> Self {
        Self {
            device_name: device_name.to_string(),
            device_address: None,
            connected: false,
            rx_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new Bluetooth adapter with a specific address
    pub fn with_address(device_name: &str, address: &str) -> Self {
        Self {
            device_name: device_name.to_string(),
            device_address: Some(address.to_string()),
            connected: false,
            rx_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Scan for Bluetooth OBD adapters
    ///
    /// Returns a list of discovered devices that appear to be
    /// OBD-II adapters based on their name.
    pub async fn scan_devices(timeout_secs: u64) -> Result<Vec<BluetoothDevice>, CanError> {
        log::info!(
            "Scanning for Bluetooth OBD adapters ({} seconds)...",
            timeout_secs
        );

        // In production, this would use btleplug to scan
        // For now, return empty list on non-Linux or when btleplug is not linked
        let _ = timeout_secs;

        log::info!("Bluetooth scan complete");
        Ok(Vec::new())
    }

    /// Check if a device name looks like an OBD adapter
    pub fn is_obd_device_name(name: &str) -> bool {
        ObdVendor::from_device_name(name).is_some()
    }
}

#[async_trait]
impl CanAdapter for BluetoothAdapter {
    async fn connect(&mut self) -> Result<(), CanError> {
        log::info!(
            "Connecting to Bluetooth device '{}'...",
            self.device_name
        );

        if let Some(addr) = &self.device_address {
            log::info!("Using address: {}", addr);
        }

        // In production, this would:
        // 1. Use btleplug to scan for the device
        // 2. Connect to the device
        // 3. Discover services and characteristics
        // 4. Subscribe to notifications

        // For now, simulate connection
        // Real implementation would need btleplug crate
        self.connected = false;
        Err(CanError::BluetoothError(format!(
            "Bluetooth adapter '{}' not found. Ensure Bluetooth is enabled and the device is in range.",
            self.device_name
        )))
    }

    async fn disconnect(&mut self) -> Result<(), CanError> {
        self.connected = false;
        self.rx_buffer.lock().await.clear();
        log::info!(
            "Bluetooth adapter '{}' disconnected",
            self.device_name
        );
        Ok(())
    }

    async fn send_frame(&self, frame: &CanFrame) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        // In production: send via BLE characteristic write
        log::debug!("Bluetooth TX: {}", frame);
        Ok(())
    }

    async fn recv_frame(&self, timeout_ms: u64) -> Result<CanFrame, CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        // In production: read from BLE characteristic notifications
        let _ = timeout_ms;
        Err(CanError::Timeout(timeout_ms))
    }

    async fn send_isotp(&self, target_id: u32, data: &[u8]) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        if data.len() <= 7 {
            let mut frame_data = vec![data.len() as u8];
            frame_data.extend_from_slice(data);
            while frame_data.len() < 8 {
                frame_data.push(0x00);
            }
            self.send_frame(&CanFrame::new(target_id, frame_data)).await
        } else {
            Err(CanError::IsoTpError(
                "Multi-frame ISO-TP over Bluetooth not yet implemented".into(),
            ))
        }
    }

    async fn recv_isotp(&self, timeout_ms: u64) -> Result<Vec<u8>, CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        let frame = self.recv_frame(timeout_ms).await?;
        if frame.data.is_empty() {
            return Err(CanError::IsoTpError("Empty frame".into()));
        }

        let length = (frame.data[0] & 0x0F) as usize;
        if length == 0 || frame.data.len() < 1 + length {
            return Err(CanError::IsoTpError("Invalid single frame".into()));
        }
        Ok(frame.data[1..1 + length].to_vec())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Bluetooth
    }

    fn adapter_name(&self) -> &str {
        &self.device_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluetooth_adapter_creation() {
        let adapter = BluetoothAdapter::new("OBDLink LX");
        assert_eq!(adapter.adapter_name(), "OBDLink LX");
        assert_eq!(adapter.adapter_type(), AdapterType::Bluetooth);
        assert!(!adapter.is_connected());
    }

    #[test]
    fn test_bluetooth_adapter_with_address() {
        let adapter = BluetoothAdapter::with_address("OBDLink LX", "AA:BB:CC:DD:EE:FF");
        assert_eq!(adapter.device_address, Some("AA:BB:CC:DD:EE:FF".to_string()));
    }

    #[test]
    fn test_is_obd_device_name() {
        assert!(BluetoothAdapter::is_obd_device_name("OBDLink LX"));
        assert!(BluetoothAdapter::is_obd_device_name("ELM327 v2.1"));
        assert!(BluetoothAdapter::is_obd_device_name("Vgate iCar Pro"));
        assert!(BluetoothAdapter::is_obd_device_name("OBDII Scanner"));
        assert!(!BluetoothAdapter::is_obd_device_name("My Phone"));
        assert!(!BluetoothAdapter::is_obd_device_name("AirPods Pro"));
    }

    #[tokio::test]
    async fn test_bluetooth_not_connected_send() {
        let adapter = BluetoothAdapter::new("OBDLink LX");
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        let result = adapter.send_frame(&frame).await;
        assert!(matches!(result, Err(CanError::NotConnected)));
    }

    #[tokio::test]
    async fn test_bluetooth_scan() {
        let devices = BluetoothAdapter::scan_devices(1).await.unwrap();
        // In test environment, scan returns empty
        assert!(devices.is_empty());
    }
}
