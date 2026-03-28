//! # Canary Premium
//!
//! Premium features for the Canary automotive diagnostics platform:
//! - License management with tiered feature gating
//! - Cloud sync with E2E encryption
//! - Data marketplace integration

pub mod license;
pub mod cloud_sync;
pub mod marketplace;
pub mod encryption;
pub mod error;

pub use error::PremiumError;
pub use license::{License, LicenseTier, LicenseManager, FeatureGate};
pub use cloud_sync::{CloudSyncClient, SyncConfig};
pub use marketplace::{MarketplaceClient, MarketplaceListing, MarketplacePurchase};
pub use encryption::E2EEncryption;
