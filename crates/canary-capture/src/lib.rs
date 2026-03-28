//! # canary-capture
//!
//! Real-time CAN bus capture, filtering, SQLite logging, and replay.
//!
//! ## Features
//!
//! - **Capture**: Real-time CAN frame capture at 1000+ frames/sec
//! - **Filter**: Filter by CAN ID, ID range, or custom predicates
//! - **Logger**: SQLite-based persistent logging of capture sessions
//! - **Replay**: Replay captured sessions at original timing
//!
//! ## Example
//!
//! ```rust,no_run
//! use canary_capture::{CaptureSession, CaptureConfig, CanFilter};
//!
//! # async fn example() -> Result<(), canary_capture::CaptureError> {
//! let config = CaptureConfig::new()
//!     .with_filter(CanFilter::id_range(0x700, 0x7FF))
//!     .with_db_path("/tmp/capture.db");
//!
//! // Start a capture session
//! let session = CaptureSession::new(config)?;
//! # Ok(())
//! # }
//! ```

pub mod capture;
pub mod filter;
pub mod logger;
pub mod replay;

// Re-exports
pub use capture::{CaptureConfig, CaptureSession, CapturedFrame};
pub use filter::CanFilter;
pub use logger::SqliteLogger;
pub use replay::ReplayEngine;

use thiserror::Error;

/// Capture-related errors
#[derive(Error, Debug)]
pub enum CaptureError {
    /// CAN adapter error
    #[error("CAN adapter error: {0}")]
    AdapterError(#[from] canary_hardware::CanError),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// No active capture session
    #[error("No active capture session")]
    NoActiveSession,

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(i64),

    /// Replay error
    #[error("Replay error: {0}")]
    ReplayError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<rusqlite::Error> for CaptureError {
    fn from(e: rusqlite::Error) -> Self {
        CaptureError::DatabaseError(e.to_string())
    }
}
