use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Log,
    Warn,
    Error,
    Info,
    Debug,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub source: Option<String>,
    pub line: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub status_text: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
    pub duration_ms: Option<u64>,
    pub timestamp: DateTime<Utc>,
    pub initiator: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsState {
    pub is_open: bool,
    pub active_panel: String,
    pub console_entries: VecDeque<ConsoleEntry>,
    pub network_entries: VecDeque<NetworkEntry>,
    pub max_entries: usize,
}
impl Default for DevToolsState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_panel: "console".into(),
            console_entries: VecDeque::new(),
            network_entries: VecDeque::new(),
            max_entries: 1000,
        }
    }
}
impl DevToolsState {
    pub fn new() -> Self { Self::default() }
    pub fn toggle(&mut self) -> bool {
        self.is_open = !self.is_open;
        self.is_open
    }
    pub fn set_panel(&mut self, panel: &str) {
        self.active_panel = panel.to_string();
    }
    pub fn log_console(&mut self, level: LogLevel, message: &str, source: Option<&str>, line: Option<u32>) {
        if self.console_entries.len() >= self.max_entries { self.console_entries.pop_front(); }
        self.console_entries.push_back(ConsoleEntry {
            level, message: message.to_string(), timestamp: Utc::now(),
            source: source.map(|s| s.into()), line,
        });
    }
    pub fn log_network(&mut self, entry: NetworkEntry) {
        if self.network_entries.len() >= self.max_entries { self.network_entries.pop_front(); }
        self.network_entries.push_back(entry);
    }
    pub fn clear_console(&mut self) { self.console_entries.clear(); }
    pub fn clear_network(&mut self) { self.network_entries.clear(); }
    pub fn console_json(&self) -> String {
        let entries: Vec<&ConsoleEntry> = self.console_entries.iter().collect();
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".into())
    }
    pub fn network_json(&self) -> String {
        let entries: Vec<&NetworkEntry> = self.network_entries.iter().collect();
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".into())
    }
    pub fn state_json(&self) -> String {
        serde_json::to_string(&DevToolsSummary {
            is_open: self.is_open,
            active_panel: self.active_panel.clone(),
            console_count: self.console_entries.len(),
            network_count: self.network_entries.len(),
        }).unwrap_or_else(|_| "{}".into())
    }
}
#[derive(Serialize)]
struct DevToolsSummary {
    is_open: bool,
    active_panel: String,
    console_count: usize,
    network_count: usize,
}
