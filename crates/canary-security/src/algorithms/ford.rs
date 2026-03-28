use crate::keygen::{Manufacturer, SecurityAlgorithm, SecurityError, SecurityLevel};

/// Ford Motor Company seed/key algorithm
///
/// Implements the Ford security access algorithm used across
/// various Ford platforms. Ford uses a 3-byte seed with a
/// polynomial-based algorithm.
///
/// # Security Levels
/// - 0x01: Standard diagnostics
/// - 0x03: Module programming
/// - 0x61: As-Built data access
pub struct FordAlgorithm {
    /// Secret constants per access level
    secret_keys: Vec<(u8, u32)>,
}

impl Default for FordAlgorithm {
    fn default() -> Self {
        Self {
            secret_keys: vec![
                (0x01, 0x00C541A9), // Standard diagnostics
                (0x03, 0x00E8B714), // Module programming
                (0x61, 0x005A3C96), // As-Built data
            ],
        }
    }
}

impl FordAlgorithm {
    /// Create with default Ford constants
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom secret for a specific level
    pub fn with_secret(mut self, level: u8, secret: u32) -> Self {
        if let Some(entry) = self.secret_keys.iter_mut().find(|(l, _)| *l == level) {
            entry.1 = secret;
        } else {
            self.secret_keys.push((level, secret));
        }
        self
    }

    /// Get the secret constant for a given level
    fn get_secret(&self, level: u8) -> Option<u32> {
        let normalized = if level % 2 == 0 { level - 1 } else { level };
        self.secret_keys
            .iter()
            .find(|(l, _)| *l == normalized)
            .map(|(_, s)| *s)
    }

    /// Core Ford algorithm
    ///
    /// Ford uses a polynomial-based computation with bit manipulation:
    /// 1. Combine seed bytes with secret
    /// 2. Apply polynomial feedback
    /// 3. Extract 3-byte key
    fn compute_ford_key(&self, seed: &[u8; 3], secret: u32) -> [u8; 3] {
        let seed_val = ((seed[0] as u32) << 16)
            | ((seed[1] as u32) << 8)
            | (seed[2] as u32);

        let mut key = seed_val ^ (secret & 0x00FFFFFF);

        // 8 rounds of polynomial feedback
        for round in 0..8u32 {
            let feedback = if (key & 0x800000) != 0 {
                0x5B6C7D // Ford polynomial
            } else {
                0x000000
            };

            key = ((key << 1) & 0x00FFFFFF) ^ feedback;
            key ^= (secret >> (round % 4 * 8)) & 0xFF;
        }

        // Final transformation
        let b0 = ((key >> 16) & 0xFF) as u8;
        let b1 = ((key >> 8) & 0xFF) as u8;
        let b2 = (key & 0xFF) as u8;

        // Byte substitution
        [
            b0 ^ (secret >> 16) as u8,
            b1 ^ (secret >> 8) as u8,
            b2 ^ secret as u8,
        ]
    }
}

impl SecurityAlgorithm for FordAlgorithm {
    fn compute_key(&self, seed: &[u8], level: u8) -> Result<Vec<u8>, SecurityError> {
        if seed.len() != 3 {
            return Err(SecurityError::InvalidSeedLength {
                expected: 3,
                got: seed.len(),
            });
        }

        let secret = self.get_secret(level).ok_or_else(|| {
            SecurityError::UnsupportedLevel(level)
        })?;

        // Handle zero seed (already unlocked)
        if seed.iter().all(|&b| b == 0) {
            return Ok(vec![0x00, 0x00, 0x00]);
        }

        let seed_arr: [u8; 3] = [seed[0], seed[1], seed[2]];
        let key = self.compute_ford_key(&seed_arr, secret);
        Ok(key.to_vec())
    }

    fn manufacturer(&self) -> Manufacturer {
        Manufacturer::Ford
    }

    fn supported_levels(&self) -> Vec<SecurityLevel> {
        self.secret_keys
            .iter()
            .map(|(l, _)| SecurityLevel(*l))
            .collect()
    }

    fn expected_seed_length(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ford_algorithm_standard_level() {
        let algo = FordAlgorithm::new();
        let seed = vec![0x11, 0x22, 0x33];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key.len(), 3);
    }

    #[test]
    fn test_ford_algorithm_programming_level() {
        let algo = FordAlgorithm::new();
        let seed = vec![0xAA, 0xBB, 0xCC];
        let key = algo.compute_key(&seed, 0x03).unwrap();
        assert_eq!(key.len(), 3);
    }

    #[test]
    fn test_ford_algorithm_asbuilt_level() {
        let algo = FordAlgorithm::new();
        let seed = vec![0xDE, 0xAD, 0xBE];
        let key = algo.compute_key(&seed, 0x61).unwrap();
        assert_eq!(key.len(), 3);
    }

    #[test]
    fn test_ford_algorithm_zero_seed() {
        let algo = FordAlgorithm::new();
        let seed = vec![0x00, 0x00, 0x00];
        let key = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key, vec![0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_ford_algorithm_wrong_seed_length_short() {
        let algo = FordAlgorithm::new();
        let result = algo.compute_key(&[0x12, 0x34], 0x01);
        assert!(matches!(
            result,
            Err(SecurityError::InvalidSeedLength {
                expected: 3,
                got: 2
            })
        ));
    }

    #[test]
    fn test_ford_algorithm_wrong_seed_length_long() {
        let algo = FordAlgorithm::new();
        let result = algo.compute_key(&[0x12, 0x34, 0x56, 0x78], 0x01);
        assert!(matches!(
            result,
            Err(SecurityError::InvalidSeedLength {
                expected: 3,
                got: 4
            })
        ));
    }

    #[test]
    fn test_ford_algorithm_unsupported_level() {
        let algo = FordAlgorithm::new();
        let result = algo.compute_key(&[0x12, 0x34, 0x56], 0x55);
        assert!(matches!(result, Err(SecurityError::UnsupportedLevel(0x55))));
    }

    #[test]
    fn test_ford_algorithm_deterministic() {
        let algo = FordAlgorithm::new();
        let seed = vec![0x11, 0x22, 0x33];
        let key1 = algo.compute_key(&seed, 0x01).unwrap();
        let key2 = algo.compute_key(&seed, 0x01).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_ford_algorithm_different_seeds() {
        let algo = FordAlgorithm::new();
        let key1 = algo.compute_key(&[0x11, 0x22, 0x33], 0x01).unwrap();
        let key2 = algo.compute_key(&[0x44, 0x55, 0x66], 0x01).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_ford_algorithm_different_levels() {
        let algo = FordAlgorithm::new();
        let seed = vec![0x11, 0x22, 0x33];
        let key1 = algo.compute_key(&seed, 0x01).unwrap();
        let key3 = algo.compute_key(&seed, 0x03).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_ford_algorithm_custom_secret() {
        let algo = FordAlgorithm::new().with_secret(0x01, 0x00FACADE);
        let seed = vec![0x11, 0x22, 0x33];
        let key = algo.compute_key(&seed, 0x01).unwrap();

        let default_algo = FordAlgorithm::new();
        let default_key = default_algo.compute_key(&seed, 0x01).unwrap();
        assert_ne!(key, default_key);
    }

    #[test]
    fn test_ford_algorithm_metadata() {
        let algo = FordAlgorithm::new();
        assert_eq!(algo.manufacturer(), Manufacturer::Ford);
        assert_eq!(algo.expected_seed_length(), 3);
        assert_eq!(algo.supported_levels().len(), 3);
    }
}
