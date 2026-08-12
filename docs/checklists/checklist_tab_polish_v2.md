# Checklist: canonical tab order + kbd-focus v2 (0.11.0)

- [x] Audit uncommitted improvements (tabs.rs, platform/webview.rs, ui/webview.rs, ui/chrome.rs)
- [x] Verify to_json order change safe for session save/restore (raw iteration confirmed)
- [x] Backup all four files to backups/*.v0.11.0-kbdfocus.bak
- [x] Fix: drop bare .tab:focus ring (permanent ring after mouse click) — :focus-visible + .kbd-focus only
- [x] Fix: remove kn.focus() steals (typing after Ctrl+Tab must stay on page content)
- [x] cargo check clean (default + servo-engine)
- [x] Release build + launch smoke (session restore, zero leaked processes)
- [x] CHANGELOG.md + architecture_map.md updated
- [x] Committed + pushed
