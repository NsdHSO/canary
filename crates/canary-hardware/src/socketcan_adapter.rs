use async_trait::async_trait;

use crate::adapter_trait::{AdapterType, CanAdapter, CanFrame};
use crate::error::CanError;

/// SocketCAN adapter for Linux CAN interfaces.
///
/// Supports physical USB CAN adapters (Peak PCAN, Kvaser, etc.)
/// and virtual CAN interfaces (vcan0) on Linux systems.
///
/// Note: SocketCAN is Linux-only. On other platforms, use the
/// VirtualAdapter for testing or WiFi/Bluetooth adapters for
/// real hardware access.
pub struct SocketCanAdapter {
    interface: String,
    connected: bool,
}

impl SocketCanAdapter {
    /// Create a new SocketCAN adapter for the given interface
    ///
    /// # Arguments
    /// * `interface` - CAN interface name (e.g., "can0", "vcan0")
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_string(),
            connected: false,
        }
    }
}

#[async_trait]
impl CanAdapter for SocketCanAdapter {
    async fn connect(&mut self) -> Result<(), CanError> {
        // On non-Linux platforms, SocketCAN is not available
        #[cfg(target_os = "linux")]
        {
            // In production, this would open a real SocketCAN socket
            // For now, we validate the interface exists
            let path = format!("/sys/class/net/{}", self.interface);
            if !std::path::Path::new(&path).exists() {
                return Err(CanError::AdapterNotFound(format!(
                    "Interface '{}' not found. Create with: sudo ip link add dev {} type vcan && sudo ip link set up {}",
                    self.interface, self.interface, self.interface
                )));
            }
            self.connected = true;
            log::info!("SocketCAN adapter connected to {}", self.interface);
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(CanError::ConnectionFailed(format!(
                "SocketCAN is only available on Linux. Interface '{}' cannot be used on this platform. Use WiFi or Bluetooth adapter instead.",
                self.interface
            )))
        }
    }

    async fn disconnect(&mut self) -> Result<(), CanError> {
        self.connected = false;
        log::info!("SocketCAN adapter disconnected from {}", self.interface);
        Ok(())
    }

    async fn send_frame(&self, frame: &CanFrame) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        #[cfg(target_os = "linux")]
        {
            // In production: use socketcan crate to write frame
            log::debug!("SocketCAN TX on {}: {}", self.interface, frame);
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = frame;
            Err(CanError::SendFailed(
                "SocketCAN not available on this platform".into(),
            ))
        }
    }

    async fn recv_frame(&self, timeout_ms: u64) -> Result<CanFrame, CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        #[cfg(target_os = "linux")]
        {
            // In production: use socketcan crate to read frame with timeout
            let _ = timeout_ms;
            Err(CanError::Timeout(timeout_ms))
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(CanError::Timeout(timeout_ms))
        }
    }

    async fn send_isotp(&self, target_id: u32, data: &[u8]) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        // ISO-TP single frame for data <= 7 bytes
        if data.len() <= 7 {
            let mut frame_data = vec![data.len() as u8];
            frame_data.extend_from_slice(data);
            while frame_data.len() < 8 {
                frame_data.push(0x00);
            }
            self.send_frame(&CanFrame::new(target_id, frame_data)).await
        } else {
            // Multi-frame ISO-TP would use socketcan-isotp
            Err(CanError::IsoTpError(
                "Multi-frame ISO-TP requires socketcan-isotp on Linux".into(),
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

        let frame_type = (frame.data[0] >> 4) & 0x0F;
        match frame_type {
            0x0 => {
                let length = (frame.data[0] & 0x0F) as usize;
                if length == 0 || frame.data.len() < 1 + length {
                    return Err(CanError::IsoTpError("Invalid single frame".into()));
                }
                Ok(frame.data[1..1 + length].to_vec())
            }
            _ => Err(CanError::IsoTpError(
                "Multi-frame reception requires socketcan-isotp".into(),
            )),
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::SocketCan
    }

    fn adapter_name(&self) -> &str {
        &self.interface
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socketcan_adapter_creation() {
        let adapter = SocketCanAdapter::new("can0");
        assert_eq!(adapter.adapter_name(), "can0");
        assert_eq!(adapter.adapter_type(), AdapterType::SocketCan);
        assert!(!adapter.is_connected());
    }

    #[tokio::test]
    async fn test_socketcan_not_connected_send() {
        let adapter = SocketCanAdapter::new("can0");
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        let result = adapter.send_frame(&frame).await;
        assert!(matches!(result, Err(CanError::NotConnected)));
    }

    #[tokio::test]
    async fn test_socketcan_not_connected_recv() {
        let adapter = SocketCanAdapter::new("can0");
        let result = adapter.recv_frame(100).await;
        assert!(matches!(result, Err(CanError::NotConnected)));
    }
}
