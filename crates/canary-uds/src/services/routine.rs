use crate::error::{nrc_description, UdsError};

/// UDS Service 0x31 - RoutineControl
///
/// Controls diagnostic routines in the ECU such as actuator tests,
/// self-diagnostics, calibration procedures, and more.

/// Routine control sub-functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutineControlType {
    /// Start a routine (0x01)
    StartRoutine = 0x01,
    /// Stop a routine (0x02)
    StopRoutine = 0x02,
    /// Request routine results (0x03)
    RequestResults = 0x03,
}

impl RoutineControlType {
    /// Get from byte value
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(RoutineControlType::StartRoutine),
            0x02 => Some(RoutineControlType::StopRoutine),
            0x03 => Some(RoutineControlType::RequestResults),
            _ => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            RoutineControlType::StartRoutine => "Start Routine",
            RoutineControlType::StopRoutine => "Stop Routine",
            RoutineControlType::RequestResults => "Request Results",
        }
    }
}

/// Common routine identifiers used in automotive diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonRoutine {
    /// Erase memory (0xFF00)
    EraseMemory = 0xFF00,
    /// Check programming dependencies (0xFF01)
    CheckProgrammingDependencies = 0xFF01,
    /// Fuel pump actuator test (0x0200)
    FuelPumpTest = 0x0200,
    /// Injector test (0x0201)
    InjectorTest = 0x0201,
    /// Fan actuator test (0x0202)
    FanTest = 0x0202,
    /// ABS pump test (0x0300)
    AbsPumpTest = 0x0300,
    /// Headlight test (0x0400)
    HeadlightTest = 0x0400,
    /// Horn test (0x0401)
    HornTest = 0x0401,
    /// Window control test (0x0402)
    WindowTest = 0x0402,
}

impl CommonRoutine {
    /// Get the 2-byte routine identifier
    pub fn value(&self) -> u16 {
        *self as u16
    }
}

/// Build a RoutineControl request
///
/// # Arguments
/// * `control_type` - Start, stop, or request results
/// * `routine_id` - 2-byte routine identifier
/// * `option_record` - Optional parameters for the routine
pub fn build_request(
    control_type: RoutineControlType,
    routine_id: u16,
    option_record: &[u8],
) -> Vec<u8> {
    let mut request = vec![
        0x31,
        control_type as u8,
        (routine_id >> 8) as u8,
        (routine_id & 0xFF) as u8,
    ];
    request.extend_from_slice(option_record);
    request
}

/// Build a request to start a routine
pub fn build_start_routine(routine_id: u16, option_record: &[u8]) -> Vec<u8> {
    build_request(RoutineControlType::StartRoutine, routine_id, option_record)
}

/// Build a request to stop a routine
pub fn build_stop_routine(routine_id: u16) -> Vec<u8> {
    build_request(RoutineControlType::StopRoutine, routine_id, &[])
}

/// Build a request to get routine results
pub fn build_request_results(routine_id: u16) -> Vec<u8> {
    build_request(RoutineControlType::RequestResults, routine_id, &[])
}

/// Routine status in the response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutineStatus {
    /// Routine completed successfully
    Completed,
    /// Routine is currently running
    Running,
    /// Routine failed
    Failed,
    /// Routine was stopped
    Stopped,
    /// Unknown status
    Unknown(u8),
}

/// Response from RoutineControl
#[derive(Debug, Clone)]
pub struct RoutineControlResponse {
    /// Sub-function echo
    pub control_type: RoutineControlType,
    /// Routine identifier
    pub routine_id: u16,
    /// Routine status/result info
    pub routine_info: u8,
    /// Status record (routine-specific result data)
    pub status_record: Vec<u8>,
}

impl RoutineControlResponse {
    /// Interpret the routine info byte as a status
    pub fn status(&self) -> RoutineStatus {
        match self.routine_info {
            0x00 => RoutineStatus::Completed,
            0x01 => RoutineStatus::Running,
            0x02 => RoutineStatus::Failed,
            0x03 => RoutineStatus::Stopped,
            other => RoutineStatus::Unknown(other),
        }
    }
}

/// Parse a RoutineControl positive response
pub fn parse_response(data: &[u8]) -> Result<RoutineControlResponse, UdsError> {
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

    // Positive response SID = 0x71
    if data[0] != 0x71 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x71, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 4 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let control_type = RoutineControlType::from_byte(data[1]).ok_or_else(|| {
        UdsError::InvalidResponse(format!("Unknown control type: 0x{:02X}", data[1]))
    })?;

    let routine_id = u16::from_be_bytes([data[2], data[3]]);

    let routine_info = if data.len() > 4 { data[4] } else { 0x00 };

    let status_record = if data.len() > 5 {
        data[5..].to_vec()
    } else {
        Vec::new()
    };

    Ok(RoutineControlResponse {
        control_type,
        routine_id,
        routine_info,
        status_record,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_start_routine() {
        let req = build_start_routine(0xFF00, &[]);
        assert_eq!(req, vec![0x31, 0x01, 0xFF, 0x00]);
    }

    #[test]
    fn test_build_start_routine_with_params() {
        let req = build_start_routine(0x0200, &[0x01, 0x05]);
        assert_eq!(req, vec![0x31, 0x01, 0x02, 0x00, 0x01, 0x05]);
    }

    #[test]
    fn test_build_stop_routine() {
        let req = build_stop_routine(0x0200);
        assert_eq!(req, vec![0x31, 0x02, 0x02, 0x00]);
    }

    #[test]
    fn test_build_request_results() {
        let req = build_request_results(0xFF00);
        assert_eq!(req, vec![0x31, 0x03, 0xFF, 0x00]);
    }

    #[test]
    fn test_parse_positive_response_completed() {
        let data = vec![0x71, 0x01, 0xFF, 0x00, 0x00];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.control_type, RoutineControlType::StartRoutine);
        assert_eq!(resp.routine_id, 0xFF00);
        assert_eq!(resp.status(), RoutineStatus::Completed);
    }

    #[test]
    fn test_parse_positive_response_running() {
        let data = vec![0x71, 0x01, 0x02, 0x00, 0x01];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.routine_id, 0x0200);
        assert_eq!(resp.status(), RoutineStatus::Running);
    }

    #[test]
    fn test_parse_response_with_status_record() {
        let data = vec![0x71, 0x03, 0xFF, 0x00, 0x00, 0xDE, 0xAD];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.control_type, RoutineControlType::RequestResults);
        assert_eq!(resp.status_record, vec![0xDE, 0xAD]);
    }

    #[test]
    fn test_parse_negative_response() {
        let data = vec![0x7F, 0x31, 0x31]; // Request out of range
        let result = parse_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x31,
                nrc: 0x31,
                ..
            })
        ));
    }

    #[test]
    fn test_parse_response_too_short() {
        let data = vec![0x71, 0x01, 0xFF];
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::InvalidResponse(_))));
    }

    #[test]
    fn test_common_routine_values() {
        assert_eq!(CommonRoutine::EraseMemory.value(), 0xFF00);
        assert_eq!(CommonRoutine::FuelPumpTest.value(), 0x0200);
        assert_eq!(CommonRoutine::HornTest.value(), 0x0401);
    }

    #[test]
    fn test_routine_control_type_from_byte() {
        assert_eq!(
            RoutineControlType::from_byte(0x01),
            Some(RoutineControlType::StartRoutine)
        );
        assert_eq!(
            RoutineControlType::from_byte(0x03),
            Some(RoutineControlType::RequestResults)
        );
        assert_eq!(RoutineControlType::from_byte(0xFF), None);
    }

    #[test]
    fn test_parse_minimal_response() {
        let data = vec![0x71, 0x01, 0x02, 0x00];
        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.routine_info, 0x00);
        assert!(resp.status_record.is_empty());
    }
}
