//! End-to-end encryption for cloud sync.
//!
//! Uses AES-256-GCM for symmetric encryption with key derivation
//! from a user-provided passphrase. Zero-knowledge: the server
//! never sees plaintext data or encryption keys.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::Digest;

use crate::error::{PremiumError, Result};

/// E2E encryption provider using AES-256-GCM
pub struct E2EEncryption {
    cipher: Aes256Gcm,
}

impl E2EEncryption {
    /// Create a new encryption instance from a passphrase.
    /// Derives a 256-bit key using SHA-256.
    pub fn from_passphrase(passphrase: &str) -> Result<Self> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(passphrase.as_bytes());
        // Add salt for key derivation
        hasher.update(b"canary-e2e-salt-v1");
        let key_bytes = hasher.finalize();

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| PremiumError::Encryption(format!("Key init failed: {}", e)))?;

        Ok(Self { cipher })
    }

    /// Create a new encryption instance from raw key bytes (32 bytes)
    pub fn from_key(key: &[u8; 32]) -> Result<Self> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| PremiumError::Encryption(format!("Key init failed: {}", e)))?;
        Ok(Self { cipher })
    }

    /// Encrypt data with a random nonce. Returns nonce || ciphertext.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| PremiumError::Encryption(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data (expects nonce || ciphertext format)
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(PremiumError::Encryption(
                "Data too short for nonce".into(),
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| PremiumError::Encryption(format!("Decryption failed: {}", e)))
    }

    /// Generate a random 256-bit encryption key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let enc = E2EEncryption::from_passphrase("my-secret-passphrase").unwrap();
        let plaintext = b"Hello, Canary diagnostics data!";

        let encrypted = enc.encrypt(plaintext).unwrap();
        assert_ne!(&encrypted[12..], plaintext); // ciphertext differs

        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_passphrase_fails() {
        let enc1 = E2EEncryption::from_passphrase("correct-passphrase").unwrap();
        let enc2 = E2EEncryption::from_passphrase("wrong-passphrase").unwrap();

        let encrypted = enc1.encrypt(b"secret data").unwrap();
        assert!(enc2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_tamper_detection() {
        let enc = E2EEncryption::from_passphrase("passphrase").unwrap();
        let mut encrypted = enc.encrypt(b"important data").unwrap();

        // Tamper with ciphertext
        if let Some(last) = encrypted.last_mut() {
            *last ^= 0xFF;
        }

        assert!(enc.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_from_raw_key() {
        let key = E2EEncryption::generate_key();
        let enc = E2EEncryption::from_key(&key).unwrap();

        let plaintext = b"test data";
        let encrypted = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_unique_nonces() {
        let enc = E2EEncryption::from_passphrase("test").unwrap();
        let e1 = enc.encrypt(b"data").unwrap();
        let e2 = enc.encrypt(b"data").unwrap();
        // Same plaintext should produce different ciphertexts (different nonces)
        assert_ne!(e1, e2);
    }

    #[test]
    fn test_short_data_error() {
        let enc = E2EEncryption::from_passphrase("test").unwrap();
        assert!(enc.decrypt(&[0u8; 5]).is_err());
    }
}
