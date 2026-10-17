use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwidDetails {
    pub hwid: String,
    pub machine_guid: String,
    pub processor: String,
    pub os_info: String,
}

/// Retrieve the unique MachineGuid from Windows Registry
#[cfg(windows)]
fn read_windows_machine_guid() -> Option<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(crypto) = hklm.open_subkey("SOFTWARE\\Microsoft\\Cryptography") {
        if let Ok(guid) = crypto.get_value::<String, _>("MachineGuid") {
            let trimmed = guid.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn read_windows_machine_guid() -> Option<String> {
    None
}

/// Fallback or cross-platform Machine ID detection
fn read_cross_platform_id() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                if line.contains("IOPlatformUUID") {
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 4 {
                        let uuid = parts[3].trim().to_string();
                        if !uuid.is_empty() {
                            return uuid;
                        }
                    }
                }
            }
        }
    }

    // Windows native registry fallback or non-windows fallback
    if let Some(guid) = read_windows_machine_guid() {
        return guid;
    }

    // Environmental fallback
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "DEFAULT_USER".to_string());
    let domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| "LOCAL_DOMAIN".to_string());
    format!("{}-{}", domain, user)
}

/// Get hardware processor identifier
fn get_processor_info() -> String {
    std::env::var("PROCESSOR_IDENTIFIER")
        .or_else(|_| std::env::var("PROCESSOR_ARCHITECTURE"))
        .unwrap_or_else(|_| "GENERIC_X86_64".to_string())
}

/// Get computer name or host
fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "NODEHOTKEY_HOST".to_string())
}

/// Generate a standardized HWID formatted as: NHKV4-XXXX-XXXX-XXXX-XXXX
pub fn generate_hwid() -> String {
    let machine_id = read_cross_platform_id();
    let processor = get_processor_info();
    let computer_name = get_computer_name();

    let raw_payload = format!(
        "NHKV4_SALT_V1|{}|{}|{}|WINDOWS_MACHINE_ID",
        machine_id, processor, computer_name
    );

    let mut hasher = Sha256::new();
    hasher.update(raw_payload.as_bytes());
    let hash = hasher.finalize();
    let hex_str = format!("{:X}", hash);

    // Format as 4 groups of 4 uppercase alphanumeric characters: NHKV4-XXXX-XXXX-XXXX-XXXX
    let part1 = &hex_str[0..4];
    let part2 = &hex_str[4..8];
    let part3 = &hex_str[8..12];
    let part4 = &hex_str[12..16];

    format!("NHKV4-{}-{}-{}-{}", part1, part2, part3, part4)
}

/// Retrieve detailed HWID info for API diagnostics and Launcher display
pub fn get_hwid_details() -> HwidDetails {
    let hwid = generate_hwid();
    let machine_guid = read_cross_platform_id();
    let processor = get_processor_info();
    let os_info = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);

    HwidDetails {
        hwid,
        machine_guid,
        processor,
        os_info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hwid_generation_format() {
        let hwid = generate_hwid();
        assert!(hwid.starts_with("NHKV4-"));
        let parts: Vec<&str> = hwid.split('-').collect();
        assert_eq!(parts.len(), 5, "HWID must have 5 segments separated by dash");
        for segment in &parts[1..] {
            assert_eq!(segment.len(), 4, "Each hash segment must be 4 characters");
        }
    }

    #[test]
    fn test_hwid_deterministic() {
        let hwid1 = generate_hwid();
        let hwid2 = generate_hwid();
        assert_eq!(hwid1, hwid2, "HWID must be deterministic on the same machine");
    }

    #[test]
    fn test_hwid_details() {
        let details = get_hwid_details();
        assert!(!details.hwid.is_empty());
        assert!(!details.machine_guid.is_empty());
        assert!(!details.processor.is_empty());
    }
}
