use canary_hardware::CanAdapter;

use crate::error::UdsError;
use crate::services::{read_data, read_dtc, session_control};
use crate::services::session_control::SessionType;
use crate::services::read_dtc::{DtcSubFunction, ReadDtcResponse};
use crate::services::read_data::ReadDataResponse;

/// Default response timeout in milliseconds
const DEFAULT_TIMEOUT_MS: u64 = 2000;

/// Default response CAN ID offset from request ID
const RESPONSE_ID_OFFSET: u32 = 0x08;

/// UDS Diagnostic Session Manager
///
/// Manages a diagnostic session with a specific ECU, handling
/// request/response communication over the CAN bus adapter.
///
/// # Example
///
/// ```rust,no_run
/// use canary_hardware::{VirtualAdapter, CanAdapter};
/// use canary_uds::UdsSession;
///
/// # async fn example() -> Result<(), canary_uds::UdsError> {
/// let mut adapter = VirtualAdapter::new("vcan0");
/// adapter.connect().await.map_err(canary_uds::UdsError::AdapterError)?;
///
/// let session = UdsSession::new(Box::new(adapter), 0x7E0);
/// # Ok(())
/// # }
/// ```
pub struct UdsSession {
    adapter: Box<dyn CanAdapter>,
    /// ECU request CAN ID (e.g., 0x7E0)
    ecu_request_id: u32,
    /// ECU response CAN ID (typically request_id + 8)
    ecu_response_id: u32,
    /// Current session type
    session_type: SessionType,
    /// Security access level
    security_level: u8,
    /// Response timeout in milliseconds
    timeout_ms: u64,
}

impl UdsSession {
    /// Create a new UDS session for a specific ECU
    ///
    /// # Arguments
    /// * `adapter` - Connected CAN adapter
    /// * `ecu_request_id` - CAN ID for requests (e.g., 0x7E0)
    pub fn new(adapter: Box<dyn CanAdapter>, ecu_request_id: u32) -> Self {
        Self {
            adapter,
            ecu_request_id,
            ecu_response_id: ecu_request_id + RESPONSE_ID_OFFSET,
            session_type: SessionType::Default,
            security_level: 0,
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }

    /// Create a UDS session with custom response ID
    pub fn with_response_id(
        adapter: Box<dyn CanAdapter>,
        ecu_request_id: u32,
        ecu_response_id: u32,
    ) -> Self {
        Self {
            adapter,
            ecu_request_id,
            ecu_response_id,
            session_type: SessionType::Default,
            security_level: 0,
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }

    /// Set the response timeout
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }

    /// Get the current session type
    pub fn session_type(&self) -> SessionType {
        self.session_type
    }

    /// Get the current security level
    pub fn security_level(&self) -> u8 {
        self.security_level
    }

    /// Get the ECU request ID
    pub fn ecu_request_id(&self) -> u32 {
        self.ecu_request_id
    }

    /// Get the ECU response ID
    pub fn ecu_response_id(&self) -> u32 {
        self.ecu_response_id
    }

    /// Send a raw UDS request and receive the response
    pub async fn send_request(&self, request: &[u8]) -> Result<Vec<u8>, UdsError> {
        // Send via ISO-TP
        self.adapter
            .send_isotp(self.ecu_request_id, request)
            .await?;

        // Receive response via ISO-TP
        let response = self.adapter.recv_isotp(self.timeout_ms).await?;

        // Check for "response pending" (NRC 0x78)
        if response.len() >= 3 && response[0] == 0x7F && response[2] == 0x78 {
            // ECU needs more time - wait and retry
            log::debug!("ECU response pending, waiting...");
            let extended_response = self
                .adapter
                .recv_isotp(self.timeout_ms * 5)
                .await?;
            return Ok(extended_response);
        }

        Ok(response)
    }

    /// Service 0x10: Start a diagnostic session
    pub async fn start_session(
        &mut self,
        session_type: SessionType,
    ) -> Result<session_control::SessionControlResponse, UdsError> {
        let request = session_control::build_request(session_type);
        let response = self.send_request(&request).await?;
        let result = session_control::parse_response(&response)?;
        self.session_type = result.session_type;
        Ok(result)
    }

    /// Service 0x19: Read DTCs
    pub async fn read_dtcs(&self) -> Result<ReadDtcResponse, UdsError> {
        let request = read_dtc::build_report_all_dtcs();
        let response = self.send_request(&request).await?;
        read_dtc::parse_response(&response)
    }

    /// Service 0x19: Read only active DTCs
    pub async fn read_active_dtcs(&self) -> Result<ReadDtcResponse, UdsError> {
        let request = read_dtc::build_report_active_dtcs();
        let response = self.send_request(&request).await?;
        read_dtc::parse_response(&response)
    }

    /// Service 0x19: Read DTCs with custom sub-function and mask
    pub async fn read_dtcs_custom(
        &self,
        sub_function: DtcSubFunction,
        status_mask: u8,
    ) -> Result<ReadDtcResponse, UdsError> {
        let request = read_dtc::build_request(sub_function, status_mask);
        let response = self.send_request(&request).await?;
        read_dtc::parse_response(&response)
    }

    /// Service 0x22: Read data by identifier
    pub async fn read_data_by_id(&self, did: u16) -> Result<ReadDataResponse, UdsError> {
        let request = read_data::build_request(did);
        let response = self.send_request(&request).await?;
        read_data::parse_response(&response)
    }

    /// Service 0x22: Read VIN
    pub async fn read_vin(&self) -> Result<String, UdsError> {
        let request = read_data::build_request(read_data::CommonDid::Vin.value());
        let response = self.send_request(&request).await?;
        read_data::parse_vin_response(&response)
    }

    /// Service 0x22: Read multiple DIDs at once
    pub async fn read_multiple_dids(
        &self,
        dids: &[u16],
    ) -> Result<Vec<ReadDataResponse>, UdsError> {
        let mut results = Vec::new();
        for did in dids {
            match self.read_data_by_id(*did).await {
                Ok(resp) => results.push(resp),
                Err(e) => {
                    log::warn!("Failed to read DID 0x{:04X}: {}", did, e);
                }
            }
        }
        Ok(results)
    }

    /// Service 0x14: Clear DTCs
    pub async fn clear_dtcs(&self) -> Result<(), UdsError> {
        let request = vec![0x14, 0xFF, 0xFF, 0xFF]; // Clear all groups
        let response = self.send_request(&request).await?;

        if response.is_empty() {
            return Err(UdsError::InvalidResponse("Empty response".into()));
        }

        if response[0] == 0x7F {
            if response.len() >= 3 {
                return Err(UdsError::NegativeResponse {
                    service: response[1],
                    nrc: response[2],
                    description: crate::error::nrc_description(response[2]).to_string(),
                });
            }
            return Err(UdsError::InvalidResponse("Malformed negative response".into()));
        }

        if response[0] != 0x54 {
            return Err(UdsError::InvalidResponse(format!(
                "Expected 0x54, got 0x{:02X}",
                response[0]
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canary_hardware::VirtualAdapter;

    async fn create_test_session() -> UdsSession {
        let mut adapter = VirtualAdapter::new("test_vcan");
        adapter.connect().await.unwrap();
        UdsSession::new(Box::new(adapter), 0x7E0)
    }

    #[test]
    fn test_session_creation() {
        let adapter = VirtualAdapter::new("test_vcan");
        let session = UdsSession::new(Box::new(adapter), 0x7E0);
        assert_eq!(session.ecu_request_id(), 0x7E0);
        assert_eq!(session.ecu_response_id(), 0x7E8);
        assert_eq!(session.session_type(), SessionType::Default);
        assert_eq!(session.security_level(), 0);
    }

    #[test]
    fn test_session_custom_response_id() {
        let adapter = VirtualAdapter::new("test_vcan");
        let session = UdsSession::with_response_id(Box::new(adapter), 0x7E0, 0x7E8);
        assert_eq!(session.ecu_response_id(), 0x7E8);
    }

    #[test]
    fn test_session_timeout() {
        let adapter = VirtualAdapter::new("test_vcan");
        let mut session = UdsSession::new(Box::new(adapter), 0x7E0);
        session.set_timeout(5000);
        // Timeout is set internally, no public getter to verify
        // but the method should not panic
    }

    #[tokio::test]
    async fn test_send_request_receives_loopback() {
        let session = create_test_session().await;

        // VirtualAdapter loops back sent frames, so we'll receive our own request
        // In real usage, the ECU would send a different response
        let result = session.send_request(&[0x10, 0x01]).await;

        // Virtual adapter will loopback, so we get back the ISO-TP frame
        // The response will be the single-frame ISO-TP we sent
        assert!(result.is_ok());
    }
}
