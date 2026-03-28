use thiserror::Error;

/// UDS protocol errors
#[derive(Error, Debug)]
pub enum UdsError {
    /// CAN adapter error
    #[error("CAN adapter error: {0}")]
    AdapterError(#[from] canary_hardware::CanError),

    /// Negative response from ECU
    #[error("ECU negative response: service 0x{service:02X}, NRC 0x{nrc:02X} ({description})")]
    NegativeResponse {
        service: u8,
        nrc: u8,
        description: String,
    },

    /// Invalid response format
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Session not active
    #[error("No active diagnostic session")]
    NoActiveSession,

    /// Service not supported by ECU
    #[error("Service 0x{0:02X} not supported")]
    ServiceNotSupported(u8),

    /// Timeout waiting for response
    #[error("Response timeout after {0}ms")]
    Timeout(u64),

    /// Security access denied
    #[error("Security access denied: {0}")]
    SecurityDenied(String),
}

/// Get human-readable description for a UDS Negative Response Code (NRC)
pub fn nrc_description(nrc: u8) -> &'static str {
    match nrc {
        0x10 => "General reject",
        0x11 => "Service not supported",
        0x12 => "Sub-function not supported",
        0x13 => "Incorrect message length or invalid format",
        0x14 => "Response too long",
        0x21 => "Busy - repeat request",
        0x22 => "Conditions not correct",
        0x24 => "Request sequence error",
        0x25 => "No response from sub-net component",
        0x26 => "Failure prevents execution",
        0x31 => "Request out of range",
        0x33 => "Security access denied",
        0x35 => "Invalid key",
        0x36 => "Exceeded number of attempts",
        0x37 => "Required time delay not expired",
        0x70 => "Upload/download not accepted",
        0x71 => "Transfer data suspended",
        0x72 => "General programming failure",
        0x73 => "Wrong block sequence counter",
        0x78 => "Request correctly received - response pending",
        0x7E => "Sub-function not supported in active session",
        0x7F => "Service not supported in active session",
        _ => "Unknown NRC",
    }
}
