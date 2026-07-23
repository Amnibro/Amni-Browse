# Checklist — Remaining Bugs v0.10.2 (Servo-primary hybrid)

**Goal:** Servo for normal browsing; Chromium/WebView **only** for DRM/MSE services (Netflix, YouTube, etc.).

## Bugs addressed
- [x] URL-bar `navigate` / `new_tab` bypassed media routing (loaded in Servo anyway)
- [x] `MEDIA_PATTERNS` too narrow vs `drm_fallback` (e.g. bare `netflix.com` missed)
- [x] Dual domain lists diverged (media_engine vs drm_fallback)
- [x] `is_embed` used `player.` / `/player/` and blocked legitimate media top-level nav
- [x] Media tabs in chrome state had empty URL/title; close/switch ignored `mN` ids
- [x] Unit tests for `route()` / media classification
- [x] Guardian council + backups + ARCHITECTURE/CHANGELOG/workspace map
- [x] Runtime smoke: DDG on Servo verified 2026-07-22 (window title syncs, 500MB WS); Netflix/YouTube media window (user verify pending)
- [x] Full release rebuild with GStreamer env — root cause was missing GStreamer *devel* package (runtime MSI only, no lib\pkgconfig); fixed via admin-extract of gstreamer-1.0-devel-msvc-x86_64-1.26.11.msi to C:\gstreamer (no elevation needed); 131/131 tests pass; release exe rebuilt 2026-07-22
- [x] Version alignment: Cargo.toml 0.7.0 → 0.10.2; About page reads env!("CARGO_PKG_VERSION")

## Non-goals this pass
- Favicons, full settings panel, embed wry as child HWND
- Fixing default `cargo run` feature (still `webview` for light builds; `run.bat` is Servo path)
