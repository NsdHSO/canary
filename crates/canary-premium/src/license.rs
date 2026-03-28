//! License management with tiered feature gating.
//!
//! Supports Free, Premium ($99/yr), Professional ($299/yr), and Enterprise ($999/yr) tiers.
//! Uses hardware fingerprint binding, expiration validation, and 14-day trial.

use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;

use crate::error::{PremiumError, Result};

type HmacSha256 = Hmac<Sha256>;

/// License tier defining available features and pricing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LicenseTier {
    /// Free tier: 50 ECUs, no security access
    Free = 0,
    /// Trial: all Premium features for 14 days
    Trial = 1,
    /// Premium ($99/year): unlimited ECUs, security access
    Premium = 2,
    /// Professional ($299/year): fleet management, API access
    Professional = 3,
    /// Enterprise ($999/year): white-label, on-premise deployment
    Enterprise = 4,
}

impl std::fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseTier::Free => write!(f, "Free"),
            LicenseTier::Trial => write!(f, "Trial"),
            LicenseTier::Premium => write!(f, "Premium"),
            LicenseTier::Professional => write!(f, "Professional"),
            LicenseTier::Enterprise => write!(f, "Enterprise"),
        }
    }
}

/// Features that can be gated by license tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    /// Basic ECU diagnostics (Free: limited to 50 ECUs)
    BasicDiagnostics,
    /// Unlimited ECU access
    UnlimitedEcus,
    /// Security access algorithms (seed/key)
    SecurityAccess,
    /// Cloud sync across devices
    CloudSync,
    /// Data marketplace access
    Marketplace,
    /// REST API access
    ApiAccess,
    /// WebSocket live streaming
    LiveStreaming,
    /// Fleet management
    FleetManagement,
    /// White-label branding
    WhiteLabel,
    /// On-premise deployment
    OnPremise,
    /// Priority support
    PrioritySupport,
    /// Multi-ECU monitoring (5+ simultaneous)
    MultiEcuMonitoring,
    /// CAN capture and replay
    CanCapture,
    /// Custom report generation
    CustomReports,
}

impl Feature {
    /// Minimum tier required for this feature
    pub fn required_tier(&self) -> LicenseTier {
        match self {
            Feature::BasicDiagnostics => LicenseTier::Free,
            Feature::CanCapture => LicenseTier::Free,

            Feature::UnlimitedEcus => LicenseTier::Premium,
            Feature::SecurityAccess => LicenseTier::Premium,
            Feature::CloudSync => LicenseTier::Premium,
            Feature::Marketplace => LicenseTier::Premium,
            Feature::MultiEcuMonitoring => LicenseTier::Premium,

            Feature::ApiAccess => LicenseTier::Professional,
            Feature::LiveStreaming => LicenseTier::Professional,
            Feature::FleetManagement => LicenseTier::Professional,
            Feature::CustomReports => LicenseTier::Professional,
            Feature::PrioritySupport => LicenseTier::Professional,

            Feature::WhiteLabel => LicenseTier::Enterprise,
            Feature::OnPremise => LicenseTier::Enterprise,
        }
    }
}

/// A validated license instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// Unique license identifier
    pub id: String,
    /// License tier
    pub tier: LicenseTier,
    /// License holder email
    pub email: String,
    /// When the license was issued
    pub issued_at: DateTime<Utc>,
    /// When the license expires
    pub expires_at: DateTime<Utc>,
    /// Hardware fingerprint this license is bound to
    pub hardware_fingerprint: String,
    /// Maximum number of ECUs (0 = unlimited)
    pub max_ecus: u32,
    /// HMAC signature for tamper detection
    pub signature: String,
}

impl License {
    /// Create a new trial license valid for 14 days
    pub fn new_trial(email: &str, signing_key: &[u8]) -> Result<Self> {
        let now = Utc::now();
        let fingerprint = Self::generate_hardware_fingerprint();
        let mut license = Self {
            id: uuid::Uuid::new_v4().to_string(),
            tier: LicenseTier::Trial,
            email: email.to_string(),
            issued_at: now,
            expires_at: now + Duration::days(14),
            hardware_fingerprint: fingerprint,
            max_ecus: 0, // unlimited during trial
            signature: String::new(),
        };
        license.signature = license.compute_signature(signing_key)?;
        Ok(license)
    }

    /// Validate the license: check expiration, hardware, and signature
    pub fn validate(&self, signing_key: &[u8]) -> Result<()> {
        // Check expiration
        if self.is_expired() {
            if self.tier == LicenseTier::Trial {
                return Err(PremiumError::TrialExpired { days: 14 });
            }
            return Err(PremiumError::LicenseExpired(
                self.expires_at.to_rfc3339(),
            ));
        }

        // Check hardware fingerprint
        let current_fingerprint = Self::generate_hardware_fingerprint();
        if self.hardware_fingerprint != current_fingerprint {
            return Err(PremiumError::HardwareFingerprint {
                expected: self.hardware_fingerprint.clone(),
                actual: current_fingerprint,
            });
        }

        // Verify signature (tamper detection)
        let expected_signature = self.compute_signature(signing_key)?;
        if self.signature != expected_signature {
            return Err(PremiumError::InvalidLicenseKey(
                "Signature verification failed".into(),
            ));
        }

        Ok(())
    }

    /// Check if the license has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if a specific feature is available under this license
    pub fn has_feature(&self, feature: Feature) -> bool {
        let required = feature.required_tier();
        // Trial gets Premium-level access
        let effective_tier = if self.tier == LicenseTier::Trial {
            LicenseTier::Premium
        } else {
            self.tier
        };
        effective_tier >= required
    }

    /// Days remaining on the license
    pub fn days_remaining(&self) -> i64 {
        let remaining = self.expires_at - Utc::now();
        remaining.num_days().max(0)
    }

    /// Generate a hardware fingerprint based on system properties.
    /// Uses hostname + OS + architecture as a stable fingerprint.
    pub fn generate_hardware_fingerprint() -> String {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();

        // Use hostname as primary identifier
        if let Ok(hostname) = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .or_else(|_| hostname_fallback())
        {
            hasher.update(hostname.as_bytes());
        }

        // Add OS info for additional uniqueness
        hasher.update(std::env::consts::OS.as_bytes());
        hasher.update(std::env::consts::ARCH.as_bytes());

        let result = hasher.finalize();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
    }

    /// Compute HMAC-SHA256 signature over license fields
    fn compute_signature(&self, signing_key: &[u8]) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(signing_key)
            .map_err(|e| PremiumError::Encryption(format!("HMAC key error: {}", e)))?;

        // Sign critical fields
        mac.update(self.id.as_bytes());
        mac.update(&(self.tier as u8).to_le_bytes());
        mac.update(self.email.as_bytes());
        mac.update(self.issued_at.to_rfc3339().as_bytes());
        mac.update(self.expires_at.to_rfc3339().as_bytes());
        mac.update(self.hardware_fingerprint.as_bytes());
        mac.update(&self.max_ecus.to_le_bytes());

        let result = mac.finalize();
        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            result.into_bytes(),
        ))
    }

    /// Load license from a file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| PremiumError::Serialization(format!("Failed to parse license: {}", e)))
    }

    /// Save license to a file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| PremiumError::Serialization(format!("Failed to serialize license: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Fallback hostname resolution
fn hostname_fallback() -> std::result::Result<String, std::env::VarError> {
    // Try reading /etc/hostname on Unix systems
    if let Ok(hostname) = std::fs::read_to_string("/etc/hostname") {
        return Ok(hostname.trim().to_string());
    }
    Ok("unknown-host".to_string())
}

/// Feature gating enforcement
pub struct FeatureGate {
    license: License,
}

impl FeatureGate {
    /// Create a new feature gate from a license
    pub fn new(license: License) -> Self {
        Self { license }
    }

    /// Create a feature gate for the free tier (no license file)
    pub fn free_tier() -> Self {
        let license = License {
            id: "free".to_string(),
            tier: LicenseTier::Free,
            email: String::new(),
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(36500), // 100 years
            hardware_fingerprint: License::generate_hardware_fingerprint(),
            max_ecus: 50,
            signature: String::new(),
        };
        Self { license }
    }

    /// Check if a feature is available, returning an error if not
    pub fn require(&self, feature: Feature) -> Result<()> {
        if self.license.has_feature(feature) {
            Ok(())
        } else {
            Err(PremiumError::FeatureGated {
                feature: format!("{:?}", feature),
                required_tier: format!("{}", feature.required_tier()),
            })
        }
    }

    /// Check ECU limit
    pub fn check_ecu_limit(&self, ecu_count: u32) -> Result<()> {
        if self.license.max_ecus > 0 && ecu_count > self.license.max_ecus {
            Err(PremiumError::FeatureGated {
                feature: format!("More than {} ECUs", self.license.max_ecus),
                required_tier: "Premium".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Get the current license tier
    pub fn tier(&self) -> LicenseTier {
        self.license.tier
    }

    /// Get reference to the underlying license
    pub fn license(&self) -> &License {
        &self.license
    }
}

/// License manager handling startup validation and persistence
pub struct LicenseManager {
    license_dir: PathBuf,
    signing_key: Vec<u8>,
    current_license: Option<License>,
}

impl LicenseManager {
    /// Create a new license manager
    pub fn new(license_dir: PathBuf, signing_key: Vec<u8>) -> Self {
        Self {
            license_dir,
            signing_key,
            current_license: None,
        }
    }

    /// Load and validate the license on startup
    pub fn initialize(&mut self) -> Result<FeatureGate> {
        let license_path = self.license_dir.join("license.json");

        if license_path.exists() {
            let license = License::load_from_file(&license_path)?;

            match license.validate(&self.signing_key) {
                Ok(()) => {
                    let tier = license.tier;
                    let days = license.days_remaining();
                    log::info!(
                        "License validated: {} tier, {} days remaining",
                        tier,
                        days
                    );
                    self.current_license = Some(license.clone());
                    Ok(FeatureGate::new(license))
                }
                Err(PremiumError::LicenseExpired(_)) | Err(PremiumError::TrialExpired { .. }) => {
                    log::warn!("License expired, falling back to Free tier");
                    self.current_license = None;
                    Ok(FeatureGate::free_tier())
                }
                Err(e) => {
                    log::error!("License validation failed: {}", e);
                    Err(e)
                }
            }
        } else {
            log::info!("No license found, using Free tier");
            Ok(FeatureGate::free_tier())
        }
    }

    /// Activate a new license key
    pub fn activate_license(&mut self, license: License) -> Result<FeatureGate> {
        license.validate(&self.signing_key)?;

        let license_path = self.license_dir.join("license.json");
        std::fs::create_dir_all(&self.license_dir)?;
        license.save_to_file(&license_path)?;

        self.current_license = Some(license.clone());
        Ok(FeatureGate::new(license))
    }

    /// Start a 14-day trial
    pub fn start_trial(&mut self, email: &str) -> Result<FeatureGate> {
        let trial_marker = self.license_dir.join(".trial_used");
        if trial_marker.exists() {
            return Err(PremiumError::TrialExpired { days: 14 });
        }

        let license = License::new_trial(email, &self.signing_key)?;

        let license_path = self.license_dir.join("license.json");
        std::fs::create_dir_all(&self.license_dir)?;
        license.save_to_file(&license_path)?;

        // Mark trial as used to prevent re-activation
        std::fs::write(&trial_marker, "trial_activated")?;

        self.current_license = Some(license.clone());
        Ok(FeatureGate::new(license))
    }

    /// Get current license info
    pub fn current_license(&self) -> Option<&License> {
        self.current_license.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const TEST_KEY: &[u8] = b"test-signing-key-for-canary-2026";

    #[test]
    fn test_license_tier_ordering() {
        assert!(LicenseTier::Free < LicenseTier::Premium);
        assert!(LicenseTier::Premium < LicenseTier::Professional);
        assert!(LicenseTier::Professional < LicenseTier::Enterprise);
    }

    #[test]
    fn test_trial_license_creation() {
        let license = License::new_trial("test@example.com", TEST_KEY).unwrap();
        assert_eq!(license.tier, LicenseTier::Trial);
        assert_eq!(license.email, "test@example.com");
        assert!(license.days_remaining() <= 14);
        assert!(license.days_remaining() >= 13);
        assert!(!license.is_expired());
    }

    #[test]
    fn test_trial_has_premium_features() {
        let license = License::new_trial("test@example.com", TEST_KEY).unwrap();
        assert!(license.has_feature(Feature::BasicDiagnostics));
        assert!(license.has_feature(Feature::UnlimitedEcus));
        assert!(license.has_feature(Feature::SecurityAccess));
        assert!(license.has_feature(Feature::CloudSync));
        // Trial should NOT have Professional features
        assert!(!license.has_feature(Feature::ApiAccess));
        assert!(!license.has_feature(Feature::FleetManagement));
    }

    #[test]
    fn test_free_tier_feature_gating() {
        let gate = FeatureGate::free_tier();
        assert!(gate.require(Feature::BasicDiagnostics).is_ok());
        assert!(gate.require(Feature::CanCapture).is_ok());
        assert!(gate.require(Feature::SecurityAccess).is_err());
        assert!(gate.require(Feature::ApiAccess).is_err());
        assert!(gate.require(Feature::WhiteLabel).is_err());
    }

    #[test]
    fn test_free_tier_ecu_limit() {
        let gate = FeatureGate::free_tier();
        assert!(gate.check_ecu_limit(50).is_ok());
        assert!(gate.check_ecu_limit(51).is_err());
    }

    #[test]
    fn test_license_signature_validation() {
        let license = License::new_trial("test@example.com", TEST_KEY).unwrap();
        // Valid signature should pass
        assert!(license.validate(TEST_KEY).is_ok());
    }

    #[test]
    fn test_license_tamper_detection() {
        let mut license = License::new_trial("test@example.com", TEST_KEY).unwrap();
        // Tamper with tier
        license.tier = LicenseTier::Enterprise;
        // Should fail validation due to signature mismatch
        assert!(license.validate(TEST_KEY).is_err());
    }

    #[test]
    fn test_license_persistence() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("license.json");

        let license = License::new_trial("test@example.com", TEST_KEY).unwrap();
        license.save_to_file(&path).unwrap();

        let loaded = License::load_from_file(&path).unwrap();
        assert_eq!(loaded.id, license.id);
        assert_eq!(loaded.tier, license.tier);
        assert_eq!(loaded.email, license.email);
    }

    #[test]
    fn test_license_manager_no_license() {
        let tmp = TempDir::new().unwrap();
        let mut manager = LicenseManager::new(tmp.path().to_path_buf(), TEST_KEY.to_vec());
        let gate = manager.initialize().unwrap();
        assert_eq!(gate.tier(), LicenseTier::Free);
    }

    #[test]
    fn test_license_manager_trial() {
        let tmp = TempDir::new().unwrap();
        let mut manager = LicenseManager::new(tmp.path().to_path_buf(), TEST_KEY.to_vec());

        let gate = manager.start_trial("test@example.com").unwrap();
        assert_eq!(gate.tier(), LicenseTier::Trial);

        // Second trial should fail
        assert!(manager.start_trial("test@example.com").is_err());
    }

    #[test]
    fn test_license_manager_startup_validation() {
        let tmp = TempDir::new().unwrap();
        let mut manager = LicenseManager::new(tmp.path().to_path_buf(), TEST_KEY.to_vec());

        // Start trial and save
        manager.start_trial("test@example.com").unwrap();

        // New manager should load and validate existing license
        let mut manager2 = LicenseManager::new(tmp.path().to_path_buf(), TEST_KEY.to_vec());
        let gate = manager2.initialize().unwrap();
        // Should be Trial tier (loaded from file)
        assert_eq!(gate.tier(), LicenseTier::Trial);
    }

    #[test]
    fn test_feature_display() {
        assert_eq!(format!("{}", LicenseTier::Free), "Free");
        assert_eq!(format!("{}", LicenseTier::Premium), "Premium");
        assert_eq!(format!("{}", LicenseTier::Enterprise), "Enterprise");
    }

    #[test]
    fn test_hardware_fingerprint_consistency() {
        let fp1 = License::generate_hardware_fingerprint();
        let fp2 = License::generate_hardware_fingerprint();
        assert_eq!(fp1, fp2, "Hardware fingerprint should be stable");
    }
}
