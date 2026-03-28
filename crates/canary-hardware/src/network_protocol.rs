use crate::adapter_trait::CanFrame;
use crate::error::CanError;

/// Protocol type for network communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// TCP/IP for reliable connection
    Tcp,
    /// UDP for low-latency streaming
    Udp,
}

/// Network protocol handler for encoding/decoding CAN frames
/// over TCP/UDP connections.
///
/// Frame wire format:
/// ```text
/// [0-3]  CAN ID (big-endian u32)
/// [4]    Flags (bit 0 = extended, bits 1-7 reserved)
/// [5]    Data length (0-64)
/// [6..N] Data bytes
/// ```
pub struct NetworkProtocol {
    protocol_type: ProtocolType,
}

impl NetworkProtocol {
    /// Create a new network protocol handler
    pub fn new(protocol_type: ProtocolType) -> Self {
        Self { protocol_type }
    }

    /// Get the protocol type
    pub fn protocol_type(&self) -> ProtocolType {
        self.protocol_type
    }

    /// Encode a CAN frame into network bytes
    pub fn encode_frame(&self, frame: &CanFrame) -> Vec<u8> {
        let mut packet = Vec::with_capacity(6 + frame.data.len());

        // CAN ID (4 bytes, big-endian)
        packet.extend_from_slice(&frame.id.to_be_bytes());

        // Flags byte
        let flags = if frame.extended { 0x01 } else { 0x00 };
        packet.push(flags);

        // Data length
        packet.push(frame.data.len() as u8);

        // Data
        packet.extend_from_slice(&frame.data);

        packet
    }

    /// Decode network bytes into a CAN frame
    pub fn decode_frame(&self, bytes: &[u8]) -> Result<CanFrame, CanError> {
        if bytes.len() < 6 {
            return Err(CanError::InvalidFrame(
                "Network packet too short (need at least 6 bytes)".into(),
            ));
        }

        let id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let extended = (bytes[4] & 0x01) != 0;
        let data_len = bytes[5] as usize;

        if bytes.len() < 6 + data_len {
            return Err(CanError::InvalidFrame(format!(
                "Packet claims {} data bytes but only {} available",
                data_len,
                bytes.len() - 6
            )));
        }

        let data = bytes[6..6 + data_len].to_vec();

        Ok(CanFrame {
            id,
            data,
            extended,
            timestamp_us: 0,
        })
    }

    /// Calculate packet size for a given data length
    pub fn packet_size(data_len: usize) -> usize {
        6 + data_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let protocol = NetworkProtocol::new(ProtocolType::Tcp);
        let frame = CanFrame::new(0x7E0, vec![0x02, 0x10, 0x01]);

        let encoded = protocol.encode_frame(&frame);
        let decoded = protocol.decode_frame(&encoded).unwrap();

        assert_eq!(decoded.id, frame.id);
        assert_eq!(decoded.data, frame.data);
        assert_eq!(decoded.extended, frame.extended);
    }

    #[test]
    fn test_encode_decode_extended_frame() {
        let protocol = NetworkProtocol::new(ProtocolType::Tcp);
        let frame = CanFrame::new_extended(0x18DA00F1, vec![0x02, 0x10, 0x01]);

        let encoded = protocol.encode_frame(&frame);
        let decoded = protocol.decode_frame(&encoded).unwrap();

        assert_eq!(decoded.id, frame.id);
        assert!(decoded.extended);
    }

    #[test]
    fn test_decode_too_short() {
        let protocol = NetworkProtocol::new(ProtocolType::Tcp);
        let result = protocol.decode_frame(&[0x00, 0x01]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_data_length_mismatch() {
        let protocol = NetworkProtocol::new(ProtocolType::Tcp);
        // Header says 8 bytes of data but only 2 provided
        let bytes = [0x00, 0x00, 0x07, 0xE0, 0x00, 0x08, 0x02, 0x10];
        let result = protocol.decode_frame(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_empty_data() {
        let protocol = NetworkProtocol::new(ProtocolType::Tcp);
        let frame = CanFrame::new(0x000, vec![]);

        let encoded = protocol.encode_frame(&frame);
        assert_eq!(encoded.len(), 6); // 4 ID + 1 flags + 1 length + 0 data

        let decoded = protocol.decode_frame(&encoded).unwrap();
        assert!(decoded.data.is_empty());
    }

    #[test]
    fn test_packet_size() {
        assert_eq!(NetworkProtocol::packet_size(0), 6);
        assert_eq!(NetworkProtocol::packet_size(8), 14);
        assert_eq!(NetworkProtocol::packet_size(64), 70);
    }

    #[test]
    fn test_protocol_type() {
        let tcp = NetworkProtocol::new(ProtocolType::Tcp);
        assert_eq!(tcp.protocol_type(), ProtocolType::Tcp);

        let udp = NetworkProtocol::new(ProtocolType::Udp);
        assert_eq!(udp.protocol_type(), ProtocolType::Udp);
    }
}
