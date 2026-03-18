/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use it except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Generic key encryption and signing key generation utilities.
//!
//! Provides AES-256-GCM encrypt/decrypt, key_id computation, and ES256 key pair generation.
//! Used for securing private keys at rest (e.g. machine identity signing keys).

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, KeyInit};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rcgen::KeyPair;
use sha2::{Digest, Sha256};

/// Error type for key encryption and generation operations.
#[derive(Debug, thiserror::Error)]
pub enum KeyEncryptionError {
    #[error("key generation failed: {0}")]
    KeyGen(String),
    #[error("encryption failed: {0}")]
    Encrypt(String),
    #[error("decryption failed: {0}")]
    Decrypt(String),
}

/// Encrypts plaintext with the given encryption key.
/// Uses AES-256-GCM. Key is derived via SHA256 of the encryption key string.
/// Returns base64(nonce || ciphertext || tag).
pub fn encrypt(plaintext: &[u8], encryption_key: &str) -> Result<String, KeyEncryptionError> {
    let key_bytes = Sha256::digest(encryption_key.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(key_bytes.as_slice())
        .map_err(|e| KeyEncryptionError::Encrypt(e.to_string()))?;
    let mut nonce = [0u8; 12];
    rand::Rng::fill(&mut rand::rng(), &mut nonce);
    let ciphertext = cipher
        .encrypt(&nonce.into(), plaintext)
        .map_err(|e| KeyEncryptionError::Encrypt(e.to_string()))?;
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&combined))
}

/// Decrypts ciphertext from base64(nonce || ciphertext || tag).
pub fn decrypt(
    encrypted_base64: &str,
    encryption_key: &str,
) -> Result<Vec<u8>, KeyEncryptionError> {
    let combined = BASE64
        .decode(encrypted_base64)
        .map_err(|e| KeyEncryptionError::Decrypt(e.to_string()))?;
    if combined.len() < 12 + 16 {
        return Err(KeyEncryptionError::Decrypt("ciphertext too short".into()));
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key_bytes = Sha256::digest(encryption_key.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(key_bytes.as_slice())
        .map_err(|e| KeyEncryptionError::Decrypt(e.to_string()))?;
    let nonce = aes_gcm::aead::generic_array::GenericArray::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| KeyEncryptionError::Decrypt(e.to_string()))
}

/// Computes key_id as hex(sha256(public_key)).
/// Works with any public key representation (PEM, DER, etc.).
pub fn key_id_from_public_key(public_key: &str) -> String {
    let hash = Sha256::digest(public_key.as_bytes());
    hex::encode(hash)
}

/// Generates an ES256 (ECDSA P-256) signing key pair.
/// Returns (private_key_pem, public_key_pem).
pub fn generate_es256_key_pair() -> Result<(Vec<u8>, String), KeyEncryptionError> {
    let key_pair = KeyPair::generate().map_err(|e| KeyEncryptionError::KeyGen(e.to_string()))?;
    let private_pem = key_pair.serialize_pem().into_bytes();
    let public_pem = key_pair.public_key_pem();
    Ok((private_pem, public_pem))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let plaintext = b"secret data";
        let key = "my-encryption-key";
        let encrypted = encrypt(plaintext, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn key_id_from_public_key_is_deterministic() {
        let pub_key = "-----BEGIN PUBLIC KEY-----\nMFkw...\n-----END PUBLIC KEY-----";
        let id1 = key_id_from_public_key(pub_key);
        let id2 = key_id_from_public_key(pub_key);
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64); // hex(sha256) = 64 chars
    }

    #[test]
    fn generate_es256_key_pair_produces_valid_outputs() {
        let (private_pem, public_pem) = generate_es256_key_pair().unwrap();
        assert!(private_pem.starts_with(b"-----BEGIN"));
        assert!(public_pem.contains("PUBLIC KEY"));
        let key_id = key_id_from_public_key(&public_pem);
        assert_eq!(key_id.len(), 64);
    }
}
