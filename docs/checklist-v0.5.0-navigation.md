# Amni Browse v0.5.0 — Navigation Pipeline Fixes

## Changes Checklist

- [x] **engine/tabs.rs** — Add URL dedup in Tab::navigate() to prevent duplicate history entries  
- [x] **engine/adblocker.rs** — Add `is_blocked_url()` static method for navigation handler
- [x] **app.rs** — Fix Back/Forward to return NavigateTo response instead of None
- [x] **app.rs** — Guard internal `amnibrowse://` URLs from being recorded in browsing history
- [x] **platform/webview.rs** — Remove JS history.back()/forward() hacks, simplify IPC handler
- [x] **platform/webview.rs** — Add `with_navigation_handler` for ad blocking at navigation level
- [x] **ui/webview.rs** — Fix updateTabs to trigger actual WebView navigation for real URLs on tab switch
- [x] **Build** — cargo check + cargo build with 0 errors  
- [x] **Test** — Launch browser, verify startup logs clean, no panics
- [x] **Docs** — Update CHANGELOG.md, ARCHITECTURE.md, README.md
