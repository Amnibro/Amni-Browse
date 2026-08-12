# Amni-Browse Architecture Map
## Chrome UI
- `assets/chrome/toolbar.html` — entire browser chrome (tab strip + nav bar + progress). CSS tokens in `:root`, 74px shell (36 tab + 36 nav). Interactive targets 32px. Nav is a **left cluster**: `#nav-start` | `#url-wrap{flex:0 1 960px}` | `#nav-end` (no margin-left:auto). Free ultrawide space is after the menu. Keyboard: roving tabs; `:focus`/`:focus-visible`; URL via `#url-wrap:focus-within`. Lock via `setLock` + `.secure`/`.insecure`/`.local`. Canary: `window.__amni.chromeRev` (`0.10.8-cluster`).
- Servo composites chrome over content; body transparent/pointer-events none so content hits fall through.
- **Critical:** `load_toolbar_html()` in `src/platform/servo_real.rs` reads disk first (cwd then exe-dir), falls back to `include_str!` embed. Chrome HTML hot-loads on launch — **relaunch, don’t rebuild**, after toolbar edits. `cargo check` without `servo-real` does not compile this file.
## Launch
- `run.bat` / `run-fast.bat` probe `C:\gstreamer\1.0\msvc_x86_64` then Program Files; gate on `bin\gstreamer-1.0-0.dll`. Real Servo binary is ~100MB+; a ~9MB exe is the feature-stripped default build — do not use it to verify chrome. `build_servo_real.bat` encodes the real feature set.
## Engine
- Servo-primary hybrid; media/DRM routes to WebView2 via media_engine.
## Recent chrome UI
- v0.10.8: URL flex-grow 0 so left cluster actually sticks on ultrawide (Grok)
- v0.10.7: unpin nav-end, scheme lock, ghost-close, bookmark 26px, zoom .off (Claude)
- v0.10.6: (superseded) right-edge pin attempt
- v0.10.5: roving tabs + nav clusters
- e8aa647: runtime disk load of toolbar.html
- v0.10.4 / 2c725e1: 32px targets + focus rings
- f6e1b60: tokenized dark surfaces
