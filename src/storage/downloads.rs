use chrono::{DateTime, Utc};
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_path: PathBuf,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub mime_type: Option<String>,
    pub error: Option<String>,
}
impl DownloadItem {
    pub fn new(url: &str, filename: &str, save_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            filename: filename.to_string(),
            save_path,
            total_bytes: None,
            downloaded_bytes: 0,
            status: DownloadStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            mime_type: None,
            error: None,
        }
    }
    pub fn progress_pct(&self) -> f64 {
        self.total_bytes.map_or(0.0, |t| if t == 0 { 100.0 } else { (self.downloaded_bytes as f64 / t as f64) * 100.0 })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadManager {
    pub downloads: Vec<DownloadItem>,
    pub default_dir: Option<String>,
}
impl DownloadManager {
    pub fn new() -> Self {
        let mut mgr = Self::load();
        mgr.downloads.retain(|d| d.status == DownloadStatus::Completed || d.status == DownloadStatus::Failed);
        mgr
    }
    fn file_path() -> PathBuf {
        BrowserConfig::config_dir().join("downloads.json")
    }
    pub fn downloads_dir() -> PathBuf {
        let dir = dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        fs::create_dir_all(&dir).ok();
        dir
    }
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
    fn extract_filename(url: &str, content_disp: Option<&str>) -> String {
        if let Some(cd) = content_disp {
            if let Some(pos) = cd.find("filename=") {
                let name = cd[pos + 9..].trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                let name = name.split(';').next().unwrap_or(name).trim();
                if !name.is_empty() { return name.to_string(); }
            }
        }
        url::Url::parse(url).ok()
            .and_then(|u| u.path_segments()?.last().map(|s| s.to_string()))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("download_{}", &Uuid::new_v4().to_string()[..8]))
    }
    pub fn start_download(&mut self, url: &str) -> String {
        let filename = Self::extract_filename(url, None);
        let save_dir = self.default_dir.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(Self::downloads_dir);
        let save_path = save_dir.join(&filename);
        let mut item = DownloadItem::new(url, &filename, save_path);
        item.status = DownloadStatus::Downloading;
        let id = item.id.clone();
        let id_ret = id.clone();
        let url_owned = url.to_string();
        let path = item.save_path.clone();
        self.downloads.push(item);
        self.save();
        info!("📥 Download started: {} -> {:?}", filename, path);
        let _ = std::thread::spawn(move || {
            Self::download_blocking(&id, &url_owned, &path);
        });
        id_ret
    }
    fn download_blocking(id: &str, url: &str, path: &PathBuf) {
        match reqwest::blocking::get(url) {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    error!("📥 Download HTTP error {}: {}", status, url);
                    return;
                }
                match resp.bytes() {
                    Ok(bytes) => {
                        if let Some(parent) = path.parent() { fs::create_dir_all(parent).ok(); }
                        match fs::write(path, &bytes) {
                            Ok(_) => info!("📥 Download complete: {:?} ({} bytes)", path, bytes.len()),
                            Err(e) => error!("📥 Download write error: {}", e),
                        }
                    }
                    Err(e) => error!("📥 Download read error: {}", e),
                }
            }
            Err(e) => error!("📥 Download request error: {}", e),
        }
    }
    pub fn cancel_download(&mut self, id: &str) {
        self.downloads.iter_mut()
            .find(|d| d.id == id && d.status == DownloadStatus::Downloading)
            .map(|d| { d.status = DownloadStatus::Cancelled; });
        self.save();
    }
    pub fn remove_download(&mut self, id: &str) {
        self.downloads.retain(|d| d.id != id);
        self.save();
    }
    pub fn clear_completed(&mut self) {
        self.downloads.retain(|d| d.status == DownloadStatus::Downloading || d.status == DownloadStatus::Pending);
        self.save();
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.downloads).unwrap_or_else(|_| "[]".to_string())
    }
}
