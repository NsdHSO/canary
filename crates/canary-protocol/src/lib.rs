use canary_data::PROTOCOLS;
use canary_models::{
    embedded::{CanFrame, KLineFrame, ProtocolSpec},
    CanaryError, Result,
};
use chrono::Utc;

/// Trait for all protocol decoders (Interface Segregation principle)
pub trait ProtocolDecoder {
    type Frame;

    fn decode(&self, raw: &[u8]) -> Result<Self::Frame>;
    fn encode(&self, frame: &Self::Frame) -> Result<Vec<u8>>;
}

/// CAN Bus 2.0B decoder
pub struct CanDecoder {
    #[allow(dead_code)]
    spec: &'static ProtocolSpec,
}

impl CanDecoder {
    pub fn new() -> Result<Self> {
        let spec = PROTOCOLS
            .get("can_2.0b")
            .ok_or_else(|| CanaryError::NotFound("CAN 2.0B protocol".into()))?;

        Ok(Self { spec })
    }
}

impl ProtocolDecoder for CanDecoder {
    type Frame = CanFrame;

    fn decode(&self, raw: &[u8]) -> Result<Self::Frame> {
        if raw.len() < 4 {
            return Err(CanaryError::ProtocolError(
                "CAN frame too short (minimum 4 bytes)".into(),
            ));
        }

        // Extract CAN ID (first 4 bytes, big-endian)
        let id = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);

        // Data is everything after ID
        let data = raw
            .get(4..)
            .map(|d| d.to_vec())
            .unwrap_or_default();

        Ok(CanFrame {
            id,
            data,
            timestamp: Utc::now(),
        })
    }

    fn encode(&self, frame: &Self::Frame) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&frame.id.to_be_bytes());
        buf.extend_from_slice(&frame.data);
        Ok(buf)
    }
}

/// K-Line (KWP2000) decoder
pub struct KLineDecoder {
    #[allow(dead_code)]
    spec: &'static ProtocolSpec,
}

impl KLineDecoder {
    pub fn new() -> Result<Self> {
        // K-Line uses CAN protocol spec for now (simplified)
        let spec = PROTOCOLS
            .get("can_2.0b")
            .ok_or_else(|| CanaryError::NotFound("K-Line protocol".into()))?;

        Ok(Self { spec })
    }
}

impl ProtocolDecoder for KLineDecoder {
    type Frame = KLineFrame;

    fn decode(&self, raw: &[u8]) -> Result<Self::Frame> {
        if raw.len() < 3 {
            return Err(CanaryError::ProtocolError(
                "K-Line frame too short".into(),
            ));
        }

        let checksum = raw.last().copied().unwrap_or(0);
        let header = raw.get(0..2).unwrap_or(&[]).to_vec();
        let data = raw
            .get(2..raw.len() - 1)
            .unwrap_or(&[])
            .to_vec();

        Ok(KLineFrame {
            header,
            data,
            checksum,
            timestamp: Utc::now(),
        })
    }

    fn encode(&self, frame: &Self::Frame) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&frame.header);
        buf.extend_from_slice(&frame.data);
        buf.push(frame.checksum);
        Ok(buf)
    }
}

/// Factory for creating protocol decoders
pub struct ProtocolFactory;

impl ProtocolFactory {
    pub fn create_can_decoder() -> Result<CanDecoder> {
        CanDecoder::new()
    }

    pub fn create_kline_decoder() -> Result<KLineDecoder> {
        KLineDecoder::new()
    }

    pub fn list_available_protocols() -> Vec<&'static str> {
        PROTOCOLS.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_decoder_creation() {
        let decoder = CanDecoder::new();
        assert!(decoder.is_ok());
    }

    #[test]
    fn test_can_decode_encode_symmetry() {
        let decoder = CanDecoder::new().unwrap();

        let original_frame = CanFrame {
            id: 0x123,
            data: vec![0x01, 0x02, 0x03],
            timestamp: Utc::now(),
        };

        let encoded = decoder.encode(&original_frame).unwrap();
        let decoded = decoder.decode(&encoded).unwrap();

        assert_eq!(original_frame.id, decoded.id);
        assert_eq!(original_frame.data, decoded.data);
    }

    #[test]
    fn test_kline_decoder_creation() {
        let decoder = KLineDecoder::new();
        assert!(decoder.is_ok());
    }

    #[test]
    fn test_protocol_factory() {
        let protocols = ProtocolFactory::list_available_protocols();
        assert!(!protocols.is_empty());
    }
}
