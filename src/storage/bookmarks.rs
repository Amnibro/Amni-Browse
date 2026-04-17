use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;
use crate::storage::config::BrowserConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
    pub created_at: DateTime<Utc>,
    pub favicon: Option<String>,
}

impl Bookmark {
    pub fn new(title: &str, url: &str, folder: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            url: url.to_string(),
            folder: folder.map(|f| f.to_string()),
            created_at: Utc::now(),
            favicon: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkManager {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self::load()
    }

    fn file_path() -> std::path::PathBuf {
        BrowserConfig::config_dir().join("bookmarks.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(&path, data).ok();
        }
    }

    pub fn add(&mut self, title: &str, url: &str, folder: Option<&str>) -> Bookmark {
        let bookmark = Bookmark::new(title, url, folder);
        self.bookmarks.push(bookmark.clone());
        self.save();
        bookmark
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.bookmarks.len();
        self.bookmarks.retain(|b| b.id != id);
        let removed = self.bookmarks.len() < len_before;
        if removed {
            self.save();
        }
        removed
    }

    pub fn find_by_url(&self, url: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.url == url)
    }

    pub fn list_folder(&self, folder: Option<&str>) -> Vec<&Bookmark> {
        self.bookmarks
            .iter()
            .filter(|b| b.folder.as_deref() == folder)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Bookmark> {
        let q = query.to_lowercase();
        self.bookmarks
            .iter()
            .filter(|b| b.title.to_lowercase().contains(&q) || b.url.to_lowercase().contains(&q))
            .collect()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.bookmarks).unwrap_or_else(|_| "[]".to_string())
    }
}
