# Amni Browse v0.7.0

**A privacy-first, zero-telemetry web browser built from the ground up in Rust.**
**Now with functional web browsing: navigate to real URLs, injected privacy toolbar, ad blocking at navigation level.**

![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)
![License](https://img.shields.io/badge/License-CC%20BY--NC%204.0-00d4ff)
![Privacy](https://img.shields.io/badge/Telemetry-ZERO-green)
![Backends](https://img.shields.io/badge/Backends-WebView%20%7C%20Servo-purple)
![Source](https://img.shields.io/badge/Source-Available-lightgrey)

---

## 🔒 Privacy by Default

Amni Browse is designed with a single principle: **your browsing is yours.**

- ✅ **Zero telemetry** — no data ever leaves your device
- ✅ **Built-in ad & tracker blocker** — no extensions needed
- ✅ **Tracking parameter stripping** — UTM, fbclid, gclid, etc. are auto-removed
- ✅ **No third-party cookies** by default
- ✅ **Do Not Track** header sent by default
- ✅ **DuckDuckGo** as default search engine
- ✅ **Local-only storage** — bookmarks, settings, everything stays on your machine
- ✅ **Private browsing tabs** — no history recorded for private tabs
- ✅ **DNS-over-HTTPS** — encrypted DNS with Cloudflare/Google/Quad9 providers

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.70+)

**WebView backend (default):**
- Windows: WebView2 Runtime (pre-installed on Windows 10/11)
- Linux: `libwebkit2gtk-4.1-dev` and `libgtk-3-dev`
- macOS: No extra deps (uses WKWebView)

**Servo-egui backend:**
- GPU with Vulkan, DX12, or Metal support
- No system WebView required

### Build & Run

```bash
# Quick launch (Windows)
run.bat

# Or build manually
cargo build --release
cargo run --release

# Create desktop shortcut (Windows, pinnable to taskbar)
powershell scripts/create_shortcut.ps1
cargo run --no-default-features --features servo-engine

# Release build
cargo build --release
cargo build --release --no-default-features --features servo-engine
```

## Architecture (v0.5.0 — Functional Navigation Pipeline)

```
src/
├── main.rs           Entry point (feature-gated backend selection)
├── app.rs            Shared BrowserState + IPC command handler
├── ui/               [UI Pillar]
│   ├── chrome.rs     Native egui browser chrome [servo-engine]
│   ├── webview.rs    HTML/CSS/JS SPA chrome [webview]
│   ├── theme.rs      5 built-in + custom themes
│   └── reader.rs     Reader mode
├── net/              [Communication Pillar]
│   ├── ipc.rs        IPC protocol (70+ message types)
│   └── dns.rs        DNS-over-HTTPS resolver
├── storage/          [Storage Pillar]
│   ├── config.rs     Settings, paths, defaults
│   ├── bookmarks.rs  Local bookmark storage
│   ├── history.rs    Browsing history with search
│   ├── session.rs    Session save/restore, crash recovery
│   ├── downloads.rs  Async file downloads
│   └── profiles.rs   Multi-profile support
├── crypto/           [Encryption Pillar]
│   ├── vault.rs      AES-256-GCM vault (PBKDF2-HMAC-SHA256)
│   └── autofill.rs   Encrypted payment cards + addresses
├── media/            [Media Pillar — v0.5+]
├── platform/         [OS/Platform Pillar]
│   ├── webview.rs    tao+wry WebView launcher [webview]
│   └── servo.rs      winit+wgpu+egui compositor [servo-engine]
└── engine/           [Engine Pillar]
    ├── tabs.rs       Tab lifecycle, split view, zoom
    ├── adblocker.rs  Ad/tracker blocking
    ├── extensions.rs Extension system
    ├── permissions.rs Per-site permissions
    ├── devtools.rs   Developer console + network
    └── app_launcher.rs Amni Apps registry + process spawner
```

### Backends

| Backend | Feature Flag | Rendering | Dependencies |
|---------|-------------|-----------|-------------|
| **WebView** (default) | `webview` | System WebView (Chromium/WebKit) | tao, wry |
| **Servo-egui** | `servo-engine` | Custom wgpu + egui | winit, wgpu, egui, egui-wgpu |

The Servo-egui backend provides fully native browser chrome via egui/wgpu, independent of any system WebView. This is the foundation for the planned **AmniShunt** rendering engine (v0.5+).

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+L` | Focus URL bar |
| `Ctrl+D` | Bookmark page |
| `Ctrl+R` | Refresh |
| `Ctrl+F` | Find in page |
| `Ctrl+H` | History |
| `Ctrl+J` | Downloads |
| `Ctrl+=` | Zoom in |
| `Ctrl+-` | Zoom out |
| `Ctrl+0` | Reset zoom |
| `Ctrl+Shift+P` | Password vault |
| `Ctrl+Shift+I` | Developer tools |
| `Ctrl+Shift+N` | New private tab |
| `Alt+←` | Go back |
| `Alt+→` | Go forward |
| `Esc` | Close panel / find bar |

## 🔐 Password Vault

- AES-256-GCM authenticated encryption
- PBKDF2-HMAC-SHA256 key derivation (600,000 iterations)
- Master password never stored — only derived key held in memory while unlocked
- Password generator with configurable length
- Credential autofill integration

## 📥 Downloads

- Async file downloading with progress tracking
- Auto-filename extraction from URL and Content-Disposition headers
- Cancel, remove, and clear completed downloads
- Persistent download history

## 🕐 History & Sessions

- Full browsing history with search and date grouping
- Visit count tracking and deduplication
- Session save/restore on startup
- Crash recovery via lock-file detection

## 🧩 Extensions

- Manifest-based extension loading from `extensions/` directory
- Content script injection with URL pattern matching
- Enable/disable/remove extensions at runtime

## � Amni Apps

Launch other Amni-Scient software directly from the browser via the **Amni Apps** panel (context menu or command palette):

| App | Type | Description |
|-----|------|-------------|
| Amni AI | Local | Qwen3.5-122B AI assistant (Gradio) |
| Azno v2 | Local | GPU-accelerated trading platform |
| Amni Mail | Local | Privacy-first email client |
| Amni Gen | Local | AI image generation (ROCm/ZLUDA) |
| Amni Calc | Local | Septidecimal WASM calculator |
| Amni Explore | Local | 3D exoplanet exploration |
| Amni Miner | Local | Data mining dashboard |
| Amni Game | Local | Rust game engine |
| Amni Coder | Web | AI code editor (example.com) |
| Amni-Scient | Web | Main product site |

## �👤 Multi-Profile

- Isolated data directories per profile
- Create, switch, rename, and delete profiles
- Default profile always available

## 🛡️ Ad & Tracker Blocking

The built-in blocker covers:
- Major ad networks (DoubleClick, Google Ads, etc.)
- Facebook/Meta tracking pixels
- Analytics platforms (Google Analytics, Mixpanel, Hotjar, etc.)
- Social media trackers
- Fingerprinting scripts
- URL tracking parameters (UTM, click IDs, etc.)

All filter rules are bundled in the binary — no external downloads needed.

## 📁 Data Storage

All data is stored locally in your OS config directory:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\amni-browse\` |
| macOS | `~/Library/Application Support/amni-browse/` |
| Linux | `~/.config/amni-browse/` |

Files stored:
- `config.json` — Browser settings
- `bookmarks.json` — Bookmarks
- `vault.json` — Encrypted password vault
- `history.json` — Browsing history
- `session.json` — Session state
- `downloads.json` — Download records
- `autofill.json` — Autofill data (cards encrypted)
- `permissions.json` — Site permissions
- `profiles.json` — Profile metadata

## Tech Stack

**Shared (both backends):**
- **Rust** — Systems programming, 2021 edition
- **serde/serde_json** — Serialization
- **tokio** — Async runtime
- **reqwest** — HTTP client
- **aes-gcm + pbkdf2** — AES-256-GCM encryption, PBKDF2-SHA256 KDF
- **chrono, uuid, regex, dirs** — Utilities

**WebView backend:**
- **wry 0.46** — Cross-platform WebView rendering
- **tao 0.30** — Cross-platform windowing

**Servo-egui backend:**
- **winit 0.30** — Cross-platform windowing
- **wgpu 22** — GPU rendering (Vulkan/DX12/Metal)
- **egui 0.29** — Immediate-mode GUI
- **egui-wgpu 0.29** — egui GPU renderer
- **pollster** — Blocking async executor

## Roadmap

| Version | Focus | Key Features |
|---------|-------|-------------|
| v0.5.0 (current) | Navigation | Functional browsing, toolbar injection, ad blocking, IPC round-trip |
| v0.6 | AmniShunt | Custom HTML tokenizer, DOM tree, septidecimal IR, sandbox |
| v0.7 | Styling | CSS cascade + specificity, block/flex layout |
| v0.8 | Compositing | GPU paint pipeline, layer compositing, scrolling |
| v0.9 | Scripting | JavaScript engine integration |
| v1.0 | Independence | Full standalone browser, no legacy dependencies |

See [AMNISHUNT_DESIGN.md](AMNISHUNT_DESIGN.md) for the v0.5+ technical architecture.

## License

**CC BY-NC 4.0** — [Creative Commons Attribution-NonCommercial 4.0 International](https://creativecommons.org/licenses/by-nc/4.0/).

- ✅ View, study, fork, and modify the source
- ✅ Share and redistribute non-commercially with attribution
- ❌ No commercial use, no resale, no SaaS, no paid redistribution
- ❌ No use of the "Amni-Browse", "Amni-Scient", or "Amnibro" trademarks

**Source-available, not permissive.** This is open source you can learn from, contribute to, and self-host — but not one you can package and sell. For commercial licensing, email `the maintainer (via GitHub)`.

See [`LICENSE`](LICENSE) for full terms.

---

*Built with Rust and a deep respect for privacy. By Amni-Scient.*
