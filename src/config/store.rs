use crate::model::{GlobalConfig, Profile};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-safe Configuration Store with In-Memory Caching & SHA1 Invalidation
#[derive(Debug, Clone)]
pub struct ConfigStore {
    pub base_dir: PathBuf,
    profiles_dir: PathBuf,
    global_config_path: PathBuf,
    cached_global: Arc<RwLock<Option<GlobalConfig>>>,
    cached_profiles: Arc<RwLock<HashMap<String, Profile>>>,
}

impl ConfigStore {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        let input = base_dir.as_ref();
        let base = if input.is_absolute() {
            input.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(input)
        };
        let configs_dir = base.join("configs");
        let profiles_dir = configs_dir.join("profiles");
        let global_config_path = configs_dir.join("global.json");

        // Ensure directories exist
        let _ = fs::create_dir_all(&profiles_dir);

        Self {
            base_dir: base,
            profiles_dir,
            global_config_path,
            cached_global: Arc::new(RwLock::new(None)),
            cached_profiles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Read global.json with fallback to default
    pub async fn get_global_config(&self) -> GlobalConfig {
        {
            let read_lock = self.cached_global.read().await;
            if let Some(cfg) = &*read_lock {
                return cfg.clone();
            }
        }

        let cfg = self.load_global_config_from_disk();
        let mut write_lock = self.cached_global.write().await;
        *write_lock = Some(cfg.clone());
        cfg
    }

    fn load_global_config_from_disk(&self) -> GlobalConfig {
        if self.global_config_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.global_config_path) {
                if let Ok(cfg) = serde_json::from_str::<GlobalConfig>(&content) {
                    return cfg;
                }
            }
        }
        GlobalConfig::default()
    }

    /// Save global.json
    pub async fn save_global_config(&self, cfg: &GlobalConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
        fs::write(&self.global_config_path, content).map_err(|e| e.to_string())?;

        let mut write_lock = self.cached_global.write().await;
        *write_lock = Some(cfg.clone());
        Ok(())
    }

    /// Read all profiles from configs/profiles/*.json
    pub async fn get_all_profiles(&self) -> Vec<Profile> {
        {
            let read_lock = self.cached_profiles.read().await;
            if !read_lock.is_empty() {
                return read_lock.values().cloned().collect();
            }
        }

        let profiles = self.load_all_profiles_from_disk();
        let mut write_lock = self.cached_profiles.write().await;
        for p in &profiles {
            write_lock.insert(p.name.clone(), p.clone());
        }
        profiles
    }

    fn load_all_profiles_from_disk(&self) -> Vec<Profile> {
        let mut profiles = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(mut profile) = serde_json::from_str::<Profile>(&content) {
                            profile.normalize();
                            profile.sanitize_profile_keys();
                            profiles.push(profile);
                        }
                    }
                }
            }
        }
        profiles
    }

    /// Read single profile by name
    pub async fn get_profile(&self, name: &str) -> Option<Profile> {
        {
            let read_lock = self.cached_profiles.read().await;
            if let Some(p) = read_lock.get(name) {
                return Some(p.clone());
            }
        }

        let sanitized = sanitize_filename(name);
        let path = self.profiles_dir.join(format!("{}.json", sanitized));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut profile) = serde_json::from_str::<Profile>(&content) {
                    profile.normalize();
                    profile.sanitize_profile_keys();
                    let mut write_lock = self.cached_profiles.write().await;
                    write_lock.insert(name.to_string(), profile.clone());
                    return Some(profile);
                }
            }
        }
        None
    }

    /// Save a profile to configs/profiles/<name>.json
    pub async fn save_profile(&self, profile: &Profile) -> Result<(), String> {
        let mut clean_prof = profile.clone();
        clean_prof.normalize();
        clean_prof.sanitize_profile_keys();
        let sanitized = sanitize_filename(&clean_prof.name);
        let path = self.profiles_dir.join(format!("{}.json", sanitized));
        let content = serde_json::to_string_pretty(&clean_prof).map_err(|e| e.to_string())?;

        fs::write(&path, content).map_err(|e| e.to_string())?;

        let mut write_lock = self.cached_profiles.write().await;
        write_lock.insert(clean_prof.name.clone(), clean_prof);
        Ok(())
    }

    /// Delete a profile from disk
    pub async fn delete_profile(&self, name: &str) -> Result<(), String> {
        let sanitized = sanitize_filename(name);
        let path = self.profiles_dir.join(format!("{}.json", sanitized));
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        let mut write_lock = self.cached_profiles.write().await;
        write_lock.remove(name);
        Ok(())
    }

    /// Rename a profile
    pub async fn rename_profile(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some(mut profile) = self.get_profile(old_name).await {
            profile.name = new_name.to_string();
            self.delete_profile(old_name).await?;
            self.save_profile(&profile).await?;
            Ok(())
        } else {
            Err("Profile not found".to_string())
        }
    }

    /// Get or generate persistent Creator Identity
    pub async fn get_creator_identity(&self) -> crate::model::CreatorIdentity {
        let mut global = self.get_global_config().await;
        if let Some(ref id) = global.creator_identity {
            if !id.author_id.is_empty() {
                return id.clone();
            }
        }

        let new_id = crate::model::CreatorIdentity {
            author_id: format!("usr_{:012x}", rand::random::<u64>() & 0xffffffffffff),
            author_secret: format!("sec_{:032x}", rand::random::<u128>()),
            author_name: format!("User_{}", rand::random::<u16>() % 9000 + 1000),
        };

        global.creator_identity = Some(new_id.clone());
        let _ = self.save_global_config(&global).await;
        new_id
    }

    /// Update Creator Nickname
    pub async fn update_creator_identity(&self, author_name: &str) -> Result<crate::model::CreatorIdentity, String> {
        let mut global = self.get_global_config().await;
        let mut id = match global.creator_identity {
            Some(i) => i,
            None => self.get_creator_identity().await,
        };
        id.author_name = author_name.trim().to_string();
        global.creator_identity = Some(id.clone());
        self.save_global_config(&global).await?;
        Ok(id)
    }

    pub async fn invalidate_cache(&self) {
        let mut g_lock = self.cached_global.write().await;
        *g_lock = None;
        let mut p_lock = self.cached_profiles.write().await;
        p_lock.clear();
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | '?' | '%' | '*' | ':' | '|' | '"' | '<' | '>' => '_',
            other => other,
        })
        .collect()
}
