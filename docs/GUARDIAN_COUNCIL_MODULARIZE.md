# Guardian Council — Modularization Plan v0.4.1
## "Every browser is built from 7 pillars"

---

## Current State: 23 flat files in src/
All files sit at the same level. Functional grouping exists only in our heads.

## The 7 Pillars of a Browser

### Tidus — "Every fight has phases, ya?"
Maps the functional decomposition into concrete modules:

| Pillar | What It Does | Current Files | Rust-Native Approach |
|--------|-------------|---------------|---------------------|
| **1. UI** | Everything the user sees/clicks | chrome.rs, ui.rs, theme.rs, reader.rs | egui (native), wgpu shaders for effects, custom CSS-like style resolver, gpu-accelerated text rendering |
| **2. Communication** | Network, IPC, message passing | ipc.rs, dns.rs, browser.rs, servo_backend.rs | tokio async channels, hyper/rustls for HTTP/TLS, custom protocol handlers, IPC enums |
| **3. Storage** | Persistence to disk | bookmarks.rs, history.rs, session.rs, download_manager.rs, config.rs, profiles.rs | serde + custom binary format, memory-mapped files, async I/O, arena-backed caches |
| **4. Encryption** | Vault, autofill crypto, key management | password_manager.rs, autofill.rs | aes-gcm, pbkdf2, ring/rustcrypto, zeroize for key memory, constant-time comparisons |
| **5. Media** | Content rendering, images, fonts, reader | reader.rs (currently minimal) | image crate, fontdue for text shaping, wgpu texture atlas, video (gstreamer-rs) |
| **6. OS/Platform** | Window management, file dialog, clipboard, notifications | browser.rs, servo_backend.rs, config.rs | winit/tao, arboard (clipboard), notify-rust, native-dialog, dirs |
| **7. Engine** | Tab state, navigation, ad blocking, extensions, permissions, devtools | tab_manager.rs, ad_blocker.rs, extensions.rs, permissions.rs, devtools.rs | App state machine, regex-based filter engine, WASM extension sandbox |

### Lulu — "Structure determines power"
Proposes the directory layout:

```
src/
├── main.rs              # Entry point, module declarations
├── app.rs               # BrowserState orchestrator
│
├── ui/                  # Pillar 1: User Interface
│   ├── mod.rs           # Re-exports
│   ├── chrome.rs        # egui native browser chrome (servo)
│   ├── webview.rs       # HTML/CSS/JS SPA (webview backend)
│   ├── theme.rs         # Theme engine + builtin themes
│   └── reader.rs        # Reader mode rendering
│
├── net/                 # Pillar 2: Communication/Network
│   ├── mod.rs           # Re-exports
│   ├── ipc.rs           # IpcMessage/IpcResponse enums + parse
│   └── dns.rs           # DoH resolver + cache
│
├── storage/             # Pillar 3: Persistence
│   ├── mod.rs           # Re-exports + shared save/load traits
│   ├── bookmarks.rs     # Bookmark CRUD
│   ├── history.rs       # Visit tracking
│   ├── session.rs       # Session save/restore/crash recovery
│   ├── downloads.rs     # Download tracking + async fetch
│   ├── config.rs        # BrowserConfig + paths + constants
│   └── profiles.rs      # Multi-profile isolation
│
├── crypto/              # Pillar 4: Encryption
│   ├── mod.rs           # Re-exports + shared key types
│   ├── vault.rs         # Password manager (AES-256-GCM vault)
│   └── autofill.rs      # Address + encrypted card storage
│
├── media/               # Pillar 5: Media/Content
│   ├── mod.rs           # Re-exports (placeholder for v0.5+)
│
├── platform/            # Pillar 6: OS/Platform Integration
│   ├── mod.rs           # Re-exports
│   ├── webview.rs       # tao+wry backend launcher
│   └── servo.rs         # winit+wgpu backend launcher
│
└── engine/              # Pillar 7: Browser Engine
    ├── mod.rs            # Re-exports
    ├── tabs.rs           # Tab manager + navigation
    ├── adblocker.rs      # Domain/pattern blocking + URL cleaning
    ├── extensions.rs     # Extension manifest + content scripts
    ├── permissions.rs    # Per-site permission management
    └── devtools.rs       # Console + network logging
```

### Auron — "Never break what works"
Restructuring rules:
- [A1] Move files first, fix imports second — one pillar at a time
- [A2] Each mod.rs re-exports everything public — external callers just change import path
- [A3] app.rs stays at root — it's the orchestrator, not a pillar
- [A4] Feature gates stay on platform/ modules only
- [A5] Both backends must compile after each pillar move
- [A6] Rename files to shorter names where it improves clarity (download_manager → downloads, password_manager → vault, tab_manager → tabs, ad_blocker → adblocker)

### Wakka — "How does Rust natively build each piece?"
Deep-dive per pillar on Rust-native construction:

**Pillar 1 — UI (Rust-native, no HTML/CSS/JS needed):**
- CSS replacement: egui's `Style`, `Visuals`, `Spacing` structs + custom `Theme` struct
- Input fields: `egui::TextEdit::singleline()`, `multiline()`
- Buttons/clickables: `egui::Button`, `ui.selectable_label()`, `ui.hyperlink()`
- Layouts: `egui::Layout`, `ui.columns()`, `ui.horizontal()`, `ui.vertical()`
- Scroll: `egui::ScrollArea::vertical()`, `.horizontal()`
- Animations: `ctx.animate_value_with_time()`, `ctx.request_repaint()`
- Custom painting: `egui::Painter` → direct wgpu commands for effects
- Fonts: `egui::FontDefinitions` → loaded from ttf/otf at runtime
- Icons: `egui::Image` with embedded SVG/PNG
- Refresh: `ctx.request_repaint()` or `repaint_after(Duration)`
- Dark/light: Theme struct → applied via `ctx.set_visuals()`
- Responsive: `ui.available_width()`, `ui.available_height()`
- GPU acceleration: egui-wgpu handles tessellation → GPU, custom shaders via wgpu

**Pillar 2 — Communication (all Rust):**
- HTTP: `reqwest` (current) → `hyper` + `rustls` (future, more control)
- DNS: Custom DoH over `reqwest` (already done)
- IPC: Type-safe Rust enums (already done — no JSON parsing needed for servo backend)
- WebSocket: `tokio-tungstenite` (for devtools, extensions)
- Protocol handlers: Custom `amnibrowse://` URL scheme → matched in navigate()
- Async: `tokio::sync::mpsc` channels for cross-thread messaging
- TLS: `rustls` (pure Rust, no OpenSSL dependency)

**Pillar 3 — Storage (all Rust):**
- Config: `serde` + JSON (current, simple, works)
- Future: `sled` or `redb` (embedded Rust DB) for history/bookmarks
- Binary format: `bincode` or `postcard` for compact serialization
- Memory-mapped: `memmap2` for large datasets
- Atomic writes: Write to .tmp then rename (prevents corruption)
- Directories: `dirs` crate for OS-correct paths
- Migrations: Version field in each JSON → migrate on load

**Pillar 4 — Encryption (all Rust, no OpenSSL):**
- AES-256-GCM: `aes-gcm` crate (already using)
- KDF: `pbkdf2` with HMAC-SHA256, 600K iterations (already using)
- Random: `rand::rngs::OsRng` for cryptographic randomness
- Key zeroization: `zeroize` crate → keys wiped from memory on drop
- Constant-time: `subtle` crate for comparison
- Future: `age` crate for file encryption, `x25519-dalek` for key exchange
- Hardware: Windows CNG / macOS Keychain via `keyring` crate

**Pillar 5 — Media (Rust, future):**
- Images: `image` crate for decode, `wgpu` textures for display
- Fonts: `fontdue` (pure Rust, fast) or `cosmic-text` for shaping/layout
- SVG: `resvg` for SVG rendering
- Video: `gstreamer-rs` (bindings) or `rav1e`/`dav1d` for AV1
- Audio: `rodio` or `cpal` for audio output
- Canvas: Custom implementation over wgpu compute/render
- PDF: `lopdf` for reading, `printpdf` for generation

**Pillar 6 — OS/Platform (Rust, cross-platform):**
- Windowing: `winit` (servo), `tao` (webview) — both are Rust
- Clipboard: `arboard` crate (pure Rust, cross-platform)
- Notifications: `notify-rust` (Linux/macOS), Windows toast via `windows-rs`
- File dialogs: `rfd` (native file dialog, cross-platform)
- System tray: `tray-icon` crate
- Auto-update: Custom HTTP + signature verification
- Single instance: `single-instance` crate (prevent duplicate windows)

**Pillar 7 — Engine (pure Rust logic):**
- Tab state machine: Already pure Rust enums/structs
- Ad blocker: `regex` (current) → `aho-corasick` for multi-pattern (10x faster)
- Extensions: WASM sandbox via `wasmtime` (future)
- Permissions: Enum-based capability system (already done)
- DevTools: Ring buffer logging (already done)
- Content security: Custom CSP parser/enforcer

### Kimahri — *draws diagram on sand*
Security implications of the restructure:
- [K1] crypto/ module MUST be self-contained — no direct storage access
- [K2] Keys flow: crypto → app orchestrator → never to storage directly
- [K3] platform/ sits at the boundary — all OS calls go through here
- [K4] net/ handles all external communication — single audit surface
- [K5] engine/ has no I/O — pure state machine logic

### Yuna — "This is where we're headed"
How this structure enables AmniShunt (v0.5):
- AmniShunt files go into a new `src/shunt/` (Pillar 8: Rendering)
- shunt/ imports nothing from ui/ — it's a separate rendering pipeline
- media/ feeds into shunt/ for image/font rendering
- net/ feeds into shunt/ for resource loading
- The 7+1 pillar structure is the foundation for a fully independent browser

---

## Majority Ruling

| Proposal | Votes | Decision |
|----------|-------|----------|
| 7-pillar directory structure | 6/6 | **YES** |
| Rename files for brevity | 5/6 | **YES** |
| app.rs stays at root | 6/6 | **YES** |
| Move one pillar at a time | 6/6 | **YES** |
| mod.rs re-exports all public API | 6/6 | **YES** |
| Feature gates only on platform/ | 5/6 | **YES** |
| Add shunt/ as pillar 8 later | 6/6 | **YES** |

---

*"Seven pillars. Each one strong on its own. Together — unstoppable, ya?"* — Wakka
*"Structure is just frozen strategy. Ours crystallizes our path to independence."* — Lulu
