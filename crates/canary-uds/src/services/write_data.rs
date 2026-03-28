use crate::error::{nrc_description, UdsError};

/// UDS Service 0x2E - WriteDataByIdentifier
///
/// Writes data to an ECU by a 2-byte Data Identifier (DID).
/// Typically requires an extended diagnostic session and
/// security access before writing.

/// Build a WriteDataByIdentifier request
///
/// # Arguments
/// * `did` - 2-byte Data Identifier
/// * `data` - Data to write
pub fn build_request(did: u16, data: &[u8]) -> Vec<u8> {
    let mut request = vec![0x2E, (did >> 8) as u8, (did & 0xFF) as u8];
    request.extend_from_slice(data);
    request
}

/// Response from WriteDataByIdentifier
#[derive(Debug, Clone)]
pub struct WriteDataResponse {
    /// Data Identifier that was written
    pub did: u16,
}

/// Parse a WriteDataByIdentifier positive response
pub fn parse_response(data: &[u8]) -> Result<WriteDataResponse, UdsError> {
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        if data.len() >= 3 {
            return Err(UdsError::NegativeResponse {
                service: data[1],
                nrc: data[2],
                description: nrc_description(data[2]).to_string(),
            });
        }
        return Err(UdsError::InvalidResponse("Malformed negative response".into()));
    }

    // Positive response SID = 0x6E
    if data[0] != 0x6E {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x6E, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 3 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let did = u16::from_be_bytes([data[1], data[2]]);

    Ok(WriteDataResponse { did })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request() {
        let req = build_request(0xF190, &[0x41, 0x42, 0x43]);
        assert_eq!(req, vec![0x2E, 0xF1, 0x90, 0x41, 0x42, 0x43]);
    }

    #[test]
    fn test_build_request_empty_data() {
        let req = build_request(0x0100, &[]);
        assert_eq!(req, vec![0x2E, 0x01, 0x00]);
    }

    #[test]
    fn test_parse_positive_response() {
        let data = vec![0x6E, 0xF1, 0x90];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.did, 0xF190);
    }

    #[test]
    fn test_parse_negative_response_security_denied() {
        let data = vec![0x7F, 0x2E, 0x33]; // Security access denied
        let result = parse_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x2E,
                nrc: 0x33,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_negative_response_conditions() {
        let data = vec![0x7F, 0x2E, 0x22]; // Conditions not correct
        let result = parse_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x2E,
                nrc: 0x22,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_empty_response() {
        let result = parse_response(&[]);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }

    #[test]
    fn test_parse_wrong_service_id() {
        let data = vec![0x62, 0xF1, 0x90]; // ReadData response SID
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }

    #[test]
    fn test_parse_response_too_short() {
        let data = vec![0x6E, 0xF1];
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }
}
