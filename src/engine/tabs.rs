use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SplitViewMode { None, Horizontal, Vertical }
impl Default for SplitViewMode { fn default() -> Self { Self::None } }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TabEngine { Servo, Media }
impl Default for TabEngine { fn default() -> Self { Self::Servo } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub history: Vec<String>,
    pub history_index: i32,
    pub is_loading: bool,
    pub split_mode: SplitViewMode,
    pub split_url: Option<String>,
    pub split_title: Option<String>,
    pub panel_group: Option<String>,
    pub is_private: bool,
    pub zoom_level: f64,
    #[serde(default)]
    pub engine: TabEngine,
}
impl Tab {
    pub fn new(url: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: "New Tab".into(),
            url: url.into(),
            is_active: false,
            history: vec![url.to_string()],
            history_index: 0,
            is_loading: false,
            split_mode: SplitViewMode::None,
            split_url: None,
            split_title: None,
            panel_group: None,
            is_private: false,
            zoom_level: 1.0,
            engine: TabEngine::Servo,
        }
    }
    pub fn new_private(url: &str) -> Self {
        let mut tab = Self::new(url);
        tab.is_private = true;
        tab.title = "Private Tab".into();
        tab
    }

    pub fn set_split(&mut self, mode: SplitViewMode, url: Option<&str>) {
        self.split_mode = mode;
        self.split_url = url.map(|u| u.to_string());
        self.split_title = url.map(|_| "Split View".to_string());
    }

    pub fn clear_split(&mut self) {
        self.split_mode = SplitViewMode::None;
        self.split_url = None;
        self.split_title = None;
    }

    pub fn navigate(&mut self, url: &str) {
        if self.url == url { self.is_loading = true; return; }
        let idx = self.history_index as usize;
        if idx + 1 < self.history.len() { self.history.truncate(idx + 1); }
        self.history.push(url.to_string());
        self.history_index = (self.history.len() - 1) as i32;
        self.url = url.to_string();
        self.is_loading = true;
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        (self.history_index as usize) < self.history.len().saturating_sub(1)
    }

    pub fn go_back(&mut self) -> Option<&str> {
        if self.can_go_back() {
            self.history_index -= 1;
            self.url = self.history[self.history_index as usize].clone();
            Some(&self.url)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<&str> {
        if self.can_go_forward() {
            self.history_index += 1;
            self.url = self.history[self.history_index as usize].clone();
            Some(&self.url)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabManager {
    pub tabs: Vec<Tab>,
    active_tab_id: Option<String>,
}
impl TabManager {
    pub fn new() -> Self {
        let mut mgr = Self { tabs: Vec::new(), active_tab_id: None };
        mgr.new_tab("amnibrowse://newtab");
        mgr
    }
    pub fn new_tab(&mut self, url: &str) -> String {
        let mut tab = Tab::new(url);
        tab.is_active = true;
        self.tabs.iter_mut().for_each(|t| t.is_active = false);
        let id = tab.id.clone();
        self.active_tab_id = Some(id.clone());
        self.tabs.push(tab);
        id
    }
    pub fn new_private_tab(&mut self, url: &str) -> String {
        let mut tab = Tab::new_private(url);
        tab.is_active = true;
        self.tabs.iter_mut().for_each(|t| t.is_active = false);
        let id = tab.id.clone();
        self.active_tab_id = Some(id.clone());
        self.tabs.push(tab);
        id
    }
    pub fn close_tab(&mut self, id: &str) -> bool {
        let was_active = self.tabs.iter().find(|t| t.id == id).map(|t| t.is_active).unwrap_or(false);
        let idx = self.tabs.iter().position(|t| t.id == id);
        if let Some(i) = idx {
            self.tabs.remove(i);
            if was_active && !self.tabs.is_empty() {
                let new_active = i.min(self.tabs.len() - 1);
                self.tabs[new_active].is_active = true;
                self.active_tab_id = Some(self.tabs[new_active].id.clone());
            } else if self.tabs.is_empty() {
                self.active_tab_id = None;
            }
            true
        } else { false }
    }
    pub fn switch_tab(&mut self, id: &str) {
        self.tabs.iter_mut().for_each(|t| t.is_active = t.id == id);
        self.active_tab_id = Some(id.into());
    }
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab_id.as_ref().and_then(|id| self.tabs.iter().find(|t| t.id == *id))
    }
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let id = self.active_tab_id.clone();
        id.and_then(move |id| self.tabs.iter_mut().find(|t| t.id == id))
    }
    pub fn tab_count(&self) -> usize { self.tabs.len() }
    pub fn private_tab_count(&self) -> usize { self.tabs.iter().filter(|t| t.is_private).count() }
    pub fn has_private_tabs(&self) -> bool { self.tabs.iter().any(|t| t.is_private) }
    pub fn zoom_in(&mut self) -> Option<f64> {
        self.active_tab_mut().map(|t| { t.zoom_level = (t.zoom_level + 0.1).min(5.0); t.zoom_level })
    }
    pub fn zoom_out(&mut self) -> Option<f64> {
        self.active_tab_mut().map(|t| { t.zoom_level = (t.zoom_level - 0.1).max(0.25); t.zoom_level })
    }
    pub fn zoom_reset(&mut self) -> Option<f64> {
        self.active_tab_mut().map(|t| { t.zoom_level = 1.0; t.zoom_level })
    }
    pub fn zoom_set(&mut self, level: f64) -> Option<f64> {
        self.active_tab_mut().map(|t| { t.zoom_level = level.clamp(0.25, 5.0); t.zoom_level })
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.tabs).unwrap_or_else(|_| "[]".to_string())
    }
}
