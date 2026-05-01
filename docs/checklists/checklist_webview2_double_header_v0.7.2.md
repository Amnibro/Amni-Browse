# Checklist — v0.7.2: WebView2 Double Header + Home Button Fix

**Date:** 2026-04-17
**Scope:** `src/platform/webview.rs` (webview backend only)

## Symptoms (from screenshots)
- On `amnibrowse://newtab/` the URL bar shows `http://amnibrowse.newtab/` — Windows WebView2 rewrites custom protocols to `http://<scheme>.<host>/`.
- Shadow toolbar from `chrome_init_js` injects because `location.protocol === 'http:'` → double header (shadow toolbar + SPA chrome).
- Home button sends `amnibrowse://newtab` to `webview.load_url`. wry 0.46 on Windows does not remap that to the `http://amnibrowse.newtab/` internal URL on a subsequent `load_url` call, so nothing navigates.

## Root Cause
Single cause, two surfaces: the `amnibrowse://` custom scheme is served by WebView2 under an internal `http://amnibrowse.*` origin. The init script's protocol guard and the native `load_url` path both assumed the external scheme would be visible at runtime.

## Changes
- [x] Backup `src/platform/webview.rs` → `backups/platform_webview.v0.7.1.bak`
- [x] Guardian council convened (see `docs/guardian_councils/guardian_council_webview2_double_header.md`)
- [x] `chrome_init_js()` — add hostname guard: bail if `location.hostname` starts with `amnibrowse.` OR `location.host === 'amnibrowse.newtab'`
- [x] `Act::Nav` — when `nav_url` starts with `amnibrowse://`, rewrite to `http://amnibrowse.<host>/<path>` before `webview.load_url` so WebView2 routes to the registered custom-protocol handler
- [x] Rebuild: `cargo build --release` clean
- [x] Update `ARCHITECTURE.md` — note WebView2 internal-origin remap
- [x] Update `CHANGELOG.md` — v0.7.2 entry
- [x] User confirms: no more double header on newtab, home button returns to homepage from any http/https page (2026-04-17)
