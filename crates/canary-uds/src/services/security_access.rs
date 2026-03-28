use crate::error::{nrc_description, UdsError};

/// UDS Service 0x27 - SecurityAccess
///
/// Manages the security handshake with an ECU:
/// 1. Request seed (odd sub-function)
/// 2. Send key (even sub-function)
///
/// Security access is required before using services like
/// WriteDataByIdentifier (0x2E), RoutineControl (0x31),
/// or RequestDownload (0x34).

/// Build a SecurityAccess seed request
///
/// # Arguments
/// * `access_level` - Security access level (odd number: 0x01, 0x03, etc.)
pub fn build_seed_request(access_level: u8) -> Vec<u8> {
    // Ensure odd sub-function for seed request
    let level = if access_level % 2 == 0 {
        access_level - 1
    } else {
        access_level
    };
    vec![0x27, level]
}

/// Build a SecurityAccess key response
///
/// # Arguments
/// * `access_level` - Security access level (even number: 0x02, 0x04, etc.)
/// * `key` - The computed security key
pub fn build_key_response(access_level: u8, key: &[u8]) -> Vec<u8> {
    // Ensure even sub-function for key response
    let level = if access_level % 2 == 0 {
        access_level
    } else {
        access_level + 1
    };
    let mut request = vec![0x27, level];
    request.extend_from_slice(key);
    request
}

/// Response from SecurityAccess seed request
#[derive(Debug, Clone)]
pub struct SeedResponse {
    /// Security access level
    pub access_level: u8,
    /// Seed bytes from ECU
    pub seed: Vec<u8>,
}

impl SeedResponse {
    /// Check if the ECU is already unlocked (zero seed)
    pub fn is_already_unlocked(&self) -> bool {
        self.seed.iter().all(|&b| b == 0)
    }
}

/// Response from SecurityAccess key submission
#[derive(Debug, Clone)]
pub struct KeyResponse {
    /// Security access level
    pub access_level: u8,
}

/// Parse a SecurityAccess seed response
pub fn parse_seed_response(data: &[u8]) -> Result<SeedResponse, UdsError> {
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

    // Positive response SID = 0x67
    if data[0] != 0x67 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x67, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 3 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let access_level = data[1];
    let seed = data[2..].to_vec();

    Ok(SeedResponse {
        access_level,
        seed,
    })
}

/// Parse a SecurityAccess key response (positive = unlocked)
pub fn parse_key_response(data: &[u8]) -> Result<KeyResponse, UdsError> {
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        if data.len() >= 3 {
            let nrc = data[2];
            let desc = match nrc {
                0x35 => "Invalid key",
                0x36 => "Exceeded number of attempts",
                0x37 => "Required time delay not expired",
                _ => nrc_description(nrc),
            };
            return Err(UdsError::NegativeResponse {
                service: data[1],
                nrc,
                description: desc.to_string(),
            });
        }
        return Err(UdsError::InvalidResponse("Malformed negative response".into()));
    }

    if data[0] != 0x67 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x67, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 2 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    Ok(KeyResponse {
        access_level: data[1],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_seed_request_odd() {
        let req = build_seed_request(0x01);
        assert_eq!(req, vec![0x27, 0x01]);
    }

    #[test]
    fn test_build_seed_request_even_normalizes() {
        let req = build_seed_request(0x02);
        assert_eq!(req, vec![0x27, 0x01]);
    }

    #[test]
    fn test_build_key_response_even() {
        let req = build_key_response(0x02, &[0xAB, 0xCD, 0xEF, 0x01]);
        assert_eq!(req, vec![0x27, 0x02, 0xAB, 0xCD, 0xEF, 0x01]);
    }

    #[test]
    fn test_build_key_response_odd_normalizes() {
        let req = build_key_response(0x01, &[0xAB, 0xCD]);
        assert_eq!(req, vec![0x27, 0x02, 0xAB, 0xCD]);
    }

    #[test]
    fn test_parse_seed_response() {
        let data = vec![0x67, 0x01, 0x12, 0x34, 0x56, 0x78];
        let resp = parse_seed_response(&data).unwrap();
        assert_eq!(resp.access_level, 0x01);
        assert_eq!(resp.seed, vec![0x12, 0x34, 0x56, 0x78]);
        assert!(!resp.is_already_unlocked());
    }

    #[test]
    fn test_parse_seed_response_already_unlocked() {
        let data = vec![0x67, 0x01, 0x00, 0x00, 0x00, 0x00];
        let resp = parse_seed_response(&data).unwrap();
        assert!(resp.is_already_unlocked());
    }

    #[test]
    fn test_parse_key_response_success() {
        let data = vec![0x67, 0x02];
        let resp = parse_key_response(&data).unwrap();
        assert_eq!(resp.access_level, 0x02);
    }

    #[test]
    fn test_parse_seed_negative_response() {
        let data = vec![0x7F, 0x27, 0x12]; // Sub-function not supported
        let result = parse_seed_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x27,
                nrc: 0x12,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_key_invalid_key() {
        let data = vec![0x7F, 0x27, 0x35]; // Invalid key
        let result = parse_key_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x27,
                nrc: 0x35,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_key_exceeded_attempts() {
        let data = vec![0x7F, 0x27, 0x36]; // Exceeded attempts
        let result = parse_key_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x27,
                nrc: 0x36,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_key_time_delay() {
        let data = vec![0x7F, 0x27, 0x37]; // Time delay required
        let result = parse_key_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x27,
                nrc: 0x37,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_seed_empty() {
        let result = parse_seed_response(&[]);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }

    #[test]
    fn test_parse_seed_wrong_sid() {
        let data = vec![0x62, 0x01, 0x12, 0x34];
        let result = parse_seed_response(&data);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }
}
