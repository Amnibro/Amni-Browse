# Amni-Browse Architecture Map
## 2026-08-12 Release ship gate (0.11.0)
- Public truth is **0.11.0** only: crate `CARGO_PKG_VERSION`, README, chrome canary `0.11.0-settings`, site tags, GitHub Latest release.
- WebView chrome: `tokens` TAB38+NAV44+BOOK28 = TOTAL 110 / push 114; shadow DOM toolbar mounts on external http(s); SPA home keeps tab/nav/bookmarks bars; OS decorations on.
- Amni Apps → `https://amni-scient.com` (IPC AmniAppList/LaunchApp navigate; list_apps_json empty).
- Theme: `__AMNI_SYNC_THEME` + boot ThemeConfig seed. Tabs: host TabManager + `__AMNI_TAB_SEED` / get_tabs resync.
- Checklist: `docs/checklists/checklist_release_0.11.0_push_v1.md` + `checklist_ui_version_parity_v0.11.0.md`.
## Chrome UI
- `assets/chrome/toolbar.html` — entire browser chrome (tab strip + nav bar + progress). CSS tokens in `:root`, 66px shell (32 tab + 32 nav + 2 progress). Interactive targets 28px (26px in-pill). Nav is a **left cluster**: `#nav-start` | `#url-wrap{flex:0 1 960px}` | `#nav-end` (no margin-left:auto). Free ultrawide space is after the menu. Tab strip: `#tab-list` horizontal scroll + `scrollIntoView` on active/roving. Keyboard: roving tabs; `:focus`/`:focus-visible`; URL via `#url-wrap:focus-within`. Lock via `setLock` + `.secure`/`.insecure`/`.local` (incl. `data:` local). Progress: loading → 72%, complete → 100% then fade (`finishing`). Canary: `window.__amni.chromeRev` (`0.11.0-settings`). Tab favicons are hue-hashed monograms; shield/bookmark buttons reflect live state (`state.shield`, `state.bookmarked`).
- Menu opens the real Settings page (data URL from `settings_page_html()`): search engine, homepage, shield, default zoom, UA override, bookmarks. Mutations go through `amnibrowse://cmd/setting_set` gated by a per-boot `cmd_token`; persisted via `BrowserConfig::save()` (AppData/amni-browse). Blank homepage serves the built-in start page (`newtab_html()`, bookmark tiles).
- Servo composites chrome over content; body transparent/pointer-events none so content hits fall through.
- **Critical:** `load_toolbar_html()` in `src/platform/servo_real.rs` reads disk first (cwd then exe-dir), falls back to `include_str!` embed. Chrome HTML hot-loads on launch — **relaunch, don’t rebuild**, after toolbar edits. About/Shield HTML is compiled into the binary — needs rebuild. `CHROME_HEIGHT_CSS` must stay 66 matching `#shell` (both changed together in v0.11.0). `cargo check` without `servo-real` does not compile this file.
## Launch
- `run.bat` is the single entrypoint: mtime-skips cargo when `target\release\amni-browse.exe` is newer than `src/`, `build.rs`, `Cargo.toml`, `Cargo.lock`. `run-fast.bat` remains as a no-build alias. GStreamer probe: `C:\gstreamer\1.0\msvc_x86_64` then Program Files; gate on `bin\gstreamer-1.0-0.dll`. Real Servo binary is ~100MB+; a ~9MB exe is the feature-stripped default build — do not use it to verify chrome.
## Engine
- Servo-primary hybrid; media/DRM routes to WebView2 via media_engine.
## WebView chrome (default `webview` feature)
- `src/ui/tokens.rs`: TAB 38 + NAV 44 + BOOKMARKS 28 = `TOTAL_CHROME_H` 110; content push 114.
- `src/platform/webview.rs` injects shadow-DOM chrome on external http(s) pages: tab strip + nav + bookmarks, themed from live `ThemeConfig` via `__AMNI_SYNC_THEME`. Tab state survives leave-home via `__AMNI_TAB_SEED` + page-load resync + `get_tabs`. OS frame: `WindowBuilder.with_decorations(true)`.
- Amni Apps menu/ctx/palette/IPC (`amni_app_list`, `launch_app`) → `https://amni-scient.com` only (no local inventory panel or launch list).
- Crate / UA / site download tag aligned at **0.11.0** (single source: `Cargo.toml` → `CARGO_PKG_VERSION`). Site index+faq tags must match product page (were stuck on 0.10.3).
## Recent chrome UI
- v0.11.0 / webview parity: theme sync home↔external, multi-tab seed on nav, chrome host height matches tokens (was 82px clip), decorations forced on, site+tag 0.11.0
- v0.10.10 / a3ddb5b: dark About+Shield, progress finish-and-fade, tab scroll-into-view (Grok/Claude)
- v0.11.0 / ff18986: real Settings + start page, live shield toggle (config.block_ads gates adblock), real bookmarks (BookmarkManager wired), search-engine prefix honored by URL bar, default zoom, UA override via Preferences, amnibrowse:// cmd/state locked to chrome webview or token, 66px chrome
- v0.10.8: URL flex-grow 0 so left cluster actually sticks on ultrawide (Grok)
- v0.10.7: unpin nav-end, scheme lock, ghost-close, bookmark 26px, zoom .off (Claude)
- v0.10.6: (superseded) right-edge pin attempt
- v0.10.5: roving tabs + nav clusters
- e8aa647: runtime disk load of toolbar.html
- v0.10.4 / 2c725e1: 32px targets + focus rings
- f6e1b60: tokenized dark surfaces
