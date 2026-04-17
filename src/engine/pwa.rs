use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayMode { Fullscreen, Standalone, MinimalUi, Browser }

impl Default for DisplayMode {
    fn default() -> Self { DisplayMode::Browser }
}

impl<'de> Deserialize<'de> for DisplayMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "fullscreen" => Ok(DisplayMode::Fullscreen),
            "standalone" => Ok(DisplayMode::Standalone),
            "minimal-ui" => Ok(DisplayMode::MinimalUi),
            "browser" => Ok(DisplayMode::Browser),
            _ => Ok(DisplayMode::Browser),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestIcon {
    pub src: String,
    pub sizes: Option<String>,
    #[serde(rename = "type")]
    pub icon_type: Option<String>,
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebAppManifest {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub start_url: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub display: DisplayMode,
    pub orientation: Option<String>,
    pub theme_color: Option<String>,
    pub background_color: Option<String>,
    #[serde(default)]
    pub icons: Vec<ManifestIcon>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstalledPwa {
    pub id: String,
    pub manifest: WebAppManifest,
    pub installed_url: String,
    pub installed_at: u64,
}

pub struct PwaManager {
    installed_apps: HashMap<String, InstalledPwa>,
}

impl PwaManager {
    pub fn new() -> Self { Self { installed_apps: HashMap::new() } }

    pub fn parse_manifest(json: &str) -> Result<WebAppManifest, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn detect_manifest_link(html: &str) -> Option<String> {
        let lower = html.to_lowercase();
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find("<link") {
            let abs = search_from + pos;
            let end = match lower[abs..].find('>') {
                Some(e) => abs + e + 1,
                None => break,
            };
            let tag = &html[abs..end];
            let tag_lower = &lower[abs..end];
            if tag_lower.contains("rel=\"manifest\"") || tag_lower.contains("rel='manifest'") {
                if let Some(href_pos) = tag_lower.find("href=") {
                    let after = &tag[href_pos + 5..];
                    let quote = after.chars().next()?;
                    if quote == '"' || quote == '\'' {
                        let rest = &after[1..];
                        if let Some(close) = rest.find(quote) {
                            return Some(rest[..close].to_string());
                        }
                    }
                }
            }
            search_from = end;
        }
        None
    }

    pub fn install_app(&mut self, manifest: &WebAppManifest, url: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let pwa = InstalledPwa {
            id: id.clone(),
            manifest: manifest.clone(),
            installed_url: url.to_string(),
            installed_at: now,
        };
        self.installed_apps.insert(url.to_string(), pwa);
        id
    }

    pub fn uninstall_app(&mut self, id: &str) -> bool {
        let key = self.installed_apps.iter()
            .find(|(_, v)| v.id == id)
            .map(|(k, _)| k.clone());
        match key {
            Some(k) => { self.installed_apps.remove(&k); true }
            None => false,
        }
    }

    pub fn is_installed(&self, url: &str) -> bool {
        self.installed_apps.contains_key(url)
    }

    pub fn list_installed(&self) -> Vec<&InstalledPwa> {
        self.installed_apps.values().collect()
    }

    pub fn get_display_mode(&self, url: &str) -> DisplayMode {
        self.installed_apps.get(url)
            .map(|p| p.manifest.display.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let json = r##"{
            "name": "My App",
            "short_name": "App",
            "start_url": "/",
            "display": "standalone",
            "theme_color": "#ffffff",
            "icons": [{"src": "/icon.png", "sizes": "192x192", "type": "image/png"}]
        }"##;
        let m = PwaManager::parse_manifest(json).unwrap();
        assert_eq!(m.name, Some("My App".to_string()));
        assert_eq!(m.display, DisplayMode::Standalone);
        assert_eq!(m.icons.len(), 1);
        assert_eq!(m.icons[0].sizes, Some("192x192".to_string()));
    }

    #[test]
    fn test_detect_manifest_link() {
        let html = r#"<html><head><link rel="manifest" href="/manifest.json"><title>Test</title></head></html>"#;
        assert_eq!(PwaManager::detect_manifest_link(html), Some("/manifest.json".to_string()));
        assert_eq!(PwaManager::detect_manifest_link("<html><head></head></html>"), None);
    }

    #[test]
    fn test_install_uninstall() {
        let mut mgr = PwaManager::new();
        let json = r#"{"name":"Test","start_url":"/","display":"browser"}"#;
        let manifest = PwaManager::parse_manifest(json).unwrap();
        let id = mgr.install_app(&manifest, "https://example.com/");
        assert!(mgr.is_installed("https://example.com/"));
        assert_eq!(mgr.list_installed().len(), 1);
        assert!(mgr.uninstall_app(&id));
        assert!(!mgr.is_installed("https://example.com/"));
        assert!(!mgr.uninstall_app(&id));
    }

    #[test]
    fn test_get_display_mode() {
        let mut mgr = PwaManager::new();
        assert_eq!(mgr.get_display_mode("https://example.com/"), DisplayMode::Browser);
        let json = r#"{"name":"App","display":"fullscreen"}"#;
        let manifest = PwaManager::parse_manifest(json).unwrap();
        mgr.install_app(&manifest, "https://example.com/");
        assert_eq!(mgr.get_display_mode("https://example.com/"), DisplayMode::Fullscreen);
    }

    #[test]
    fn test_parse_minimal_manifest() {
        let json = r#"{}"#;
        let m = PwaManager::parse_manifest(json).unwrap();
        assert_eq!(m.name, None);
        assert_eq!(m.display, DisplayMode::Browser);
        assert!(m.icons.is_empty());
        assert!(m.categories.is_empty());
    }

    #[test]
    fn test_detect_manifest_single_quotes() {
        let html = r#"<html><head><link rel='manifest' href='/app.webmanifest'></head></html>"#;
        assert_eq!(PwaManager::detect_manifest_link(html), Some("/app.webmanifest".to_string()));
    }
}
