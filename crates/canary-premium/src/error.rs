use thiserror::Error;

/// Errors that can occur in the premium module
#[derive(Debug, Error)]
pub enum PremiumError {
    #[error("License expired at {0}")]
    LicenseExpired(String),

    #[error("Invalid license key: {0}")]
    InvalidLicenseKey(String),

    #[error("Hardware fingerprint mismatch: expected {expected}, got {actual}")]
    HardwareFingerprint { expected: String, actual: String },

    #[error("Feature '{feature}' requires {required_tier} tier or higher")]
    FeatureGated {
        feature: String,
        required_tier: String,
    },

    #[error("Trial expired after {days} days")]
    TrialExpired { days: u32 },

    #[error("Cloud sync error: {0}")]
    CloudSync(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Marketplace error: {0}")]
    Marketplace(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Stripe payment error: {0}")]
    PaymentError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PremiumError>;
