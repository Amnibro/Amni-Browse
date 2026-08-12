# Checklist — keyboard travel + internal hash routing (v0.11.9-pre)

- [x] Backup originals: backups/platform_webview.rs.v0.11.8-kbdtravel.bak, backups/ui_webview.rs.v0.11.8-kbdtravel.bak
- [x] Overlay keydown (shipped WebView2 path): add Ctrl+L (focus+select URL bar), Ctrl+D (bookmark_add current page), Ctrl+H (history), Ctrl+J (downloads)
- [x] Fragment bug root cause: internal target builder folded `#frag` into hostname (`http://amnibrowse.newtab#history/`) — split fragment before host/path, reattach after path
- [x] Menu History/Downloads: route to `amnibrowse://newtab#history` / `#downloads` (was bare newtab = dead-end on home)
- [x] SPA: hash → openPanel on load (history/downloads/vault/devtools)
- [x] Side-effect fix: Developer menu deep-links (#themes/#ext/#bug) previously received `#themes/` — data-p selector missed silently; now clean
- [x] Verified overlay skips internal pages (hostname guard line ~266) — no double keydown handling
- [x] cargo check clean (only pre-existing warnings)
- [x] changelog.md + architecture_map.md updated
- [x] Runtime spot-check (Google, f3d00a3): menu History opens panel; menu Themes lands on Themes tab; Ctrl+L focuses URL bar: menu → History opens panel; Ctrl+L from external page focuses URL bar
- [x] SHIP GATE v0.11.9: tag pushed (3d729c4), release sole Latest, live asset cold-pulled 170 entries / 0 backslash / toolbar present / 0.11.9 baked in exe, site chips + download href live on v0.11.9 (was silently still serving v0.11.8 zip)
