use crate::keygen::{Manufacturer, SecurityAlgorithm, SecurityError, SecurityLevel};

/// General Motors seed/key algorithm
///
/// Implements the GM security access algorithm used across
/// various GM platforms (Global A/B architecture).
///
/// GM typically uses a 2-byte seed with a lookup-table based
/// algorithm combined with CRC-like operations.
///
/// # Security Levels
/// - 0x01: Service mode diagnostics
/// - 0x03: Enhanced diagnostics
/// - 0x0B: Programming (SPS)
pub struct GmAlgorithm {
    /// Secret constants per access level
    secret_keys: Vec<(u8, u16)>,
}

impl Default for GmAlgorithm {
    fn default() -> Self {
        Self {
            secret_keys: vec![
                (0x01, 0x8421), // Service mode
                (0x03, 0xC6A5), // Enhanced diagnostics
                (0x0B, 0xD294), // Programming (SPS)
            ],
        }
    }
}

impl GmAlgorithm {
    /// Create with default GM constants
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom secret for a specific level
    pub fn with_secret(mut self, level: u8, secret: u16) -> Self {
        if let Some(entry) = self.secret_keys.iter_mut().find(|(l, _)| *l == level) {
            entry.1 = secret;
        } else {
            self.secret_keys.push((level, secret));
        }
        self
    }

    /// Get the secret constant for a given level
    fn get_secret(&self, level: u8) -> Option<u16> {
        let normalized = if level % 2 == 0 { level - 1 } else { level };
        self.secret_keys
            .iter()
            .find(|(l, _)| *l == normalized)
            .map(|(_, s)| *s)
    }

    /// GM CRC-like lookup table (simplified)
    const CRC_TABLE: [u8; 16] = [
        0x03, 0x0E, 0x05, 0x0A, 0x09, 0x06, 0x0F, 0x04,
        0x07, 0x0C, 0x01, 0x08, 0x0B, 0x02, 0x0D, 0x00,
    ];

    /// Core GM algorithm
    ///
    /// Uses a combination of XOR, table lookup, and bit shifting
    /// to compute the key from seed and secret.
    fn compute_gm_key(&self, seed: u16, secret: u16) -> u16 {
        let mut key = seed ^ secret;

        // 4 rounds of transformation
        for _ in 0..4 {
            let high_nibble = ((key >> 12) & 0x0F) as usize;
            let table_val = Self::CRC_TABLE[high_nibble] as u16;
            key = (key << 4) ^ (table_val << 8) ^ (table_val);
        }

        // Final XOR with rotated secret
        key ^= secret.rotate_left(5);

        key
    }
}

impl SecurityAlgorithm for GmAlgorithm {
    fn compute_key(&self, seed: &[u8], level: u8) -> Result<Vec<u8>, SecurityError> {
        if seed.len() != 2 {
            return Err(SecurityError::InvalidSeedLength {
                expected: 2,
                got: seed.len(),
            });
        }

        let secret = self.get_secret(level).ok_or_else(|| {
            SecurityError::UnsupportedLevel(level)
        })?;

        let seed_u16 = u16::from_be_bytes([seed[0], seed[1]]);

        // Handle zero seed (already unlocked)
        if seed_u16 == 0 {
            return Ok(vec![0x00, 0x00]);
        }

        let key = self.compute_gm_key(seed_u16, secret);
        Ok(key.to_be_bytes().to_vec())
    }

    fn manufacturer(&self) -> Manufacturer {
        Manufacturer::Gm
    }

    fn supported_levels(&self) -> Vec<SecurityLevel> {
        self.secret_keys
            .iter()
            .map(|(l, _)| SecurityLevel(*l))
            .collect()
    }

    fn expected_seed_length(&self) -> usize {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gm_algorithm_basic_level() {
        let algo = GmAlgorithm::new();
        let seed = vec![0xAB, 0xCD];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key.len(), 2);
    }

    #[test]
    fn test_gm_algorithm_enhanced_level() {
        let algo = GmAlgorithm::new();
        let seed = vec![0x12, 0x34];
        let key = algo.compute_key(&seed, 0x03).unwrap();
        assert_eq!(key.len(), 2);
    }

    #[test]
    fn test_gm_algorithm_programming_level() {
        let algo = GmAlgorithm::new();
        let seed = vec![0xFF, 0xEE];
        let key = algo.compute_key(&seed, 0x0B).unwrap();
        assert_eq!(key.len(), 2);
    }

    #[test]
    fn test_gm_algorithm_zero_seed() {
        let algo = GmAlgorithm::new();
        let seed = vec![0x00, 0x00];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key, vec![0x00, 0x00]);
    }

    #[test]
    fn test_gm_algorithm_wrong_seed_length() {
        let algo = GmAlgorithm::new();
        let result = algo.compute_key(&[0x12, 0x34, 0x56, 0x78], 0x01);
        assert!(matches!(
            result,
            Err(SecurityError::InvalidSeedLength {
                expected: 2,
                got: 4
            })
        ));
    }

    #[test]
    fn test_gm_algorithm_unsupported_level() {
        let algo = GmAlgorithm::new();
        let result = algo.compute_key(&[0x12, 0x34], 0x55);
        assert!(matches!(result, Err(SecurityError::UnsupportedLevel(0x55))));
    }

    #[test]
    fn test_gm_algorithm_deterministic() {
        let algo = GmAlgorithm::new();
        let seed = vec![0xAB, 0xCD];
        let key1 = algo.compute_key(&seed, 0x01).unwrap();
        let key2 = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_gm_algorithm_different_seeds() {
        let algo = GmAlgorithm::new();
        let key1 = algo.compute_key(&[0x12, 0x34], 0x01).unwrap();
        let key2 = algo.compute_key(&[0x56, 0x78], 0x01).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_gm_algorithm_different_levels() {
        let algo = GmAlgorithm::new();
        let seed = vec![0x12, 0x34];
        let key1 = algo.compute_key(&seed, 0x01).unwrap();
        let key3 = algo.compute_key(&seed, 0x03).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_gm_algorithm_custom_secret() {
        let algo = GmAlgorithm::new().with_secret(0x01, 0xBEEF);
        let seed = vec![0x12, 0x34];
        let key = algo.compute_key(&seed, 0x01).unwrap();

        let default_algo = GmAlgorithm::new();
        let default_key = default_algo.compute_key(&seed, 0x01).unwrap();
        assert_ne!(key, default_key);
    }

    #[test]
    fn test_gm_algorithm_metadata() {
        let algo = GmAlgorithm::new();
        assert_eq!(algo.manufacturer(), Manufacturer::Gm);
        assert_eq!(algo.expected_seed_length(), 2);
        assert_eq!(algo.supported_levels().len(), 3);
    }
}
