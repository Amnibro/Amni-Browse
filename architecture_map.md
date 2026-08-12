# Amni-Browse Architecture Map
## Chrome UI
- `assets/chrome/toolbar.html` — entire browser chrome (tab strip + nav bar + progress). CSS tokens in `:root`, 66px shell (32 tab + 32 nav + 2 progress). Interactive targets 28px (26px in-pill). Nav is a **left cluster**: `#nav-start` | `#url-wrap{flex:0 1 960px}` | `#nav-end` (no margin-left:auto). Free ultrawide space is after the menu. Tab strip: `#tab-list` horizontal scroll + `scrollIntoView` on active/roving. Keyboard: roving tabs; `:focus`/`:focus-visible`; URL via `#url-wrap:focus-within`. Lock via `setLock` + `.secure`/`.insecure`/`.local` (incl. `data:` local). Progress: loading → 72%, complete → 100% then fade (`finishing`). Canary: `window.__amni.chromeRev` (`0.11.0-settings`). Tab favicons are hue-hashed monograms; shield/bookmark buttons reflect live state (`state.shield`, `state.bookmarked`).
- Menu opens the real Settings page (data URL from `settings_page_html()`): search engine, homepage, shield, default zoom, UA override, bookmarks. Mutations go through `amnibrowse://cmd/setting_set` gated by a per-boot `cmd_token`; persisted via `BrowserConfig::save()` (AppData/amni-browse). Blank homepage serves the built-in start page (`newtab_html()`, bookmark tiles).
- Servo composites chrome over content; body transparent/pointer-events none so content hits fall through.
- **Critical:** `load_toolbar_html()` in `src/platform/servo_real.rs` reads disk first (cwd then exe-dir), falls back to `include_str!` embed. Chrome HTML hot-loads on launch — **relaunch, don’t rebuild**, after toolbar edits. About/Shield HTML is compiled into the binary — needs rebuild. `CHROME_HEIGHT_CSS` must stay 66 matching `#shell` (both changed together in v0.11.0). `cargo check` without `servo-real` does not compile this file.
## Launch
- `run.bat` is the single entrypoint: mtime-skips cargo when `target\release\amni-browse.exe` is newer than `src/`, `build.rs`, `Cargo.toml`, `Cargo.lock`. `run-fast.bat` remains as a no-build alias. GStreamer probe: `C:\gstreamer\1.0\msvc_x86_64` then Program Files; gate on `bin\gstreamer-1.0-0.dll`. Real Servo binary is ~100MB+; a ~9MB exe is the feature-stripped default build — do not use it to verify chrome.
## Engine
- Servo-primary hybrid; media/DRM routes to WebView2 via media_engine.
## Recent chrome UI
- v0.10.10 / a3ddb5b: dark About+Shield, progress finish-and-fade, tab scroll-into-view (Grok/Claude)
- v0.11.0 / ff18986: real Settings + start page, live shield toggle (config.block_ads gates adblock), real bookmarks (BookmarkManager wired), search-engine prefix honored by URL bar, default zoom, UA override via Preferences, amnibrowse:// cmd/state locked to chrome webview or token, 66px chrome
- v0.10.8: URL flex-grow 0 so left cluster actually sticks on ultrawide (Grok)
- v0.10.7: unpin nav-end, scheme lock, ghost-close, bookmark 26px, zoom .off (Claude)
- v0.10.6: (superseded) right-edge pin attempt
- v0.10.5: roving tabs + nav clusters
- e8aa647: runtime disk load of toolbar.html
- v0.10.4 / 2c725e1: 32px targets + focus rings
- f6e1b60: tokenized dark surfaces
