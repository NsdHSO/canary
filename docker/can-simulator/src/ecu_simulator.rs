use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Diagnostic Session Types (UDS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSession {
    Default = 0x01,
    Programming = 0x02,
    Extended = 0x03,
}

/// Diagnostic Trouble Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dtc {
    pub code: String,
    pub status: u8,
    pub description: String,
}

/// ECU Simulator Error Types
#[derive(Error, Debug)]
pub enum EcuError {
    #[error("Service not supported: 0x{0:02X}")]
    ServiceNotSupported(u8),

    #[error("Invalid request format")]
    InvalidRequest,

    #[error("Security access denied")]
    SecurityAccessDenied,

    #[error("Session not allowed")]
    SessionNotAllowed,
}

/// Base ECU Simulator
pub struct EcuSimulator {
    /// CAN ID for this ECU
    pub can_id: u32,

    /// Current diagnostic session
    pub session: DiagnosticSession,

    /// Security access level (0 = locked)
    pub security_level: u8,

    /// DTC database
    pub dtc_database: HashMap<String, Dtc>,

    /// Live data (PID → value)
    pub live_data: HashMap<u16, Vec<u8>>,
}

impl EcuSimulator {
    /// Create a new ECU simulator
    pub fn new(can_id: u32) -> Self {
        Self {
            can_id,
            session: DiagnosticSession::Default,
            security_level: 0,
            dtc_database: HashMap::new(),
            live_data: HashMap::new(),
        }
    }

    /// Handle UDS request and return response
    pub fn handle_uds_request(&mut self, request: &[u8]) -> Vec<u8> {
        if request.is_empty() {
            return self.negative_response(0x00, 0x13); // Incorrect message length
        }

        let service_id = request[0];
        let data = &request[1..];

        match service_id {
            0x10 => self.handle_session_control(data),
            0x19 => self.handle_read_dtc(data),
            0x22 => self.handle_read_data(data),
            0x27 => self.handle_security_access(data),
            0x3E => self.handle_tester_present(data),
            _ => self.negative_response(service_id, 0x11), // Service not supported
        }
    }

    /// UDS Service 0x10: Diagnostic Session Control
    fn handle_session_control(&mut self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return self.negative_response(0x10, 0x13); // Incorrect message length
        }

        let session_type = data[0];

        // Validate session type
        self.session = match session_type {
            0x01 => DiagnosticSession::Default,
            0x02 => DiagnosticSession::Programming,
            0x03 => DiagnosticSession::Extended,
            _ => return self.negative_response(0x10, 0x12), // Sub-function not supported
        };

        // Positive response: 0x50 (0x10 + 0x40) + session type + timing parameters
        vec![0x50, session_type, 0x00, 0x32, 0x01, 0xF4]
    }

    /// UDS Service 0x19: Read DTC Information
    fn handle_read_dtc(&mut self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return self.negative_response(0x19, 0x13); // Incorrect message length
        }

        let sub_function = data[0];

        match sub_function {
            0x02 => self.read_dtc_by_status_mask(data),
            0x04 => self.read_dtc_snapshot_record(data),
            0x06 => self.read_dtc_extended_data(data),
            _ => self.negative_response(0x19, 0x12), // Sub-function not supported
        }
    }

    /// Read DTCs by status mask (0x19 0x02)
    fn read_dtc_by_status_mask(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 2 {
            return self.negative_response(0x19, 0x13);
        }

        let status_mask = data[1];
        let mut response = vec![0x59, 0x02, 0x00]; // Positive response + availability mask

        // Filter DTCs by status mask
        for dtc in self.dtc_database.values() {
            if dtc.status & status_mask != 0 {
                // Convert DTC code to bytes (e.g., "P0301" -> [0x01, 0x03, 0x01])
                if let Some(dtc_bytes) = self.encode_dtc_code(&dtc.code) {
                    response.extend_from_slice(&dtc_bytes);
                    response.push(dtc.status);
                }
            }
        }

        response
    }

    /// Read DTC snapshot record (stub)
    fn read_dtc_snapshot_record(&self, _data: &[u8]) -> Vec<u8> {
        // Return empty snapshot for now
        vec![0x59, 0x04, 0x00]
    }

    /// Read DTC extended data (stub)
    fn read_dtc_extended_data(&self, _data: &[u8]) -> Vec<u8> {
        // Return no extended data for now
        vec![0x59, 0x06, 0x00]
    }

    /// UDS Service 0x22: Read Data By Identifier
    fn handle_read_data(&mut self, data: &[u8]) -> Vec<u8> {
        if data.len() < 2 {
            return self.negative_response(0x22, 0x13); // Incorrect message length
        }

        // Data identifier (2 bytes, big-endian)
        let did = u16::from_be_bytes([data[0], data[1]]);

        // Check if we have this data ID
        if let Some(value) = self.live_data.get(&did) {
            let mut response = vec![0x62]; // Positive response
            response.extend_from_slice(&did.to_be_bytes());
            response.extend_from_slice(value);
            response
        } else {
            self.negative_response(0x22, 0x31) // Request out of range
        }
    }

    /// UDS Service 0x27: Security Access
    fn handle_security_access(&mut self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return self.negative_response(0x27, 0x13);
        }

        let sub_function = data[0];

        match sub_function {
            0x01 => {
                // Request seed (level 1)
                let seed = vec![0x12, 0x34, 0x56, 0x78]; // Fixed seed for testing
                let mut response = vec![0x67, 0x01];
                response.extend_from_slice(&seed);
                response
            }
            0x02 => {
                // Send key (level 1)
                if data.len() < 5 {
                    return self.negative_response(0x27, 0x13);
                }

                // Simple validation: key = seed XOR 0xFF
                let expected_key = vec![0xED, 0xCB, 0xA9, 0x87];
                let provided_key = &data[1..5];

                if provided_key == expected_key {
                    self.security_level = 1;
                    vec![0x67, 0x02]
                } else {
                    self.negative_response(0x27, 0x35) // Invalid key
                }
            }
            _ => self.negative_response(0x27, 0x12), // Sub-function not supported
        }
    }

    /// UDS Service 0x3E: Tester Present
    fn handle_tester_present(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return self.negative_response(0x3E, 0x13);
        }

        let sub_function = data[0];
        if sub_function == 0x00 {
            vec![0x7E, 0x00] // Positive response
        } else {
            self.negative_response(0x3E, 0x12)
        }
    }

    /// Generate negative response
    fn negative_response(&self, service_id: u8, nrc: u8) -> Vec<u8> {
        vec![0x7F, service_id, nrc]
    }

    /// Encode DTC code string to bytes
    fn encode_dtc_code(&self, code: &str) -> Option<Vec<u8>> {
        if code.len() != 5 {
            return None;
        }

        let chars: Vec<char> = code.chars().collect();

        // First character (P/C/B/U) determines upper 2 bits
        let first_byte = match chars[0] {
            'P' => 0x00, // Powertrain
            'C' => 0x40, // Chassis
            'B' => 0x80, // Body
            'U' => 0xC0, // Network
            _ => return None,
        };

        // Convert remaining digits
        let digit1 = chars[1].to_digit(10)? as u8;
        let digit2 = chars[2].to_digit(10)? as u8;
        let digit3 = chars[3].to_digit(10)? as u8;
        let digit4 = chars[4].to_digit(10)? as u8;

        Some(vec![
            first_byte | (digit1 << 4) | digit2,
            (digit3 << 4) | digit4,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_control() {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Test default session
        let response = ecu.handle_uds_request(&[0x10, 0x01]);
        assert_eq!(response[0], 0x50); // Positive response
        assert_eq!(response[1], 0x01); // Session type

        // Test extended session
        let response = ecu.handle_uds_request(&[0x10, 0x03]);
        assert_eq!(response[0], 0x50);
        assert_eq!(response[1], 0x03);
        assert_eq!(ecu.session, DiagnosticSession::Extended);
    }

    #[test]
    fn test_read_dtc_empty_database() {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Read DTCs with status mask 0xFF
        let response = ecu.handle_uds_request(&[0x19, 0x02, 0xFF]);
        assert_eq!(response[0], 0x59); // Positive response
        assert_eq!(response[1], 0x02);
        assert_eq!(response.len(), 3); // No DTCs
    }

    #[test]
    fn test_read_dtc_with_data() {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Add test DTC
        ecu.dtc_database.insert("P0301".to_string(), Dtc {
            code: "P0301".to_string(),
            status: 0x08,
            description: "Cylinder 1 Misfire".to_string(),
        });

        let response = ecu.handle_uds_request(&[0x19, 0x02, 0xFF]);
        assert_eq!(response[0], 0x59);
        assert!(response.len() > 3); // Should have DTC data
    }

    #[test]
    fn test_read_data_by_id() {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Add test data
        ecu.live_data.insert(0xF190, vec![0x12, 0x34, 0x56, 0x78]);

        // Read VIN (0xF190)
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0x90]);
        assert_eq!(response[0], 0x62); // Positive response
        assert_eq!(response[1..3], [0xF1, 0x90]); // DID echo
        assert_eq!(response[3..], [0x12, 0x34, 0x56, 0x78]); // Data
    }

    #[test]
    fn test_read_data_not_found() {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Read non-existent data
        let response = ecu.handle_uds_request(&[0x22, 0xF1, 0x90]);
        assert_eq!(response[0], 0x7F); // Negative response
        assert_eq!(response[1], 0x22); // Service ID
        assert_eq!(response[2], 0x31); // Request out of range
    }

    #[test]
    fn test_security_access() {
        let mut ecu = EcuSimulator::new(0x7E0);

        // Request seed
        let response = ecu.handle_uds_request(&[0x27, 0x01]);
        assert_eq!(response[0], 0x67); // Positive response
        assert_eq!(response[1], 0x01);
        assert_eq!(response.len(), 6); // Seed + header

        // Send correct key
        let response = ecu.handle_uds_request(&[0x27, 0x02, 0xED, 0xCB, 0xA9, 0x87]);
        assert_eq!(response[0], 0x67); // Positive response
        assert_eq!(ecu.security_level, 1);

        // Send incorrect key
        ecu.security_level = 0;
        let response = ecu.handle_uds_request(&[0x27, 0x02, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(response[0], 0x7F); // Negative response
        assert_eq!(response[2], 0x35); // Invalid key
    }

    #[test]
    fn test_tester_present() {
        let mut ecu = EcuSimulator::new(0x7E0);

        let response = ecu.handle_uds_request(&[0x3E, 0x00]);
        assert_eq!(response[0], 0x7E); // Positive response
        assert_eq!(response[1], 0x00);
    }

    #[test]
    fn test_service_not_supported() {
        let mut ecu = EcuSimulator::new(0x7E0);

        let response = ecu.handle_uds_request(&[0xFF, 0x00]);
        assert_eq!(response[0], 0x7F); // Negative response
        assert_eq!(response[1], 0xFF); // Service ID
        assert_eq!(response[2], 0x11); // Service not supported
    }

    #[test]
    fn test_encode_dtc_code() {
        let ecu = EcuSimulator::new(0x7E0);

        // Test P0301
        let bytes = ecu.encode_dtc_code("P0301").unwrap();
        assert_eq!(bytes, vec![0x03, 0x01]);

        // Test P0420
        let bytes = ecu.encode_dtc_code("P0420").unwrap();
        assert_eq!(bytes, vec![0x04, 0x20]);

        // Test C0123
        let bytes = ecu.encode_dtc_code("C0123").unwrap();
        assert_eq!(bytes, vec![0x41, 0x23]);
    }
}
