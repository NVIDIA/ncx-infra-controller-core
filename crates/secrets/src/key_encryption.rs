/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
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
//! # Envelope format (scheme version 1)
//!
//! Stored in DB as **standard base64** of the binary layout:
//!
//! `scheme_version` (1) \| `key_id_len` (1) \| `key_id` (utf-8) \| `nonce` (12) \|
//! `ciphertext_len` (4, big-endian u32) \| `ciphertext` (AES-GCM output: ciphertext + tag)
//!
//! - **scheme_version** `1`: AES-256-GCM, no HKDF; key material is 32 bytes from
//!   base64-decoding the configured encryption secret (`openssl rand -base64 32`).
//! - **key_id**: map key under `machine_identity.encryption_keys` (e.g. `kv1`), must match site
//!   `current_encryption_key_id` (from a secrets file, env-backed credentials, or another store).

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, KeyInit};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rcgen::KeyPair;
use sha2::{Digest, Sha256};

/// Scheme version 1: AES-256-GCM, 32-byte key from base64-decoded encryption secret, envelope below.
pub const SCHEME_VERSION_V1: u8 = 1;

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

/// Decodes an encryption secret: standard base64 of exactly 32 bytes (e.g. `openssl rand -base64 32`).
pub fn aes256_key_from_secret_b64(secret_b64: &str) -> Result<[u8; 32], KeyEncryptionError> {
    let trimmed = secret_b64.trim();
    let raw = BASE64.decode(trimmed).map_err(|e| {
        KeyEncryptionError::Encrypt(format!("encryption secret is not valid base64: {e}"))
    })?;
    raw.try_into().map_err(|v: Vec<u8>| {
        KeyEncryptionError::Encrypt(format!(
            "encryption secret must decode to exactly 32 bytes for AES-256 (got {} bytes); use e.g. `openssl rand -base64 32`",
            v.len()
        ))
    })
}

fn serialize_envelope_v1(
    key_id: &str,
    nonce: &[u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>, KeyEncryptionError> {
    let kid = key_id.as_bytes();
    if kid.is_empty() || kid.len() > 255 {
        return Err(KeyEncryptionError::Encrypt(
            "encryption_key_id (envelope) must be 1..=255 UTF-8 bytes".into(),
        ));
    }
    let ct_len: u32 = ciphertext
        .len()
        .try_into()
        .map_err(|_| KeyEncryptionError::Encrypt("ciphertext too large".into()))?;
    let mut out = Vec::with_capacity(2 + kid.len() + 12 + 4 + ciphertext.len());
    out.push(SCHEME_VERSION_V1);
    out.push(kid.len() as u8);
    out.extend_from_slice(kid);
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ct_len.to_be_bytes());
    out.extend_from_slice(ciphertext);
    Ok(out)
}

fn parse_envelope_v1(data: &[u8]) -> Result<(&str, &[u8], &[u8]), KeyEncryptionError> {
    if data.len() < 2 {
        return Err(KeyEncryptionError::Decrypt("truncated envelope".into()));
    }
    if data[0] != SCHEME_VERSION_V1 {
        return Err(KeyEncryptionError::Decrypt(
            "unsupported scheme_version".into(),
        ));
    }
    let kid_len = data[1] as usize;
    if data.len() < 2 + kid_len + 12 + 4 {
        return Err(KeyEncryptionError::Decrypt("truncated envelope".into()));
    }
    let key_id = std::str::from_utf8(&data[2..2 + kid_len])
        .map_err(|_| KeyEncryptionError::Decrypt("invalid key_id UTF-8".into()))?;
    let off = 2 + kid_len;
    let nonce = &data[off..off + 12];
    let ct_len = u32::from_be_bytes(data[off + 12..off + 16].try_into().unwrap()) as usize;
    let ct_end = off + 16 + ct_len;
    if data.len() != ct_end {
        return Err(KeyEncryptionError::Decrypt(
            "envelope length mismatch".into(),
        ));
    }
    let ciphertext = &data[off + 16..ct_end];
    Ok((key_id, nonce, ciphertext))
}

/// Encrypts plaintext with AES-256-GCM using envelope v1.
///
/// `encryption_secret_b64` must be standard base64 of exactly 32 random bytes (from the credential
/// store: e.g. local secrets file or any other provider).
/// `encryption_key_id` must match the entry under `machine_identity.encryption_keys` and site
/// `current_encryption_key_id`.
/// Returns standard base64 of the binary envelope (safe for `TEXT` columns).
pub fn encrypt(
    plaintext: &[u8],
    encryption_secret_b64: &str,
    encryption_key_id: &str,
) -> Result<String, KeyEncryptionError> {
    let key = aes256_key_from_secret_b64(encryption_secret_b64)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| KeyEncryptionError::Encrypt(e.to_string()))?;
    let mut nonce = [0u8; 12];
    rand::Rng::fill(&mut rand::rng(), &mut nonce);
    let ciphertext = cipher
        .encrypt(&nonce.into(), plaintext)
        .map_err(|e| KeyEncryptionError::Encrypt(e.to_string()))?;
    let envelope = serialize_envelope_v1(encryption_key_id, &nonce, &ciphertext)?;
    Ok(BASE64.encode(&envelope))
}

/// Decrypts a DB value produced by [`encrypt`]: envelope v1 with a 32-byte base64 encryption secret.
pub fn decrypt(
    encrypted_base64: &str,
    encryption_secret: &str,
) -> Result<Vec<u8>, KeyEncryptionError> {
    let combined = BASE64
        .decode(encrypted_base64.trim())
        .map_err(|e| KeyEncryptionError::Decrypt(e.to_string()))?;

    let (_, nonce, ciphertext) = parse_envelope_v1(&combined)?;
    let key = aes256_key_from_secret_b64(encryption_secret).map_err(|e| match e {
        KeyEncryptionError::Encrypt(msg) => KeyEncryptionError::Decrypt(msg),
        other => other,
    })?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| KeyEncryptionError::Decrypt(e.to_string()))?;
    let nonce_ga = aes_gcm::aead::generic_array::GenericArray::from_slice(nonce);
    cipher
        .decrypt(nonce_ga, ciphertext)
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

    /// 32 zero bytes, standard base64.
    fn test_secret_key_b64() -> String {
        BASE64.encode([0u8; 32])
    }

    #[test]
    fn encrypt_decrypt_roundtrip_v1() {
        let plaintext = b"secret data";
        let key_b64 = test_secret_key_b64();
        let encrypted = encrypt(plaintext, &key_b64, "kv1").unwrap();
        let decrypted = decrypt(&encrypted, &key_b64).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn key_id_from_public_key_is_deterministic() {
        let pub_key = "-----BEGIN PUBLIC KEY-----\nMFkw...\n-----END PUBLIC KEY-----";
        let id1 = key_id_from_public_key(pub_key);
        let id2 = key_id_from_public_key(pub_key);
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64);
    }

    #[test]
    fn generate_es256_key_pair_produces_valid_outputs() {
        let (private_pem, public_pem) = generate_es256_key_pair().unwrap();
        assert!(private_pem.starts_with(b"-----BEGIN"));
        assert!(public_pem.contains("PUBLIC KEY"));
        let key_id = key_id_from_public_key(&public_pem);
        assert_eq!(key_id.len(), 64);
    }

    #[test]
    fn encryption_secret_wrong_length_errors() {
        let short = BASE64.encode([0u8; 16]);
        let err = aes256_key_from_secret_b64(&short).unwrap_err();
        assert!(err.to_string().contains("32"));
    }

    #[test]
    fn encrypt_decrypt_token_delegation_json_utf8_roundtrip() {
        let key_b64 = test_secret_key_b64();
        let json = r#"{"client_id":"c","client_secret":"s"}"#;
        let enc = encrypt(json.as_bytes(), &key_b64, "kv1").unwrap();
        let plain = decrypt(&enc, &key_b64).unwrap();
        let out = String::from_utf8(plain).unwrap();
        assert_eq!(out, json);
    }

    #[test]
    fn decrypt_rejects_plaintext_token_delegation_json() {
        let key_b64 = test_secret_key_b64();
        let plaintext_json = r#"{"client_id":"c","client_secret":"s"}"#;
        assert!(decrypt(plaintext_json, &key_b64).is_err());
    }
}
