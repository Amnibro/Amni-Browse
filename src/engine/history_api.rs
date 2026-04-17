pub struct HistoryState {
    pub entries: Vec<HistoryEntry>,
    pub current_index: usize,
}
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub state_data: String,
}
impl HistoryState {
    pub fn new(initial_url: &str) -> Self {
        Self {
            entries: vec![HistoryEntry { url: initial_url.to_string(), title: String::new(), state_data: String::new() }],
            current_index: 0,
        }
    }
    pub fn push_state(&mut self, state: &str, title: &str, url: &str) {
        self.entries.truncate(self.current_index + 1);
        self.entries.push(HistoryEntry { url: url.to_string(), title: title.to_string(), state_data: state.to_string() });
        self.current_index = self.entries.len() - 1;
    }
    pub fn replace_state(&mut self, state: &str, title: &str, url: &str) {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            entry.url = url.to_string();
            entry.title = title.to_string();
            entry.state_data = state.to_string();
        }
    }
    pub fn go_back(&mut self) -> Option<&HistoryEntry> {
        if self.current_index > 0 { self.current_index -= 1; self.entries.get(self.current_index) } else { None }
    }
    pub fn go_forward(&mut self) -> Option<&HistoryEntry> {
        if self.current_index + 1 < self.entries.len() { self.current_index += 1; self.entries.get(self.current_index) } else { None }
    }
    pub fn go(&mut self, delta: i32) -> Option<&HistoryEntry> {
        let target = self.current_index as i64 + delta as i64;
        if target >= 0 && (target as usize) < self.entries.len() {
            self.current_index = target as usize;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }
    pub fn length(&self) -> usize { self.entries.len() }
    pub fn current(&self) -> Option<&HistoryEntry> { self.entries.get(self.current_index) }
}
