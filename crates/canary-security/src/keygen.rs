use thiserror::Error;

/// Security access errors
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Invalid seed length
    #[error("Invalid seed length: expected {expected}, got {got}")]
    InvalidSeedLength { expected: usize, got: usize },

    /// Unsupported security level
    #[error("Unsupported security level: 0x{0:02X}")]
    UnsupportedLevel(u8),

    /// Algorithm computation failed
    #[error("Algorithm error: {0}")]
    AlgorithmError(String),

    /// Manufacturer not supported
    #[error("Unsupported manufacturer: {0}")]
    UnsupportedManufacturer(String),
}

/// Supported manufacturers for security access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Manufacturer {
    /// Volkswagen Group (VW, Audi, Skoda, SEAT)
    VwGroup,
    /// General Motors (Chevrolet, Cadillac, GMC, Buick)
    Gm,
    /// Ford Motor Company (Ford, Lincoln, Mercury)
    Ford,
}

impl Manufacturer {
    /// Get manufacturer display name
    pub fn name(&self) -> &'static str {
        match self {
            Manufacturer::VwGroup => "VW Group",
            Manufacturer::Gm => "General Motors",
            Manufacturer::Ford => "Ford",
        }
    }

    /// List all supported manufacturers
    pub fn all() -> Vec<Manufacturer> {
        vec![Manufacturer::VwGroup, Manufacturer::Gm, Manufacturer::Ford]
    }
}

/// Security access level (UDS Service 0x27)
///
/// Odd levels are seed requests, even levels are key responses.
/// Level 0x01/0x02 = basic, 0x03/0x04 = extended, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityLevel(pub u8);

impl SecurityLevel {
    /// Basic security level (0x01)
    pub fn basic() -> Self {
        Self(0x01)
    }

    /// Extended security level (0x03)
    pub fn extended() -> Self {
        Self(0x03)
    }

    /// Programming security level (0x11)
    pub fn programming() -> Self {
        Self(0x11)
    }

    /// Get the seed request sub-function value
    pub fn seed_request(&self) -> u8 {
        // Seed requests use odd levels
        if self.0 % 2 == 0 {
            self.0 - 1
        } else {
            self.0
        }
    }

    /// Get the key response sub-function value
    pub fn key_response(&self) -> u8 {
        // Key responses use even levels
        if self.0 % 2 == 0 {
            self.0
        } else {
            self.0 + 1
        }
    }
}

/// Trait for manufacturer-specific seed/key algorithms
///
/// Each manufacturer implements this trait with their proprietary
/// algorithm for computing the security key from a given seed.
pub trait SecurityAlgorithm: Send + Sync {
    /// Compute the security key from a seed
    ///
    /// # Arguments
    /// * `seed` - The seed bytes received from the ECU
    /// * `level` - The security access level
    ///
    /// # Returns
    /// The computed key bytes to send back to the ECU
    fn compute_key(&self, seed: &[u8], level: u8) -> Result<Vec<u8>, SecurityError>;

    /// Get the manufacturer this algorithm is for
    fn manufacturer(&self) -> Manufacturer;

    /// Get supported security levels
    fn supported_levels(&self) -> Vec<SecurityLevel>;

    /// Get expected seed length for this algorithm
    fn expected_seed_length(&self) -> usize;
}

/// Key generator that wraps manufacturer-specific algorithms
///
/// Provides a unified interface for seed/key operations
/// regardless of the underlying manufacturer algorithm.
pub struct KeyGenerator {
    algorithm: Box<dyn SecurityAlgorithm>,
}

impl KeyGenerator {
    /// Create a key generator for a specific manufacturer
    pub fn for_manufacturer(manufacturer: Manufacturer) -> Self {
        let algorithm: Box<dyn SecurityAlgorithm> = match manufacturer {
            Manufacturer::VwGroup => {
                Box::new(crate::algorithms::vw_group::VwGroupAlgorithm::default())
            }
            Manufacturer::Gm => Box::new(crate::algorithms::gm::GmAlgorithm::default()),
            Manufacturer::Ford => Box::new(crate::algorithms::ford::FordAlgorithm::default()),
        };
        Self { algorithm }
    }

    /// Create a key generator with a custom algorithm
    pub fn with_algorithm(algorithm: Box<dyn SecurityAlgorithm>) -> Self {
        Self { algorithm }
    }

    /// Compute the security key from a seed
    pub fn compute_key(&self, seed: &[u8], level: u8) -> Result<Vec<u8>, SecurityError> {
        self.algorithm.compute_key(seed, level)
    }

    /// Get the manufacturer
    pub fn manufacturer(&self) -> Manufacturer {
        self.algorithm.manufacturer()
    }

    /// Get supported security levels
    pub fn supported_levels(&self) -> Vec<SecurityLevel> {
        self.algorithm.supported_levels()
    }

    /// Get expected seed length
    pub fn expected_seed_length(&self) -> usize {
        self.algorithm.expected_seed_length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manufacturer_names() {
        assert_eq!(Manufacturer::VwGroup.name(), "VW Group");
        assert_eq!(Manufacturer::Gm.name(), "General Motors");
        assert_eq!(Manufacturer::Ford.name(), "Ford");
    }

    #[test]
    fn test_manufacturer_all() {
        let all = Manufacturer::all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_security_level_basic() {
        let level = SecurityLevel::basic();
        assert_eq!(level.seed_request(), 0x01);
        assert_eq!(level.key_response(), 0x02);
    }

    #[test]
    fn test_security_level_extended() {
        let level = SecurityLevel::extended();
        assert_eq!(level.seed_request(), 0x03);
        assert_eq!(level.key_response(), 0x04);
    }

    #[test]
    fn test_security_level_even_input() {
        let level = SecurityLevel(0x04);
        assert_eq!(level.seed_request(), 0x03);
        assert_eq!(level.key_response(), 0x04);
    }

    #[test]
    fn test_keygen_for_all_manufacturers() {
        for mfr in Manufacturer::all() {
            let keygen = KeyGenerator::for_manufacturer(mfr);
            assert_eq!(keygen.manufacturer(), mfr);
            assert!(!keygen.supported_levels().is_empty());
            assert!(keygen.expected_seed_length() > 0);
        }
    }

    #[test]
    fn test_keygen_vw_compute() {
        let keygen = KeyGenerator::for_manufacturer(Manufacturer::VwGroup);
        let seed = vec![0x12, 0x34, 0x56, 0x78];
        let result = keygen.compute_key(&seed, 0x01);
        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key.len(), 4);
    }

    #[test]
    fn test_keygen_gm_compute() {
        let keygen = KeyGenerator::for_manufacturer(Manufacturer::Gm);
        let seed = vec![0xAB, 0xCD];
        let result = keygen.compute_key(&seed, 0x01);
        assert!(result.is_ok());
    }

    #[test]
    fn test_keygen_ford_compute() {
        let keygen = KeyGenerator::for_manufacturer(Manufacturer::Ford);
        let seed = vec![0x11, 0x22, 0x33];
        let result = keygen.compute_key(&seed, 0x01);
        assert!(result.is_ok());
    }
}
