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
//! - **0x27** SecurityAccess
//! - **0x2E** WriteDataByIdentifier
//! - **0x2F** InputOutputControlByIdentifier
//! - **0x31** RoutineControl
//! - **0x34** RequestDownload
//! - **0x35** RequestUpload
//! - **0x36** TransferData
//! - **0x37** RequestTransferExit
//!
//! ## Architecture
//!
//! Uses `canary-hardware` for CAN bus communication. The `UdsSession`
//! struct manages the diagnostic session state and provides high-level
//! methods for each UDS service.

pub mod error;
pub mod monitor;
pub mod services;
pub mod session;

// Re-exports
pub use error::UdsError;
pub use session::UdsSession;
pub use services::session_control::SessionType;
pub use services::read_dtc::{DtcEntry, DtcStatus, DtcSubFunction, ReadDtcResponse};
pub use services::read_data::{CommonDid, ReadDataResponse};
pub use services::write_data::WriteDataResponse;
pub use services::io_control::{IoControlParameter, IoControlResponse};
pub use services::routine::{CommonRoutine, RoutineControlResponse, RoutineControlType, RoutineStatus};
pub use services::download::{
    DataFormatIdentifier, RequestDownloadResponse, RequestUploadResponse,
    TransferConfig, TransferDataResponse, TransferDirection, TransferExitResponse,
};
pub use services::security_access::{SeedResponse, KeyResponse};
pub use monitor::{MultiEcuMonitor, MonitoredEcu, EcuData, MonitorStats};
