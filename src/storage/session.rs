use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTab {
    pub url: String,
    pub title: String,
    pub is_active: bool,
    pub history: Vec<String>,
    pub history_index: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub tabs: Vec<SessionTab>,
    pub window_width: f64,
    pub window_height: f64,
    pub saved_at: DateTime<Utc>,
    pub was_clean_exit: bool,
}
impl Default for SessionState {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            window_width: 1400.0,
            window_height: 900.0,
            saved_at: Utc::now(),
            was_clean_exit: true,
        }
    }
}
#[derive(Debug)]
pub struct SessionManager {
    pub state: SessionState,
    pub restore_on_start: bool,
}
impl SessionManager {
    pub fn new(restore: bool) -> Self {
        Self { state: SessionState::default(), restore_on_start: restore }
    }
    fn session_path() -> PathBuf { BrowserConfig::config_dir().join("session.json") }
    fn lock_path() -> PathBuf { BrowserConfig::config_dir().join("session.lock") }
    pub fn capture(&mut self, tabs: Vec<SessionTab>) {
        self.state.tabs = tabs;
        self.state.saved_at = Utc::now();
    }
    pub fn get_restore_data(&self) -> Vec<SessionTab> {
        Self::load().map(|s| s.tabs).unwrap_or_default()
    }
    pub fn save(&self) {
        let path = Self::session_path();
        serde_json::to_string_pretty(&self.state).ok().map(|d| fs::write(&path, d).ok());
    }
    pub fn save_clean_exit(&mut self) {
        self.state.was_clean_exit = true;
        self.save();
        fs::remove_file(Self::lock_path()).ok();
    }
    pub fn load() -> Option<SessionState> {
        let path = Self::session_path();
        path.exists().then(|| {
            fs::read_to_string(&path).ok().and_then(|d| serde_json::from_str(&d).ok())
        }).flatten()
    }
    pub fn create_lock() {
        let lock = Self::lock_path();
        fs::write(&lock, Utc::now().to_rfc3339()).ok();
    }
    pub fn was_crash() -> bool {
        Self::lock_path().exists()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_else(|_| "{}".to_string())
    }
}
