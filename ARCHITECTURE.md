# Amni-Browse Architecture Map
## v0.10.0-pre — Servo-Rendered Chrome (2026-04-19)
Browser chrome (tab strip, nav bar, URL input, progress) is now rendered *by Servo itself* via a second `WebView` loaded from `assets/chrome/toolbar.html`. The old egui chrome (referenced throughout this doc below) is superseded on the `servo-real` backend. Composition uses a child `OffscreenRenderingContext` for content; per-frame order: chrome.paint() → content.paint() → `offscreen.render_to_parent_callback()` (Servo-provided `glBlitFramebuffer`) onto the main `WindowRenderingContext` at `Rect(0, chrome_px, w, h - chrome_px)` → present. Input routes by pointer y: chrome strip (first 74 CSS px) → chrome webview; below → content webview with y translated. Sections below referring to `ui/chrome.rs` describe the v0.9.x-and-earlier egui chrome, which still compiles on the `servo-engine` (custom engine) backend but is bypassed on `servo-real`.
## v0.9.0 — Hybrid Media Engine (2026-04-18)
Amni-Browse is now a **two-engine browser**: Servo renders general web content (the unique non-Chromium engine story we committed to) while the system-native WebView handles modern media sites that require Media Source Extensions or Widevine/FairPlay DRM — engines where Servo has structural gaps that a single-session fix can't close. URL-pattern routing keeps this transparent to the user: hit YouTube or Netflix, a media window opens; hit Wikipedia or a blog, Servo paints.

**Engine matrix per OS:**
| OS | Media engine (via `wry 0.46`) | MSE | Widevine DRM | FairPlay DRM | Binary cost | Privacy posture |
|---|---|---|---|---|---|---|
| Windows | WebView2 (Edge/Chromium) | yes | yes (system CDM) | no | ~10 MB glue lib, runtime system-provided | SmartScreen + sync + background-net disabled via `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, profile under `%APPDATA%/amni-browse/webview2-data/` |
| macOS | WKWebView (Safari/WebKit) | yes | yes (Safari CDM, 10.14+) | yes (native) | ~0 (system framework) | WKWebView uses app sandbox, no telemetry by default |
| Linux | WebKitGTK 4.1 | yes | opt-in via `libwidevinecdm.so` install | no | ~0 (system `webkit2gtk-4.1`) | WebKitGTK is non-telemetry; DRM only fires if user explicitly installs Widevine (Google TOS opt-in) |

**Dispatch flow:**
```
User navigation → media_engine::route(url) → EngineKind
    ├── Servo  → Servo webview (existing path, unchanged)
    └── Media  → media_engine::spawn_media_window(event_loop, url)
                 ├── Windows: winit Window + wry WebView (WebView2 backend)
                 ├── macOS:   winit Window + wry WebView (WKWebView backend)
                 └── Linux:   winit Window + wry WebView (WebKitGTK backend)
```

**Module layout additions:**
- `src/platform/media_engine.rs` — `EngineKind` enum, `MEDIA_PATTERNS` list, `route(url)`, `spawn_media_window(event_loop, url)`, per-OS `configure_privacy_env()`, Linux `install_widevine()` / `widevine_installed()`, `platform_label()`.
- `src/engine/tabs.rs` — `TabEngine` enum (`Servo` / `Media`), `Tab.engine` field with `#[serde(default)]` for session back-compat.
- `src/platform/servo_real.rs` — `AppState.media_windows: HashMap<WindowId, MediaWindow>`, startup scans restored tabs for media-pattern matches and spawns child windows, `window_event` routes by `WindowId` so Servo input isn't poisoned by media-window events.

**Why this design (guardian council, 2026-04-18):**
- *Architect*: Two windows per media tab keeps the event loops cleanly separated — Servo's `spin_event_loop` and wry's internal pump don't step on each other. Later we can embed wry as a child `HWND` / `NSView` / `GdkWindow` inside the Servo winit window for a unified-window UX, but v1 ships sooner.
- *Sentinel*: Widevine is already on Windows + macOS via system-signed CDMs. No binary redistribution from us, no licensing ambiguity. Linux stays opt-in precisely to avoid that.
- *Scholar*: Servo's `servo_media_gstreamer` handles simple `<video src="file.mp4">` fine but its MSE implementation is stub-level; routing out to WebView for MSE-heavy sites is the pragmatic answer the Servo team themselves give.
- *Engineer*: `wry 0.46` already supports `raw-window-handle 0.6`, which is exactly what `winit 0.30` + our Servo embed already use. No new window system, no `tao` on this path, no dual event loop.
- *Pathfinder*: Ships today; we can iterate UX (tab indicator, right-click "open in media mode", settings panel) without changing the engine layer.

**Anti-goals:**
- We do NOT redistribute Widevine or FairPlay binaries. Widevine is Google-owned; FairPlay is Apple-only.
- We do NOT use WebView2 as the *primary* engine. That was explicitly rejected during v0.8.x for the "non-Chromium browsing" commitment — media mode is a narrow, signposted escape hatch.
- Servo's stub MSE implementation is not on the fix list for this release; we route around it instead of waiting on upstream.

## v0.8.2.2 — Real CSS Color Parser (2026-04-17)
Native (`servo-engine`) backend now parses the full CSS colour grammar. `engine/style.rs::parse_color` was a hex-only stub whose non-hex fallback was `Color { r:0, g:0, b:0, a:1.0 }` — **opaque black** — so every real-world page (DDG, Wikipedia, every stylesheet that uses `rgb()`, `rgba()`, named colors, `var(...)`, CSS shorthand with `url(...)`, or keywords like `transparent`/`currentcolor`) painted an opaque black rectangle over the whole page. The rewrite introduces three functions: `parse_color` dispatches (handles `transparent`/`inherit`/`initial`/`unset`/`currentcolor`/`none`/empty as fully transparent, tokenizes whitespace-separated shorthand values and returns the first parseable colour, passes single-token values to `parse_color_one`); `parse_color_one` handles hex (`#rgb`/`#rgba`/`#rrggbb`/`#rrggbbaa`) and `rgb(...)` / `rgba(...)` with comma or space/slash separators and percentage or 0–255 components; `parse_named_color` covers ~130 CSS3 named colors in a single dense match. Unknown values now return `Color { a:0.0 }` (transparent) instead of opaque black — the correct default for "I don't know what this is, don't paint over the existing canvas". Combined with v0.8.2.1's default-text-color fix, the render pipeline finally produces visible page content.

## v0.8.2.1 — Default Text Color + wgpu Log Filter (2026-04-17)
Native (`servo-engine`) backend now paints unstyled text as opaque black. `engine/paint.rs::RenderTree::walk` and `engine/pipeline.rs::RenderPipeline::build_tree` initialise `cs.color = Color { r:0, g:0, b:0, a:1.0 }` right after `ComputedStyle::default()` and before the CSS cascade runs. Default-derived `Color` has `a = 0.0` which silently made every un-coloured glyph render with zero coverage — the page was fetched, parsed, laid out, and rasterized correctly; only the glyphs were invisible. Opaque-black default matches browser UA behaviour. `src/main.rs` `env_logger` default filter now scopes `wgpu_core`/`wgpu_hal`/`naga`/`egui_wgpu` to WARN so v0.8.2's `Maintain::Poll` drain no longer spams the terminal — the Poll call itself is fine, the log line comes from `wgpu_core::device::resource` at INFO on every frame regardless of Poll vs Wait.

## v0.8.2 — Block Layout + URL-Bar Sync + wgpu Queue Drain (2026-04-17)
Native (`servo-engine`) backend now lays out normal web content correctly. `engine/layout.rs::to_taffy_style` maps `CssDisplay::Block` (and all non-Flex/Grid/None CSS displays) to `taffy::Display::Block` — taffy 0.7.7's native block-flow mode. Previously, the `_` arm shipped as `taffy::Display::Flex`, forcing every `<body>`/`<div>`/`<p>` into a single horizontal flex row with no wrap; `content_h` collapsed to the viewport minimum and pages painted as a tiny dark rectangle. `flex_direction`/`flex_grow`/`flex_shrink`/`gap` fields are inert under Block display and remain populated unchanged. `platform/servo.rs` gains a `last_tab_url: String` field on `AmniApp`; `render()` hoists `active_url` above `chrome.render()` and writes it into `chrome.url_input` on tab-change edges — no stomp on user typing because the gate is edge-triggered, not per-frame. `platform/servo.rs` also calls `gpu.device.poll(wgpu::Maintain::Poll)` after each `queue.submit` + `frame.present`, draining completed GPU submissions so `wgpu_core` stops logging `Device::maintain: waiting for submission index <N>` at INFO on every frame (the repro showed index > 29,000). Non-blocking variant — no stall, just drain. Inline flow (true mixed text+inline content) still falls through to Block and is deferred to the `engine/inline_layout.rs` integration in v0.9.x.

## v0.8.1 — Theme→egui Bridge + Event-Driven Reflow (2026-04-17)
Native (`servo-engine`) backend now bridges our `Theme` struct into `egui::Visuals` via `apply_theme_to_egui()` in `src/platform/servo.rs`. Called every frame when the active theme ID changes. Maps `bg_primary/secondary/tertiary/hover`, `border`, `text_primary/secondary`, and `accent` hex strings into `Visuals` fields (`panel_fill`, `window_fill`, `widgets.{noninteractive,inactive,hovered,active,open}.{bg_fill,fg_stroke,bg_stroke}`, `selection`, `hyperlink_color`, `override_text_color`). Auto-picks light or dark base by luma of `bg_primary`. wgpu clear color also derived from `bg_primary`. Before this bridge existed, egui ran in its default light visuals regardless of theme selection. Resize reflow is now event-driven: `WindowEvent::Resized` sets `pending_reflow = true`; render loop consumes the flag to trigger one `render_to_pixels` call without re-fetching. Chrome theme buttons now dispatch real theme IDs (`amni-dark` etc.) instead of `theme_0..theme_N`.

## v0.8.0 P3b-v2 — Click-Through-Egui + Resize Reflow (2026-04-17)
Native (`servo-engine`) clicks on the rendered page now route through egui's `Response` API instead of the raw `WindowEvent::MouseInput` path (which egui consumed before our handler saw it). The page texture is rendered via `egui::Image::from_texture(...).sense(egui::Sense::click())`; on `clicked()`, the content-space coordinate is computed as `(pointer_pos - resp.rect.min) / display_scale` and fed into `Interactor::dispatch_click`. Resize now reflows: `AmniApp` tracks `rendered_vw` + `rendered_css`, and when `ui.available_width()` drifts >8 px from `rendered_vw` (and no render is in flight) the frame spawns a render-only task that calls `pipeline.render_to_pixels(html, css, vw, vh)` directly — no network re-fetch. Reflow replies carry `reflow: true` so `check_rendered_pages` skips `run_page_scripts` (scripts already ran on the original paint). Single-flight guard (`render_pending`) prevents reflow storms during drag-resize.

## v0.8.0 P3b — Auto-Height Native Render + Live Viewport (2026-04-17)
Native (`servo-engine`) backend no longer paints to a hardcoded 1280×2048 canvas. `RenderPipeline::render_to_pixels` now derives the output height from layout: `content_h = max(rect.y + rect.h)` across all layout rects, floored at the viewport height and clamped to 16384 px (guard against stylesheet-driven OOM). Width stays `vw` (window content-box). `platform/servo.rs` passes live `ui.available_width()` / `ui.available_height()` (floored at 640×400) instead of the hardcoded constants, so the first paint after each navigation matches the real window. `ScrollArea::both` in the UI already scrolls the full image. Deferred to P3b-v2: re-render on window resize without re-fetch; click-coordinate translation through scroll offset + image scale.

## v0.7.2 — WebView2 Custom-Scheme Remap Awareness (2026-04-17)
Windows WebView2 rewrites every `amnibrowse://<host>/<path>` registered via `with_custom_protocol` to an internal `http://amnibrowse.<host>/<path>` origin at runtime. Both the init-script guard (`chrome_init_js`) and the `Act::Nav` dispatcher in `src/platform/webview.rs` must treat that internal origin as "ours": the guard bails on `location.hostname` starting with `amnibrowse.`, and `Act::Nav` pre-remaps any `amnibrowse://` target to the internal `http://amnibrowse.*/` form before `webview.load_url`.

## v0.7.0 — Amni Apps Launcher + Desktop Shortcut & Icon

## Core Stack
- **Language**: Rust 2021 Edition
- **Backend A (WebView)**: tao 0.30 + wry 0.46
- **Backend B (Servo-egui)**: winit 0.30 + wgpu 22 + egui 0.29 + egui-wgpu 0.29
- **Async**: tokio 1 (full)
- **Crypto**: AES-256-GCM + PBKDF2-HMAC-SHA256 (600K iter)
- **Network**: hyper 1 + hyper-rustls 0.27 + rustls 0.23 (custom HTTPS)
- **DOM**: html5ever 0.38 + markup5ever_rcdom 0.38
- **CSS**: cssparser 0.34 + selectors 0.26
- **Layout**: taffy 0.7 (flexbox/grid)

## 7-Pillar Module Structure
```
src/
├── main.rs                    Entry point, feature-gated backend selection
├── app.rs                     Shared BrowserState, handle_command dispatcher
├── ui/                        [Pillar 1: UI]
│   ├── mod.rs                 chrome(servo), webview(webview), theme, reader, emoji
│   ├── chrome.rs              Native egui browser chrome (servo backend)
│   ├── webview.rs             HTML/CSS/JS SPA (webview backend)
│   ├── theme.rs               5 builtin themes + custom, CSS var generation
│   ├── reader.rs              Reader mode, content extraction
│   └── emoji.rs               Centralized emoji atlas (65+ symbols)
├── net/                       [Pillar 2: Communication]
│   ├── mod.rs                 ipc, dns, http, cookies
│   ├── ipc.rs                 IpcMessage/IpcResponse enums (70+ types)
│   ├── dns.rs                 DoH resolver (Cloudflare/Google/Quad9)
│   ├── http.rs                Custom HTTPS client (hyper+rustls, caching, DNT)
│   └── cookies.rs             Privacy cookie jar (3rd-party blocking)
├── storage/                   [Pillar 3: Storage]
│   ├── mod.rs                 config, bookmarks, history, session, downloads, profiles
│   ├── config.rs              Settings, paths, data clearing
│   ├── bookmarks.rs           Bookmark CRUD, JSON persistence
│   ├── history.rs             Visit tracking, search, date grouping
│   ├── session.rs             Session save/restore, crash recovery
│   ├── downloads.rs           Async file downloads, progress
│   └── profiles.rs            Multi-profile with isolated data dirs
├── crypto/                    [Pillar 4: Encryption]
│   ├── mod.rs                 vault, autofill
│   ├── vault.rs               AES-256-GCM password vault, PBKDF2 KDF
│   └── autofill.rs            Addresses + encrypted cards
├── media/                     [Pillar 5: Media — placeholder]
│   └── mod.rs
├── platform/                  [Pillar 6: OS/Platform]
│   ├── mod.rs                 webview(webview), servo(servo-engine)
│   ├── webview.rs             tao+wry WebView launcher
│   └── servo.rs               winit+wgpu+egui compositor
└── engine/                    [Pillar 7: Engine]
    ├── mod.rs                 tabs, adblocker, extensions, permissions, devtools, dom, layout, style
    ├── tabs.rs                Tab lifecycle, nav history, split view, zoom
    ├── adblocker.rs           Domain/pattern block, URL cleaning
    ├── extensions.rs          Manifest loading, content scripts
    ├── permissions.rs         Per-site permissions (8 types)
    ├── devtools.rs            Console + network logging
    ├── dom.rs                 html5ever DOM parser, meta extraction, reader content
    ├── style.rs               CSS parser (cssparser), computed styles, cascade
    ├── layout.rs              Flexbox/grid layout engine (taffy)
    ├── pipeline.rs            Render pipeline: HTTP→DOM→CSS→Layout orchestrator
    └── app_launcher.rs        Amni Apps registry, process spawner (allowlist)
```

## Dual-Backend Architecture
```
┌────────────────────────────────────────────────────┐
│                    main.rs                          │
│  Feature gate: webview (default) OR servo-engine   │
└──────────────┬────────────────┬────────────────────┘
               │                │
    ┌──────────▼────────┐  ┌───▼──────────────────┐
    │ platform/webview   │  │ platform/servo        │
    │ (tao + wry)        │  │ (winit + wgpu)        │
    │                    │  │                        │
    │ ┌────────────────┐ │  │ ┌──────────────────┐  │
    │ │ui/webview(HTML)│ │  │ │ui/chrome (egui)  │  │
    │ └────────────────┘ │  │ └──────────────────┘  │
    └────────┬───────────┘  └──────────┬────────────┘
             │                         │
             └──────────┬──────────────┘
                        │
             ┌──────────▼──────────┐
             │      app.rs         │
             │  BrowserState       │
             │  handle_command()   │
             │  17 fields          │
             │  async_tx + notify  │
             └──────────┬──────────┘
                        │
    ┌───────┬───────┬───┴───┬─────────┬──────────┐
    │engine/│storage│crypto/│  net/   │ ui/theme  │
    │tabs   │config │vault  │  ipc    │ ui/reader │
    │adblock│bmarks │autfil │  dns    │ ui/emoji  │
    │extns  │history│       │  http   │           │
    │perms  │sessn  │       │  cookie │           │
    │devtls │dloads │       │         │           │
    │dom    │profs  │       │         │           │
    │style  │       │       │         │           │
    │layout │       │       │         │           │
    │pipeln │       │       │         │           │
    └───────┴───────┴───────┴─────────┴───────────┘
```

## Data Flow
```
WebView path (v0.5.0 — full IPC round-trip):
  SPA URL bar → navigate() → sendIpc({type:'navigate',url}) 
    → wry ipc_handler → parse_ipc_message() → IpcMessage::Navigate
    → BrowserState::handle_command() → NavigateTo{url}
    → Act::Nav(url) → webview.load_url(url) → real page loads
    → chrome_init_js() injects toolbar on http/https pages
    → toolbar buttons → window.ipc.postMessage(JSON) → back to IPC handler

  Home button → sendIpc({type:'navigate',url:'amnibrowse://newtab'})
    → NavigateTo → Act::Nav → detects amnibrowse:// → base64 data URI
    → SPA reloads → sendIpc(get_tabs/get_stats/...) → IPC round-trip

  Back/Forward → handle_command → Tab::go_back/forward → NavigateTo{url}
    → Act::Nav → webview.load_url (navigates to previous/next URL)

  Ad blocking → with_navigation_handler → AdBlocker::is_blocked_url
    → blocks 60+ ad/tracker domains at main-frame navigation level

Engine-independent path (v0.6.1 — async delivery LIVE):
  SPA Ctrl+K → "Engine Fetch Page" (Ctrl+Shift+E)
    → sendIpc({type:'fetch_page',url})
    → app.rs FetchPage handler → tokio::spawn(async)
    → RenderPipeline::fetch_and_parse(url):
      → AmniClient::get(url) [hyper+rustls, DNT, Sec-GPC, caching]
      → AmniDom::parse(html) [html5ever, scoped block]
      → extract_meta() → PageSummary
      → resolve & fetch CSS <link> stylesheets → css_sources
    → IpcResponse::PageRendered{url, title, html, meta}
    → async_tx.send(resp.to_js_call())
    → async_notify() → EventLoopProxy::send_event(())
    → UserEvent → async_rx.try_recv() → webview.evaluate_script()
    → window.__amni_receive({type:'page_rendered'})
    → engine-viewer overlay displays fetched content

  Reader mode (engine-powered, async delivery):
    SPA → "Reader Fetch (Engine)" → sendIpc({type:'reader_fetch',url})
    → tokio::spawn → RenderPipeline::fetch_reader(url)
    → AmniClient::get → AmniDom::parse → extract_reader_content()
    → ReaderMode::render_html() → IpcResponse::ReaderHtml
    → async_tx → notify → UserEvent → webview → SPA reader overlay

  Page meta (engine-powered, async delivery):
    SPA → "Page Meta (Engine)" → sendIpc({type:'page_meta',url})
    → tokio::spawn → fetch_and_parse → PageMetaResp
    → async_tx → notify → UserEvent → webview → status bar update

  Reader mode (IPC content, now with Rust parser):
    ReaderContent{title, content} → AmniDom::parse(content)
    → extract_reader_content() → cleaned article HTML
    → ReaderMode::render_html() → styled output

  Full layout computation (available, not yet surfaced):
    RenderPipeline::parse_and_layout(html, css, viewport)
    → AmniDom::parse → StyleSheet::parse (per CSS source)
    → build_tree: selector_matches → apply_declarations → ComputedStyle
    → LayoutEngine::add_node/add_leaf → LayoutEngine::compute(taffy)
    → LayoutResult{rects, node_count}

Servo-egui path:
  User Action → egui widget events → chrome::BrowserChrome::cmd()
    → drain_commands() → app::BrowserState::handle_command()
    → apply_to_chrome() updates chrome state
    → egui re-renders on next frame
```

## Feature Flags (Cargo.toml)
| Feature | Deps | Purpose |
|---------|------|---------|
| `webview` (default) | tao, wry | System WebView rendering |
| `servo-engine` | winit, wgpu, egui, egui-winit, egui-wgpu, raw-window-handle, pollster | Custom GPU rendering |

## Storage (local only, zero cloud)
| File | Purpose |
|------|---------|
| config.json | Browser settings |
| bookmarks.json | Bookmarks |
| vault.enc.json | AES-256-GCM encrypted passwords |
| theme.json | Active theme + customs |
| history.json | Browsing history |
| session.json | Session state + crash recovery |
| downloads.json | Download records |
| autofill.json | Addresses + encrypted cards |
| permissions.json | Per-site permissions |
| profiles.json | Profile metadata |
| extensions.json | Extension registry |

## File Inventory (37 files: 30 source + 7 mod.rs, ~6,800+ LOC)
| File | Backend | Pillar | Purpose |
|------|---------|--------|---------|
| main.rs | Both | — | Entry point, feature-gated backend selection |
| app.rs | Both | — | Shared BrowserState, 80+ IPC command dispatch |
| ui/chrome.rs | Servo | UI | Native egui browser chrome (10 panels) |
| ui/webview.rs | WebView | UI | HTML/CSS/JS SPA browser chrome + Ctrl+K palette |
| ui/theme.rs | Both | UI | 5 builtin themes + custom, CSS vars |
| ui/reader.rs | Both | UI | Reader mode, content extraction |
| ui/emoji.rs | Both | UI | Centralized emoji atlas (65+ symbols) |
| net/ipc.rs | Both | Communication | IpcMessage/IpcResponse enums (70+ types) |
| net/dns.rs | Both | Communication | DoH resolver (Cloudflare/Google/Quad9) |
| net/http.rs | Both | Communication | Custom HTTPS client (hyper+rustls, caching) |
| net/cookies.rs | Both | Communication | Privacy cookie jar (3rd-party blocking) |
| storage/config.rs | Both | Storage | Settings, paths, data clearing |
| storage/bookmarks.rs | Both | Storage | Bookmark CRUD, JSON persistence |
| storage/history.rs | Both | Storage | Visit tracking, search, date grouping |
| storage/session.rs | Both | Storage | Session save/restore, crash recovery |
| storage/downloads.rs | Both | Storage | Async file downloads, progress |
| storage/profiles.rs | Both | Storage | Multi-profile, isolated data dirs |
| crypto/vault.rs | Both | Encryption | AES-256-GCM vault, PBKDF2 KDF |
| crypto/autofill.rs | Both | Encryption | Addresses + encrypted cards |
| media/mod.rs | — | Media | Placeholder |
| platform/webview.rs | WebView | Platform | tao+wry WebView launcher |
| platform/servo.rs | Servo | Platform | winit+wgpu+egui compositor |
| engine/tabs.rs | Both | Engine | Tab lifecycle, split view, zoom |
| engine/adblocker.rs | Both | Engine | Domain/pattern block, URL cleaning |
| engine/extensions.rs | Both | Engine | Manifest loading, content scripts |
| engine/permissions.rs | Both | Engine | Per-site permissions (8 types) |
| engine/devtools.rs | Both | Engine | Console + network logging |
| engine/dom.rs | Both | Engine | html5ever DOM parser, meta/article extraction |
| engine/style.rs | Both | Engine | CSS parser (cssparser), computed styles |
| engine/layout.rs | Both | Engine | Flexbox/grid layout engine (taffy) |
| engine/pipeline.rs | Both | Engine | Render pipeline: HTTP→DOM→CSS→Layout |

## Roadmap: AmniShunt (v0.5+)
See AMNISHUNT_DESIGN.md and GUARDIAN_COUNCIL_v0.5.md for the next-gen rendering pipeline:
- Custom WebKit-Servo translation/shunt layer
- Septidecimal (base-17) encoded intermediate representation
- Process-isolated sandbox with capability model
- GPU-accelerated HTML tokenization
- Full independence from WebView and Servo rendering

## Feature Matrix
| Feature | Status | Module | Backend |
|---------|--------|--------|---------|
| Multi-tab browsing | Done | engine/tabs | Both |
| Back/Forward/Refresh | Done | engine/tabs | Both |
| URL bar + search | Done | platform/webview, ui/chrome | Both |
| Split view | Done | engine/tabs | Both |
| Bookmarks | Done | storage/bookmarks | Both |
| Ad/tracker blocking | Done | engine/adblocker | Both |
| Tracking param strip | Done | engine/adblocker | Both |
| Password vault (AES-256) | Done | crypto/vault | Both |
| 5 themed skins + custom | Done | ui/theme | Both |
| Data clearing | Done | storage/config | Both |
| Keyboard shortcuts | Done | ui/webview, ui/chrome | Both |
| Privacy stats | Done | ui/webview, ui/chrome | Both |
| Download manager | Done | storage/downloads | Both |
| Browsing history | Done | storage/history | Both |
| Find in page | Done | ui/webview, ui/chrome | Both |
| Autofill (forms+cards) | Done | crypto/autofill | Both |
| Session restore | Done | storage/session | Both |
| Private browsing | Done | engine/tabs | Both |
| Zoom controls | Done | engine/tabs | Both |
| Reader mode | Done | ui/reader | Both |
| Permissions mgr | Done | engine/permissions | Both |
| DNS over HTTPS | Done | net/dns | Both |
| Dev tools | Done | engine/devtools | Both |
| Extensions API | Done | engine/extensions | Both |
| Profile manager | Done | storage/profiles | Both |
| Dual backend (WebView+Servo) | Done | platform/* | Both |
| Native egui chrome | Done | ui/chrome | Servo |
| wgpu GPU rendering | Done | platform/servo | Servo |
| 7-Pillar modular structure | Done | v0.4.1 | Both |
| Amni Apps Launcher | Done | engine/app_launcher | WebView |
| Desktop shortcut + icon | Done | scripts/create_shortcut.ps1 | Windows |
| Embedded .ico in binary | Done | build.rs + embed-resource | Windows |
| AmniShunt rendering | Planned | v0.5 | Servo |
| Septidecimal IR encoding | Planned | v0.5 | Servo |
| Process-isolated sandbox | Planned | v0.5 | Servo |
| Custom HTML tokenizer | Planned | v0.5 | Servo |
| CSS engine | Planned | v0.6 | Servo |
| Layout engine | Planned | v0.6 | Servo |
| JavaScript engine | Planned | v0.8 | Servo |
| Amni-Ai integration | Planned | v0.9+ | Both |
