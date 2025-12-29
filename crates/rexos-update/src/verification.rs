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
        let bytes =
            hex::decode(hex_key).map_err(|e| VerificationError::InvalidPublicKey(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(VerificationError::InvalidPublicKey(format!(
                "Key must be 32 bytes, got {}",
                bytes.len()
            )));
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
            return Err(VerificationError::InvalidSignature(format!(
                "Signature must be 64 bytes, got {}",
                sig_bytes.len()
            )));
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
            return Err(VerificationError::InvalidSignature(format!(
                "Signature must be 64 bytes, got {}",
                sig_bytes.len()
            )));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);

        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        self.public_key
            .verify(data, &signature)
            .map_err(|_| VerificationError::SignatureMismatch)
    }
}

/// Verifies file hashes using SHA256
pub struct HashVerifier;

impl HashVerifier {
    /// Compute SHA256 hash of a file
    pub fn sha256_file(path: &Path) -> Result<String, VerificationError> {
        use sha2::{Digest, Sha256};

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
        use sha2::{Digest, Sha256};

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

/// Certificate chain verification for HTTPS pinning
///
/// Used to verify that update server certificates match expected pinned certificates,
/// providing an additional layer of security against MITM attacks.
pub struct CertificateVerifier {
    /// Pinned certificate hashes (SHA256)
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
    ///
    /// # Production Setup
    ///
    /// Before deploying to production, replace this placeholder with the actual
    /// SHA256 hash of the RexOS update server's TLS certificate. To obtain the hash:
    ///
    /// ```bash
    /// echo | openssl s_client -connect updates.rexos.io:443 2>/dev/null | \
    ///   openssl x509 -pubkey -noout | \
    ///   openssl pkey -pubin -outform der | \
    ///   openssl dgst -sha256 -binary | \
    ///   xxd -p -c 64
    /// ```
    pub fn with_rexos_cert(mut self) -> Self {
        // TODO(production): Replace with actual RexOS update server certificate hash
        // This is a placeholder - the real hash should be obtained from the production
        // server certificate and hardcoded here for certificate pinning security.
        //
        // For development/testing, this placeholder allows the system to function
        // without certificate pinning enabled.
        const REXOS_CERT_HASH: &str =
            "0000000000000000000000000000000000000000000000000000000000000000";

        self.pinned_certs.push(REXOS_CERT_HASH.to_string());
        self
    }

    /// Check if certificate pinning is properly configured (not using placeholder)
    pub fn is_configured(&self) -> bool {
        const PLACEHOLDER: &str =
            "0000000000000000000000000000000000000000000000000000000000000000";
        !self.pinned_certs.iter().all(|c| c == PLACEHOLDER)
    }
}

/// Generate a signing keypair for update signing (build/release tooling)
///
/// This function is used by the RexOS build system to generate Ed25519 keypairs
/// for signing update packages. The generated keys are used as follows:
///
/// - **Private key**: Kept secure on the build server, used to sign updates
/// - **Public key**: Embedded in the firmware, used to verify update signatures
///
/// # Returns
///
/// A tuple of (private_key_hex, public_key_hex) where both are hex-encoded strings.
///
/// # Example (Build Tool Usage)
///
/// ```ignore
/// // In release tooling:
/// let (private, public) = generate_keypair();
/// // Store private key securely, embed public key in config
/// ```
///
/// # Security Note
///
/// The private key should NEVER be committed to version control or distributed
/// with firmware. Store it securely using a secrets management system.
#[allow(dead_code)] // Used by build/release tooling, not runtime code
pub fn generate_keypair() -> (String, String) {
    use ed25519_dalek::SigningKey;
    use rand::RngCore;
    use rand::rngs::OsRng;

    let mut secret_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_key_bytes);

    let signing_key = SigningKey::from_bytes(&secret_key_bytes);
    let verifying_key = signing_key.verifying_key();

    let private_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());

    (private_hex, public_hex)
}

/// Sign data with a private key (build/release tooling)
///
/// This function is used by the RexOS release tooling to sign update packages
/// before distribution. The signature is verified on the device using the
/// corresponding public key.
///
/// # Arguments
///
/// * `data` - The data to sign (typically the update file contents)
/// * `private_key_hex` - The hex-encoded Ed25519 private key (32 bytes = 64 hex chars)
///
/// # Returns
///
/// The hex-encoded Ed25519 signature (64 bytes = 128 hex chars)
///
/// # Example (Build Tool Usage)
///
/// ```ignore
/// // In release tooling:
/// let update_data = std::fs::read("update.tar.gz")?;
/// let signature = sign_data(&update_data, &private_key)?;
/// // Include signature in update manifest
/// ```
///
/// # Errors
///
/// Returns an error if the private key is invalid or incorrectly formatted.
#[allow(dead_code)] // Used by build/release tooling, not runtime code
pub fn sign_data(data: &[u8], private_key_hex: &str) -> Result<String, VerificationError> {
    use ed25519_dalek::{Signer, SigningKey};

    let key_bytes = hex::decode(private_key_hex)
        .map_err(|e| VerificationError::InvalidPublicKey(e.to_string()))?;

    if key_bytes.len() != 32 {
        return Err(VerificationError::InvalidPublicKey(format!(
            "Key must be 32 bytes, got {}",
            key_bytes.len()
        )));
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
