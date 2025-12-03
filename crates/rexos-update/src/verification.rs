//! Cryptographic verification of updates

use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Signature verification failed")]
    SignatureMismatch,

    #[error("Hash verification failed: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Verifies update signatures using Ed25519
pub struct SignatureVerifier {
    public_key: ed25519_dalek::VerifyingKey,
}

impl SignatureVerifier {
    /// Create verifier from hex-encoded public key
    pub fn from_hex(hex_key: &str) -> Result<Self, VerificationError> {
        let bytes = hex::decode(hex_key)
            .map_err(|e| VerificationError::InvalidPublicKey(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(VerificationError::InvalidPublicKey(
                format!("Key must be 32 bytes, got {}", bytes.len())
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);

        let public_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| VerificationError::InvalidPublicKey(e.to_string()))?;

        Ok(Self { public_key })
    }

    /// Verify a file's signature
    pub fn verify_file(&self, path: &Path, signature_hex: &str) -> Result<(), VerificationError> {
        use ed25519_dalek::Verifier;

        // Read file content
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        // Parse signature
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| VerificationError::InvalidSignature(e.to_string()))?;

        if sig_bytes.len() != 64 {
            return Err(VerificationError::InvalidSignature(
                format!("Signature must be 64 bytes, got {}", sig_bytes.len())
            ));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);

        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        // Verify
        self.public_key
            .verify(&content, &signature)
            .map_err(|_| VerificationError::SignatureMismatch)
    }

    /// Verify data signature
    pub fn verify_data(&self, data: &[u8], signature_hex: &str) -> Result<(), VerificationError> {
        use ed25519_dalek::Verifier;

        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| VerificationError::InvalidSignature(e.to_string()))?;

        if sig_bytes.len() != 64 {
            return Err(VerificationError::InvalidSignature(
                format!("Signature must be 64 bytes, got {}", sig_bytes.len())
            ));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);

        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        self.public_key
            .verify(data, &signature)
            .map_err(|_| VerificationError::SignatureMismatch)
    }
}

/// Verifies file hashes
pub struct HashVerifier;

impl HashVerifier {
    /// Compute SHA256 hash of a file
    pub fn sha256_file(path: &Path) -> Result<String, VerificationError> {
        use sha2::{Sha256, Digest};

        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Compute SHA256 hash of data
    pub fn sha256_data(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Verify file matches expected hash
    pub fn verify_file(path: &Path, expected_hash: &str) -> Result<(), VerificationError> {
        let actual = Self::sha256_file(path)?;

        if actual != expected_hash.to_lowercase() {
            return Err(VerificationError::HashMismatch {
                expected: expected_hash.to_string(),
                actual,
            });
        }

        Ok(())
    }

    /// Verify data matches expected hash
    pub fn verify_data(data: &[u8], expected_hash: &str) -> Result<(), VerificationError> {
        let actual = Self::sha256_data(data);

        if actual != expected_hash.to_lowercase() {
            return Err(VerificationError::HashMismatch {
                expected: expected_hash.to_string(),
                actual,
            });
        }

        Ok(())
    }
}

/// Certificate chain verification (for future HTTPS pinning)
pub struct CertificateVerifier {
    /// Pinned certificate hashes
    pinned_certs: Vec<String>,
}

impl CertificateVerifier {
    /// Create with pinned certificate hashes
    pub fn new(pinned_certs: Vec<String>) -> Self {
        Self { pinned_certs }
    }

    /// Verify certificate matches one of the pinned certs
    pub fn verify_cert(&self, cert_hash: &str) -> bool {
        self.pinned_certs.iter().any(|pinned| pinned == cert_hash)
    }

    /// Add RexOS update server certificate
    pub fn with_rexos_cert(mut self) -> Self {
        // SHA256 of RexOS update server certificate (placeholder)
        self.pinned_certs.push(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        );
        self
    }
}

/// Generate a signing keypair (for build tools)
pub fn generate_keypair() -> (String, String) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use rand::RngCore;

    let mut secret_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_key_bytes);

    let signing_key = SigningKey::from_bytes(&secret_key_bytes);
    let verifying_key = signing_key.verifying_key();

    let private_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());

    (private_hex, public_hex)
}

/// Sign data with a private key (for build tools)
pub fn sign_data(data: &[u8], private_key_hex: &str) -> Result<String, VerificationError> {
    use ed25519_dalek::{Signer, SigningKey};

    let key_bytes = hex::decode(private_key_hex)
        .map_err(|e| VerificationError::InvalidPublicKey(e.to_string()))?;

    if key_bytes.len() != 32 {
        return Err(VerificationError::InvalidPublicKey(
            format!("Key must be 32 bytes, got {}", key_bytes.len())
        ));
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);

    let signing_key = SigningKey::from_bytes(&key_array);
    let signature = signing_key.sign(data);

    Ok(hex::encode(signature.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data() {
        let data = b"hello world";
        let hash = HashVerifier::sha256_data(data);

        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_verification() {
        let data = b"test data";
        let hash = HashVerifier::sha256_data(data);

        assert!(HashVerifier::verify_data(data, &hash).is_ok());
        assert!(HashVerifier::verify_data(data, "wronghash").is_err());
    }

    #[test]
    fn test_keypair_generation() {
        let (private, public) = generate_keypair();

        assert_eq!(private.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(public.len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let (private, public) = generate_keypair();
        let data = b"test message";

        let signature = sign_data(data, &private).unwrap();
        let verifier = SignatureVerifier::from_hex(&public).unwrap();

        assert!(verifier.verify_data(data, &signature).is_ok());
    }

    #[test]
    fn test_invalid_signature() {
        let (_, public) = generate_keypair();
        let data = b"test message";

        let verifier = SignatureVerifier::from_hex(&public).unwrap();
        let fake_sig = "0".repeat(128);

        assert!(verifier.verify_data(data, &fake_sig).is_err());
    }
}
