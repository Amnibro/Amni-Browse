use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub visit_count: u32,
    pub last_visited: DateTime<Utc>,
    pub first_visited: DateTime<Utc>,
    pub favicon: Option<String>,
}
impl HistoryEntry {
    pub fn new(url: &str, title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.to_string(),
            visit_count: 1,
            last_visited: now,
            first_visited: now,
            favicon: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryManager {
    pub entries: Vec<HistoryEntry>,
}
impl HistoryManager {
    pub fn new() -> Self { Self::load() }
    fn file_path() -> PathBuf { BrowserConfig::config_dir().join("history.json") }
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
    pub fn record_visit(&mut self, url: &str, title: &str) {
        if url.starts_with("amnibrowse://") || url.is_empty() { return; }
        match self.entries.iter_mut().find(|e| e.url == url) {
            Some(existing) => {
                existing.visit_count += 1;
                existing.last_visited = Utc::now();
                if !title.is_empty() { existing.title = title.to_string(); }
            }
            None => self.entries.push(HistoryEntry::new(url, title)),
        }
        self.save();
    }
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let q = query.to_lowercase();
        self.entries.iter()
            .filter(|e| e.title.to_lowercase().contains(&q) || e.url.to_lowercase().contains(&q))
            .collect()
    }
    pub fn get_by_date(&self, date: NaiveDate) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|e| e.last_visited.date_naive() == date)
            .collect()
    }
    pub fn recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut sorted: Vec<&HistoryEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.last_visited.cmp(&a.last_visited));
        sorted.into_iter().take(limit).collect()
    }
    pub fn delete_entry(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
        self.save();
    }
    pub fn delete_by_url(&mut self, url: &str) {
        self.entries.retain(|e| e.url != url);
        self.save();
    }
    pub fn clear_range(&mut self, from: DateTime<Utc>, to: DateTime<Utc>) {
        self.entries.retain(|e| e.last_visited < from || e.last_visited > to);
        self.save();
    }
    pub fn clear_all(&mut self) {
        self.entries.clear();
        self.save();
    }
    pub fn entry_count(&self) -> usize { self.entries.len() }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.entries).unwrap_or_else(|_| "[]".to_string())
    }
    pub fn recent_json(&self, limit: usize) -> String {
        let recent: Vec<&HistoryEntry> = self.recent(limit);
        serde_json::to_string(&recent).unwrap_or_else(|_| "[]".to_string())
    }
}
