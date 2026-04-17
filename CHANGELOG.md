# Changelog

## v0.7.0 — 2026-03-15
### Amni Apps Launcher + Desktop Shortcut & Icon
- **engine/app_launcher.rs** — NEW: Hardcoded `AMNI_APPS` registry (10 apps) with `AmniApp` struct (id, name, desc, emoji, LaunchType, AppCategory); `launch_app()` validates against allowlist then spawns via `std::process::Command` (Bat→cmd /C start, Cargo→cmd /K cargo run --release, Web→navigate); `list_apps_json()` serializes for IPC
- **engine/mod.rs** — Added `pub mod app_launcher` re-export
- **net/ipc.rs** — Added `AmniAppList` and `LaunchApp{id}` IPC messages; Added `AmniApps{data}`, `AppLaunched{message}`, `AppNavigate{url}` IPC responses
- **app.rs** — Wired `AmniAppList` → returns app registry JSON; `LaunchApp` → validates + spawns process or returns NavigateTo for web apps
- **ui/webview.rs** — NEW "Amni Apps" slide panel with grouped cards (Local Apps / Web Apps), emoji icons, name, description, Launch/Open buttons; context menu entry; command palette entry; JS `renderAmniApps()` renderer; response handlers for `amni_apps`, `app_launched`, `app_navigate`; panel auto-requests `amni_app_list` on open
- **ui/emoji.rs** — `bolt`, `diamond`, `crown` emojis already in atlas (used by app cards)
- **assets/amni-browse.ico** — NEW: Multi-resolution Windows icon (16/32/48/64/128/256px) — dark navy shield with cyan "A" and privacy dot
- **assets/amni-browse.svg** — NEW: Vector source for the icon
- **assets/windows_app.rc** — NEW: Windows resource file linking ICO to binary
- **build.rs** — NEW: Uses `embed-resource` to compile .ico into .exe for native taskbar icon
- **scripts/create_shortcut.ps1** — NEW: Creates pinnable desktop shortcut with icon
- **run.bat** — NEW: Simple launcher builds release if needed then starts browser
- **assets/amni-browse.png** — User-provided PNG icon now treated as canonical source; `assets/amni-browse.ico` regenerated from this PNG for shortcut/taskbar use
- **engine/app_launcher.rs** — `list_apps_json()` now emits optional `icon_src` for app cards; Amni Mail card auto-loads an image from `assets/` (`amni-mail.png`, `amni-mail.jpg`, `amni-mail.jpeg`, or best image fallback)
- **ui/webview.rs** — `renderAmniApps()` now renders per-app PNG icons when available, with emoji fallback
- **app.rs** — `ThemeSet` now returns `ActiveTheme` response so theme changes apply instantly without requiring a second IPC round-trip
- **ui/webview.rs** — Hardened `__amni_receive` with try/catch and defensive JSON handling; tab rendering now validates payload shape and guards against malformed entries to prevent tab UI crashes
- **platform/webview.rs** — Replaced fragile injected site toolbar with a shadow-DOM self-healing toolbar; it now re-injects after SPA/body rerenders and preserves Home/Back/Forward controls instead of disappearing on subsequent navigations
- **platform/webview.rs** — Fixed WebView2 crash when returning to Home (`wry` panic: `InvalidUri(Empty)` in IPC source URI path) by replacing `data:` home navigation with `webview.load_html(...)` and adding empty-URL navigation guards
- **platform/webview.rs + ui/webview.rs** — Navigation now stays inside the Amni shell (in-page `loadUrl(...)`) instead of replacing the root WebView document; external pages render in `#web-content` iframe so tab bar, themes, and settings panels remain available while browsing
- **platform/webview.rs** — Prevented injected page toolbar from running inside iframe content (`window.self !== window.top` guard), removing duplicate header bars when browsing sites inside the shell
- **ui/webview.rs** — Fixed hamburger/context-menu actions by not immediately closing panels on menu-item clicks; menu options now open Themes/Settings/Downloads/etc. correctly
- **ui/webview.rs + ui/theme.rs** — Theme panel now highlights active theme, supports deleting custom themes directly from cards, and includes 4 new creative presets: Solarflare, Mint Matrix, Paper Sunset, and Deep Space
- **engine/paint.rs** — Fixed pre-existing non-exhaustive match on PaintCommand (added wildcard arm)
### Apps Available
| App | Type | Launch |
|-----|------|--------|
| Amni AI | Local | run.bat (Gradio @ :7700) |
| Azno v2 | Local | Run.bat (Trading @ :8050) |
| Amni Mail | Local | run.bat (FastAPI+React) |
| Amni Gen | Local | run.bat (Gradio @ :7860) |
| Amni Calc | Local | run.bat (WASM @ :8090) |
| Amni Explore | Local | run.bat (Ursina 3D) |
| Amni Miner | Local | run_dashboard.bat @ :8080 |
| Amni Game | Local | cargo run --release |
| Amni Coder | Web | example.com/coder |
| Amni-Scient | Web | example.com |

### Fixes
- **platform/webview.rs** — Corrected navigation handler to use the runtime ad-block toggle state (`state.ad_blocker.enabled`) so sites are not blocked when ad blocking is disabled.

## v0.6.1 — 2026-03-14
### Async Response Delivery — Engine Pipeline Now Fully Wired
- **main.rs** — Created `tokio::runtime::Runtime` with `rt.enter()` guard; `tokio::spawn()` now has proper async context for task execution on worker threads
- **app.rs** — Added `async_tx: Option<std::sync::mpsc::Sender<String>>` and `async_notify: Option<Arc<dyn Fn() + Send + Sync>>` to BrowserState; all 3 async IPC handlers (FetchPage, PageMetaReq, ReaderFetch) now clone tx+notify, send `IpcResponse::to_js_call()` through channel, and wake event loop via notify callback; previously responses were created and silently dropped
- **platform/webview.rs** — Created `std::sync::mpsc::channel::<String>()` for async response delivery; sender+notify callback set on BrowserState after construction; `async_rx.try_recv()` drained in UserEvent handler before sync act queue, feeding responses to `webview.evaluate_script()`; imported `Arc` for notify callback
- **engine/pipeline.rs** — `fetch_and_parse()` now resolves relative CSS `<link>` hrefs against page base URL and fetches stylesheet content via AmniClient; CSS text stored in `PageResult::css_sources`; DOM scoped to block for early drop (Rc<Node> not Send across await points); added `fetch_full_layout()` convenience method for end-to-end fetch+parse+layout; `fetch_reader()` refactored to drop DOM before returning
### Data Flow (Fixed)
- Engine Fetch: Cmd Palette → sendIpc(fetch_page) → handle_command → tokio::spawn → pipeline.fetch_and_parse() → async_tx.send(PageRendered.to_js_call()) → async_notify() → UserEvent → async_rx.try_recv() → webview.evaluate_script() → window.__amni_receive({type:'page_rendered'}) → engine-viewer overlay displayed
- Page Meta: Same flow → PageMetaResp → status bar update
- Reader Fetch: Same flow → ReaderHtml → reader overlay displayed

## v0.6.0 — 2026-03-14
### Engine Independence — Custom Network, DOM, CSS, Layout & Pipeline Integration
- **ui/emoji.rs** (NEW) — Centralized emoji atlas with 65+ static mappings, dynamic `register()`, `e()`/`eh()` accessors for raw/HTML entity output
- **ui/webview.rs** — All hardcoded HTML entities replaced with format variables from emoji atlas; Command Palette (Ctrl+K) with 22 commands (including Engine Fetch, Reader Fetch, Page Meta), fuzzy search, keyboard nav; `page_rendered` and `page_meta` response handlers with engine-viewer overlay
- **net/http.rs** (NEW) — Custom HTTPS client via hyper 1 + hyper-rustls 0.27 + rustls 0.23; response caching (cache-control aware, max 3600s TTL); DNT + Sec-GPC privacy headers; custom user agent; GET/POST with redirect detection
- **net/cookies.rs** (NEW) — Privacy-controlled cookie jar; third-party cookie blocking; domain matching; Set-Cookie header parsing; allow/deny lists; JSON persistence via serde
- **engine/dom.rs** (NEW) — Custom DOM parser wrapping html5ever 0.38 + markup5ever_rcdom 0.38; `parse()` from HTML string; `extract_meta()` (title, description, charset, lang, links, scripts, stylesheets, images, headings, text_content, meta_tags); `extract_reader_content()` for article extraction with nav/header/footer/script filtering; `query_by_tag()`, `query_by_id()`
- **engine/style.rs** (NEW) — CSS parser wrapping cssparser 0.34; `StyleSheet::parse()` tokenizes CSS into rules/declarations; `ComputedStyle` with 25+ properties (display, position, flex, color, font, margin, padding, border, opacity, z-index, etc.); color parsing (#hex), dimension parsing (px/em/rem/vh/vw/%), font-weight keywords
- **engine/layout.rs** (NEW) — Layout engine wrapping taffy 0.7; `LayoutEngine` manages node tree with `add_node()`/`add_leaf()`; `compute()` runs flexbox/grid layout against viewport dimensions; CSS-to-taffy style conversion (display, position, sizing, margins, padding, borders, flex properties, overflow, gap)
- **engine/pipeline.rs** (NEW) — Render pipeline orchestrator connecting HTTP→DOM→CSS→Layout; `fetch_and_parse()` fetches URL via AmniClient, parses with AmniDom, extracts PageMeta; `fetch_reader()` for reader mode; `parse_and_layout()` full CSS cascade + taffy layout computation; selector matching (tag, #id, .class); inline style support
- **app.rs** — BrowserState now holds `Arc<TokioMutex<RenderPipeline>>`; new IPC handlers: `FetchPage` (async engine fetch), `PageMetaReq` (metadata extraction), `ReaderFetch` (server-side reader content via AmniDom); `ReaderContent` now uses AmniDom for server-side article extraction instead of just wrapping raw HTML
- **net/ipc.rs** — Added 3 new IPC messages (`FetchPage`, `PageMetaReq`, `ReaderFetch`) and 2 new responses (`PageRendered`, `PageMetaResp`)
- **main.rs** — Added `rustls::crypto::ring::default_provider().install_default()` for TLS initialization
### Dependencies Added
- hyper 1 (client, http1, http2), hyper-util 0.1, hyper-rustls 0.27, http-body-util 0.1
- rustls 0.23, webpki-roots 0.26, bytes 1
- html5ever 0.38, markup5ever_rcdom 0.38
- cssparser 0.34, selectors 0.26, taffy 0.7
### Infrastructure
- Version bump 0.5.0 → 0.6.0
- Backups at backups/v0.6.0-pre/, v0.6.0-phase34/, v0.6.0-wired/
- Build: 0 errors, browser launches and runs successfully
- Rustls crypto provider (ring) initialized at startup

## v0.5.0 — 2026-03-14
### Functional Browser — Navigation Pipeline
- **platform/webview.rs** — Complete IPC response dispatching via action queue pattern (Act::Nav/Js/Title), EventLoopProxy for cross-callback signaling, initialization script toolbar injection on external pages, base64 data URI for home navigation, navigation handler for ad blocking at domain level
- **Navigation flow**: SPA URL bar → IPC → handle_command → NavigateTo → Act::Nav → webview.load_url → real page with injected toolbar → IPC back to Rust for Back/Forward/Home/Bookmark
- **Back/Forward**: Now return NavigateTo responses from internal tab history (no longer use JS history API), ensuring proper navigation through tab-managed URL history
- **Tab switching**: SPA updateTabs triggers actual WebView navigation for tabs with real URLs via IPC navigate
- **Ad blocking (navigation level)**: with_navigation_handler blocks main-frame navigations to 60+ known ad/tracker domains
- **URL dedup**: Tab::navigate skips duplicate history entries for same-URL navigations
- **Internal URL guard**: amnibrowse:// URLs excluded from browsing history recording
- **Toolbar (chrome_init_js)**: Floating dark toolbar on http/https pages with Back/Forward/Reload/Home/URL input/Bookmark/Shield, auto-updates URL bar and ad-blocked count via IPC
- **Home navigation**: base64 data URI approach loads SPA HTML (with_html for initial load panics on file:// URLs in wry 0.46)

## v0.4.1 — 2025-07-18
### 7-Pillar Modular Restructure
- **UI Pillar** (ui/) — chrome.rs, webview.rs, theme.rs, reader.rs
- **Communication Pillar** (net/) — ipc.rs, dns.rs
- **Storage Pillar** (storage/) — config.rs, bookmarks.rs, history.rs, session.rs, downloads.rs, profiles.rs
- **Encryption Pillar** (crypto/) — vault.rs, autofill.rs
- **Media Pillar** (media/) — placeholder for v0.5+
- **Platform Pillar** (platform/) — webview.rs (was browser.rs), servo.rs (was servo_backend.rs)
- **Engine Pillar** (engine/) — tabs.rs (was tab_manager.rs), adblocker.rs (was ad_blocker.rs), extensions.rs, permissions.rs, devtools.rs
### File Renames
- browser.rs → platform/webview.rs
- servo_backend.rs → platform/servo.rs
- ui.rs → ui/webview.rs
- chrome.rs → ui/chrome.rs
- tab_manager.rs → engine/tabs.rs
- ad_blocker.rs → engine/adblocker.rs
- download_manager.rs → storage/downloads.rs
- password_manager.rs → crypto/vault.rs
### Infrastructure
- All imports updated from flat crate:: paths to pillar-qualified paths
- 7 mod.rs re-export files with feature-gated visibility
- main.rs rewritten for module hierarchy (7 top-level mod declarations)
- Both backends compile clean (0 errors, warnings only)
- v0.4.0-flat backed up to backups/ directory
- ARCHITECTURE.md updated for pillar topology
- GUARDIAN_COUNCIL_MODULARIZE.md — council proposals per pillar with Rust-native approaches
- 30 files (23 source + 7 mod.rs), ~5,900 LOC total

## v0.4.0 — 2026-03-14
### Dual-Backend Architecture (Servo Integration)
- **app.rs** (NEW) — Extracted shared BrowserState from browser.rs; central handle_command() dispatcher used by both backends
- **chrome.rs** (NEW) — Native egui browser chrome for Servo backend: tab bar, nav bar, status bar, find bar, 10 side panels (vault, themes, settings, downloads, history, devtools, extensions, profiles, autofill, permissions), keyboard shortcuts
- **servo_backend.rs** (NEW) — winit event loop + wgpu GPU compositor + egui rendering pipeline; ApplicationHandler implementation with GPU state management; forget_lifetime() pattern for wgpu 22 render pass arcanization
- **browser.rs** — Refactored to WebView-only backend; feature-gated under `#[cfg(feature = "webview")]`; delegates all state to app.rs
- **main.rs** — Feature-gated module declarations and backend selection: `webview` (default) or `servo-engine`
- **Cargo.toml** — Dual feature flags: `webview = ["dep:wry", "dep:tao"]` and `servo-engine = ["dep:winit", "dep:wgpu", "dep:egui", "dep:egui-winit", "dep:egui-wgpu", "dep:raw-window-handle", "dep:pollster"]`; wgpu pinned to v22 for egui-wgpu 0.29 compatibility
### Build Commands
- `cargo build` — WebView backend (default, uses system WebView)
- `cargo build --no-default-features --features servo-engine` — Servo backend (custom wgpu rendering)
### Design Documents
- GUARDIAN_COUNCIL_v0.5.md — Guardian council proposals for v0.5 AmniShunt vision
- AMNISHUNT_DESIGN.md — Technical design for WebKit-Servo translation shunt layer, septidecimal IR encoding, process-isolated sandbox
### Infrastructure
- Version bump 0.3.0 → 0.4.0
- All v0.3.0 and v0.4.0 files backed up to backups/ directory
- Architecture map updated for dual-backend topology
- 23 source files, ~5,900 LOC total

## v0.3.0 — 2025-07-15
### New Modules (13 features)
- **download_manager.rs** — Async file downloads with progress tracking, cancel/remove/clear
- **history.rs** — Browsing history with search, date grouping, visit count deduplication
- **session.rs** — Session save/restore on startup, crash recovery via lock-file detection
- **autofill.rs** — Address profiles + AES-256-GCM encrypted payment cards, vault key sharing
- **permissions.rs** — Per-site permission management (Camera/Mic/Location/Notifications/Clipboard/Fullscreen/Autoplay/Popups)
- **dns.rs** — DNS-over-HTTPS resolver with TTL cache (Cloudflare/Google/Quad9/Custom providers)
- **devtools.rs** — Console + network logging with 1000-entry ring buffer
- **extensions.rs** — Manifest-based extension system, content script injection, URL matching
- **profiles.rs** — Multi-profile support with isolated data directories
- **reader.rs** — Reader mode with content extraction, Light/Dark/Sepia themes
### Core Changes
- **ipc.rs** — Completely rewritten with 60+ IpcMessage variants and 25+ IpcResponse variants
- **browser.rs** — Rewritten with BrowserState holding 15 subsystem managers, full dispatch
- **tab_manager.rs** — Added private browsing tabs (is_private, no history) and zoom controls (0.25x-5.0x)
- **config.rs** — Added restore_session, enable_doh, doh_provider, default_zoom, enable_reader_mode, downloads_dir
- **password_manager.rs** — Added public key accessor for vault key sharing with autofill
### UI Updates (ui.rs)
- Downloads panel with list/cancel/remove/clear
- History panel with search and delete
- Find-in-page bar (Ctrl+F)
- Zoom controls with visual indicator (Ctrl+=/-/0)
- Private tab badge on tab bar
- DevTools panel (Console + Network tabs)
- Extensions manager panel
- Profiles manager panel
- Autofill addresses and cards panel
- Permissions management panel
- DNS-over-HTTPS toggle in settings
- Session restore toggle in settings
- Reader mode button
- Expanded context menu with all features
- 10 new keyboard shortcuts
### Infrastructure
- Version bump 0.2.0 → 0.3.0
- All v0.2.0 files backed up to backups/ directory
- Architecture map updated
- README fully rewritten with all features documented
## v0.2.0
- Initial release with tabs, bookmarks, ad blocker, password vault, themes, split view
F i x :   N a v i g a t i o n   h a n d l e r   b l o c k e d   w e b s i t e s . 
 
 2026-03-17 v0.7.1 - Fixed logic issue in adblocker's Wry navigation handler that caused all websites to be blocked.

## Date: 2026-03-18
- Rerouted HTTP navigation to use the internal RenderPipeline over Webview's native loader to prevent UI override and bypass X-Frame-Options
- Upgraded the JS hydration layer to properly inject and sandbox PageRendered payloads using element manipulation on the #engine-viewer.
