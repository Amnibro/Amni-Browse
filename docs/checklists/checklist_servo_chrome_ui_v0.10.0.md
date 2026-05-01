# Checklist — Servo-Rendered Browser Chrome (v0.10.0)
Goal: full browser UI (tabs, address bar, nav buttons) rendered *by Servo itself* as a second webview composited via `OffscreenRenderingContext::render_to_parent_callback()`. No egui, no native widgets — HTML/CSS/JS all the way down.
## Architecture (Option C — offscreen FB blit)
- **Chrome webview** paints directly into main `WindowRenderingContext`. Chrome HTML is full-window; bottom region gets overwritten by the blit.
- **Content webviews** (one per tab) paint into a shared `OffscreenRenderingContext` sized `(w, h - chrome_px)`. Only the active tab is painted each frame.
- **Per frame**: chrome.paint() → active_content.paint() → `offscreen.render_to_parent_callback()(gl, Rect(0, chrome_px, w, h-chrome_px))` → `rendering_context.present()`.
- **Input**: pointer y < `chrome_px` → chrome webview (absolute coords); else content webview (y translated by `-chrome_px`).
- **Command bus**: chrome JS → `fetch('amnibrowse://cmd/<name>?args', {mode:'no-cors'})` → intercepted in `WebViewDelegate::load_web_resource` → dispatched to `AppState::execute_command` → acts on active content webview.
- **State push**: chrome JS polls `amnibrowse://state` every 250ms → returns JSON (`url`, `title`, `loading`, `canBack`, `canForward`, `tabs[]`) → chrome DOM reconciles.
## Phase 1 — Visual shell
- [x] `assets/chrome/toolbar.html` — single-file (HTML+CSS+JS inline).
- [x] `servo_real.rs` — `chrome_webview`, `offscreen_context`, `scale_factor`, `active_content_index`, `self_weak`, constants `CHROME_HEIGHT_CSS=74`, `TOOLBAR_HTML`.
- [x] `resumed` — build chrome webview on main context + first content webview on offscreen context (via `Rc::new_cyclic` for self-reference).
- [x] `RedrawRequested` — `paint_and_present` paints chrome + active content then blits offscreen into lower region of main, then presents.
- [x] `Resized` — `resize_all` resizes main context, offscreen context, chrome webview, and every content webview.
- [x] `ScaleFactorChanged` — recompute chrome_px and re-resize everything.
- [x] Input routing — y < chrome_px → chrome (absolute); else active content (translated). CursorLeft sent to the webview being exited on seam-crossing.
- [x] `cargo check --no-default-features --features servo-real` clean (0 errors).
- [ ] Runtime smoke: launch, confirm chrome strip renders at top, page renders below without stripes/flicker.
## Phase 2 — Command bus
- [x] `load_web_resource` intercepts `amnibrowse://` scheme.
- [x] `amnibrowse://cmd/<name>?args` executes via `AppState::execute_command`. Responds 200 + CORS headers.
- [x] Commands: `back`, `forward`, `reload`, `navigate` (with URL resolver — plain domain → https://, bare text → DDG search).
- [x] Stubs (wire Phase 4d): `bookmark`, `menu`.
- [x] Chrome JS replaces stub `cmd()` with `fetch('amnibrowse://cmd/*', {mode:'no-cors'})`.
## Phase 3 — State push
- [x] `amnibrowse://state` returns JSON with URL, title, loading, canBack, canForward, tabs[].
- [x] Response includes `Content-Type: application/json` + CORS headers.
- [x] Chrome JS `poll()` every 250ms with `setInterval`. Initial poll fires on load.
- [x] `applyState` syncs URL input (only when not focused), progress bar, back/forward disabled class, tab DOM (innerHTML reconciliation with html-equality check to avoid thrash).
## Phase 4a — Multi-tab
- [x] `AppState::active_content_index: Cell<usize>` replaces "last is active".
- [x] `new_tab` creates new content webview on offscreen context, sets active to new.
- [x] `switch_tab` parses `t<i>` ID, bounds-checks, sets active_index.
- [x] `close_tab` parses ID, removes, adjusts active_index; refuses to close the last tab.
- [x] `build_state_json` marks `active: i == active_idx` instead of position-based.
- [x] Tab DOM in chrome is rendered from state (was hardcoded in Phase 1).
## Phase 4c — Keyboard shortcuts
- [x] `handle_shortcut` intercepts before routing KeyboardInput to webviews.
- [x] Ctrl/Cmd+T → `new_tab`.
- [x] Ctrl/Cmd+W → `close_tab` active.
- [x] Ctrl/Cmd+R / F5 → `reload`.
- [x] Ctrl/Cmd+L → focus URL bar (via `chrome.evaluate_javascript`).
- [x] Ctrl/Cmd+Tab / Ctrl+Shift+Tab → cycle tabs.
- [x] Alt+Left / Alt+Right → back / forward.
## Phase 4e — Chrome-parity batch (v0.10.0 post-smoke)
- [x] `notify_page_title_changed` → `window.set_title("{title} — Amni Browse")` (active tab only).
- [x] Per-tab zoom via `tab_zoom: RefCell<Vec<f32>>`, commands `zoom_in` / `zoom_out` / `zoom_reset` → `WebView::set_page_zoom` clamped [0.25, 5.0].
- [x] Ctrl+= / Ctrl+- / Ctrl+0 zoom shortcuts.
- [x] Closed-tab stack (`closed_tabs: RefCell<Vec<Url>>`); `reopen_tab` command + Ctrl+Shift+T shortcut.
- [x] F11 fullscreen toggle (`window.set_fullscreen(Some(Fullscreen::Borderless(None)))`); Esc exits when fullscreen.
- [x] Ctrl+1..8 jump to tab N; Ctrl+9 → last tab (Chrome convention).
- [x] Middle-click on tab → close tab (via chrome JS `auxclick` button===1).
- [x] `new_tab` accepts optional `url` arg (future-ready for link→new-tab).
- [x] State JSON exposes `zoom`, `fullscreen`, `canReopen`; chrome UI renders zoom % indicator.
- [x] BUGFIX: input routing used `.last()` (stale pre-multi-tab) → now uses `active_content()`; tab-switch actually routes to the correct webview.
## Phase 4b — Favicons (deferred)
- [ ] `WebViewDelegate::notify_page_favicon_changed` — store `Option<Url>` per webview.
- [ ] Serve favicon bytes via `amnibrowse://favicon/<tab-id>` or embed as data URL in state JSON.
- [ ] Chrome CSS: `.tab .favicon { background: url(...) }` per tab.
## Phase 4d — Menu panel (deferred)
- [ ] Thread `BrowserState` (or Rc<RefCell<>>) into `AppState`.
- [ ] `bookmark` command toggles via `BookmarkManager::add/remove`.
- [ ] Menu HTML panel with: vault, themes, downloads, history, devtools, settings.
- [ ] Route subcommands (`open_vault`, `open_history`, ...) to relevant BrowserState actions.
