//! # canary-uds
//!
//! UDS (Unified Diagnostic Services) protocol implementation
//! for automotive ECU diagnostics.
//!
//! Implements ISO 14229-1 services:
//! - **0x10** DiagnosticSessionControl
//! - **0x14** ClearDiagnosticInformation
//! - **0x19** ReadDTCInformation
//! - **0x22** ReadDataByIdentifier
//!
//! ## Architecture
//!
//! Uses `canary-hardware` for CAN bus communication. The `UdsSession`
//! struct manages the diagnostic session state and provides high-level
//! methods for each UDS service.

pub mod error;
pub mod services;
pub mod session;

// Re-exports
pub use error::UdsError;
pub use session::UdsSession;
pub use services::session_control::SessionType;
pub use services::read_dtc::{DtcEntry, DtcStatus, DtcSubFunction, ReadDtcResponse};
pub use services::read_data::{CommonDid, ReadDataResponse};
