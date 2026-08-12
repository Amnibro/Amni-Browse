# Checklist: tab interaction polish v1 (0.11.0)

- [x] Backup webview.rs + chrome.rs to backups/*.v0.11.0-tabkeys.bak
- [x] Fix char-boundary panic in chrome.rs truncate (byte slice -> chars)
- [x] Surrogate-safe title truncation in tabDisplayLabel (Array.from)
- [x] Ctrl+Tab / Ctrl+Shift+Tab tab cycling (webview + egui chrome)
- [x] Ctrl+1..8 jump to tab, Ctrl+9 jump to last tab
- [x] Middle-click tab closes it
- [x] Double-click empty tab strip opens new tab
- [x] cargo check clean
- [x] Release build + smoke
- [x] architecture_map.md + CHANGELOG.md updated
- [x] Pushed to git
