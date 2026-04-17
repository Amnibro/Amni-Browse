use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub avatar_color: String,
    pub created_at: String,
    pub is_default: bool,
}
impl Profile {
    pub fn new(name: &str, color: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            avatar_color: color.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            is_default: false,
        }
    }
    pub fn default_profile() -> Self {
        Self {
            id: "default".into(),
            name: "Default".into(),
            avatar_color: "#00d4ff".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            is_default: true,
        }
    }
    pub fn data_dir(&self) -> PathBuf {
        let base = BrowserConfig::config_dir();
        if self.is_default { base } else { base.join("profiles").join(&self.id) }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManager {
    pub profiles: Vec<Profile>,
    pub active_id: String,
}
impl Default for ProfileManager {
    fn default() -> Self {
        Self {
            profiles: vec![Profile::default_profile()],
            active_id: "default".into(),
        }
    }
}
impl ProfileManager {
    pub fn new() -> Self { Self::load() }
    fn file_path() -> PathBuf { BrowserConfig::config_dir().join("profiles.json") }
    fn load() -> Self {
        let path = Self::file_path();
        path.exists().then(|| {
            fs::read_to_string(&path).ok().and_then(|d| serde_json::from_str(&d).ok())
        }).flatten().unwrap_or_default()
    }
    pub fn save(&self) {
        let path = Self::file_path();
        serde_json::to_string_pretty(self).ok().map(|d| fs::write(&path, d).ok());
    }
    pub fn create_profile(&mut self, name: &str, color: &str) -> String {
        let profile = Profile::new(name, color);
        let id = profile.id.clone();
        let dir = profile.data_dir();
        fs::create_dir_all(&dir).ok();
        self.profiles.push(profile);
        self.save();
        id
    }
    pub fn delete_profile(&mut self, id: &str) -> bool {
        if id == "default" { return false; }
        let profile = self.profiles.iter().find(|p| p.id == id).cloned();
        if let Some(p) = profile {
            fs::remove_dir_all(p.data_dir()).ok();
        }
        let before = self.profiles.len();
        self.profiles.retain(|p| p.id != id);
        let removed = self.profiles.len() < before;
        if removed {
            if self.active_id == id { self.active_id = "default".into(); }
            self.save();
        }
        removed
    }
    pub fn switch_profile(&mut self, id: &str) -> bool {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_id = id.into();
            self.save();
            true
        } else { false }
    }
    pub fn active_profile(&self) -> &Profile {
        self.profiles.iter().find(|p| p.id == self.active_id).unwrap_or(&self.profiles[0])
    }
    pub fn rename_profile(&mut self, id: &str, new_name: &str) {
        self.profiles.iter_mut().filter(|p| p.id == id).for_each(|p| p.name = new_name.into());
        self.save();
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.profiles).unwrap_or_else(|_| "[]".into())
    }
}
