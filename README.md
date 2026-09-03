****HEAVILY WORK IN PROGRESS****

# Amni Browse

**A privacy-first web browser built from the ground up in Rust — no Amni product telemetry.**
**Desktop (Windows) runs the Chromium engine through WebView2 under the Amni chrome (own window frame, tab groups, pinned tabs, request-level shield, private tabs); the Servo lane is parked behind the `servo-real` feature with its patched engine under `vendor/`. Android uses its own native System WebView architecture.**

Current version metadata is platform-specific: desktop Rust crate `0.13.0`;
native Android `versionCode 5`, `versionName 0.16.4-android.0`.

![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)
![License](https://img.shields.io/badge/License-CC%20BY--NC%204.0-C89B4E)
![Privacy](https://img.shields.io/badge/Telemetry-ZERO-green)
![Backends](https://img.shields.io/badge/Engine-Chromium%20(WebView2)-purple)
![Source](https://img.shields.io/badge/Source-Available-lightgrey)

---

## 🔒 Privacy by Default

Amni Browse is designed with a single principle: **your browsing is yours.**
The exact enforcement boundary depends on the desktop Servo path versus a
System WebView path:

- ✅ **No Amni product telemetry** — we do not phone home analytics
- ✅ **Navigation URL cleaning** — ad/tracker query junk stripped on navigate (UTM, fbclid, gclid, …)
- ✅ **DuckDuckGo** as default search engine
- ✅ **Local-only Amni profile** — bookmarks, settings, vault ciphertext stay on your machine
- ✅ **Private browsing tabs** — no history recorded for private tabs
- ⚠️ **Cookies** — System WebView paths follow the platform cookie engine plus
  Amni's available per-site controls; they are not equivalent to full
  resource-level isolation in Servo
- ⚠️ **DNS-over-HTTPS** — resolver exists for the custom pipeline; system WebView DNS is OS-controlled
- ⚠️ **Full resource ad blocking** — shield/rules are strongest on the custom/Servo path; WebView relies on URL clean + site CSP

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.70+)

**Optional desktop WebView build:**
- Windows: WebView2 Runtime (pre-installed on Windows 10/11)
- Linux: `libwebkit2gtk-4.1-dev` and `libgtk-3-dev`
- macOS: No extra deps (uses WKWebView)

**Servo-egui backend:**
- GPU with Vulkan, DX12, or Metal support
- No system WebView required

### Install (Windows, one click)

Double-click `scripts/AmniBrowse-Setup.cmd` or run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install.ps1
```

That pulls the latest zip from **https://amni-scient.com/browse/latest.json** (falls back to GitHub `Amnibro/Amni-Browse` releases), installs to `%LOCALAPPDATA%\AmniBrowse`, adds Start Menu + Desktop shortcuts, and registers as a browser. Uninstall: `scripts\uninstall.ps1`.

Host `docs/latest.json` on the site whenever you ship a GitHub release so the feed and the zip stay in sync. In-app: Settings → Updates (or the ↑ chip) checks the same feeds and can apply over an installed copy.

### Android (`0.16.4-android.0`, versionCode 5)

The Android product is a native Kotlin/AppCompat browser shell around Android
System WebView (package `com.amniscient.browse`), not the desktop Servo binary
and not a Capacitor wrapper. New tabs are private and omitted from restored
sessions; use **New open tab** for a persistent tab. Room stores local browser
metadata, Android Autofill integrates with the device password manager, and the
native chrome provides tabs, bookmarks/folders, history, import, downloads,
search suggestions, per-site JavaScript/cookie controls, file handling,
fullscreen, picture-in-picture, printing, and theme/accessibility sizing.
Servo is not packaged in the APK. Beta feed:
`https://amni-scient.com/browse/android-latest.json`.

On the PC:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\export-chrome-amni.ps1
```

That writes `%USERPROFILE%\Documents\amni-chrome-import.json` (bookmarks + history only). Copy it to the phone, open AmniBrowse → Import Chrome (PC file). Set as default from the in-app button.

Build (needs Android Studio JBR 17+):

```bat
cd android
set JAVA_HOME=C:\Program Files\Android\Android Studio\jbr
gradlew.bat assembleRelease --no-daemon
```

APK: `android/app/build/outputs/apk/release/app-release.apk`. Signing uses gitignored `android/keystore.properties` + `android/amni-browse-release.jks`.

### Password managers

Settings → Password manager: **Amni vault**, **Bitwarden** (`bw` on PATH), **1Password** (`op`, signed into the desktop app), or **KeePassXC** (`keepassxc-cli` + `.kdbx` path). Unlock once. A key icon appears in the URL bar when the page has matches — pick a login to fill, same idea as Chrome. One match can autofill on load (toggle).

### Build & Run

```bash
# First build + launch (Windows, ~30 min cold)
run.bat

# Every launch after that (no rebuild)
run-fast.bat

# Or build manually
cargo build --release
cargo run --release

# Create desktop shortcut (Windows, pinnable to taskbar)
powershell scripts/create_shortcut.ps1

# Release build — default feature is servo-real (libservo).
# Light WebView stub: cargo build --release --no-default-features --features webview
cargo build --release
```

## Current architecture

### Android

- A standalone Gradle/Kotlin application under `android/`; version metadata is
  `versionCode 5` / `versionName 0.16.4-android.0`.
- `BrowseActivity` owns native AppCompat chrome and switches between normal and
  private Android WebView instances. It applies tracker stripping, per-site
  JavaScript/cookie policy, downloads, file selection, fullscreen/PiP, print,
  import, and session rules.
- Room (`Db.kt`) stores browser metadata. `SessionStore`/`SessionCodec` exclude
  private tabs from restoration. Android Autofill remains the password-manager
  boundary; no desktop vault or browser-profile data is packaged.
- The APK contains no Servo engine and shares no desktop runtime state.

### Desktop

- Rust `main.rs` selects a feature-gated backend and `app.rs` owns shared
  browser state/IPC.
- `servo-real` is the default shipping desktop feature: libservo renders normal
  content, while native winit/wgpu hosting and an HTML chrome overlay provide
  the window and controls.
- DRM/CDM-only routes use a wry System WebView child pane attached to the active
  tab. It is not a separate product window.
- `webview` remains a lightweight System WebView build, and `servo-engine`
  remains the legacy custom egui/wgpu path.
- Shared Rust modules cover tabs, navigation policy, blocking, downloads,
  profiles, settings, permissions, extensions, local storage, and the encrypted
  vault.

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
| `Ctrl+P` | Print |
| `Ctrl+S` | Download / save URL |
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

**Amni Apps** (context menu, hamburger menu, or command palette) opens **https://amni-scient.com** — the product site — not a local app inventory.

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

## Version tracks

- Desktop source/crate: `0.12.5`, Servo-primary hybrid.
- Android app: `0.16.4-android.0`, `versionCode 5`, native System WebView.
- Future experimental engine work is documented separately in
  [AMNISHUNT_DESIGN.md](AMNISHUNT_DESIGN.md); it is not the current Android
  architecture.

## License

**CC BY-NC 4.0** — [Creative Commons Attribution-NonCommercial 4.0 International](https://creativecommons.org/licenses/by-nc/4.0/).

- ✅ View, study, fork, and modify the source
- ✅ Share and redistribute non-commercially with attribution
- ❌ No commercial use, no resale, no SaaS, no paid redistribution
- ❌ No use of the "Amni-Browse", "Amni-Scient", or "Amnibro" trademarks

**Source-available, not permissive.** This is open source you can learn from, contribute to, and self-host — but not one you can package and sell. For commercial licensing, email `amnibro7@gmail.com`.

See [`LICENSE`](LICENSE) for full terms.

---

*Built with Rust and a deep respect for privacy. By Amni-Scient.*
