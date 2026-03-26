use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSpec {
    pub id: String,
    pub name: String,
    pub bit_rate: u32,
    pub frame_format: FrameFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameFormat {
    Standard,
    Extended,
}

#[derive(Debug, Clone)]
pub struct CanFrame {
    pub id: u32,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct KLineFrame {
    pub header: Vec<u8>,
    pub data: Vec<u8>,
    pub checksum: u8,
    pub timestamp: DateTime<Utc>,
}
