//! Compile-time String Obfuscation & Secrets Protection
//!
//! Prevents sensitive strings (API URLs, secret keys, license endpoints)
//! from appearing as plaintext in the compiled binary.

pub const DEFAULT_CIPHER_KEY: &[u8] = b"NHKV4_CORE_SECURITY_KEY_2026";

/// Deobfuscate byte slice using rolling XOR key in memory at runtime
pub fn deobfuscate_bytes(encrypted: &[u8], key: &[u8]) -> String {
    if encrypted.is_empty() || key.is_empty() {
        return String::new();
    }
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();

    String::from_utf8(decrypted).unwrap_or_default()
}

/// Macro to obfuscate string literal at compile time and deobfuscate at runtime
#[macro_export]
macro_rules! secure_str {
    ($s:literal) => {{
        const KEY: &[u8] = $crate::security::obfuscate::DEFAULT_CIPHER_KEY;
        const RAW_BYTES: &[u8] = $s.as_bytes();
        const LEN: usize = RAW_BYTES.len();
        const ENCRYPTED: [u8; LEN] = {
            let mut arr = [0u8; LEN];
            let mut i = 0;
            while i < LEN {
                arr[i] = RAW_BYTES[i] ^ KEY[i % KEY.len()];
                i += 1;
            }
            arr
        };
        $crate::security::obfuscate::deobfuscate_bytes(&ENCRYPTED, KEY)
    }};
    ($s:literal, $custom_key:literal) => {{
        const KEY: &[u8] = $custom_key.as_bytes();
        const RAW_BYTES: &[u8] = $s.as_bytes();
        const LEN: usize = RAW_BYTES.len();
        const ENCRYPTED: [u8; LEN] = {
            let mut arr = [0u8; LEN];
            let mut i = 0;
            while i < LEN {
                arr[i] = RAW_BYTES[i] ^ KEY[i % KEY.len()];
                i += 1;
            }
            arr
        };
        $crate::security::obfuscate::deobfuscate_bytes(&ENCRYPTED, KEY)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deobfuscate_bytes() {
        let original = "https://api.nodehotkey.com/v1/license";
        let key = b"SECRET_KEY_123";

        // Manually encrypt
        let encrypted: Vec<u8> = original
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();

        // Must not equal original in binary
        assert_ne!(encrypted.as_slice(), original.as_bytes());

        // Deobfuscate
        let decrypted = deobfuscate_bytes(&encrypted, key);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_secure_str_macro_default_key() {
        let secret = secure_str!("DATABASE_MASTER_URL=postgresql://secret");
        assert_eq!(secret, "DATABASE_MASTER_URL=postgresql://secret");
    }

    #[test]
    fn test_secure_str_macro_custom_key() {
        let endpoint = secure_str!("https://gateway.nodehotkey.internal/auth", "MY_CUSTOM_SALT");
        assert_eq!(endpoint, "https://gateway.nodehotkey.internal/auth");
    }

    #[test]
    fn test_secure_str_macro_empty() {
        let empty = secure_str!("");
        assert_eq!(empty, "");
    }
}
