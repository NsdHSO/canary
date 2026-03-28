//! # canary-security
//!
//! Security access algorithms for automotive ECU diagnostics.
//!
//! Implements seed/key algorithms for various manufacturers:
//! - **VW Group** (VW, Audi, Skoda, SEAT)
//! - **General Motors** (GM, Chevrolet, Cadillac, GMC)
//! - **Ford** (Ford, Lincoln, Mercury)
//!
//! ## Architecture
//!
//! Uses the Strategy pattern with a `SecurityAlgorithm` trait that
//! each manufacturer implements. The `KeyGenerator` provides a
//! unified interface for seed/key operations.
//!
//! ## Example
//!
//! ```rust
//! use canary_security::{KeyGenerator, Manufacturer};
//!
//! let keygen = KeyGenerator::for_manufacturer(Manufacturer::VwGroup);
//! let seed = vec![0x12, 0x34, 0x56, 0x78];
//! let key = keygen.compute_key(&seed, 0x01).unwrap();
//! ```

pub mod keygen;
pub mod algorithms;

// Re-exports
pub use keygen::{KeyGenerator, Manufacturer, SecurityAlgorithm, SecurityError, SecurityLevel};
pub use algorithms::vw_group::VwGroupAlgorithm;
pub use algorithms::gm::GmAlgorithm;
pub use algorithms::ford::FordAlgorithm;
