use canary_hardware::CanAdapter;

use crate::error::UdsError;
use crate::services::{read_data, read_dtc, session_control, write_data, io_control, routine, download, security_access};
use crate::services::session_control::SessionType;
use crate::services::read_dtc::{DtcSubFunction, ReadDtcResponse};
use crate::services::read_data::ReadDataResponse;
use crate::services::write_data::WriteDataResponse;
use crate::services::io_control::{IoControlParameter, IoControlResponse};
use crate::services::routine::RoutineControlResponse;
use crate::services::download::{
    DataFormatIdentifier, RequestDownloadResponse, RequestUploadResponse,
    TransferDataResponse, TransferExitResponse,
};
use crate::services::security_access::{SeedResponse, KeyResponse};

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

    // ===== Phase 3: New UDS Services =====

    /// Service 0x27: Request security seed
    pub async fn request_seed(&self, access_level: u8) -> Result<SeedResponse, UdsError> {
        let request = security_access::build_seed_request(access_level);
        let response = self.send_request(&request).await?;
        security_access::parse_seed_response(&response)
    }

    /// Service 0x27: Send security key
    pub async fn send_key(
        &mut self,
        access_level: u8,
        key: &[u8],
    ) -> Result<KeyResponse, UdsError> {
        let request = security_access::build_key_response(access_level, key);
        let response = self.send_request(&request).await?;
        let result = security_access::parse_key_response(&response)?;
        self.security_level = access_level;
        Ok(result)
    }

    /// Service 0x2E: Write data by identifier
    pub async fn write_data_by_id(
        &self,
        did: u16,
        data: &[u8],
    ) -> Result<WriteDataResponse, UdsError> {
        let request = write_data::build_request(did, data);
        let response = self.send_request(&request).await?;
        write_data::parse_response(&response)
    }

    /// Service 0x2F: I/O control by identifier
    pub async fn io_control(
        &self,
        did: u16,
        control_param: IoControlParameter,
        control_state: &[u8],
    ) -> Result<IoControlResponse, UdsError> {
        let request = io_control::build_request(did, control_param, control_state);
        let response = self.send_request(&request).await?;
        io_control::parse_response(&response)
    }

    /// Service 0x2F: Return I/O control to ECU
    pub async fn io_return_control(&self, did: u16) -> Result<IoControlResponse, UdsError> {
        let request = io_control::build_return_control(did);
        let response = self.send_request(&request).await?;
        io_control::parse_response(&response)
    }

    /// Service 0x31: Start a routine
    pub async fn start_routine(
        &self,
        routine_id: u16,
        option_record: &[u8],
    ) -> Result<RoutineControlResponse, UdsError> {
        let request = routine::build_start_routine(routine_id, option_record);
        let response = self.send_request(&request).await?;
        routine::parse_response(&response)
    }

    /// Service 0x31: Stop a routine
    pub async fn stop_routine(&self, routine_id: u16) -> Result<RoutineControlResponse, UdsError> {
        let request = routine::build_stop_routine(routine_id);
        let response = self.send_request(&request).await?;
        routine::parse_response(&response)
    }

    /// Service 0x31: Request routine results
    pub async fn request_routine_results(
        &self,
        routine_id: u16,
    ) -> Result<RoutineControlResponse, UdsError> {
        let request = routine::build_request_results(routine_id);
        let response = self.send_request(&request).await?;
        routine::parse_response(&response)
    }

    /// Service 0x34: Request download (tester -> ECU)
    pub async fn request_download(
        &self,
        memory_address: u32,
        memory_size: u32,
    ) -> Result<RequestDownloadResponse, UdsError> {
        let request = download::build_request_download(
            DataFormatIdentifier::none(),
            memory_address,
            memory_size,
        );
        let response = self.send_request(&request).await?;
        download::parse_request_download_response(&response)
    }

    /// Service 0x35: Request upload (ECU -> tester)
    pub async fn request_upload(
        &self,
        memory_address: u32,
        memory_size: u32,
    ) -> Result<RequestUploadResponse, UdsError> {
        let request = download::build_request_upload(
            DataFormatIdentifier::none(),
            memory_address,
            memory_size,
        );
        let response = self.send_request(&request).await?;
        download::parse_request_upload_response(&response)
    }

    /// Service 0x36: Transfer data block
    pub async fn transfer_data(
        &self,
        block_sequence_counter: u8,
        data: &[u8],
    ) -> Result<TransferDataResponse, UdsError> {
        let request = download::build_transfer_data(block_sequence_counter, data);
        let response = self.send_request(&request).await?;
        download::parse_transfer_data_response(&response)
    }

    /// Service 0x37: Request transfer exit
    pub async fn request_transfer_exit(&self) -> Result<TransferExitResponse, UdsError> {
        let request = download::build_transfer_exit(&[]);
        let response = self.send_request(&request).await?;
        download::parse_transfer_exit_response(&response)
    }

    /// High-level: Download memory from ECU (handles block sequencing)
    pub async fn download_memory(
        &self,
        memory_address: u32,
        data: &[u8],
    ) -> Result<(), UdsError> {
        // Request download
        let dl_resp = self.request_download(memory_address, data.len() as u32).await?;
        let block_size = dl_resp.max_block_length as usize;
        let effective_block_size = if block_size > 2 { block_size - 2 } else { block_size };

        // Transfer data blocks
        let mut block_counter: u8 = 1;
        let mut offset = 0;

        while offset < data.len() {
            let end = std::cmp::min(offset + effective_block_size, data.len());
            let block = &data[offset..end];
            self.transfer_data(block_counter, block).await?;

            block_counter = block_counter.wrapping_add(1);
            if block_counter == 0 {
                block_counter = 1; // Skip 0 per ISO 14229
            }
            offset = end;
        }

        // Request transfer exit
        self.request_transfer_exit().await?;

        Ok(())
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
