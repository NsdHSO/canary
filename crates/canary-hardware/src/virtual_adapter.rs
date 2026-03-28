use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::adapter_trait::{AdapterType, CanAdapter, CanFrame};
use crate::error::CanError;

/// Virtual CAN adapter for testing without physical hardware.
///
/// Implements a loopback interface that stores sent frames and
/// returns them on receive. Useful for unit tests, CI/CD, and
/// development without access to real CAN hardware.
pub struct VirtualAdapter {
    name: String,
    connected: bool,
    /// Internal frame buffer (loopback)
    tx_buffer: Arc<Mutex<VecDeque<CanFrame>>>,
    /// Received frames buffer
    rx_buffer: Arc<Mutex<VecDeque<CanFrame>>>,
}

impl VirtualAdapter {
    /// Create a new virtual adapter with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            connected: false,
            tx_buffer: Arc::new(Mutex::new(VecDeque::new())),
            rx_buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Inject a frame into the receive buffer (for testing)
    pub fn inject_frame(&self, frame: CanFrame) {
        if let Ok(mut buf) = self.rx_buffer.lock() {
            buf.push_back(frame);
        }
    }

    /// Get all sent frames (for test verification)
    pub fn get_sent_frames(&self) -> Vec<CanFrame> {
        self.tx_buffer
            .lock()
            .map(|buf| buf.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Clear all buffers
    pub fn clear_buffers(&self) {
        if let Ok(mut buf) = self.tx_buffer.lock() {
            buf.clear();
        }
        if let Ok(mut buf) = self.rx_buffer.lock() {
            buf.clear();
        }
    }
}

#[async_trait]
impl CanAdapter for VirtualAdapter {
    async fn connect(&mut self) -> Result<(), CanError> {
        self.connected = true;
        log::info!("Virtual adapter '{}' connected", self.name);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CanError> {
        self.connected = false;
        self.clear_buffers();
        log::info!("Virtual adapter '{}' disconnected", self.name);
        Ok(())
    }

    async fn send_frame(&self, frame: &CanFrame) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        let mut buf = self.tx_buffer.lock().map_err(|e| {
            CanError::SendFailed(format!("Lock poisoned: {}", e))
        })?;
        buf.push_back(frame.clone());

        // Loopback: also add to rx buffer
        let mut rx_buf = self.rx_buffer.lock().map_err(|e| {
            CanError::SendFailed(format!("Lock poisoned: {}", e))
        })?;
        rx_buf.push_back(frame.clone());

        log::debug!("Virtual TX: {}", frame);
        Ok(())
    }

    async fn recv_frame(&self, timeout_ms: u64) -> Result<CanFrame, CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            {
                let mut buf = self.rx_buffer.lock().map_err(|e| {
                    CanError::Other(format!("Lock poisoned: {}", e))
                })?;
                if let Some(frame) = buf.pop_front() {
                    log::debug!("Virtual RX: {}", frame);
                    return Ok(frame);
                }
            }

            if start.elapsed() >= timeout {
                return Err(CanError::Timeout(timeout_ms));
            }

            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }

    async fn send_isotp(&self, target_id: u32, data: &[u8]) -> Result<(), CanError> {
        if !self.connected {
            return Err(CanError::NotConnected);
        }

        // Simple single-frame ISO-TP for data <= 7 bytes
        if data.len() <= 7 {
            let mut frame_data = vec![data.len() as u8];
            frame_data.extend_from_slice(data);
            // Pad to 8 bytes
            while frame_data.len() < 8 {
                frame_data.push(0x00);
            }
            let frame = CanFrame::new(target_id, frame_data);
            self.send_frame(&frame).await?;
        } else {
            // Multi-frame ISO-TP: First Frame + Consecutive Frames
            let total_len = data.len();

            // First Frame (FF): [1X XX] where XXX = total length
            let mut ff_data = vec![
                0x10 | ((total_len >> 8) & 0x0F) as u8,
                (total_len & 0xFF) as u8,
            ];
            ff_data.extend_from_slice(&data[..6]);
            let ff = CanFrame::new(target_id, ff_data);
            self.send_frame(&ff).await?;

            // Consecutive Frames (CF): [2N] where N = sequence number
            let mut offset = 6;
            let mut seq = 1u8;
            while offset < total_len {
                let end = std::cmp::min(offset + 7, total_len);
                let mut cf_data = vec![0x20 | (seq & 0x0F)];
                cf_data.extend_from_slice(&data[offset..end]);
                // Pad to 8 bytes
                while cf_data.len() < 8 {
                    cf_data.push(0x00);
                }
                let cf = CanFrame::new(target_id, cf_data);
                self.send_frame(&cf).await?;
                offset = end;
                seq = (seq + 1) & 0x0F;
            }
        }

        Ok(())
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
                // Single Frame
                let length = (frame.data[0] & 0x0F) as usize;
                if length == 0 || length > 7 || frame.data.len() < 1 + length {
                    return Err(CanError::IsoTpError("Invalid single frame length".into()));
                }
                Ok(frame.data[1..1 + length].to_vec())
            }
            0x1 => {
                // First Frame - collect consecutive frames
                let total_len = (((frame.data[0] & 0x0F) as usize) << 8)
                    | (frame.data[1] as usize);
                let mut data = frame.data[2..].to_vec();

                while data.len() < total_len {
                    let cf = self.recv_frame(timeout_ms).await?;
                    let cf_type = (cf.data[0] >> 4) & 0x0F;
                    if cf_type != 0x2 {
                        return Err(CanError::IsoTpError(
                            format!("Expected CF, got frame type 0x{:X}", cf_type),
                        ));
                    }
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
        AdapterType::Virtual
    }

    fn adapter_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtual_adapter_connect_disconnect() {
        let mut adapter = VirtualAdapter::new("vcan0");
        assert!(!adapter.is_connected());

        adapter.connect().await.unwrap();
        assert!(adapter.is_connected());

        adapter.disconnect().await.unwrap();
        assert!(!adapter.is_connected());
    }

    #[tokio::test]
    async fn test_virtual_adapter_send_receive() {
        let mut adapter = VirtualAdapter::new("vcan0");
        adapter.connect().await.unwrap();

        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);
        adapter.send_frame(&frame).await.unwrap();

        let received = adapter.recv_frame(1000).await.unwrap();
        assert_eq!(received.id, 0x7E0);
        assert_eq!(received.data, vec![0x02, 0x10, 0x01]);
    }

    #[tokio::test]
    async fn test_virtual_adapter_not_connected() {
        let adapter = VirtualAdapter::new("vcan0");
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);

        let result = adapter.send_frame(&frame).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_virtual_adapter_inject_frame() {
        let mut adapter = VirtualAdapter::new("vcan0");
        adapter.connect().await.unwrap();

        let response = CanFrame::new(0x7E8, vec![0x06, 0x50, 0x01, 0x00, 0x19, 0x01, 0xF4]);
        adapter.inject_frame(response);

        let received = adapter.recv_frame(1000).await.unwrap();
        assert_eq!(received.id, 0x7E8);
    }

    #[tokio::test]
    async fn test_virtual_adapter_timeout() {
        let mut adapter = VirtualAdapter::new("vcan0");
        adapter.connect().await.unwrap();
        adapter.clear_buffers();

        let result = adapter.recv_frame(10).await;
        assert!(matches!(result, Err(CanError::Timeout(10))));
    }

    #[tokio::test]
    async fn test_virtual_adapter_isotp_single_frame() {
        let mut adapter = VirtualAdapter::new("vcan0");
        adapter.connect().await.unwrap();

        // Send a short ISO-TP message (single frame)
        adapter.send_isotp(0x7E0, &[0x10, 0x01]).await.unwrap();

        // Receive and decode
        let data = adapter.recv_isotp(1000).await.unwrap();
        assert_eq!(data, vec![0x10, 0x01]);
    }

    #[tokio::test]
    async fn test_virtual_adapter_get_sent_frames() {
        let mut adapter = VirtualAdapter::new("vcan0");
        adapter.connect().await.unwrap();

        adapter
            .send_frame(&CanFrame::new(0x100, vec![0x01]))
            .await
            .unwrap();
        adapter
            .send_frame(&CanFrame::new(0x200, vec![0x02]))
            .await
            .unwrap();

        let sent = adapter.get_sent_frames();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].id, 0x100);
        assert_eq!(sent[1].id, 0x200);
    }

    #[tokio::test]
    async fn test_virtual_adapter_type() {
        let adapter = VirtualAdapter::new("vcan0");
        assert_eq!(adapter.adapter_type(), AdapterType::Virtual);
        assert_eq!(adapter.adapter_name(), "vcan0");
    }
}
