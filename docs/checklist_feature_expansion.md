# Amni-Browse Feature Expansion Checklist
## Task: Chrome/Firefox/Safari Rival Implementation
## Date: 2026-03-13

### Phase 0: Prep
- [ ] Backup all existing source files to backups/ with v0.2.0 suffix
- [ ] Update Cargo.toml with new dependencies

### Phase 1: Download Manager (download_manager.rs)
- [ ] Create DownloadItem struct (id, url, filename, path, size, progress, status, timestamp)
- [ ] Create DownloadManager with async download via tokio+reqwest
- [ ] Pause/resume/cancel support
- [ ] Auto-detect filename from Content-Disposition / URL
- [ ] Downloads directory config
- [ ] IPC: download_start, download_pause, download_resume, download_cancel, download_list
- [ ] IpcResponse: download_progress, download_complete, downloads_list

### Phase 2: Browsing History (history.rs)
- [ ] Create HistoryEntry struct (id, url, title, visit_count, last_visited, first_visited)
- [ ] Create HistoryManager with add/search/delete/clear
- [ ] Auto-record on navigation
- [ ] Date-grouped retrieval
- [ ] Search by title/URL
- [ ] IPC: history_add, history_search, history_delete, history_clear, history_list

### Phase 3: Find in Page
- [ ] JS-side find implementation using window.find() or Selection API
- [ ] Match counter (X of Y)
- [ ] Next/Previous navigation
- [ ] Highlight matches
- [ ] IPC: find_in_page, find_next, find_prev, find_close
- [ ] Keyboard: Ctrl+F to open, Escape to close, Enter for next

### Phase 4: Session Restore (session.rs)
- [ ] Create SessionState struct (tabs with URLs, active tab, window size/pos)
- [ ] Auto-save session periodically and on clean exit
- [ ] Restore tabs on startup with option
- [ ] Crash recovery detection (lock file)
- [ ] Config: restore_session toggle
- [ ] IPC: save_session, restore_session

### Phase 5: Autofill System (autofill.rs)
- [ ] Extend password_manager with form autofill
- [ ] AddressProfile struct (name, street, city, state, zip, country, phone, email)
- [ ] PaymentCard struct (cardholder, number_enc, expiry, type) — encrypted like passwords
- [ ] AutofillManager: CRUD for addresses and cards
- [ ] IPC: autofill_add_address, autofill_add_card, autofill_list, autofill_remove
- [ ] IPC: autofill_suggest (returns matching entries for current site)

### Phase 6: Private Browsing Mode
- [ ] Incognito tab flag in TabManager
- [ ] Isolated state: no history recording, no cookie persistence, no autofill save
- [ ] Visual indicator (different tab style / mask icon)
- [ ] Clear all private data when last private tab closes
- [ ] IPC: new_private_tab
- [ ] Keyboard: Ctrl+Shift+N

### Phase 7: Zoom Controls
- [ ] Per-tab zoom level tracking (25%-500%)
- [ ] Zoom in/out/reset via IPC
- [ ] Keyboard: Ctrl+Plus, Ctrl+Minus, Ctrl+0
- [ ] Zoom indicator in status bar
- [ ] IPC: zoom_in, zoom_out, zoom_reset, zoom_set

### Phase 8: Reader Mode (reader.rs)
- [ ] Content extraction (strip nav/ads/sidebars, keep article body)
- [ ] Clean typography rendering
- [ ] Font size/family controls
- [ ] Light/dark/sepia reading themes
- [ ] IPC: reader_toggle, reader_settings
- [ ] Keyboard: Ctrl+Shift+R (or button in URL bar)

### Phase 9: Permissions Manager (permissions.rs)
- [ ] Permission types: camera, microphone, location, notifications, clipboard, fullscreen
- [ ] Per-site permission storage (allow/deny/ask)
- [ ] Default policy config
- [ ] Permission request UI prompt
- [ ] IPC: permission_set, permission_get, permission_list, permission_reset

### Phase 10: DNS over HTTPS (dns.rs)
- [ ] DoH resolver using reqwest to query DNS over HTTPS
- [ ] Configurable providers (Cloudflare 1.1.1.1, Google, Quad9, custom)
- [ ] DNS cache with TTL
- [ ] Fallback to system DNS if DoH fails
- [ ] Config: enable_doh, doh_provider
- [ ] IPC: doh_config

### Phase 11: Dev Tools — Basic (devtools.rs)
- [ ] Console panel (capture console.log/warn/error from webview)
- [ ] Network panel (log requests with method/status/size/time)
- [ ] Elements inspector (show DOM tree)
- [ ] Toggle panel with F12
- [ ] IPC: devtools_toggle, devtools_console_log, devtools_network_log

### Phase 12: Extensions API (extensions.rs)
- [ ] Extension manifest format (JSON: name, version, permissions, scripts)
- [ ] Extension loader from extensions/ directory
- [ ] Content script injection
- [ ] Background script support (limited)
- [ ] Extension manager UI (enable/disable/remove)
- [ ] IPC: ext_list, ext_enable, ext_disable, ext_remove

### Phase 13: Profile Manager (profiles.rs)
- [ ] Profile struct (name, avatar_color, config_dir)
- [ ] Isolated config/data per profile
- [ ] Profile switcher UI
- [ ] Default profile
- [ ] IPC: profile_list, profile_create, profile_switch, profile_delete

### Integration
- [ ] Update ipc.rs with all new IpcMessage/IpcResponse variants
- [ ] Update browser.rs handle_ipc_message with all new dispatch arms
- [ ] Update main.rs with new module declarations
- [ ] Update ui.rs with new UI panels and keyboard shortcuts
- [ ] Run cargo check / clippy
- [ ] Test IPC round-trips
- [ ] Update ARCHITECTURE.md
- [ ] Update README.md
- [ ] Create CHANGELOG.md entry
