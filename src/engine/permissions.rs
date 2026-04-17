use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionState {
    Allow,
    Deny,
    Ask,
}
impl Default for PermissionState {
    fn default() -> Self { Self::Ask }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionType {
    Camera,
    Microphone,
    Location,
    Notifications,
    Clipboard,
    Fullscreen,
    Autoplay,
    Popups,
}
impl std::fmt::Display for PermissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Camera => write!(f, "Camera"),
            Self::Microphone => write!(f, "Microphone"),
            Self::Location => write!(f, "Location"),
            Self::Notifications => write!(f, "Notifications"),
            Self::Clipboard => write!(f, "Clipboard"),
            Self::Fullscreen => write!(f, "Fullscreen"),
            Self::Autoplay => write!(f, "Autoplay"),
            Self::Popups => write!(f, "Popups"),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePermissions {
    pub site: String,
    pub permissions: HashMap<PermissionType, PermissionState>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionsManager {
    pub sites: Vec<SitePermissions>,
    pub defaults: HashMap<PermissionType, PermissionState>,
}
impl PermissionsManager {
    pub fn new() -> Self {
        let mut mgr = Self::load();
        if mgr.defaults.is_empty() {
            mgr.defaults.insert(PermissionType::Camera, PermissionState::Ask);
            mgr.defaults.insert(PermissionType::Microphone, PermissionState::Ask);
            mgr.defaults.insert(PermissionType::Location, PermissionState::Ask);
            mgr.defaults.insert(PermissionType::Notifications, PermissionState::Ask);
            mgr.defaults.insert(PermissionType::Clipboard, PermissionState::Ask);
            mgr.defaults.insert(PermissionType::Fullscreen, PermissionState::Allow);
            mgr.defaults.insert(PermissionType::Autoplay, PermissionState::Deny);
            mgr.defaults.insert(PermissionType::Popups, PermissionState::Deny);
        }
        mgr
    }
    fn file_path() -> PathBuf { BrowserConfig::config_dir().join("permissions.json") }
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
    fn normalize_site(url: &str) -> String {
        url::Url::parse(url).ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| url.to_string())
    }
    pub fn get_permission(&self, url: &str, perm: &PermissionType) -> PermissionState {
        let site = Self::normalize_site(url);
        self.sites.iter()
            .find(|s| s.site == site)
            .and_then(|s| s.permissions.get(perm))
            .cloned()
            .unwrap_or_else(|| self.defaults.get(perm).cloned().unwrap_or_default())
    }
    pub fn set_permission(&mut self, url: &str, perm: PermissionType, state: PermissionState) {
        let site = Self::normalize_site(url);
        match self.sites.iter_mut().find(|s| s.site == site) {
            Some(sp) => { sp.permissions.insert(perm, state); }
            None => {
                let mut perms = HashMap::new();
                perms.insert(perm, state);
                self.sites.push(SitePermissions { site, permissions: perms });
            }
        }
        self.save();
    }
    pub fn reset_site(&mut self, url: &str) {
        let site = Self::normalize_site(url);
        self.sites.retain(|s| s.site != site);
        self.save();
    }
    pub fn reset_all(&mut self) {
        self.sites.clear();
        self.save();
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.sites).unwrap_or_else(|_| "[]".into())
    }
    pub fn defaults_json(&self) -> String {
        serde_json::to_string(&self.defaults).unwrap_or_else(|_| "{}".into())
    }
}
