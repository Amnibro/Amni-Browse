use std::process::Command;
pub struct ClipboardManager {
    clipboard_text: String,
}
impl ClipboardManager {
    pub fn new() -> Self { Self { clipboard_text: String::new() } }
    pub fn read_text(&self) -> &str { &self.clipboard_text }
    pub fn write_text(&mut self, text: &str) { self.clipboard_text = text.to_string(); }
    pub fn clear(&mut self) { self.clipboard_text.clear(); }
}
pub fn read_system_clipboard() -> Option<String> {
    if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-command", "Get-Clipboard"])
            .output()
            .ok()
            .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok().map(|s| s.trim_end().to_string()) } else { None })
    } else {
        None
    }
}
pub fn write_system_clipboard(text: &str) -> bool {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &format!("echo {} | clip", text)])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clipboard_manager() {
        let mut cm = ClipboardManager::new();
        assert_eq!(cm.read_text(), "");
        cm.write_text("hello clipboard");
        assert_eq!(cm.read_text(), "hello clipboard");
        cm.clear();
        assert_eq!(cm.read_text(), "");
    }
    #[test]
    fn test_write_read_cycle() {
        let mut cm = ClipboardManager::new();
        cm.write_text("first");
        cm.write_text("second");
        assert_eq!(cm.read_text(), "second");
    }
    #[test]
    fn test_system_clipboard_functions_exist() {
        let _ = read_system_clipboard();
        let _ = write_system_clipboard("test");
    }
}
