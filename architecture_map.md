# Amni-Browse Architecture Map
## Chrome UI
- `assets/chrome/toolbar.html` — entire browser chrome (tab strip + nav bar + progress). CSS tokens in `:root`, 74px shell (36 tab + 36 nav). Interactive targets 32px. Nav: `#nav-start` | `#url-wrap` (max 960px) | `#nav-end{margin-left:auto}`. Keyboard: roving tabindex on tabs; `:focus`/`:focus-visible` rings; URL via `#url-wrap:focus-within`. Canary: `window.__amni.chromeRev`.
- Servo composites chrome over content; body transparent/pointer-events none so content hits fall through.
- **Critical:** `load_toolbar_html()` in `src/platform/servo_real.rs` reads disk first (cwd then exe-dir), falls back to `include_str!` embed. Needs one `--features servo-real` rebuild after e8aa647; after that, chrome HTML hot-loads on launch. `cargo check` without `servo-real` does not compile this file.
## Launch
- `run.bat` / `run-fast.bat` probe `C:\gstreamer\1.0\msvc_x86_64` then Program Files; gate on `bin\gstreamer-1.0-0.dll`. Real Servo binary is ~100MB+; a ~9MB exe is the feature-stripped default build — do not use it to verify chrome.
## Engine
- Servo-primary hybrid; media/DRM routes to WebView2 via media_engine.
## Recent chrome UI
- v0.10.6: `#nav-end` right-edge pin + close hit target + setRoving focus flag (Grok)
- v0.10.5: roving tabs + nav clusters + Servo-safe focus
- e8aa647: runtime disk load of toolbar.html
- a2378ce: URL max-width + initial roving
- v0.10.4 / 2c725e1: 32px targets + focus rings
- f6e1b60: tokenized dark surfaces + cursor:pointer
