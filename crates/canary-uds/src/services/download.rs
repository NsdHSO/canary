use crate::error::{nrc_description, UdsError};

/// UDS Services for memory transfer:
/// - 0x34: RequestDownload (ECU receives data from tester)
/// - 0x35: RequestUpload (ECU sends data to tester)
/// - 0x36: TransferData
/// - 0x37: RequestTransferExit

/// Data compression/encryption method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFormatIdentifier {
    /// Compression method (high nibble)
    pub compression: u8,
    /// Encryption method (low nibble)
    pub encryption: u8,
}

impl DataFormatIdentifier {
    /// No compression or encryption
    pub fn none() -> Self {
        Self {
            compression: 0,
            encryption: 0,
        }
    }

    /// Create from raw byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            compression: (byte >> 4) & 0x0F,
            encryption: byte & 0x0F,
        }
    }

    /// Convert to raw byte
    pub fn to_byte(&self) -> u8 {
        ((self.compression & 0x0F) << 4) | (self.encryption & 0x0F)
    }
}

/// Address and length format identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressAndLengthFormat {
    /// Number of bytes for memory size (high nibble)
    pub memory_size_length: u8,
    /// Number of bytes for memory address (low nibble)
    pub memory_address_length: u8,
}

impl AddressAndLengthFormat {
    /// Standard 4-byte address, 4-byte size
    pub fn standard() -> Self {
        Self {
            memory_size_length: 4,
            memory_address_length: 4,
        }
    }

    /// Create from raw byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            memory_size_length: (byte >> 4) & 0x0F,
            memory_address_length: byte & 0x0F,
        }
    }

    /// Convert to raw byte
    pub fn to_byte(&self) -> u8 {
        ((self.memory_size_length & 0x0F) << 4) | (self.memory_address_length & 0x0F)
    }
}

// ===== Service 0x34: RequestDownload =====

/// Build a RequestDownload request (tester -> ECU)
///
/// # Arguments
/// * `data_format` - Compression/encryption method
/// * `memory_address` - Starting memory address
/// * `memory_size` - Number of bytes to download
pub fn build_request_download(
    data_format: DataFormatIdentifier,
    memory_address: u32,
    memory_size: u32,
) -> Vec<u8> {
    let addr_len_format = AddressAndLengthFormat::standard();
    let mut request = vec![0x34, data_format.to_byte(), addr_len_format.to_byte()];
    request.extend_from_slice(&memory_address.to_be_bytes());
    request.extend_from_slice(&memory_size.to_be_bytes());
    request
}

/// Response from RequestDownload
#[derive(Debug, Clone)]
pub struct RequestDownloadResponse {
    /// Maximum number of bytes per TransferData block (excluding SID and counter)
    pub max_block_length: u32,
}

/// Parse a RequestDownload positive response
pub fn parse_request_download_response(data: &[u8]) -> Result<RequestDownloadResponse, UdsError> {
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        return parse_negative_response(data);
    }

    if data[0] != 0x74 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x74, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 2 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let length_format = (data[1] >> 4) & 0x0F;
    let num_bytes = length_format as usize;

    if data.len() < 2 + num_bytes {
        return Err(UdsError::InvalidResponse(
            "Max block length data too short".into(),
        ));
    }

    let mut max_block_length: u32 = 0;
    for i in 0..num_bytes {
        max_block_length = (max_block_length << 8) | data[2 + i] as u32;
    }

    Ok(RequestDownloadResponse { max_block_length })
}

// ===== Service 0x35: RequestUpload =====

/// Build a RequestUpload request (ECU -> tester)
///
/// # Arguments
/// * `data_format` - Compression/encryption method
/// * `memory_address` - Starting memory address
/// * `memory_size` - Number of bytes to upload
pub fn build_request_upload(
    data_format: DataFormatIdentifier,
    memory_address: u32,
    memory_size: u32,
) -> Vec<u8> {
    let addr_len_format = AddressAndLengthFormat::standard();
    let mut request = vec![0x35, data_format.to_byte(), addr_len_format.to_byte()];
    request.extend_from_slice(&memory_address.to_be_bytes());
    request.extend_from_slice(&memory_size.to_be_bytes());
    request
}

/// Response from RequestUpload
#[derive(Debug, Clone)]
pub struct RequestUploadResponse {
    /// Maximum number of bytes per TransferData block
    pub max_block_length: u32,
}

/// Parse a RequestUpload positive response
pub fn parse_request_upload_response(data: &[u8]) -> Result<RequestUploadResponse, UdsError> {
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        return Err(parse_negative_response_err(data));
    }

    if data[0] != 0x75 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x75, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 2 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    let length_format = (data[1] >> 4) & 0x0F;
    let num_bytes = length_format as usize;

    if data.len() < 2 + num_bytes {
        return Err(UdsError::InvalidResponse(
            "Max block length data too short".into(),
        ));
    }

    let mut max_block_length: u32 = 0;
    for i in 0..num_bytes {
        max_block_length = (max_block_length << 8) | data[2 + i] as u32;
    }

    Ok(RequestUploadResponse { max_block_length })
}

// ===== Service 0x36: TransferData =====

/// Build a TransferData request
///
/// # Arguments
/// * `block_sequence_counter` - Block sequence counter (1-255, wraps)
/// * `transfer_data` - Data payload for this block
pub fn build_transfer_data(block_sequence_counter: u8, transfer_data: &[u8]) -> Vec<u8> {
    let mut request = vec![0x36, block_sequence_counter];
    request.extend_from_slice(transfer_data);
    request
}

/// Response from TransferData
#[derive(Debug, Clone)]
pub struct TransferDataResponse {
    /// Block sequence counter echo
    pub block_sequence_counter: u8,
    /// Transfer response parameter record (optional data from ECU during upload)
    pub data: Vec<u8>,
}

/// Parse a TransferData positive response
pub fn parse_transfer_data_response(data: &[u8]) -> Result<TransferDataResponse, UdsError> {
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        return Err(parse_negative_response_err(data));
    }

    if data[0] != 0x76 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x76, got 0x{:02X}",
            data[0]
        )));
    }

    if data.len() < 2 {
        return Err(UdsError::InvalidResponse("Response too short".into()));
    }

    Ok(TransferDataResponse {
        block_sequence_counter: data[1],
        data: data[2..].to_vec(),
    })
}

// ===== Service 0x37: RequestTransferExit =====

/// Build a RequestTransferExit request
pub fn build_transfer_exit(parameter_record: &[u8]) -> Vec<u8> {
    let mut request = vec![0x37];
    request.extend_from_slice(parameter_record);
    request
}

/// Response from RequestTransferExit
#[derive(Debug, Clone)]
pub struct TransferExitResponse {
    /// Optional parameter record from ECU
    pub parameter_record: Vec<u8>,
}

/// Parse a RequestTransferExit positive response
pub fn parse_transfer_exit_response(data: &[u8]) -> Result<TransferExitResponse, UdsError> {
    if data.is_empty() {
        return Err(UdsError::InvalidResponse("Empty response".into()));
    }

    if data[0] == 0x7F {
        return Err(parse_negative_response_err(data));
    }

    if data[0] != 0x77 {
        return Err(UdsError::InvalidResponse(format!(
            "Expected 0x77, got 0x{:02X}",
            data[0]
        )));
    }

    Ok(TransferExitResponse {
        parameter_record: data[1..].to_vec(),
    })
}

// ===== High-level transfer operations =====

/// Transfer direction for high-level operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    /// Download: tester sends data to ECU (0x34)
    Download,
    /// Upload: ECU sends data to tester (0x35)
    Upload,
}

/// Configuration for a memory transfer operation
#[derive(Debug, Clone)]
pub struct TransferConfig {
    /// Transfer direction
    pub direction: TransferDirection,
    /// Memory address
    pub memory_address: u32,
    /// Memory size
    pub memory_size: u32,
    /// Data format
    pub data_format: DataFormatIdentifier,
}

impl TransferConfig {
    /// Create a download config (tester -> ECU)
    pub fn download(memory_address: u32, memory_size: u32) -> Self {
        Self {
            direction: TransferDirection::Download,
            memory_address,
            memory_size,
            data_format: DataFormatIdentifier::none(),
        }
    }

    /// Create an upload config (ECU -> tester)
    pub fn upload(memory_address: u32, memory_size: u32) -> Self {
        Self {
            direction: TransferDirection::Upload,
            memory_address,
            memory_size,
            data_format: DataFormatIdentifier::none(),
        }
    }
}

/// Helper for negative response parsing
fn parse_negative_response<T>(data: &[u8]) -> Result<T, UdsError> {
    Err(parse_negative_response_err(data))
}

fn parse_negative_response_err(data: &[u8]) -> UdsError {
    if data.len() >= 3 {
        UdsError::NegativeResponse {
            service: data[1],
            nrc: data[2],
            description: nrc_description(data[2]).to_string(),
        }
    } else {
        UdsError::InvalidResponse("Malformed negative response".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === DataFormatIdentifier tests ===

    #[test]
    fn test_data_format_none() {
        let fmt = DataFormatIdentifier::none();
        assert_eq!(fmt.to_byte(), 0x00);
    }

    #[test]
    fn test_data_format_roundtrip() {
        let fmt = DataFormatIdentifier {
            compression: 0x02,
            encryption: 0x01,
        };
        let byte = fmt.to_byte();
        assert_eq!(byte, 0x21);
        let recovered = DataFormatIdentifier::from_byte(byte);
        assert_eq!(recovered.compression, 0x02);
        assert_eq!(recovered.encryption, 0x01);
    }

    // === AddressAndLengthFormat tests ===

    #[test]
    fn test_addr_len_format_standard() {
        let fmt = AddressAndLengthFormat::standard();
        assert_eq!(fmt.to_byte(), 0x44);
    }

    #[test]
    fn test_addr_len_format_roundtrip() {
        let fmt = AddressAndLengthFormat {
            memory_size_length: 2,
            memory_address_length: 3,
        };
        let byte = fmt.to_byte();
        assert_eq!(byte, 0x23);
        let recovered = AddressAndLengthFormat::from_byte(byte);
        assert_eq!(recovered.memory_size_length, 2);
        assert_eq!(recovered.memory_address_length, 3);
    }

    // === RequestDownload (0x34) tests ===

    #[test]
    fn test_build_request_download() {
        let req = build_request_download(
            DataFormatIdentifier::none(),
            0x00010000,
            0x00001000,
        );
        assert_eq!(req[0], 0x34);
        assert_eq!(req[1], 0x00); // No compression/encryption
        assert_eq!(req[2], 0x44); // 4-byte address, 4-byte size
        // Address: 0x00010000
        assert_eq!(&req[3..7], &[0x00, 0x01, 0x00, 0x00]);
        // Size: 0x00001000
        assert_eq!(&req[7..11], &[0x00, 0x00, 0x10, 0x00]);
    }

    #[test]
    fn test_parse_request_download_response() {
        // Max block length = 0x0FFA (2 bytes), length format = 0x20
        let data = vec![0x74, 0x20, 0x0F, 0xFA];
        let resp = parse_request_download_response(&data).unwrap();
        assert_eq!(resp.max_block_length, 0x0FFA);
    }

    #[test]
    fn test_parse_request_download_negative() {
        let data = vec![0x7F, 0x34, 0x70]; // Upload/download not accepted
        let result = parse_request_download_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x34,
                nrc: 0x70,
                ..
            })
        ));
    }

    // === RequestUpload (0x35) tests ===

    #[test]
    fn test_build_request_upload() {
        let req = build_request_upload(
            DataFormatIdentifier::none(),
            0x00020000,
            0x00000800,
        );
        assert_eq!(req[0], 0x35);
        assert_eq!(req[1], 0x00);
        assert_eq!(req[2], 0x44);
    }

    #[test]
    fn test_parse_request_upload_response() {
        let data = vec![0x75, 0x20, 0x0F, 0xFA];
        let resp = parse_request_upload_response(&data).unwrap();
        assert_eq!(resp.max_block_length, 0x0FFA);
    }

    // === TransferData (0x36) tests ===

    #[test]
    fn test_build_transfer_data() {
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let req = build_transfer_data(0x01, &payload);
        assert_eq!(req, vec![0x36, 0x01, 0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_parse_transfer_data_response_download() {
        // Download: ECU just echoes the block counter
        let data = vec![0x76, 0x01];
        let resp = parse_transfer_data_response(&data).unwrap();
        assert_eq!(resp.block_sequence_counter, 0x01);
        assert!(resp.data.is_empty());
    }

    #[test]
    fn test_parse_transfer_data_response_upload() {
        // Upload: ECU sends data back
        let data = vec![0x76, 0x01, 0xCA, 0xFE, 0xBA, 0xBE];
        let resp = parse_transfer_data_response(&data).unwrap();
        assert_eq!(resp.block_sequence_counter, 0x01);
        assert_eq!(resp.data, vec![0xCA, 0xFE, 0xBA, 0xBE]);
    }

    #[test]
    fn test_parse_transfer_data_negative() {
        let data = vec![0x7F, 0x36, 0x73]; // Wrong block sequence counter
        let result = parse_transfer_data_response(&data);
        assert!(matches!(
            result,
            Err(UdsError::NegativeResponse {
                service: 0x36,
                nrc: 0x73,
                ..
            })
        ));
    }

    // === TransferExit (0x37) tests ===

    #[test]
    fn test_build_transfer_exit() {
        let req = build_transfer_exit(&[]);
        assert_eq!(req, vec![0x37]);
    }

    #[test]
    fn test_build_transfer_exit_with_params() {
        let req = build_transfer_exit(&[0x01, 0x02]);
        assert_eq!(req, vec![0x37, 0x01, 0x02]);
    }

    #[test]
    fn test_parse_transfer_exit_response() {
        let data = vec![0x77];
        let resp = parse_transfer_exit_response(&data).unwrap();
        assert!(resp.parameter_record.is_empty());
    }

    #[test]
    fn test_parse_transfer_exit_response_with_params() {
        let data = vec![0x77, 0xAB, 0xCD];
        let resp = parse_transfer_exit_response(&data).unwrap();
        assert_eq!(resp.parameter_record, vec![0xAB, 0xCD]);
    }

    // === TransferConfig tests ===

    #[test]
    fn test_transfer_config_download() {
        let config = TransferConfig::download(0x10000, 0x1000);
        assert_eq!(config.direction, TransferDirection::Download);
        assert_eq!(config.memory_address, 0x10000);
        assert_eq!(config.memory_size, 0x1000);
    }

    #[test]
    fn test_transfer_config_upload() {
        let config = TransferConfig::upload(0x20000, 0x800);
        assert_eq!(config.direction, TransferDirection::Upload);
    }
}
