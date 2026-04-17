use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub permissions: Vec<String>,
    pub content_scripts: Vec<ContentScript>,
    pub background_script: Option<String>,
    pub icons: Option<ExtensionIcons>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScript {
    pub matches: Vec<String>,
    pub js: Vec<String>,
    pub css: Vec<String>,
    pub run_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionIcons {
    pub small: Option<String>,
    pub large: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub installed_at: String,
    pub path: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionRegistry {
    pub extensions: Vec<Extension>,
}
pub struct ExtensionManager {
    registry: ExtensionRegistry,
}
impl ExtensionManager {
    pub fn new() -> Self {
        Self { registry: Self::load_registry() }
    }
    fn extensions_dir() -> PathBuf {
        let dir = BrowserConfig::config_dir().join("extensions");
        fs::create_dir_all(&dir).ok();
        dir
    }
    fn registry_path() -> PathBuf { BrowserConfig::config_dir().join("extensions.json") }
    fn load_registry() -> ExtensionRegistry {
        let path = Self::registry_path();
        path.exists().then(|| {
            fs::read_to_string(&path).ok().and_then(|d| serde_json::from_str(&d).ok())
        }).flatten().unwrap_or_default()
    }
    fn save_registry(&self) {
        let path = Self::registry_path();
        serde_json::to_string_pretty(&self.registry).ok().map(|d| fs::write(&path, d).ok());
    }
    pub fn scan_extensions(&mut self) {
        let dir = Self::extensions_dir();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue; }
                let manifest_path = path.join("manifest.json");
                if !manifest_path.exists() { continue; }
                if let Ok(data) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<ExtensionManifest>(&data) {
                        if !self.registry.extensions.iter().any(|e| e.manifest.id == manifest.id) {
                            self.registry.extensions.push(Extension {
                                manifest, enabled: true,
                                installed_at: chrono::Utc::now().to_rfc3339(),
                                path,
                            });
                        }
                    }
                }
            }
        }
        self.save_registry();
    }
    pub fn install_from_dir(&mut self, source: &PathBuf) -> Result<String, String> {
        let manifest_path = source.join("manifest.json");
        if !manifest_path.exists() { return Err("No manifest.json found".into()); }
        let data = fs::read_to_string(&manifest_path).map_err(|e| format!("Read err: {}", e))?;
        let manifest: ExtensionManifest = serde_json::from_str(&data).map_err(|e| format!("Parse err: {}", e))?;
        let ext_dir = Self::extensions_dir().join(&manifest.id);
        if ext_dir.exists() { return Err("Extension already installed".into()); }
        Self::copy_dir_recursive(source, &ext_dir).map_err(|e| format!("Copy err: {}", e))?;
        let id = manifest.id.clone();
        self.registry.extensions.push(Extension {
            manifest, enabled: true,
            installed_at: chrono::Utc::now().to_rfc3339(),
            path: ext_dir,
        });
        self.save_registry();
        Ok(id)
    }
    fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }
    pub fn enable(&mut self, id: &str) {
        self.registry.extensions.iter_mut().filter(|e| e.manifest.id == id).for_each(|e| e.enabled = true);
        self.save_registry();
    }
    pub fn disable(&mut self, id: &str) {
        self.registry.extensions.iter_mut().filter(|e| e.manifest.id == id).for_each(|e| e.enabled = false);
        self.save_registry();
    }
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(ext) = self.registry.extensions.iter().find(|e| e.manifest.id == id) {
            fs::remove_dir_all(&ext.path).ok();
        }
        let before = self.registry.extensions.len();
        self.registry.extensions.retain(|e| e.manifest.id != id);
        let removed = self.registry.extensions.len() < before;
        if removed { self.save_registry(); }
        removed
    }
    pub fn get_content_scripts(&self, url: &str) -> Vec<(String, Vec<String>, Vec<String>)> {
        self.registry.extensions.iter()
            .filter(|e| e.enabled)
            .flat_map(|e| e.manifest.content_scripts.iter().map(move |cs| (e, cs)))
            .filter(|(_, cs)| cs.matches.iter().any(|m| Self::url_matches_pattern(url, m)))
            .map(|(e, cs)| {
                let js: Vec<String> = cs.js.iter().filter_map(|f| {
                    fs::read_to_string(e.path.join(f)).ok()
                }).collect();
                let css: Vec<String> = cs.css.iter().filter_map(|f| {
                    fs::read_to_string(e.path.join(f)).ok()
                }).collect();
                (e.manifest.id.clone(), js, css)
            })
            .collect()
    }
    fn url_matches_pattern(url: &str, pattern: &str) -> bool {
        pattern == "<all_urls>" || url.contains(pattern.trim_matches('*'))
    }
    pub fn list(&self) -> &[Extension] { &self.registry.extensions }
    pub fn to_json(&self) -> String {
        let summaries: Vec<ExtSummary> = self.registry.extensions.iter().map(|e| ExtSummary {
            id: e.manifest.id.clone(), name: e.manifest.name.clone(),
            version: e.manifest.version.clone(), description: e.manifest.description.clone(),
            enabled: e.enabled, author: e.manifest.author.clone(),
        }).collect();
        serde_json::to_string(&summaries).unwrap_or_else(|_| "[]".into())
    }
}
#[derive(Serialize)]
struct ExtSummary {
    id: String, name: String, version: String, description: String,
    enabled: bool, author: Option<String>,
}
