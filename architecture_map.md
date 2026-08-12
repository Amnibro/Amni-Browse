# Amni-Browse Architecture Map
## Chrome UI
- `assets/chrome/toolbar.html` — entire browser chrome (tab strip + nav bar + progress). CSS tokens in `:root`, 74px shell (36 tab + 36 nav). Interactive targets 32px (nav-btn, new-tab, url-wrap, zoom-level). Keyboard: `:focus-visible` rings; URL focus via `#url-wrap:focus-within` box-shadow (not bare `#url` outline); tabs `role=tab` + tabindex.
- Servo composites chrome over content; body transparent/pointer-events none so content hits fall through.
## Launch
- `run.bat` / `run-fast.bat` probe `C:\gstreamer\1.0\msvc_x86_64` then Program Files; gate on `bin\gstreamer-1.0-0.dll`.
## Engine
- Servo-primary hybrid; media/DRM routes to WebView2 via media_engine.
## Recent chrome UI
- v0.10.4: kb/mouse travel (hit targets + focus rings) — Grok UI path
- f6e1b60: tokenized dark surfaces + cursor:pointer affordances
