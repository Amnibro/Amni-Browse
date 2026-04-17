# Amni-Browse Architecture Map
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
