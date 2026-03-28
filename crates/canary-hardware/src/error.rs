use thiserror::Error;

/// Errors that can occur during CAN adapter operations
#[derive(Error, Debug)]
pub enum CanError {
    /// Adapter is not connected
    #[error("Adapter not connected")]
    NotConnected,

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Send operation failed
    #[error("Send failed: {0}")]
    SendFailed(String),

    /// Receive operation timed out
    #[error("Receive timeout after {0}ms")]
    Timeout(u64),

    /// Received an invalid CAN frame
    #[error("Invalid CAN frame: {0}")]
    InvalidFrame(String),

    /// ISO-TP protocol error
    #[error("ISO-TP error: {0}")]
    IsoTpError(String),

    /// Adapter not found
    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),

    /// Bluetooth-specific error
    #[error("Bluetooth error: {0}")]
    BluetoothError(String),

    /// WiFi-specific error
    #[error("WiFi error: {0}")]
    WiFiError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic adapter error
    #[error("Adapter error: {0}")]
    Other(String),
}
