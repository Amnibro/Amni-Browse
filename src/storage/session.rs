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
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub group: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub tabs: Vec<SessionTab>,
    pub window_width: f64,
    pub window_height: f64,
    #[serde(default)]
    pub window_x: Option<f64>,
    #[serde(default)]
    pub window_y: Option<f64>,
    #[serde(default)]
    pub maximized: bool,
    pub saved_at: DateTime<Utc>,
    pub was_clean_exit: bool,
}
impl Default for SessionState {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            window_width: 1400.0,
            window_height: 900.0,
            window_x: None,
            window_y: None,
            maximized: false,
            saved_at: Utc::now(),
            was_clean_exit: true,
        }
    }
}
impl SessionTab {
    pub fn is_media(&self) -> bool { self.engine == "media" }
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
        let p = Self::lock_path();
        if !p.exists() { return false; }
        let age = fs::metadata(&p).ok().and_then(|m| m.modified().ok()).and_then(|t| t.elapsed().ok()).map(|d| d.as_secs()).unwrap_or(0);
        age < 86_400
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_else(|_| "{}".to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_session_json_defaults_engine() {
        let raw = r#"{"url":"https://a.test","title":"A","is_active":true,"history":["https://a.test"],"history_index":0}"#;
        let t: SessionTab = serde_json::from_str(raw).unwrap();
        assert_eq!(t.engine, "");
        assert!(!t.is_media());
    }
    #[test]
    fn media_engine_flag() {
        let t = SessionTab { url: "https://www.netflix.com/".into(), title: "n".into(), is_active: true, history: vec![], history_index: 0, engine: "media".into(), pinned: false, group: None };
        assert!(t.is_media());
    }
}
