use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
pub const APP_NAME: &str = "Amni Browse";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_HOME: &str = "amnibrowse://newtab";
pub const DEFAULT_SEARCH_ENGINE: &str = "https://duckduckgo.com/?q=";
pub const USER_AGENT: &str = "AmniBrowse/0.3 (Privacy-First; Amni-Scient)";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SplitMode { None, Horizontal, Vertical }
impl Default for SplitMode { fn default() -> Self { Self::None } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub home_page: String,
    pub search_engine: String,
    pub block_ads: bool,
    pub block_trackers: bool,
    pub block_third_party_cookies: bool,
    pub enable_do_not_track: bool,
    pub enable_javascript: bool,
    pub clear_data_on_exit: bool,
    pub custom_user_agent: Option<String>,
    pub enable_webxr: bool,
    pub default_split_mode: SplitMode,
    pub clear_cache_on_exit: bool,
    pub clear_cookies_on_exit: bool,
    pub clear_history_on_exit: bool,
    pub clear_passwords_on_exit: bool,
    pub restore_session: bool,
    pub enable_doh: bool,
    pub doh_provider: String,
    pub default_zoom: f64,
    pub enable_reader_mode: bool,
    pub downloads_dir: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            home_page: DEFAULT_HOME.into(),
            search_engine: DEFAULT_SEARCH_ENGINE.into(),
            block_ads: true,
            block_trackers: true,
            block_third_party_cookies: true,
            enable_do_not_track: true,
            enable_javascript: true,
            clear_data_on_exit: false,
            custom_user_agent: None,
            enable_webxr: true,
            default_split_mode: SplitMode::None,
            clear_cache_on_exit: false,
            clear_cookies_on_exit: false,
            clear_history_on_exit: false,
            clear_passwords_on_exit: false,
            restore_session: true,
            enable_doh: false,
            doh_provider: "cloudflare".into(),
            default_zoom: 1.0,
            enable_reader_mode: true,
            downloads_dir: None,
        }
    }
}

impl BrowserConfig {
    pub fn config_dir() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("amni-browse");
        fs::create_dir_all(&dir).ok();
        dir
    }
    pub fn cache_dir() -> PathBuf {
        let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("amni-browse");
        fs::create_dir_all(&dir).ok();
        dir
    }
    pub fn load() -> Self {
        let path = Self::config_dir().join("config.json");
        path.exists().then(|| {
            fs::read_to_string(&path).ok().and_then(|d| serde_json::from_str(&d).ok())
        }).flatten().unwrap_or_else(|| { let c = Self::default(); c.save(); c })
    }
    pub fn save(&self) {
        let path = Self::config_dir().join("config.json");
        serde_json::to_string_pretty(self).ok().map(|d| fs::write(&path, d).ok());
    }
    pub fn clear_data(&self) {
        let (config_dir, cache_dir) = (Self::config_dir(), Self::cache_dir());
        if self.clear_cache_on_exit || self.clear_data_on_exit {
            if cache_dir.exists() { fs::remove_dir_all(&cache_dir).ok(); fs::create_dir_all(&cache_dir).ok(); }
        }
        if self.clear_cookies_on_exit || self.clear_data_on_exit { fs::remove_file(config_dir.join("cookies.json")).ok(); }
        if self.clear_history_on_exit || self.clear_data_on_exit { fs::remove_file(config_dir.join("history.json")).ok(); }
        if self.clear_passwords_on_exit || self.clear_data_on_exit { fs::remove_file(config_dir.join("vault.enc.json")).ok(); }
    }
    pub fn clear_all_data_now() {
        let (config_dir, cache_dir) = (Self::config_dir(), Self::cache_dir());
        if cache_dir.exists() { fs::remove_dir_all(&cache_dir).ok(); fs::create_dir_all(&cache_dir).ok(); }
        ["cookies.json", "history.json", "downloads.json", "autofill.json", "permissions.json"]
            .iter().for_each(|f| { fs::remove_file(config_dir.join(f)).ok(); });
    }
}
