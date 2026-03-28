use crate::error::{nrc_description, UdsError};

/// UDS Service 0x22 - ReadDataByIdentifier
///
/// Reads data from an ECU by a 2-byte Data Identifier (DID).
/// Common DIDs include VIN, calibration ID, ECU serial number,
/// and live sensor data.

/// Well-known Data Identifiers (DIDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommonDid {
    /// Vehicle Identification Number
    Vin = 0xF190,
    /// ECU Serial Number
    EcuSerialNumber = 0xF18C,
    /// ECU Hardware Version
    EcuHardwareVersion = 0xF191,
    /// ECU Software Version
    EcuSoftwareVersion = 0xF195,
    /// System Supplier ECU Software Number
    SupplierSoftwareNumber = 0xF188,
    /// Calibration ID
    CalibrationId = 0xF806,
    /// Boot Software Identification
    BootSoftwareId = 0xF180,
    /// Active Diagnostic Session
    ActiveDiagSession = 0xF186,
    /// ECU Manufacturing Date
    ManufacturingDate = 0xF18B,
}

impl CommonDid {
    /// Get the 2-byte DID value
    pub fn value(&self) -> u16 {
        *self as u16
    }
}

/// Build a ReadDataByIdentifier request for a single DID
pub fn build_request(did: u16) -> Vec<u8> {
    vec![0x22, (did >> 8) as u8, (did & 0xFF) as u8]
}

/// Build a ReadDataByIdentifier request for multiple DIDs
pub fn build_multi_request(dids: &[u16]) -> Vec<u8> {
    let mut request = vec![0x22];
    for did in dids {
        request.push((did >> 8) as u8);
        request.push((did & 0xFF) as u8);
    }
    request
}

/// Response from ReadDataByIdentifier
#[derive(Debug, Clone)]
pub struct ReadDataResponse {
    /// Data Identifier
    pub did: u16,
    /// Raw data bytes
    pub data: Vec<u8>,
}

impl ReadDataResponse {
    /// Try to interpret the data as a UTF-8 string (e.g., VIN)
    pub fn as_string(&self) -> Option<String> {
        String::from_utf8(self.data.clone())
            .ok()
            .map(|s| s.trim_end_matches('\0').to_string())
    }

    /// Get the data as a hex string
    pub fn as_hex_string(&self) -> String {
        self.data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Try to interpret as a u16 value
    pub fn as_u16(&self) -> Option<u16> {
        if self.data.len() >= 2 {
            Some(u16::from_be_bytes([self.data[0], self.data[1]]))
        } else {
            None
        }
    }

    /// Try to interpret as a u32 value
    pub fn as_u32(&self) -> Option<u32> {
        if self.data.len() >= 4 {
            Some(u32::from_be_bytes([
                self.data[0],
                self.data[1],
                self.data[2],
                self.data[3],
            ]))
        } else {
            None
        }
    }
}

/// Parse a ReadDataByIdentifier positive response
pub fn parse_response(data: &[u8]) -> Result<ReadDataResponse, UdsError> {
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

    if data[0] != 0x62 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x62, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 3 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let did = u16::from_be_bytes([data[1], data[2]]);
    let payload = data[3..].to_vec();

    Ok(ReadDataResponse {
        did,
        data: payload,
    })
}

/// Parse response for VIN specifically
pub fn parse_vin_response(data: &[u8]) -> Result<String, UdsError> {
    let response = parse_response(data)?;
    response
        .as_string()
        .ok_or_else(|| UdsError::InvalidResponse("VIN is not valid UTF-8".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_single_request() {
        let req = build_request(0xF190);
        assert_eq!(req, vec![0x22, 0xF1, 0x90]);
    }

    #[test]
    fn test_build_multi_request() {
        let req = build_multi_request(&[0xF190, 0xF18C]);
        assert_eq!(req, vec![0x22, 0xF1, 0x90, 0xF1, 0x8C]);
    }

    #[test]
    fn test_parse_positive_response() {
        // VIN response
        let vin = b"WVWZZZ3CZWE123456";
        let mut data = vec![0x62, 0xF1, 0x90];
        data.extend_from_slice(vin);

        let resp = parse_response(&data).unwrap();
        assert_eq!(resp.did, 0xF190);
        assert_eq!(resp.as_string().unwrap(), "WVWZZZ3CZWE123456");
    }

    #[test]
    fn test_parse_negative_response() {
        let data = vec![0x7F, 0x22, 0x31]; // Request out of range
        let result = parse_response(&data);
        assert!(matches!(result, Err(UdsError::NegativeResponse { service: 0x22, nrc: 0x31, .. })));
    }

    #[test]
    fn test_parse_vin_response() {
        let vin = b"WVWZZZ3CZWE123456";
        let mut data = vec![0x62, 0xF1, 0x90];
        data.extend_from_slice(vin);

        let vin_str = parse_vin_response(&data).unwrap();
        assert_eq!(vin_str, "WVWZZZ3CZWE123456");
    }

    #[test]
    fn test_read_data_response_as_hex() {
        let resp = ReadDataResponse {
            did: 0xF190,
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };
        assert_eq!(resp.as_hex_string(), "DE AD BE EF");
    }

    #[test]
    fn test_read_data_response_as_u16() {
        let resp = ReadDataResponse {
            did: 0x0100,
            data: vec![0x0C, 0x80], // RPM = 3200
        };
        assert_eq!(resp.as_u16(), Some(0x0C80));
    }

    #[test]
    fn test_common_did_values() {
        assert_eq!(CommonDid::Vin.value(), 0xF190);
        assert_eq!(CommonDid::EcuSerialNumber.value(), 0xF18C);
        assert_eq!(CommonDid::EcuSoftwareVersion.value(), 0xF195);
    }
}
