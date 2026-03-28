use crate::error::{nrc_description, UdsError};

/// UDS Service 0x10 - DiagnosticSessionControl
///
/// Controls the diagnostic session type active in the ECU.
/// Different session types enable different diagnostic services.

/// Diagnostic session types (ISO 14229-1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionType {
    /// Default session (0x01) - basic diagnostics
    Default = 0x01,
    /// Programming session (0x02) - ECU flashing
    Programming = 0x02,
    /// Extended diagnostic session (0x03) - advanced diagnostics
    Extended = 0x03,
    /// Safety system session (0x04) - safety-critical operations
    SafetySystem = 0x04,
}

impl SessionType {
    /// Get session type from byte value
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(SessionType::Default),
            0x02 => Some(SessionType::Programming),
            0x03 => Some(SessionType::Extended),
            0x04 => Some(SessionType::SafetySystem),
            _ => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            SessionType::Default => "Default",
            SessionType::Programming => "Programming",
            SessionType::Extended => "Extended Diagnostic",
            SessionType::SafetySystem => "Safety System",
        }
    }
}

/// Build a DiagnosticSessionControl request
pub fn build_request(session_type: SessionType) -> Vec<u8> {
    vec![0x10, session_type as u8]
}

/// Response from DiagnosticSessionControl
#[derive(Debug, Clone)]
pub struct SessionControlResponse {
    /// Active session type
    pub session_type: SessionType,
    /// P2 server timing (ms) - max response time
    pub p2_server_max_ms: u16,
    /// P2* server timing (ms) - extended response time
    pub p2_star_server_max_ms: u16,
}

/// Parse a DiagnosticSessionControl positive response
pub fn parse_response(data: &[u8]) -> Result<SessionControlResponse, UdsError> {
    // Positive response: [0x50, session_type, P2_high, P2_low, P2*_high, P2*_low]
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        // Negative response: [0x7F, service_id, NRC]
        if data.len() >= 3 {
            return Err(UdsError::NegativeResponse {
                service: data[1],
                nrc: data[2],
                description: nrc_description(data[2]).to_string(),
            });
        }
        return Err(UdsError::InvalidResponse("Malformed negative response".into()));
    }

    if data[0] != 0x50 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x50, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 2 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let session_type = SessionType::from_byte(data[1]).ok_or_else(|| {
        UdsError::InvalidResponse(format!("Unknown session type: 0x{:02X}", data[1]))
    })?;

    let p2_server_max_ms = if data.len() >= 4 {
        u16::from_be_bytes([data[2], data[3]])
    } else {
        50 // Default P2 timing
    };

    let p2_star_server_max_ms = if data.len() >= 6 {
        u16::from_be_bytes([data[4], data[5]]) * 10 // P2* is in 10ms units
    } else {
        5000 // Default P2* timing
    };

    Ok(SessionControlResponse {
        session_type,
        p2_server_max_ms,
        p2_star_server_max_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request_default() {
        let req = build_request(SessionType::Default);
        assert_eq!(req, vec![0x10, 0x01]);
    }

    #[test]
    fn test_build_request_extended() {
        let req = build_request(SessionType::Extended);
        assert_eq!(req, vec![0x10, 0x03]);
    }

    #[test]
    fn test_parse_positive_response() {
        let data = vec![0x50, 0x01, 0x00, 0x19, 0x01, 0xF4];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.session_type, SessionType::Default);
        assert_eq!(resp.p2_server_max_ms, 25);
        assert_eq!(resp.p2_star_server_max_ms, 5000);
    }

    #[test]
    fn test_parse_negative_response() {
        let data = vec![0x7F, 0x10, 0x12];
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::NegativeResponse { service: 0x10, nrc: 0x12, .. })));
    }

    #[test]
    fn test_parse_short_positive_response() {
        let data = vec![0x50, 0x03];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.session_type, SessionType::Extended);
        assert_eq!(resp.p2_server_max_ms, 50); // default
    }

    #[test]
    fn test_session_type_from_byte() {
        assert_eq!(SessionType::from_byte(0x01), Some(SessionType::Default));
        assert_eq!(SessionType::from_byte(0x02), Some(SessionType::Programming));
        assert_eq!(SessionType::from_byte(0x03), Some(SessionType::Extended));
        assert_eq!(SessionType::from_byte(0xFF), None);
    }

    #[test]
    fn test_session_type_name() {
        assert_eq!(SessionType::Default.name(), "Default");
        assert_eq!(SessionType::Extended.name(), "Extended Diagnostic");
    }
}
