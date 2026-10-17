//! License Verification & Safety Lock Module
//!
//! Handles HWID-bound cryptographic license validation with Ed25519 (Asymmetric)
//! and fallback HMAC-SHA256, expiration checking, persistent license storage, and engine safety guarding.

use base64::prelude::*;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_LICENSE_SALT: &[u8] = b"NODEHOTKEY_V4_HMAC_SECRET_SALT_2026";
pub const LICENSE_FILE_PATH: &str = "configs/license.key";

/// Official Master Ed25519 Public Key (32 bytes) embedded in client binary for offline signature verification
pub const MASTER_ED25519_PUBLIC_KEY_BYTES: [u8; 32] = [
    0x71, 0x4c, 0xdf, 0xa8, 0xd7, 0x26, 0x68, 0x08,
    0x44, 0xdf, 0x3e, 0x9b, 0xda, 0xeb, 0xcc, 0xb9,
    0x9e, 0xc5, 0x11, 0xa5, 0x78, 0xc6, 0xdb, 0x82,
    0x40, 0xfc, 0x51, 0x55, 0xbb, 0xb9, 0xa2, 0xba,
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LicenseClaims {
    pub hwid: String,
    pub tier: String,
    pub expires_at: i64, // Unix timestamp in seconds (-1 for lifetime)
    pub issued_at: i64,  // Unix timestamp in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

fn default_algorithm() -> String {
    "Ed25519".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseToken {
    #[serde(default = "default_algorithm")]
    pub algorithm: String, // "Ed25519" or "HMAC"
    pub claims: LicenseClaims,
    pub signature: String, // Base64 or Hex signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub is_valid: bool,
    pub status: String, // "Active", "Expired", "InvalidHWID", "Unlicensed", "Corrupted"
    pub tier: Option<String>,
    pub expires_at: Option<i64>,
    pub days_remaining: Option<i64>,
    pub hwid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_key: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LicenseError {
    InvalidFormat,
    InvalidSignature,
    HwidMismatch,
    Expired,
    IoError(String),
}

impl std::fmt::Display for LicenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseError::InvalidFormat => write!(f, "License key has invalid format or encoding"),
            LicenseError::InvalidSignature => write!(f, "Digital signature verification failed (Tampered or forged key)"),
            LicenseError::HwidMismatch => write!(f, "License is bound to another machine (HWID mismatch)"),
            LicenseError::Expired => write!(f, "License has expired"),
            LicenseError::IoError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for LicenseError {}

/// Derive the official Master Ed25519 Signing Key (For developer CLI generation tools)
/// Obfuscated with rolling XOR cipher at compile-time to prevent plaintext binary extraction
pub fn get_master_signing_key() -> SigningKey {
    let seed_str = crate::secure_str!("NODEHOTKEY_V4_ED25519_MASTER_PRIVATE_SEED_2026_PRODUCTION");
    let mut hasher = Sha256::new();
    hasher.update(seed_str.as_bytes());
    let seed: [u8; 32] = hasher.finalize().into();
    SigningKey::from_bytes(&seed)
}

/// Official Master Ed25519 Verifying Key (Directly parsed from public key bytes, zero private key dependency)
pub fn get_master_verifying_key() -> VerifyingKey {
    VerifyingKey::from_bytes(&MASTER_ED25519_PUBLIC_KEY_BYTES)
        .expect("Hardcoded master verifying key must be valid 32-byte Ed25519 public key")
}

/// Helper to serialize claims canonical string for digital signing
fn serialize_canonical_claims(claims: &LicenseClaims) -> String {
    format!(
        "HWID:{}|TIER:{}|EXPIRES:{}|ISSUED:{}|USER:{}",
        claims.hwid,
        claims.tier,
        claims.expires_at,
        claims.issued_at,
        claims.user_id.as_deref().unwrap_or("ANONYMOUS")
    )
}

/// Compute standard RFC 2104 HMAC-SHA256 (Used for legacy HMAC fallback)
pub fn compute_hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let block_size = 64;
    let mut k = [0u8; 64];
    if key.len() > block_size {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let hash = hasher.finalize();
        k[..32].copy_from_slice(&hash);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut o_key_pad = [0x5cu8; 64];
    let mut i_key_pad = [0x36u8; 64];
    for i in 0..64 {
        o_key_pad[i] ^= k[i];
        i_key_pad[i] ^= k[i];
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&i_key_pad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&o_key_pad);
    outer_hasher.update(&inner_hash);
    let final_hash = outer_hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&final_hash);
    out
}

/// Generate a signed License Key using Ed25519 Asymmetric Signature (Recommended)
pub fn generate_license_key_ed25519(claims: &LicenseClaims, signing_key: &SigningKey) -> String {
    let canonical = serialize_canonical_claims(claims);
    let signature = signing_key.sign(canonical.as_bytes());
    let sig_b64 = BASE64_STANDARD.encode(signature.to_bytes());

    let token = LicenseToken {
        algorithm: "Ed25519".to_string(),
        claims: claims.clone(),
        signature: sig_b64,
    };

    let json_bytes = serde_json::to_vec(&token).unwrap_or_default();
    BASE64_STANDARD.encode(json_bytes)
}

/// Legacy HMAC license generation
pub fn generate_license_key(claims: &LicenseClaims, secret_salt: &[u8]) -> String {
    let canonical = serialize_canonical_claims(claims);
    let sig_bytes = compute_hmac_sha256(secret_salt, canonical.as_bytes());
    let sig_hex = format!("{:X}", Sha256::digest(&sig_bytes));

    let token = LicenseToken {
        algorithm: "HMAC".to_string(),
        claims: claims.clone(),
        signature: sig_hex,
    };

    let json_bytes = serde_json::to_vec(&token).unwrap_or_default();
    BASE64_STANDARD.encode(json_bytes)
}

/// Verify a license key string (Supports both Ed25519 and legacy HMAC fallback)
pub fn verify_license_key(
    key_str: &str,
    current_hwid: &str,
    secret_salt: &[u8],
) -> Result<LicenseClaims, LicenseError> {
    let trimmed = key_str.trim();
    if trimmed.is_empty() {
        return Err(LicenseError::InvalidFormat);
    }

    let decoded_bytes = BASE64_STANDARD
        .decode(trimmed)
        .map_err(|_| LicenseError::InvalidFormat)?;

    let token: LicenseToken =
        serde_json::from_slice(&decoded_bytes).map_err(|_| LicenseError::InvalidFormat)?;

    let canonical = serialize_canonical_claims(&token.claims);

    // 1. Verify Digital Signature
    if token.algorithm == "Ed25519" {
        let verifying_key = get_master_verifying_key();
        let sig_bytes = BASE64_STANDARD
            .decode(&token.signature)
            .map_err(|_| LicenseError::InvalidSignature)?;

        if sig_bytes.len() != 64 {
            return Err(LicenseError::InvalidSignature);
        }

        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_arr);

        if verifying_key.verify(canonical.as_bytes(), &signature).is_err() {
            return Err(LicenseError::InvalidSignature);
        }
    } else {
        // Fallback HMAC verification
        let expected_sig_bytes = compute_hmac_sha256(secret_salt, canonical.as_bytes());
        let expected_sig_hex = format!("{:X}", Sha256::digest(&expected_sig_bytes));

        if token.signature != expected_sig_hex {
            return Err(LicenseError::InvalidSignature);
        }
    }

    // 2. Verify HWID (Support UNIVERSAL or Exact Match)
    if token.claims.hwid != "UNIVERSAL" && token.claims.hwid != current_hwid {
        return Err(LicenseError::HwidMismatch);
    }

    // 3. Verify Expiry Date (-1 means lifetime)
    if token.claims.expires_at != -1 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if token.claims.expires_at <= now {
            return Err(LicenseError::Expired);
        }
    }

    Ok(token.claims)
}

/// Save license key string to disk
pub fn save_license_file(key_str: &str, base_path: Option<&Path>) -> Result<(), LicenseError> {
    let path = match base_path {
        Some(p) => p.join(LICENSE_FILE_PATH),
        None => PathBuf::from(LICENSE_FILE_PATH),
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&path, key_str.trim())
        .map_err(|e| LicenseError::IoError(format!("Failed to write license file: {}", e)))
}

/// Load license key string from disk
pub fn load_license_file(base_path: Option<&Path>) -> Option<String> {
    let path = match base_path {
        Some(p) => p.join(LICENSE_FILE_PATH),
        None => PathBuf::from(LICENSE_FILE_PATH),
    };

    if path.is_file() {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

pub const SERIAL_FILE_PATH: &str = "configs/serial.key";

/// Save serial key string (e.g. NHK-PRO-XXXX) to disk
pub fn save_serial_file(serial_str: &str, base_path: Option<&Path>) -> Result<(), LicenseError> {
    let path = match base_path {
        Some(p) => p.join(SERIAL_FILE_PATH),
        None => PathBuf::from(SERIAL_FILE_PATH),
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&path, serial_str.trim())
        .map_err(|e| LicenseError::IoError(format!("Failed to write serial file: {}", e)))
}

/// Load serial key string from disk
pub fn load_serial_file(base_path: Option<&Path>) -> Option<String> {
    let path = match base_path {
        Some(p) => p.join(SERIAL_FILE_PATH),
        None => PathBuf::from(SERIAL_FILE_PATH),
    };

    if path.is_file() {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

/// Remove license and serial files from disk (purge credentials upon key deletion or ban)
pub fn purge_license_files(base_path: Option<&Path>) {
    let lic_path = match base_path {
        Some(p) => p.join(LICENSE_FILE_PATH),
        None => PathBuf::from(LICENSE_FILE_PATH),
    };
    let serial_path = match base_path {
        Some(p) => p.join(SERIAL_FILE_PATH),
        None => PathBuf::from(SERIAL_FILE_PATH),
    };
    let _ = std::fs::remove_file(lic_path);
    let _ = std::fs::remove_file(serial_path);
}

/// Get complete diagnostic license status for the application
pub fn check_license_status(base_path: Option<&Path>) -> LicenseStatus {
    let current_hwid = crate::security::generate_hwid();
    let maybe_key = load_license_file(base_path);
    let active_serial = load_serial_file(base_path);

    let key_str = match maybe_key {
        Some(k) => k,
        None => {
            return LicenseStatus {
                is_valid: false,
                status: "Unlicensed".to_string(),
                tier: None,
                expires_at: None,
                days_remaining: None,
                hwid: current_hwid,
                active_key: active_serial,
            };
        }
    };

    if key_str.trim().eq_ignore_ascii_case("SUSPENDED") {
        return LicenseStatus {
            is_valid: false,
            status: "Suspended".to_string(),
            tier: None,
            expires_at: None,
            days_remaining: None,
            hwid: current_hwid,
            active_key: active_serial,
        };
    }

    match verify_license_key(&key_str, &current_hwid, DEFAULT_LICENSE_SALT) {
        Ok(claims) => {
            let days_remaining = if claims.expires_at == -1 {
                Some(9999) // Lifetime
            } else {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let diff_secs = claims.expires_at - now;
                Some((diff_secs / 86400).max(0))
            };

            LicenseStatus {
                is_valid: true,
                status: "Active".to_string(),
                tier: Some(claims.tier),
                expires_at: Some(claims.expires_at),
                days_remaining,
                hwid: current_hwid,
                active_key: active_serial,
            }
        }
        Err(LicenseError::Expired) => LicenseStatus {
            is_valid: false,
            status: "Expired".to_string(),
            tier: None,
            expires_at: None,
            days_remaining: Some(0),
            hwid: current_hwid,
            active_key: active_serial,
        },
        Err(LicenseError::HwidMismatch) => LicenseStatus {
            is_valid: false,
            status: "InvalidHWID".to_string(),
            tier: None,
            expires_at: None,
            days_remaining: None,
            hwid: current_hwid,
            active_key: active_serial,
        },
        Err(_) => LicenseStatus {
            is_valid: false,
            status: "Corrupted".to_string(),
            tier: None,
            expires_at: None,
            days_remaining: None,
            hwid: current_hwid,
            active_key: active_serial,
        },
    }
}

/// Query Cloudflare Worker to verify HWID against D1 and fetch signed Ed25519 token
pub async fn sync_license_from_server(
    worker_url: &str,
    hwid: &str,
    base_path: Option<&Path>,
) -> Result<LicenseClaims, LicenseError> {
    let endpoint = format!("{}/api/license/check", worker_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| LicenseError::IoError(format!("HTTP client build failed: {}", e)))?;

    let body = serde_json::json!({
        "hwid": hwid
    });

    let response = client
        .post(&endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| LicenseError::IoError(format!("Failed to connect to license server: {}", e)))?;

    if !response.status().is_success() {
        let status_code = response.status().as_u16();
        let err_json: serde_json::Value = response.json().await.unwrap_or_default();
        let err_msg = err_json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Server rejected license check");
        let lower_msg = err_msg.to_lowercase();
        if lower_msg.contains("suspend") {
            let _ = save_license_file("SUSPENDED", base_path);
        } else if status_code == 404 || lower_msg.contains("banned") || lower_msg.contains("revoked") || lower_msg.contains("not registered") {
            purge_license_files(base_path);
        }
        return Err(LicenseError::IoError(err_msg.to_string()));
    }

    let res_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| LicenseError::IoError(format!("Invalid server JSON response: {}", e)))?;

    let token_str = res_json
        .get("licenseKey")
        .and_then(|v| v.as_str())
        .ok_or(LicenseError::InvalidFormat)?;

    // Verify the returned Ed25519 token against local master public key
    let claims = verify_license_key(token_str, hwid, DEFAULT_LICENSE_SALT)?;

    // Save token to disk for offline usage
    let _ = save_license_file(token_str, base_path);

    Ok(claims)
}

/// Generate a random Serial / Redeem Key (e.g. NHK-PRO-4A2B-8C1D-E9F0)
pub fn generate_random_serial_key(tier: &str) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let p1: u32 = rng.gen_range(0x1000..=0xFFFF);
    let p2: u32 = rng.gen_range(0x1000..=0xFFFF);
    let p3: u32 = rng.gen_range(0x1000..=0xFFFF);
    format!("NHK-{}-{:04X}-{:04X}-{:04X}", tier.to_uppercase(), p1, p2, p3)
}

/// Redeem a Serial/Product Key from Cloudflare Worker, binding it to the machine's HWID
pub async fn redeem_license_from_server(
    worker_url: &str,
    redeem_key: &str,
    hwid: &str,
    base_path: Option<&Path>,
) -> Result<LicenseClaims, LicenseError> {
    let endpoint = format!("{}/api/license/redeem", worker_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| LicenseError::IoError(format!("HTTP client build failed: {}", e)))?;

    let body = serde_json::json!({
        "key": redeem_key.trim(),
        "hwid": hwid.trim()
    });

    let response = client
        .post(&endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| LicenseError::IoError(format!("Failed to connect to license server: {}", e)))?;

    if !response.status().is_success() {
        let err_json: serde_json::Value = response.json().await.unwrap_or_default();
        let err_msg = err_json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Server rejected license redemption");
        return Err(LicenseError::IoError(err_msg.to_string()));
    }

    let res_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| LicenseError::IoError(format!("Invalid server JSON response: {}", e)))?;

    let token_str = res_json
        .get("licenseKey")
        .and_then(|v| v.as_str())
        .ok_or(LicenseError::InvalidFormat)?;

    // Verify the returned Ed25519 token against local master public key
    let claims = verify_license_key(token_str, hwid, DEFAULT_LICENSE_SALT)?;

    // Save token to disk for offline usage
    let _ = save_license_file(token_str, base_path);
    let _ = save_serial_file(redeem_key, base_path);

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_keypair_derivation() {
        let signing = get_master_signing_key();
        let verifying = get_master_verifying_key();
        assert_eq!(signing.verifying_key(), verifying);
    }

    #[test]
    fn test_ed25519_license_lifecycle_valid() {
        let hwid = "NHKV4-ED25519-TEST-1234";
        let claims = LicenseClaims {
            hwid: hwid.to_string(),
            tier: "PRO_LIFETIME".to_string(),
            expires_at: -1,
            issued_at: 1700000000,
            user_id: Some("vip_001".to_string()),
        };

        let signing_key = get_master_signing_key();
        let key = generate_license_key_ed25519(&claims, &signing_key);
        assert!(!key.is_empty());

        let verified = verify_license_key(&key, hwid, DEFAULT_LICENSE_SALT).unwrap();
        assert_eq!(verified.hwid, hwid);
        assert_eq!(verified.tier, "PRO_LIFETIME");
    }

    #[test]
    fn test_ed25519_license_tampered() {
        let hwid = "NHKV4-ED25519-TAMPER";
        let claims = LicenseClaims {
            hwid: hwid.to_string(),
            tier: "TRIAL".to_string(),
            expires_at: -1,
            issued_at: 1700000000,
            user_id: None,
        };

        // Sign with a DIFFERENT random signing key
        let fake_seed = [9u8; 32];
        let fake_signing_key = SigningKey::from_bytes(&fake_seed);
        let key = generate_license_key_ed25519(&claims, &fake_signing_key);

        // Verification against master public key MUST FAIL
        let err = verify_license_key(&key, hwid, DEFAULT_LICENSE_SALT).unwrap_err();
        assert_eq!(err, LicenseError::InvalidSignature);
    }

    #[test]
    fn test_legacy_hmac_still_supported() {
        let hwid = "NHKV4-LEGACY-HMAC";
        let claims = LicenseClaims {
            hwid: hwid.to_string(),
            tier: "LEGACY".to_string(),
            expires_at: -1,
            issued_at: 1700000000,
            user_id: None,
        };

        let key = generate_license_key(&claims, DEFAULT_LICENSE_SALT);
        let verified = verify_license_key(&key, hwid, DEFAULT_LICENSE_SALT).unwrap();
        assert_eq!(verified.hwid, hwid);
    }
}
