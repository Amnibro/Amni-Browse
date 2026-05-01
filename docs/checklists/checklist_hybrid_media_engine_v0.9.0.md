# Checklist — Hybrid Media Engine (v0.9.0)

Target: Servo (primary) + wry (media fallback) coexistence across Windows / macOS / Linux.

## Design
- [x] Architecture documented in `ARCHITECTURE.md` v0.9.0 section with per-OS engine matrix + dispatch flow + guardian council rationale.
- [x] `EngineKind` enum + `MEDIA_PATTERNS` list covering YouTube, Twitch, Vimeo, Netflix, Disney+, Hulu, HBO Max / Max, Prime Video, Paramount+, Crunchyroll, Funimation, Apple TV+, Spotify embed, Tidal, SoundCloud, Discovery+, ESPN+, Dailymotion.
- [x] `TabEngine` on `Tab` with `#[serde(default)]` (session back-compat).

## Code — `src/platform/media_engine.rs`
- [x] `EngineKind { Servo, Media }` + `Default = Servo`.
- [x] `route(url) -> EngineKind` pattern router (substring match, lowercased).
- [x] `MediaWindow { window, webview, url }`.
- [x] `spawn_media_window(event_loop, url)` — creates winit Window + wry WebView.
- [x] `MEDIA_UA` user-agent string so streaming sites recognise a modern browser.
- [x] `platform_label()` for logging.

## Platform gating
- [x] Windows `configure_privacy_env()` — `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` disables SmartScreen / sync / background-net / breakpad / first-run / default-browser-check.
- [x] Windows `WEBVIEW2_USER_DATA_FOLDER` pointed at `%APPDATA%/amni-browse/webview2-data/`.
- [x] macOS `configure_privacy_env()` — logs but relies on WKWebView app-sandboxed data store.
- [x] Linux `configure_privacy_env()` — per-user WebKitGTK data dir + sets `WEBKIT_FORCE_WIDEVINE_ENABLED` only if CDM present.
- [x] Fallback `cfg(not(any(...)))` no-op for any other target.

## Linux Widevine opt-in
- [x] `widevine_path()` returns `~/.config/amni-browse/widevine/libwidevinecdm.so`.
- [x] `widevine_installed()` checks existence.
- [x] `install_widevine()` returns an error message that walks the user through the manual copy step (opt-in per Google TOS, not shipped).
- [x] Non-Linux `widevine_installed()` returns `true` (system-provided).

## Cargo + wiring
- [x] `Cargo.toml` — `servo-real` feature gains `"dep:wry"`.
- [x] `src/platform/mod.rs` — `media_engine` registered behind `servo-real`.
- [x] `src/engine/tabs.rs` — `Tab.engine` field + `TabEngine` enum.
- [x] `src/platform/servo_real.rs` — `AppState.media_windows` registry + startup scan + `WindowId` routing + multi-window close logic.

## Build verification
- [x] `cargo check --no-default-features --features servo-real` compiles clean (2m 01s, warnings only, zero errors).
- [ ] `cargo build --no-default-features --features servo-real` full link (to run at next session start).
- [ ] Runtime smoke test: launch with a restored YouTube tab, confirm media window opens alongside Servo window and plays video.
- [ ] Runtime smoke test: navigate Servo to DDG, confirm no regression in input / adblock / render.
- [ ] Confirm `target/debug/amni-browse.exe` closes cleanly when all windows (Servo main + any media) are closed.

## Follow-ups deferred (v0.9.1+)
- [x] Intercept mid-session navigations in Servo's `WebViewDelegate::request_navigation` to route media URLs without needing a restart. *(Landed 2026-04-19: `AppState.pending_media_urls` queue + `drain_pending_media()` helper called after `spin_event_loop()` in both `user_event` and `window_event`. Embed/player URLs excluded to avoid double-spawn.)*
- [ ] Right-click menu "Open in media mode" for arbitrary URLs.
- [ ] Tab-row UI indicator (needs chrome rework in `ui/chrome.rs`).
- [ ] Settings toggle to disable media mode globally (pure-Servo purist mode).
- [ ] Editable streaming-domain pattern list (currently compile-time constant).
- [ ] Linux Widevine automated downloader (fetch from a Chromium package, SHA-verify, extract).
- [ ] Embed wry WebView as child inside the Servo winit window (unified-window UX rather than separate windows).
- [ ] Unit tests for `route()` to lock in pattern behaviour.
