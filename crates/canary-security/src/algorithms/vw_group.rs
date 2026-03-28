use crate::keygen::{Manufacturer, SecurityAlgorithm, SecurityError, SecurityLevel};

/// VW Group (VW, Audi, Skoda, SEAT) seed/key algorithm
///
/// Implements the common VW Group security access algorithm used
/// across the MQB and MLB platforms. The algorithm uses a combination
/// of XOR operations with manufacturer-specific constants and
/// bit rotation to derive the key from a seed.
///
/// # Security Levels
/// - 0x01: Basic diagnostics access
/// - 0x03: Extended diagnostics (adaptation channels)
/// - 0x11: Programming session (ECU flashing)
/// - 0x27: Development access
pub struct VwGroupAlgorithm {
    /// Secret constant per access level
    secret_keys: Vec<(u8, u32)>,
}

impl Default for VwGroupAlgorithm {
    fn default() -> Self {
        Self {
            secret_keys: vec![
                (0x01, 0x0A221289), // Basic diagnostics
                (0x03, 0x3E50C281), // Extended diagnostics
                (0x11, 0x57A6C801), // Programming
                (0x27, 0x891A2B3C), // Development
            ],
        }
    }
}

impl VwGroupAlgorithm {
    /// Create with default VW Group constants
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom secret key for a specific level
    pub fn with_secret(mut self, level: u8, secret: u32) -> Self {
        if let Some(entry) = self.secret_keys.iter_mut().find(|(l, _)| *l == level) {
            entry.1 = secret;
        } else {
            self.secret_keys.push((level, secret));
        }
        self
    }

    /// Get the secret constant for a given access level
    fn get_secret(&self, level: u8) -> Option<u32> {
        // Normalize to odd (seed request) level
        let normalized = if level % 2 == 0 { level - 1 } else { level };
        self.secret_keys
            .iter()
            .find(|(l, _)| *l == normalized)
            .map(|(_, s)| *s)
    }

    /// Core VW Group algorithm
    ///
    /// The algorithm performs multiple rounds of XOR and bit manipulation:
    /// 1. XOR seed with secret constant
    /// 2. Rotate bits based on seed bytes
    /// 3. Apply final transformation
    fn compute_vw_key(&self, seed: u32, secret: u32) -> u32 {
        let mut key = seed ^ secret;

        // Round 1: Bit rotation based on seed nibbles
        for i in 0..4 {
            let shift = ((seed >> (i * 8)) & 0x07) + 1;
            key = key.rotate_left(shift);
        }

        // Round 2: XOR with rotated secret
        key ^= secret.rotate_right(13);

        // Round 3: Nibble swap and final XOR
        let swapped = ((key & 0x0F0F0F0F) << 4) | ((key & 0xF0F0F0F0) >> 4);
        key = swapped ^ (secret & 0x55AA55AA);

        key
    }
}

impl SecurityAlgorithm for VwGroupAlgorithm {
    fn compute_key(&self, seed: &[u8], level: u8) -> Result<Vec<u8>, SecurityError> {
        if seed.len() != 4 {
            return Err(SecurityError::InvalidSeedLength {
                expected: 4,
                got: seed.len(),
            });
        }

        let secret = self.get_secret(level).ok_or_else(|| {
            SecurityError::UnsupportedLevel(level)
        })?;

        let seed_u32 = u32::from_be_bytes([seed[0], seed[1], seed[2], seed[3]]);

        // Handle zero seed (ECU already unlocked)
        if seed_u32 == 0 {
            return Ok(vec![0x00, 0x00, 0x00, 0x00]);
        }

        let key = self.compute_vw_key(seed_u32, secret);
        Ok(key.to_be_bytes().to_vec())
    }

    fn manufacturer(&self) -> Manufacturer {
        Manufacturer::VwGroup
    }

    fn supported_levels(&self) -> Vec<SecurityLevel> {
        self.secret_keys
            .iter()
            .map(|(l, _)| SecurityLevel(*l))
            .collect()
    }

    fn expected_seed_length(&self) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vw_algorithm_basic_level() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0x12, 0x34, 0x56, 0x78];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key.len(), 4);
        // Key should be deterministic
        let key2 = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key, key2);
    }

    #[test]
    fn test_vw_algorithm_extended_level() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0xAB, 0xCD, 0xEF, 0x01];
        let key = algo.compute_key(&seed, 0x03).unwrap();
        assert_eq!(key.len(), 4);
    }

    #[test]
    fn test_vw_algorithm_programming_level() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0xFF, 0xEE, 0xDD, 0xCC];
        let key = algo.compute_key(&seed, 0x11).unwrap();
        assert_eq!(key.len(), 4);
    }

    #[test]
    fn test_vw_algorithm_zero_seed() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0x00, 0x00, 0x00, 0x00];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key, vec![0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_vw_algorithm_wrong_seed_length() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0x12, 0x34];
        let result = algo.compute_key(&seed, 0x01);
        assert!(matches!(
            result,
            Err(SecurityError::InvalidSeedLength {
                expected: 4,
                got: 2
            })
        ));
    }

    #[test]
    fn test_vw_algorithm_unsupported_level() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0x12, 0x34, 0x56, 0x78];
        let result = algo.compute_key(&seed, 0x55);
        assert!(matches!(result, Err(SecurityError::UnsupportedLevel(0x55))));
    }

    #[test]
    fn test_vw_algorithm_different_seeds_different_keys() {
        let algo = VwGroupAlgorithm::new();
        let key1 = algo.compute_key(&[0x12, 0x34, 0x56, 0x78], 0x01).unwrap();
        let key2 = algo.compute_key(&[0x9A, 0xBC, 0xDE, 0xF0], 0x01).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_vw_algorithm_different_levels_different_keys() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0x12, 0x34, 0x56, 0x78];
        let key1 = algo.compute_key(&seed, 0x01).unwrap();
        let key3 = algo.compute_key(&seed, 0x03).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_vw_algorithm_with_custom_secret() {
        let algo = VwGroupAlgorithm::new().with_secret(0x01, 0xDEADBEEF);
        let seed = vec![0x12, 0x34, 0x56, 0x78];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key.len(), 4);

        // Should differ from default
        let default_algo = VwGroupAlgorithm::new();
        let default_key = default_algo.compute_key(&seed, 0x01).unwrap();
        assert_ne!(key, default_key);
    }

    #[test]
    fn test_vw_algorithm_supported_levels() {
        let algo = VwGroupAlgorithm::new();
        let levels = algo.supported_levels();
        assert_eq!(levels.len(), 4);
    }

    #[test]
    fn test_vw_algorithm_seed_length() {
        let algo = VwGroupAlgorithm::new();
        assert_eq!(algo.expected_seed_length(), 4);
    }

    #[test]
    fn test_vw_algorithm_manufacturer() {
        let algo = VwGroupAlgorithm::new();
        assert_eq!(algo.manufacturer(), Manufacturer::VwGroup);
    }

    #[test]
    fn test_vw_algorithm_even_level_normalizes() {
        let algo = VwGroupAlgorithm::new();
        let seed = vec![0x12, 0x34, 0x56, 0x78];
        // Level 0x02 should normalize to 0x01
        let key1 = algo.compute_key(&seed, 0x01).unwrap();
        let key2 = algo.compute_key(&seed, 0x02).unwrap();
        assert_eq!(key1, key2);
    }
}
