use crate::error::{nrc_description, UdsError};

/// UDS Service 0x2F - InputOutputControlByIdentifier
///
/// Controls ECU inputs/outputs for actuator testing.
/// Allows overriding sensor values, activating actuators,
/// and returning control to ECU.

/// I/O control parameter values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoControlParameter {
    /// Return control to ECU (0x00)
    ReturnControlToEcu = 0x00,
    /// Reset to default (0x01)
    ResetToDefault = 0x01,
    /// Freeze current state (0x02)
    FreezeCurrentState = 0x02,
    /// Short-term adjustment (0x03)
    ShortTermAdjustment = 0x03,
}

impl IoControlParameter {
    /// Get parameter from byte value
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(IoControlParameter::ReturnControlToEcu),
            0x01 => Some(IoControlParameter::ResetToDefault),
            0x02 => Some(IoControlParameter::FreezeCurrentState),
            0x03 => Some(IoControlParameter::ShortTermAdjustment),
            _ => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            IoControlParameter::ReturnControlToEcu => "Return Control to ECU",
            IoControlParameter::ResetToDefault => "Reset to Default",
            IoControlParameter::FreezeCurrentState => "Freeze Current State",
            IoControlParameter::ShortTermAdjustment => "Short Term Adjustment",
        }
    }
}

/// Build an InputOutputControl request
///
/// # Arguments
/// * `did` - Data Identifier for the I/O control
/// * `control_param` - Control action to perform
/// * `control_state` - Optional control state record (for ShortTermAdjustment)
pub fn build_request(
    did: u16,
    control_param: IoControlParameter,
    control_state: &[u8],
) -> Vec<u8> {
    let mut request = vec![
        0x2F,
        (did >> 8) as u8,
        (did & 0xFF) as u8,
        control_param as u8,
    ];
    request.extend_from_slice(control_state);
    request
}

/// Build a request to return control to ECU
pub fn build_return_control(did: u16) -> Vec<u8> {
    build_request(did, IoControlParameter::ReturnControlToEcu, &[])
}

/// Build a request to freeze the current state
pub fn build_freeze_current(did: u16) -> Vec<u8> {
    build_request(did, IoControlParameter::FreezeCurrentState, &[])
}

/// Build a short-term adjustment request
pub fn build_short_term_adjustment(did: u16, control_state: &[u8]) -> Vec<u8> {
    build_request(did, IoControlParameter::ShortTermAdjustment, control_state)
}

/// Response from InputOutputControl
#[derive(Debug, Clone)]
pub struct IoControlResponse {
    /// Data Identifier
    pub did: u16,
    /// Control parameter echo
    pub control_param: IoControlParameter,
    /// Status record (current state after control)
    pub status_record: Vec<u8>,
}

/// Parse an InputOutputControl positive response
pub fn parse_response(data: &[u8]) -> Result<IoControlResponse, UdsError> {
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

    // Positive response SID = 0x6F
    if data[0] != 0x6F {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x6F, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 4 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let did = u16::from_be_bytes([data[1], data[2]]);
    let control_param = IoControlParameter::from_byte(data[3]).ok_or_else(|| {
        UdsError::InvalidResponse(format!("Unknown control parameter: 0x{:02X}", data[3]))
    })?;
    let status_record = data[4..].to_vec();

    Ok(IoControlResponse {
        did,
        control_param,
        status_record,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_return_control() {
        let req = build_return_control(0xF200);
        assert_eq!(req, vec![0x2F, 0xF2, 0x00, 0x00]);
    }

    #[test]
    fn test_build_freeze_current() {
        let req = build_freeze_current(0xF200);
        assert_eq!(req, vec![0x2F, 0xF2, 0x00, 0x02]);
    }

    #[test]
    fn test_build_short_term_adjustment() {
        let req = build_short_term_adjustment(0xF200, &[0x01, 0xFF]);
        assert_eq!(req, vec![0x2F, 0xF2, 0x00, 0x03, 0x01, 0xFF]);
    }

    #[test]
    fn test_parse_positive_response() {
        let data = vec![0x6F, 0xF2, 0x00, 0x00, 0x01];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.did, 0xF200);
        assert_eq!(resp.control_param, IoControlParameter::ReturnControlToEcu);
        assert_eq!(resp.status_record, vec![0x01]);
    }

    #[test]
    fn test_parse_short_term_adjustment_response() {
        let data = vec![0x6F, 0xF2, 0x00, 0x03, 0x01, 0xFF];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.control_param, IoControlParameter::ShortTermAdjustment);
        assert_eq!(resp.status_record, vec![0x01, 0xFF]);
    }

    #[test]
    fn test_parse_negative_response() {
        let data = vec![0x7F, 0x2F, 0x31]; // Request out of range
        let result = parse_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x2F,
                nrc: 0x31,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_response_too_short() {
        let data = vec![0x6F, 0xF2, 0x00];
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }

    #[test]
    fn test_io_control_parameter_from_byte() {
        assert_eq!(
            IoControlParameter::from_byte(0x00),
            Some(IoControlParameter::ReturnControlToEcu)
        );
        assert_eq!(
            IoControlParameter::from_byte(0x03),
            Some(IoControlParameter::ShortTermAdjustment)
        );
        assert_eq!(IoControlParameter::from_byte(0xFF), None);
    }

    #[test]
    fn test_io_control_parameter_name() {
        assert_eq!(
            IoControlParameter::ReturnControlToEcu.name(),
            "Return Control to ECU"
        );
        assert_eq!(
            IoControlParameter::ShortTermAdjustment.name(),
            "Short Term Adjustment"
        );
    }

    #[test]
    fn test_parse_empty_response() {
        let result = parse_response(&[]);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }
}
