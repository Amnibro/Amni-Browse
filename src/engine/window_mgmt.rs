use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// The type/purpose of a window.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowType {
    Main,
    Popup,
    Dialog,
    DevTools,
}

impl Default for WindowType {
    fn default() -> Self {
        WindowType::Main
    }
}

/// A request to open a new window.
#[derive(Debug, Clone)]
pub struct WindowRequest {
    pub url: Option<String>,
    pub width: u32,
    pub height: u32,
    pub window_type: WindowType,
    pub resizable: bool,
    pub title: String,
    pub opener_id: Option<u32>,
}

impl Default for WindowRequest {
    fn default() -> Self {
        WindowRequest {
            url: None,
            width: 800,
            height: 600,
            window_type: WindowType::Main,
            resizable: true,
            title: String::new(),
            opener_id: None,
        }
    }
}

/// Dialog types that can be shown to the user (alert, confirm, prompt, beforeunload).
#[derive(Debug, Clone)]
pub enum DialogType {
    /// Simple informational alert with a message.
    Alert(String),
    /// Confirmation dialog with yes/no semantics.
    Confirm(String),
    /// Prompt dialog with message and default value.
    Prompt(String, String),
    /// Before-unload dialog shown when navigating away.
    BeforeUnload(String),
}

/// Result from a dialog interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    Ok,
    Cancel,
    Value(String),
}

/// Metadata about an open window.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub window_type: WindowType,
    pub is_open: bool,
    pub opener_id: Option<u32>,
    pub created_at: Instant,
}

/// Manages windows, popups, and dialogs for the browser.
pub struct WindowManager {
    pub windows: HashMap<u32, WindowInfo>,
    next_id: u32,
    pub pending_dialogs: VecDeque<(u32, DialogType)>,
    pub dialog_result: Option<DialogResult>,
    pub popup_blocker_enabled: bool,
    pub allowed_popup_origins: HashSet<String>,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            windows: HashMap::new(),
            next_id: 1,
            pending_dialogs: VecDeque::new(),
            dialog_result: None,
            popup_blocker_enabled: true,
            allowed_popup_origins: HashSet::new(),
        }
    }

    /// Open a new window based on the given request.
    /// Returns the window id on success, or an error if the popup was blocked.
    pub fn open_window(&mut self, request: WindowRequest) -> Result<u32, String> {
        // Check popup blocker for popup-type windows.
        if request.window_type == WindowType::Popup && self.popup_blocker_enabled {
            // Check if the opener's origin is allowed.
            let origin_allowed = if let Some(ref url) = request.url {
                let origin = extract_origin(url);
                self.is_popup_allowed(&origin)
            } else {
                false
            };

            if !origin_allowed {
                return Err("Popup blocked by popup blocker".to_string());
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let info = WindowInfo {
            id,
            url: request.url.unwrap_or_default(),
            title: request.title,
            width: request.width,
            height: request.height,
            window_type: request.window_type,
            is_open: true,
            opener_id: request.opener_id,
            created_at: Instant::now(),
        };

        self.windows.insert(id, info);
        Ok(id)
    }

    /// Close a window by id.
    pub fn close_window(&mut self, id: u32) {
        if let Some(info) = self.windows.get_mut(&id) {
            info.is_open = false;
        }
    }

    /// Get info about a window by id.
    pub fn get_window(&self, id: u32) -> Option<&WindowInfo> {
        self.windows.get(&id)
    }

    /// List all windows (both open and closed).
    pub fn list_windows(&self) -> Vec<&WindowInfo> {
        self.windows.values().collect()
    }

    /// Queue a dialog for display by the UI layer.
    pub fn show_dialog(&mut self, window_id: u32, dialog: DialogType) {
        self.pending_dialogs.push_back((window_id, dialog));
    }

    /// Resolve the current pending dialog with a result.
    pub fn resolve_dialog(&mut self, result: DialogResult) {
        self.dialog_result = Some(result);
        self.pending_dialogs.pop_front();
    }

    /// Peek at the next pending dialog without removing it.
    pub fn pending_dialog(&self) -> Option<&(u32, DialogType)> {
        self.pending_dialogs.front()
    }

    /// Check whether popups from the given origin are allowed.
    pub fn is_popup_allowed(&self, origin: &str) -> bool {
        if !self.popup_blocker_enabled {
            return true;
        }
        self.allowed_popup_origins.contains(origin)
    }

    /// Allow popups from the given origin.
    pub fn allow_popup_origin(&mut self, origin: &str) {
        self.allowed_popup_origins.insert(origin.to_string());
    }

    /// Handle a window.open() call from JavaScript.
    ///
    /// Parses the `features` string (e.g., "width=400,height=300,menubar=no,toolbar=no")
    /// and creates a popup window request.
    ///
    /// Returns the new window's id on success.
    pub fn window_open_from_js(
        &mut self,
        url: &str,
        target: &str,
        features: &str,
    ) -> Result<u32, String> {
        let parsed = parse_window_features(features);

        let width = parsed
            .get("width")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(800);
        let height = parsed
            .get("height")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(600);
        let resizable = parsed
            .get("resizable")
            .map(|v| v != "no" && v != "0")
            .unwrap_or(true);

        let title = if target.is_empty() || target == "_blank" {
            url.to_string()
        } else {
            target.to_string()
        };

        let request = WindowRequest {
            url: if url.is_empty() {
                None
            } else {
                Some(url.to_string())
            },
            width,
            height,
            window_type: WindowType::Popup,
            resizable,
            title,
            opener_id: None,
        };

        self.open_window(request)
    }
}

/// Parse a window.open() features string into key-value pairs.
///
/// Examples:
/// - "width=400,height=300" -> { "width": "400", "height": "300" }
/// - "menubar=no,toolbar=no,width=500" -> { "menubar": "no", "toolbar": "no", "width": "500" }
/// - "width=400, height=300" (with spaces) also works
fn parse_window_features(features: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if features.is_empty() {
        return map;
    }

    for part in features.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((key, val)) = part.split_once('=') {
            map.insert(
                key.trim().to_ascii_lowercase(),
                val.trim().to_string(),
            );
        } else {
            // A bare feature name like "menubar" is treated as "yes".
            map.insert(part.to_ascii_lowercase(), "yes".to_string());
        }
    }

    map
}

/// Extract the origin (scheme + host + port) from a URL.
fn extract_origin(url: &str) -> String {
    // Simple origin extraction: scheme://host[:port]
    if let Some(after_scheme) = url.split_once("://") {
        let scheme = after_scheme.0;
        let rest = after_scheme.1;
        let host_port = rest.split('/').next().unwrap_or(rest);
        format!("{}://{}", scheme, host_port)
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_main_window() {
        let mut wm = WindowManager::new();
        let req = WindowRequest {
            url: Some("https://example.com".to_string()),
            width: 1024,
            height: 768,
            window_type: WindowType::Main,
            resizable: true,
            title: "Test".to_string(),
            opener_id: None,
        };
        let id = wm.open_window(req).unwrap();
        assert_eq!(id, 1);
        let info = wm.get_window(id).unwrap();
        assert_eq!(info.url, "https://example.com");
        assert!(info.is_open);
        assert_eq!(info.window_type, WindowType::Main);
    }

    #[test]
    fn test_popup_blocker() {
        let mut wm = WindowManager::new();
        // Popup blocker is enabled by default.
        let req = WindowRequest {
            url: Some("https://evil.com/popup".to_string()),
            width: 400,
            height: 300,
            window_type: WindowType::Popup,
            resizable: false,
            title: "Popup".to_string(),
            opener_id: Some(1),
        };
        let result = wm.open_window(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("blocked"));
    }

    #[test]
    fn test_allowed_popup_origin() {
        let mut wm = WindowManager::new();
        wm.allow_popup_origin("https://trusted.com");

        let req = WindowRequest {
            url: Some("https://trusted.com/popup".to_string()),
            width: 400,
            height: 300,
            window_type: WindowType::Popup,
            resizable: true,
            title: "Allowed".to_string(),
            opener_id: None,
        };
        let result = wm.open_window(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_close_window() {
        let mut wm = WindowManager::new();
        let req = WindowRequest {
            url: Some("https://example.com".to_string()),
            window_type: WindowType::Main,
            ..Default::default()
        };
        let id = wm.open_window(req).unwrap();
        assert!(wm.get_window(id).unwrap().is_open);

        wm.close_window(id);
        assert!(!wm.get_window(id).unwrap().is_open);
    }

    #[test]
    fn test_list_windows() {
        let mut wm = WindowManager::new();
        wm.open_window(WindowRequest {
            url: Some("https://a.com".to_string()),
            window_type: WindowType::Main,
            ..Default::default()
        })
        .unwrap();
        wm.open_window(WindowRequest {
            url: Some("https://b.com".to_string()),
            window_type: WindowType::Main,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(wm.list_windows().len(), 2);
    }

    #[test]
    fn test_dialog_queue() {
        let mut wm = WindowManager::new();
        assert!(wm.pending_dialog().is_none());

        wm.show_dialog(1, DialogType::Alert("Hello!".to_string()));
        wm.show_dialog(1, DialogType::Confirm("Sure?".to_string()));

        let (win_id, dialog) = wm.pending_dialog().unwrap();
        assert_eq!(*win_id, 1);
        assert!(matches!(dialog, DialogType::Alert(_)));

        wm.resolve_dialog(DialogResult::Ok);
        let (_, dialog) = wm.pending_dialog().unwrap();
        assert!(matches!(dialog, DialogType::Confirm(_)));

        wm.resolve_dialog(DialogResult::Cancel);
        assert!(wm.pending_dialog().is_none());
    }

    #[test]
    fn test_window_open_from_js() {
        let mut wm = WindowManager::new();
        wm.popup_blocker_enabled = false;

        let id = wm
            .window_open_from_js(
                "https://example.com",
                "_blank",
                "width=400,height=300,resizable=no",
            )
            .unwrap();

        let info = wm.get_window(id).unwrap();
        assert_eq!(info.width, 400);
        assert_eq!(info.height, 300);
        assert_eq!(info.window_type, WindowType::Popup);
    }

    #[test]
    fn test_parse_window_features() {
        let features = parse_window_features("width=500, height=400, menubar=no, toolbar=yes");
        assert_eq!(features.get("width").unwrap(), "500");
        assert_eq!(features.get("height").unwrap(), "400");
        assert_eq!(features.get("menubar").unwrap(), "no");
        assert_eq!(features.get("toolbar").unwrap(), "yes");
    }

    #[test]
    fn test_parse_window_features_empty() {
        let features = parse_window_features("");
        assert!(features.is_empty());
    }

    #[test]
    fn test_extract_origin() {
        assert_eq!(
            extract_origin("https://example.com/path/to/page"),
            "https://example.com"
        );
        assert_eq!(
            extract_origin("http://localhost:8080/test"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_dialog_prompt_with_value() {
        let mut wm = WindowManager::new();
        wm.show_dialog(
            1,
            DialogType::Prompt("Enter name:".to_string(), "default".to_string()),
        );

        wm.resolve_dialog(DialogResult::Value("Alice".to_string()));
        assert_eq!(
            wm.dialog_result,
            Some(DialogResult::Value("Alice".to_string()))
        );
    }

    #[test]
    fn test_devtools_window_bypasses_blocker() {
        let mut wm = WindowManager::new();
        // Popup blocker is enabled, but DevTools windows are not popups.
        let req = WindowRequest {
            url: None,
            width: 800,
            height: 600,
            window_type: WindowType::DevTools,
            resizable: true,
            title: "DevTools".to_string(),
            opener_id: Some(1),
        };
        let result = wm.open_window(req);
        assert!(result.is_ok());
    }
}
