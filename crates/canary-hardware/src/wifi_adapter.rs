use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::adapter_trait::{AdapterType, CanAdapter, CanFrame};
use crate::error::CanError;
use crate::network_protocol::{NetworkProtocol, ProtocolType};

/// WiFi CAN adapter for wireless diagnostics.
///
/// Supports devices like:
/// - ESP32-based adapters (custom firmware)
/// - OBDLink MX+ WiFi
/// - WiFi ELM327 clones
///
/// Communicates over TCP to a device that bridges WiFi to CAN bus.
pub struct WiFiAdapter {
    host: String,
    port: u16,
    stream: Arc<Mutex<Option<TcpStream>>>,
    protocol: NetworkProtocol,
    connected: bool,
}

impl WiFiAdapter {
    /// Create a new WiFi adapter targeting the given host and port
    ///
    /// # Arguments
    /// * `host` - IP address of the WiFi adapter (e.g., "192.168.4.1")
    /// * `port` - TCP port (default: 35000 for ELM327, 23 for some adapters)
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            stream: Arc::new(Mutex::new(None)),
            protocol: NetworkProtocol::new(ProtocolType::Tcp),
            connected: false,
        }
    }

    /// Create a WiFi adapter with default ELM327 port (35000)
    pub fn new_default(host: &str) -> Self {
        Self::new(host, 35000)
    }

    /// Get the connection address
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[async_trait]
impl CanAdapter for WiFiAdapter {
    async fn connect(&mut self) -> Result<(), CanError> {
        let addr = self.address();
        log::info!("Connecting to WiFi adapter at {}", addr);

        let stream = TcpStream::connect(&addr).await.map_err(|e| {
            CanError::ConnectionFailed(format!(
                "Failed to connect to WiFi adapter at {}: {}",
                addr, e
            ))
        })?;

        // Set TCP_NODELAY for low latency
        stream.set_nodelay(true).map_err(|e| {
            CanError::ConnectionFailed(format!("Failed to set TCP_NODELAY: {}", e))
        })?;

        *self.stream.lock().await = Some(stream);
        self.connected = true;
        log::info!("WiFi adapter connected to {}", addr);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CanError> {
        if let Some(stream) = self.stream.lock().await.take() {
            drop(stream);
        }
        self.connected = false;
        log::info!("WiFi adapter disconnected from {}:{}", self.host, self.port);
        Ok(())
    }

    async fn send_frame(&self, frame: &CanFrame) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        let mut guard = self.stream.lock().await;
        let stream = guard.as_mut().ok_or(CanError::NotConnected)?;

        let packet = self.protocol.encode_frame(frame);
        stream.write_all(&packet).await.map_err(|e| {
            CanError::SendFailed(format!("WiFi send failed: {}", e))
        })?;

        log::debug!("WiFi TX to {}:{}: {}", self.host, self.port, frame);
        Ok(())
    }

    async fn recv_frame(&self, timeout_ms: u64) -> Result<CanFrame, CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        let mut guard = self.stream.lock().await;
        let stream = guard.as_mut().ok_or(CanError::NotConnected)?;

        let mut buf = [0u8; 128];
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            stream.read(&mut buf),
        )
        .await;

        match result {
            Ok(Ok(n)) if n > 0 => {
                let frame = self.protocol.decode_frame(&buf[..n])?;
                log::debug!("WiFi RX from {}:{}: {}", self.host, self.port, frame);
                Ok(frame)
            }
            Ok(Ok(_)) => Err(CanError::WiFiError("Connection closed".into())),
            Ok(Err(e)) => Err(CanError::WiFiError(format!("Read error: {}", e))),
            Err(_) => Err(CanError::Timeout(timeout_ms)),
        }
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
            // Multi-frame ISO-TP over WiFi
            let total_len = data.len();
            let mut ff_data = vec![
                0x10 | ((total_len >> 8) & 0x0F) as u8,
                (total_len & 0xFF) as u8,
            ];
            ff_data.extend_from_slice(&data[..6]);
            self.send_frame(&CanFrame::new(target_id, ff_data)).await?;

            let mut offset = 6;
            let mut seq = 1u8;
            while offset < total_len {
                let end = std::cmp::min(offset + 7, total_len);
                let mut cf_data = vec![0x20 | (seq & 0x0F)];
                cf_data.extend_from_slice(&data[offset..end]);
                while cf_data.len() < 8 {
                    cf_data.push(0x00);
                }
                self.send_frame(&CanFrame::new(target_id, cf_data)).await?;
                offset = end;
                seq = (seq + 1) & 0x0F;
            }
            Ok(())
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
            0x1 => {
                let total_len = (((frame.data[0] & 0x0F) as usize) << 8)
                    | (frame.data[1] as usize);
                let mut data = frame.data[2..].to_vec();

                while data.len() < total_len {
                    let cf = self.recv_frame(timeout_ms).await?;
                    data.extend_from_slice(&cf.data[1..]);
                }
                data.truncate(total_len);
                Ok(data)
            }
            _ => Err(CanError::IsoTpError(
                format!("Unexpected frame type: 0x{:X}", frame_type),
            )),
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::WiFi
    }

    fn adapter_name(&self) -> &str {
        &self.host
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_adapter_creation() {
        let adapter = WiFiAdapter::new("192.168.4.1", 35000);
        assert_eq!(adapter.adapter_name(), "192.168.4.1");
        assert_eq!(adapter.adapter_type(), AdapterType::WiFi);
        assert!(!adapter.is_connected());
        assert_eq!(adapter.address(), "192.168.4.1:35000");
    }

    #[test]
    fn test_wifi_adapter_default_port() {
        let adapter = WiFiAdapter::new_default("192.168.4.1");
        assert_eq!(adapter.address(), "192.168.4.1:35000");
    }

    #[tokio::test]
    async fn test_wifi_not_connected_send() {
        let adapter = WiFiAdapter::new("192.168.4.1", 35000);
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        let result = adapter.send_frame(&frame).await;
        assert!(matches!(result, Err(CanError::NotConnected)));
    }
}
