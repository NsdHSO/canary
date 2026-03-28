use flate2::read::GzDecoder;
use std::io::Read;

/// Decompress gzip data
pub fn decompress_gzip(compressed: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}

/// Compress data with gzip
#[cfg(test)]
pub fn compress_gzip(data: &str) -> Result<Vec<u8>, std::io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data.as_bytes())?;
    encoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = "This is test data for compression.";
        let compressed = compress_gzip(original).unwrap();
        let decompressed = decompress_gzip(&compressed).unwrap();

        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_compression_reduces_size() {
        let original = "A".repeat(1000); // Highly compressible
        let compressed = compress_gzip(&original).unwrap();

        // Compression should reduce size significantly
        assert!(compressed.len() < original.len() / 5);
    }

    #[test]
    fn test_decompress_empty() {
        let compressed = compress_gzip("").unwrap();
        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(decompressed, "");
    }
}
