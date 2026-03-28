use crate::error::{nrc_description, UdsError};

/// UDS Service 0x19 - ReadDTCInformation
///
/// Reads Diagnostic Trouble Codes (DTCs) from an ECU.
/// Supports multiple sub-functions for different DTC queries.

/// DTC sub-function types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcSubFunction {
    /// Report number of DTCs by status mask
    ReportNumberByStatusMask = 0x01,
    /// Report DTCs by status mask
    ReportByStatusMask = 0x02,
    /// Report DTC snapshot identification
    ReportSnapshotIdentification = 0x03,
    /// Report DTC snapshot record by DTC number
    ReportSnapshotByDtcNumber = 0x04,
    /// Report DTC extended data record by DTC number
    ReportExtendedDataByDtcNumber = 0x06,
    /// Report supported DTCs
    ReportSupportedDtcs = 0x0A,
}

/// DTC status bits (ISO 14229-1)
#[derive(Debug, Clone, Copy)]
pub struct DtcStatus {
    /// Raw status byte
    pub raw: u8,
}

impl DtcStatus {
    pub fn new(raw: u8) -> Self {
        Self { raw }
    }

    /// Test Failed - DTC is currently active
    pub fn test_failed(&self) -> bool {
        (self.raw & 0x01) != 0
    }

    /// Test Failed This Operation Cycle
    pub fn test_failed_this_cycle(&self) -> bool {
        (self.raw & 0x02) != 0
    }

    /// Pending DTC
    pub fn pending(&self) -> bool {
        (self.raw & 0x04) != 0
    }

    /// Confirmed DTC
    pub fn confirmed(&self) -> bool {
        (self.raw & 0x08) != 0
    }

    /// Test Not Completed Since Last Clear
    pub fn not_completed_since_clear(&self) -> bool {
        (self.raw & 0x10) != 0
    }

    /// Test Failed Since Last Clear
    pub fn test_failed_since_clear(&self) -> bool {
        (self.raw & 0x20) != 0
    }

    /// Test Not Completed This Operation Cycle
    pub fn not_completed_this_cycle(&self) -> bool {
        (self.raw & 0x40) != 0
    }

    /// Warning Indicator Requested (MIL/Check Engine Light)
    pub fn warning_indicator(&self) -> bool {
        (self.raw & 0x80) != 0
    }

    /// Human-readable status description
    pub fn description(&self) -> String {
        let mut parts = Vec::new();
        if self.test_failed() {
            parts.push("Active");
        }
        if self.confirmed() {
            parts.push("Confirmed");
        }
        if self.pending() {
            parts.push("Pending");
        }
        if self.warning_indicator() {
            parts.push("MIL On");
        }
        if parts.is_empty() {
            "Stored".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// A single DTC entry from the ECU
#[derive(Debug, Clone)]
pub struct DtcEntry {
    /// DTC number (3 bytes: high, mid, low)
    pub dtc_high: u8,
    pub dtc_mid: u8,
    pub dtc_low: u8,
    /// DTC status
    pub status: DtcStatus,
}

impl DtcEntry {
    /// Get the DTC as a u32 for comparison
    pub fn dtc_number(&self) -> u32 {
        ((self.dtc_high as u32) << 16) | ((self.dtc_mid as u32) << 8) | (self.dtc_low as u32)
    }

    /// Convert to standard DTC code string (e.g., "P0301")
    pub fn to_code_string(&self) -> String {
        let first_nibble = (self.dtc_high >> 6) & 0x03;
        let prefix = match first_nibble {
            0 => 'P', // Powertrain
            1 => 'C', // Chassis
            2 => 'B', // Body
            3 => 'U', // Network
            _ => '?',
        };

        let second_nibble = (self.dtc_high >> 4) & 0x03;
        let third_nibble = self.dtc_high & 0x0F;

        format!(
            "{}{}{:01X}{:02X}",
            prefix, second_nibble, third_nibble, self.dtc_mid
        )
    }
}

/// Build a ReadDTCInformation request
pub fn build_request(sub_function: DtcSubFunction, status_mask: u8) -> Vec<u8> {
    vec![0x19, sub_function as u8, status_mask]
}

/// Build a request to report all confirmed DTCs
pub fn build_report_all_dtcs() -> Vec<u8> {
    build_request(DtcSubFunction::ReportByStatusMask, 0xFF)
}

/// Build a request to report only active DTCs
pub fn build_report_active_dtcs() -> Vec<u8> {
    build_request(DtcSubFunction::ReportByStatusMask, 0x09) // Active + Confirmed
}

/// Response from ReadDTCInformation
#[derive(Debug, Clone)]
pub struct ReadDtcResponse {
    /// Sub-function echo
    pub sub_function: u8,
    /// Status availability mask
    pub status_availability_mask: u8,
    /// List of DTCs
    pub dtcs: Vec<DtcEntry>,
}

/// Parse a ReadDTCInformation response
pub fn parse_response(data: &[u8]) -> Result<ReadDtcResponse, UdsError> {
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

    if data[0] != 0x59 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x59, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 3 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let sub_function = data[1];
    let status_availability_mask = data[2];

    // Parse DTCs (each is 4 bytes: DTC_high, DTC_mid, DTC_low, status)
    let mut dtcs = Vec::new();
    let dtc_data = &data[3..];

    for chunk in dtc_data.chunks(4) {
        if chunk.len() == 4 {
            dtcs.push(DtcEntry {
                dtc_high: chunk[0],
                dtc_mid: chunk[1],
                dtc_low: chunk[2],
                status: DtcStatus::new(chunk[3]),
            });
        }
    }

    Ok(ReadDtcResponse {
        sub_function,
        status_availability_mask,
        dtcs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_report_all_dtcs() {
        let req = build_report_all_dtcs();
        assert_eq!(req, vec![0x19, 0x02, 0xFF]);
    }

    #[test]
    fn test_build_report_active_dtcs() {
        let req = build_report_active_dtcs();
        assert_eq!(req, vec![0x19, 0x02, 0x09]);
    }

    #[test]
    fn test_parse_response_with_dtcs() {
        // Positive response with 2 DTCs
        let data = vec![
            0x59, 0x02, 0xFF, // Header
            0x00, 0x03, 0x01, 0x09, // P0301 - Active + Confirmed
            0x01, 0x04, 0x20, 0x08, // P0420 - Confirmed only
        ];

        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.dtcs.len(), 2);
        assert!(resp.dtcs[0].status.test_failed());
        assert!(resp.dtcs[0].status.confirmed());
        assert!(!resp.dtcs[1].status.test_failed());
        assert!(resp.dtcs[1].status.confirmed());
    }

    #[test]
    fn test_parse_empty_dtc_response() {
        let data = vec![0x59, 0x02, 0xFF];
        let resp = parse_response(&data).unwrap();
        assert!(resp.dtcs.is_empty());
    }

    #[test]
    fn test_parse_negative_response() {
        let data = vec![0x7F, 0x19, 0x11];
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::NegativeResponse { service: 0x19, .. })));
    }

    #[test]
    fn test_dtc_status_bits() {
        let status = DtcStatus::new(0x89); // Active + Confirmed + MIL
        assert!(status.test_failed());
        assert!(status.confirmed());
        assert!(status.warning_indicator());
        assert!(!status.pending());
    }

    #[test]
    fn test_dtc_status_description() {
        let active = DtcStatus::new(0x09);
        assert!(active.description().contains("Active"));
        assert!(active.description().contains("Confirmed"));

        let stored = DtcStatus::new(0x00);
        assert_eq!(stored.description(), "Stored");
    }

    #[test]
    fn test_dtc_entry_to_code_string() {
        // P0301 = DTC bytes: 0x00, 0x03, 0x01
        let entry = DtcEntry {
            dtc_high: 0x03,
            dtc_mid: 0x01,
            dtc_low: 0x00,
            status: DtcStatus::new(0x09),
        };
        let code = entry.to_code_string();
        assert_eq!(code, "P0301");
    }
}
