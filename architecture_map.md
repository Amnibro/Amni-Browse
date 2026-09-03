## 2026-09-03 v0.13.0 Chromium lane (WebView2) is the shipped Windows engine
- `src/platform/chromium.rs` (tao window + wry 0.46 `build_as_child` WebView2s): `App { chrome, tabs: Vec<Tab{uid, view, url, title, private, loading, zoom, navs}>, active, ... }`. Chrome child = `http://amnibrowse.chrome/` (toolbar.html + `FETCH_SHIM` that rewrites `amnibrowse://x` fetches to `http://amnibrowse.x/`, which is how wry maps custom protocols on Windows). Content children get `KEY_SCRIPT` (Ctrl shortcuts over `window.ipc`), the same protocol handler (internal pages), navigation handler (top-level ad block), `new_window_req_handler` (auth hosts -> native popup, else `Ev::Popup` -> new tab), title/page-load handlers, download handler.
- **Re-entrancy rule:** wry callbacks never touch `App`; they push `Ev` into `events` and wake the tao loop (`EventLoopProxy`). `amnibrowse://state` reads `App` via `try_borrow` and falls back to the last JSON while a command is being applied.
- **Z-order:** the chrome child is created first and raised with `SetWindowPos(HWND_TOP)` after every content child (`raise_chrome`); `overlay` cmd grows the chrome rect so toolbar flyouts/menus can extend over the page.
- **Engine wiring (`wire_engine`):** per-tab COM hooks on `ICoreWebView2`: WebResourceRequested shield, HistoryChanged -> canBack/canForward, FaviconChanged (ICoreWebView2_15) -> tab icon URL, ContainsFullScreenElementChanged -> borderless + chrome hidden, IsDocumentPlayingAudioChanged (ICoreWebView2_8), DownloadStarting (ICoreWebView2_4) + per-operation BytesReceived/StateChanged -> `DownloadManager` items, Settings4 password autosave/autofill. Frameless: `FRAME_CSS`=5 inset for tao's undecorated hit-test, window background = theme bg, `win_drag`/`win_max`/`win_min`/`win_close` from the toolbar, empty tab strip = drag region.
- **Shield:** `install_request_filter` (now inside `wire_engine`) (webview2-com 0.33 / windows 0.58): `AddWebResourceRequestedFilter("*")` + `WebResourceRequested` -> `AdBlocker::should_block` -> 403 `CreateWebResourceResponse`; DNT/Sec-GPC headers when `enable_do_not_track`.
- Servo lane parked: `platform/servo_real.rs` + `vendor/` untouched, feature `servo-real`; `scripts/package_release.sh` now refuses a servo-real exe. Templates shared via `ui/internal_pages.rs`.
- Rig: `scripts/probe_click_tabs.ps1` (real clicks on tab chips, `+`, GitHub's Google button) and `scripts/probe_sites.ps1`.
## 2026-09-03 v0.12.8 engine in-tree (vendor/servo + vendor/stylo)
- **Law:** site glitches that trace to the engine get fixed in `vendor/servo` / `vendor/stylo` (copies of servo c91fc17 / stylo b3e6425, `[patch]` in Cargo.toml; `scripts/gen_servo_patch.py` regenerates the 59-crate servo table; the patch key must be the `.git` URL used by the dependency). Stylo atoms for new media features live in `vendor/stylo/stylo_atoms/static_atoms.txt`.
- **Stylo prefs** that Servo never wires: `stylo_static_prefs::set_pref!` in `servo_prefs()` (has-selector, nth-child-of, starting-style, light-dark images).
- **Servo layout patches:** `display_list/mod.rs` text path paints `-webkit-text-fill-color` (falls back to `background::representative_color` when clip is text); `background-clip: text` layers skipped; `mask_image_layer` + `background::layout_mask` resolve `mask-image: url()` and paint it through `FilterOp::ColorMatrix` tint (WebRender 0.70 image-mask clips panic on rect primitives and paint nothing on pictures); gradient masks return a DUMMY key so the box is not painted. `flow/inline`: `WordFragment` slices + `process_weak_wrap_opportunity` implement emergency breaking; `style_ext.rs` maps `-webkit-box` to flex. `IndependentFormattingContext::text_area_rows` (layout) raises the automatic block size of a `<textarea>` to rows x line-height (1.3em for `normal`); the presentational height hint in `script/dom/element` is gone.
- **User scripts:** `AppState.user_content` (Servo `UserContentManager`) injects `servo_compat::document_start_script()` before page scripts on every content webview (requestIdleCallback, scrollIntoViewIfNeeded).
- **Desktop:** `restore_window_position` (windows-sys `MonitorFromPoint`/`GetMonitorInfoW` work area) + `window_x/window_y/maximized` in session.json; `shortcut_keys` swallows releases of consumed shortcut keys; `sync_window_title` on switch_tab; `file_subresource_allowed` admits local subresources when the document is unknown (referrer is stripped for file: documents).
- **Gates:** `scripts/check.cmd` (~3 min), `scripts/build_release.cmd` (10-14 min), `scripts/probe_sites.ps1 -Urls @(...)` via `powershell -Command` (window picked by PID + area; probe output travels through `document.title`), `test/engine/features.html` + `probe_dom.js` / `probe_text.js`.
## 2026-08-23 omnibox typing: kbd claim is synchronous on chrome mousedown
- `kbd_after_mouse(in_chrome, down, prev)`: down in chrome → true, down in content → false. Applied in `WindowEvent::MouseInput` before the event is forwarded. Toolbar `#url-wrap` mousedown focuses the input.
## 2026-08-26 Android 0.16.4 current release metadata
- Package `com.amniscient.browse`, versionCode 5, versionName
  `0.16.4-android.0`; Gradle is the authoritative Android version source.
- Native Kotlin/AppCompat architecture: `BrowseActivity` owns native chrome
  over normal/private Android WebView instances; Room stores local browser
  metadata; `SessionStore`/`SessionCodec` omit private tabs from restore.
- Helpers include `SearchEngine`, `TabOrder`, `BookmarkFolders`, `CookieHosts`,
  `ImportParser`, `BrowserPolicies`, and `NavResolver` tracker stripping.
- Android does not package Servo or desktop profile/vault data. The desktop
  remains a separate Rust Servo-primary hybrid with an in-tab wry WebView pane
  only for DRM/CDM routes.
- Generated APK/AAB and R8 output stay outside source control. Feed metadata is
  site-managed at `browse/android-latest.json`; signing and publication remain
  external release gates.
## 2026-08-21 origin favicon is a Servo subresource
- Missing `<link rel=icon>` pages get `/favicon.ico` by injecting that link into the loaded document (`prime_origin_favicon`). The GET is Servo's. Out-of-band `reqwest` and `favicon_fetch_inflight` threads are gone.
- Kick/poll split (v0.12.5): `favicon_prime_script` fires an **async** XHR (`x.open(...,true)`, `x.timeout=8000`), decodes in `onload` (BOM strip → x-user-defined plane scan → 900 KB cap → base64), stashes the tag on `window['__amniFav_'+cmd_token].s` (token-keyed; a page `window.__amniFav` cannot feed the embedder cache), returns `'started'`. Later `build_state_json` passes run `favicon_poll_script`, which only reads that stash. `origin_favicon_polls: HashMap<String,u32>` counts polls; `FAVICON_POLL_MAX=30` then `poll-timeout`. `'gone'` (full navigation wiped the global) is not terminal: the next poll re-primes, still counting toward 30. `finish_origin_favicon()` is the single terminal path (cache write + primed + counter drop); `favicon_tag_is_terminal` keeps `started`/`pending`/`gone` alive. No sync XHR anywhere, so the document thread never blocks on the icon fetch.
## 2026-08-21 chrome compositing truth (pixels, not theory)
- Idle paint order is content → chrome → blit. Chrome painted first gets wiped by the content painter's framebuffer clear; chrome painted last wipes the content. `AMNI_PAINT_ORDER=legacy` = pre-fix order.
- Overlay frames (`chrome_overlay_px > 0`, color picker / menus / dialogs): content → blit → chrome, otherwise the blit covers `#colorpop` at CSS y>chrome. Measured 2026-08-21: picker logged at 32,180 and was absent from `color_click.png` until this swap.
- **There is no WebRender Y-flip.** CSS top renders at the visual top. Chrome lays out `flex-start`; `content_blit_rect` origin is GL (0,0) so the untouched band is the visual top; hit band `p.y < chrome_px` matches the pixels. Ruler-probe proof: `test/shot_ruler.png`.
- Overlay anchoring rule: everything under the toolbar uses `top:`; `bottom:` anchoring (v0.11-v0.12.5 flyouts, this round's context menu/dialogs) was built on the flip myth.
- Session stores LOGICAL window size and the restore is clamped to the monitor work area (physical-in/logical-out grew the window 25% per launch).
- `load_toolbar_html()` hot-loads in release too: `AMNI_CHROME_HTML` → cwd → exe dir → `include_str!`.
- ON-SCREEN GATE (2026-08-21): `scripts/smoke_lap5.ps1` = launch + seeded session + real pointer drag + stderr capture. MUST call `SetProcessDPIAware()` first or GetClientRect/screenshots are virtualized (a 1750x1125 window reads 1400x900 and the nav right cluster looks 'missing' when it is only cropped). `build_servo_real.bat` now calls `scripts/vsenv.cmd` - without MSVC env mozangle dies and the exe silently stays stale while the log reads Finished. Window is FRAMELESS by default (`AMNI_DECORATIONS=1` to restore); `resize_edge()` + `drag_resize_window` provide the 6px resize frame. Tab ids `t{i}` are POSITIONAL, never stable identities - optimistic client-side reorder re-applies forever; drag-drop does a direct DOM reorder + 500ms render hold instead. `apply_media_visibility` show()/hide()s content webviews - without it the compositor paints a background tab over the active one.
- Tab drag-reorder (`assets/chrome/toolbar.html` + `move_tab` in `servo_real.rs`, 2026-08-21): pointer-driven (servo delivers no HTML5 dragstart/drop), 5px threshold, neighbours shift by tab-width, drop posts `move_tab{from:tN,to:idx}`. `renderTabs` early-returns while `dragTab` is set — the 250ms poll would otherwise nuke the dragged node. `pendingTabMove` applies optimistic reorder until backend state catches up (2026-08-21 Cursor lap 3). Rust side rotates `content_webviews`/`tab_zoom`/`media_panes` together and remaps `active_content_index` (4 cases). View source: `request_view_source()` → `pending_source` → `open_html_tab()` via `load_html_data` (single encode, 2026-08-21 Cursor lap 4), line-numbered themed tab, 512KB cap. Ctrl+U bound.
- Simple dialogs + file picker (2026-08-21 Cursor lap 3): `EmbedderControl::SimpleDialog` → chrome `#dlg-scrim` modal + `dialog_ok`/`dialog_cancel`; `EmbedderControl::FilePicker` → native `rfd` dialog, `submit()`/`dismiss()`. Unblocks pages that call `alert()`/`confirm()`/`prompt()` or `<input type=file>`.
- `file_url()` strips the `\?\` canonicalize prefix, so the Archivo @font-face actually resolves.
## 2026-08-20 show_embedder_control (context menu + select)
- `WebViewDelegate::show_embedder_control` / `hide_embedder_control` now live on `AppState`. Context menus and `<select>` render in the chrome overlay (`#ctx-scrim` / `#ctxmenu`).
- Anchor is CSS `bottom:${y}px;left:${x}px` (same flip as `#panel{bottom:84px}`). Device rect + `chrome_px`, divided by hidpi scale. Overlay height = full window while the menu is open (`overlay` cap is window height, was 480).
- Servo `ContextMenuItem`s call `ContextMenu::select`. Extra Amni rows: open link in new tab, save image, view source (Ctrl+U or ctx menu → `open_html_tab`). Select uses `SelectElement::select` + `submit`.
- Handled in delegate (2026-08-21): file picker (`rfd`), alert/confirm/prompt (`#dlg-scrim`), color picker. Still unhandled: IME composition.
- Local files (2026-08-21 Grok): session restore used to drop `file://` (only http/https/data survived) so a smoke `file:///` tab silently opened the home page. `content_scheme_ok` now admits `file` and `about`. `load_web_resource` intercepts `file://`, reads the path (`Url::to_file_path`), and finishes with bytes + mime. Omnibox `C:\path.html` that exists on disk becomes a `file://` URL.
## 2026-08-20 desktop input routing + favicons (servo-real)
- Keyboard target is now `AppState.kbd_in_chrome`, not the cursor band. Chrome overlay reports focus via `cmd/kbd?on=1|0` (`focusin`/`focusout` over `input,button,.tab,[tabindex]`); a mouse-down in the content band clears it and blurs `document.activeElement` in the overlay. Ctrl+L / Ctrl+F set the flag in Rust before the JS focus lands.
- `"stop"` = `window.stop()` (was `reload()`). Reload button glyph swaps to ✕ from state `loading`; Esc → stop when a load is in flight and chrome does not hold focus. `"home"` loads `home_url()`; Alt+Home bound. Ctrl+wheel over content = zoom.
- `favicon_data_url()` (servo_real.rs): `WebView::favicon()` → `servo::PixelFormat` swizzle → RGBA → >32px downscaled to 32 → PNG → base64 data URL. Cached in `AppState.favicons` keyed by tab URL, cleared by the `notify_favicon_changed` delegate, capped at 64 entries. State JSON carries `icon` per tab; `renderTabs` emits `<img>` and keeps the monogram fallback.
- Gates: `scripts/check.cmd` (vsenv + GStreamer → `cargo check --release --features servo-real`, ~5s warm) and `scripts/build_release.cmd`. A bash `cargo check` cannot build mozangle (bindgen needs MSVC `INCLUDE`).
- Context menu + `<select>` landed (next section). File picker / dialogs / color / IME still drop-through.
## 2026-08-20 Android v0 WebView daily driver
- New tree `android/`: `BrowseActivity` chrome over System WebView, Room, import JSON, Autofill.
- Export: `scripts/export_chrome_amni.py` / `export-chrome-amni.ps1` from Chrome `User Data\Default`.
- Spec: `docs/superpowers/specs/2026-08-20-android-daily-driver-design.md`.
- Out of v0: Servo, vault copy, cookie steal, Play.
## 2026-08-16 v0.12.5 chrome Y-flip + cursor/media hit
- Screenshot: chrome strip on the window *bottom*, hits still on the *top* → huge cursor offset; video clicks never landed.
- Cause: Webrender flips the full-window chrome webview (HTML top → visual bottom) while winit Y and content blit used top-left. v0.11.13 blit at GL (0,0) parked page content on the top band.
- Fix: `#shell` sits at HTML flex-end so the flip puts chrome at visual top; suggest/panel `bottom:84px`; content blit `target_rect.y = chrome_px`. Hits (y < chrome_px) now match pixels. Media pane already used that band.
## 2026-08-16 v0.12.4 session restore + duplicate tab
- Servo path writes `session.json` on navigate/tab change and a clean file on close. Next launch restores every tab (Servo + in-tab DRM) and window size.
- `Tab::navigate` / back / forward / new set `engine` from `is_drm_required`. `servo_real` initial tabs use `media_engine::route(url)` only.
- Settings → Privacy: restore-on-start toggle; crash lock note if last run died dirty.
- Ctrl+Shift+K duplicates the current tab. `window.open` to a DRM URL stays in-tab.
- Tutorial no longer claims YouTube is a second window. win_close `__exit__` no longer relaunches.
## 2026-08-16 v0.12.3 DRM is an in-tab pane, not a second window
- `spawn_media_pane` is `WebViewBuilder::build_as_child` on the main winit window, bounds = below chrome (`content_bounds`).
- Media lives on the same tab index as Servo content. Switch/close/reload stay in one window. Extra "Amni Media" chrome bar is gone.
- WebView still CDM-only; it just looks like a normal tab.
## 2026-08-16 v0.12.2 Servo-primary: own player + PDF, WebView DRM-only
- `media_engine::route` is DRM-only. YouTube/Twitch/Vimeo stay on Servo. No Chromium hatch for MSE hosts.
- In-tree YouTube progressive extract (`engine/stream_extract.rs`, ANDROID Innertube) → Servo `<video>` player page when a muxed format exists.
- PDF: fetch bytes + pdf.js canvas viewer; system open is fallback only.
- Law: ship Amni code where Servo lacks; WebView only for Widevine/FairPlay CDM.
## 2026-08-12 v0.12.1 click-install + auto-update + BYO password manager
- Install: `scripts/AmniBrowse-Setup.cmd` / `scripts/install.ps1` downloads latest zip from `https://amni-scient.com/browse/latest.json` then GitHub `Amnibro/Amni-Browse` releases, extracts to `%LOCALAPPDATA%\AmniBrowse`, Start Menu + Desktop shortcuts, registers HTTP/HTTPS, launches. Host `docs/latest.json` on the site. NSIS stub `scripts/amni-browse.nsi`. Uninstall: `scripts/uninstall.ps1`.
- In-app updater (`src/net/updater.rs`): startup check (if `check_updates`), Settings Check/Install, chrome ↑ badge. Applies zip over install dir via `apply-update.cmd` then exits.
- BYO passwords (`src/crypto/pm.rs`): Amni vault, Bitwarden `bw`, 1Password `op`, KeePassXC CLI. Unlock in Settings. Key icon in URL bar lists matches; one-match autofill on load (toggle). Chrome-style fill via injected form script. CSV parse helper for Chrome exports.
- Config: `password_provider`, `pm_cli_path`, `pm_keepass_db`, `autofill_on_load`, `check_updates`, `update_feed` (serde defaults so old config.json still loads).
## 2026-08-12 v0.12.0 servo default + daily-driver chrome
- Cargo `default = ["servo-real"]`. Plain `cargo build --release` is libservo. WebView stub is `--no-default-features --features webview`. `run.bat` builds `--features servo-real` without stripping defaults.
- `BrowserConfig::config_dir_root` vs `config_dir` (AMNI_PROFILE). Profile switch relaunches with that env.
- Servo path: history record on URL change; downloads on file/PDF nav; PDF viewer page + system open; find-in-page via `window.find`; print; omnibox `amnibrowse://suggest`; downloads/history flyouts; vault unlock + autofill inject; extension content scripts/CSS on load complete; `window.open` → new tab; CLI start URL; Windows default-browser registry + Settings.
- Chrome overlay hit region (`cmd overlay`) so suggest/panels receive mouse below the 84px shell. Toolbar: find bar, suggest, Ctrl+F/P/S/J/H, Ctrl+Shift+N.
- New: `src/engine/daily_driver.rs`, `src/platform/os_default.rs`. Backups: `backups/*.v0.11.13.bak`.
## 2026-08-12 v0.11.13 content blit fix + glass polish (SHIPPED)
- `src/platform/servo_real.rs` paint_and_present: GL target_rect origin (0,chrome_px)->(0,0) — bottom-left GL origin meant the old offset pushed content UP over the chrome band; this was the remaining "no header bar" pixel cause. Newtab footer `v{ver} · Real Servo · Amni-Scient`; theme_root_vars emits dual aliases (--bg/--bg-primary, --dim/--text-muted, --chrome, tab tokens).
- Chrome contract 84px (SERVO_TAB_H 40 + NAV 42 + progress 2) in `src/ui/tokens.rs` + `assets/chrome/toolbar.html`; CLOSE_HITBOX 28 (28×28 .close); NAV_HIT 36; url bar hides data:/about:blank/amnibrowse://newtab. Gates: scripts/run_glass_gates.ps1 4/4 pass on cold zip extract. Release v0.11.13 sole Latest, live asset md5 == local; site about/index/amni-browse/faq flipped + Pages build 773435e2 verified live.

## 2026-08-12 v0.11.12 tab strip mouse polish
- `assets/chrome/toolbar.html`: wheel over #tab-list pans it horizontally (strip is overflow-x:auto + scrollbar-width:none — wheel was the only missing mouse path to off-screen tabs); dblclick/middle-click on empty #tabs space -> new_tab (auxclick moved #tab-list -> #tabs so dead space right of last tab counts; tab middle-click still closes, + button excluded). Cargo 0.11.12; rebuild+repack+publish pending.

## 2026-08-12 v0.11.11 servo backend ship guard
- **SHIP LAW:** `default = ["webview"]` in Cargo.toml; plain `cargo build --release` produces a 9.3MB wry/tao exe with NO chrome overlay (no header bar, no theme radios) that clobbers `target/release/amni-browse.exe`. The v0.11.10 zip shipped exactly that. Release exes must come from `--no-default-features --features servo-real` (build_servo_real.bat) and be packed ONLY via `scripts/package_release.sh`, which greps the exe for "Real Servo (libservo)" / absence of "WebView (wry/tao)" and zips with python zipfile (forward slashes).
- **Cold zip smoke (Grok):** `_tmp_v01111_verify` extract of `amni-browse-v0.11.11-win64.zip` → log `Backend: Real Servo (libservo)`, toolbar loaded 15601 bytes, `chromeRev:'0.11.11'`. Theme multi-tab: `setting_set theme` walks every `data:text/html` content tab. Toolbar polish: dropped Servo-invalid `#tab-list::-webkit-scrollbar` (keep `scrollbar-width:none`).
- Runtime tell in any log: `Backend:` line at startup — `Real Servo (libservo)` good, `WebView (wry/tao)` means wrong build shipped.
## 2026-08-12 v0.11.10 servo internal-page theme parity + theme picker
- Public truth: Cargo 0.11.10 / GH Latest v0.11.10. chromeRev now auto-injected: toolbar.html carries `__CHROMEREV__`, replaced with CARGO_PKG_VERSION in `chrome_data_url()` (servo_real.rs) — never hand-bump the rev string again.
- servo_real.rs NEWTAB_TPL + SETTINGS_TPL: palettes are `__THEME__`-substituted from `ThemeConfig::active_theme()` via `theme_root_vars()` (--bg/--elev/--stroke/--text/--dim/--accent/--accent-dim). No hardcoded gold/dark hexes left except .mono glyph white.
- Settings page has a Theme section (radio chips, all_themes(), active checked) -> `setting_set theme` -> `ThemeConfig::set_theme` (persists) + settings page reloads themed; toolbar re-themes on next 250ms state poll. This was the ONLY way to switch themes on the servo backend.

# Amni-Browse Architecture Map
## 2026-08-12 v0.11.9 keyboard travel + internal hash routing
- Public truth: Cargo 0.11.9 / GH Latest v0.11.9, v0.11.8 demoted to Pre-release.
- Injected overlay keydown (platform/webview.rs, shipped WebView2 path): + Ctrl+L focus/select URL bar, Ctrl+D bookmark_add, Ctrl+H/Ctrl+J -> newtab#history/#downloads (was only W/T/Tab/1-9/K).
- Internal URL builder splits #frag before host/path: amnibrowse://x#y -> http://amnibrowse.x/#y (fragment previously fused into hostname; developer#themes deep-links silently landed on default tab).
- Home SPA (ui/webview.rs): location.hash -> openPanel(history|downloads|vault|devtools) on load; menu History/Downloads now deep-link instead of dead-ending on home.
- Hash consumers strip trailing slash so a regressing builder cannot remount the silent tab-miss class.
## 2026-08-12 v0.11.8 text_secondary parity (developer.rs dim == ui/webview.rs dim)
- Hub saveTheme text_secondary shade(tx,-40) -> dim(tx,40); dim() ported verbatim from webview.rs. Public truth: Cargo 0.11.8 / GH Latest v0.11.8, v0.11.7 demoted to Pre-release.
## 2026-08-12 v0.11.7 custom-theme parity (developer.rs == ui/webview.rs)
- developer.rs saveTheme derives all secondary colors via shade() from user BG/accent/text (was hardcoded navy + cyan accent_glow); glow = accent+'26'.
- active_theme IPC seeds th-bg/th-ac/th-tx pickers; home SPA custom pickers default to shell tokens. Gradient stops + font stack identical across both editors.
## 2026-08-12 v0.11.6 WebView tab reflow + brand gold residual
- Home SPA (ui/webview.rs) and injected chrome (platform/webview.rs) active tab: inset box-shadow 0 2px 0 accent (parity with toolbar.html); no border-bottom height change.
- Injected URL focus: 3px glow. Media bar + Developer/Servo start/settings + profile/custom accent defaults: Amni Scient gold #C89B4E (kills leftover cyan chrome).
- App icon (assets/amni-browse.svg + .ico via build.rs) recolored cyan -> gold; README badge gold. Public truth: Cargo 0.11.6 / GH Latest v0.11.6, v0.11.5 demoted to Pre-release.
## 2026-08-12 v0.11.5 chrome polish + changelog repair
- `assets/chrome/toolbar.html`: all `.tab` chips carry `border:1px solid transparent` (active recolors via `border-color` — no more 2px strip reflow on tab switch); URL bar tracks `lastUrl`, Escape restores it instantly (no poll-lag ghost text) and empty Enter is a no-op; close glyph labeled (`title`+`aria-label`), menu `aria-haspopup`. Canary `0.11.5-chrome-polish`.
- `changelog.md` had UTF-16LE blocks appended by PowerShell `>>` (read as binary); tail transcoded back to UTF-8, all entries recovered. Append to changelog with UTF-8 tools only (`Add-Content -Encoding utf8` or python).
## 2026-08-12 Tab group + private strip polish (home ↔ external)
- Group labels cap ~12 chars (ellipsis) with full name in `title`; stable within-group order (index-preserving sort).
- External chrome: `'Private'` title fallback parity with home; private tabs get `.priv` ring + `P` pill; fingerprint includes `is_private`.
- Bookmark chips use theme `--radius` / `p.radius` (no hard 4/6px drift). Active tab `scrollIntoView` on home strip.
## 2026-08-12 Home/external parity + brand theme + tab groups
- SPA home tabs match external chrome: fixed **148×30** chips, host-derived labels (`Tab::title_from_url` — Home / host / Developer), not stuck on "New Tab".
- Default theme **Amni Scient** tracks amni-scient.com: `#08090B` / `#0D0F12` panels, gold accent `#C89B4E`, soft text `#A7ADB6`, radius 4px.
- Tab groups via `panel_group` + IPC `tab_set_group`; right-click tab (home or external) or context "Group active tab…".
- Page titles: navigate sets host label; external load posts `update_title` → TabsUpdated.
- Status: **WebView2 · Chromium** (default `webview` feature). Servo is opt-in (`servo-real` / `servo-engine`).
- Home stats cards (ads/tabs/bookmarks/passwords/history/downloads) are live `get_stats` counters — not marketing placeholders.
## 2026-08-12 Tab strip hit reliability (WebView chrome)
- External injected chrome (`src/platform/webview.rs` / `f3eb1c8`): fixed 148×30 tab chips, title cap ~22 chars, fingerprint skip-repaint (`__AMNI_TABS_FP`).
- Hit path: `pointerdown`+`click` via `bindHit` + `stopImmediatePropagation`; close control `.x`; chrome host uses `popover=manual` top-layer so DRM overlays cannot steal strip clicks; Ctrl+W/T/Tab escape hatches.
## 2026-08-12 Content push under chrome (YouTube)
- Fixed overlay chrome was painting over in-page mastheads (`#masthead-container` top:0). `applyContentPush` rewrites a live `<style id="__amni_push_style">` every ensure: html padding-top = measured host height+4, offsets YouTube fixed chrome (`#masthead-container`, `ytd-masthead`, mini-guide) to `top: var(--amni-chrome-h)`.
## 2026-08-12 Developer hub + security + bug report
- Internal page `amnibrowse://developer` (tabs: Themes, Extensions, Security, Bug report).
- Themes: pick/save custom, export/import JSON IPC (`theme_export` / `theme_import`).
- Extensions: scan AppData `extensions/`, open dir, install sample `hello-amni`, enable/disable/remove.
- Page safety heuristics (`engine/page_safety.rs`): HTTP, IP hosts, lookalikes, risky TLDs, userinfo-in-URL, punycode → SAFE..CRITICAL; live chip + caution strip on external chrome.
- Bug report: prefilled GitHub issue (`engine/bug_report.rs`) with version/OS/page diag — no auto upload.
- Menu + command palette: Developer / Extensions / Security / Report bug.
## 2026-08-12 YouTube chrome (Trusted Types)
- YouTube enables Trusted Types — `bar.innerHTML = '...'` / `strip.innerHTML = ''` threw and chrome never mounted on external pages.
- Fix: pure DOM (`createElement`/`textContent`/`removeChild`); closed shadow + `__AMNI_SR`; host on `documentElement` with max z-index; 250ms watchdog; page-load re-`__AMNI_ENSURE`.
## 2026-08-12 DRM chrome + privacy copy truth
- Media/DRM `spawn_media_window` no longer bare: init-script bar + `amni_media_close` + decorations; `drain_close_requests` in servo_real.
- WebView external chrome: remount watchdog (400ms), force fixed styles, re-prepend if host detached (streaming sites).
- Home SPA + README + settings footer: drop false “3P cookies blocked by default”; state system WebView cookie policy + URL-bar stripping + no Amni telemetry.
## 2026-08-12 Full-chrome polish sweep (0.11.0)
- Home SPA (`ui/webview.rs`): split view DOM + drag-resize live (`#split-resize` 13px hit, `split-on` flex); `#cmd-palette` excluded from doc-click panel guard; engine iframe `top:110px;bottom:22px;z-index:5` (chrome = 38+44+28, status 22); bookmark star true-toggles via `bookmarkIds` map + `bookmark_remove`; toggles are `role=switch` keyboard-reachable, knob `--text-primary`; shared focus-visible ring; menu viewport clamp; Ctrl+Shift+E bound; BG image/opacity wired to newtab ::before.
- Servo overlay (`assets/chrome/toolbar.html`): :root seeded with amni-dark tokens (no palette flash if state poll fails); radii from `--radius`; no baked URL; aria-pressed on star/shield.
- Ship gate: zip repacked (exe 04:01 ≥ sources), boot-verified, 133/133 tests. Checklist: `docs/checklists/checklist_chrome_polish_v0.11.1.md`. Backups: `backups/*.v0.11.1_pre_polish.bak`.
## 2026-08-12 Selected contrast lock (0.11.0)
- Light themes (`amni-light`, `amni-paper-sunset`): strip `bg_secondary` darker than content; `tab_active` = `bg_primary` (content match); `tab_inactive` = strip. Dark themes already used content-match active.
- Paint: `.tab.active` fill + bottom accent + elevation shadow; `.grp` left rail only. Home SPA (`ui/webview.rs`) and injected (`platform/webview.rs`) share the rule; `toolbar.html` ring is `:focus-visible`/`.kbd-focus` only.
## 2026-08-12 Close successor strip-order (0.11.0)
- `TabManager::close_tab`: when closing the active tab, pick successor from `ordered_tabs()` (next strip index, else previous) — not raw `Vec` index after `remove`. Matches paint/cycle/jump. Unit tests in `tabs.rs`.
- UI paint rule unchanged: `.tab.active` = fill + bottom accent; `.grp` / `.tab-group-label` = thin left rail + uppercase label only (selected always beats group hue).
- Package: `target/release/amni-browse-v0.11.0-win64.zip` must contain post-fix `amni-browse.exe` (zip mtime ≥ exe).
## 2026-08-12 Canonical tab order + kbd-focus (0.11.0)
- `src/engine/tabs.rs`: `ordered_tabs()` (group -> insertion index); `to_json` emits that order, so every `tabs_updated` consumer sees strip order. Session snapshot still reads raw `self.tabs.tabs`.
- All three chromes (`src/ui/webview.rs` home, `src/platform/webview.rs` injected, `src/ui/chrome.rs` egui) share the ordering rule via local `orderedTabs`/`ordered_tab_values`; Ctrl+Tab cycle + Ctrl+1..9 jump index off it.
- `markKbdTab(id)` sets `window.__AMNI_KBD_TAB` + 900ms timeout; repaint adds `.kbd-focus` ring class. Ring CSS is `:focus-visible,.kbd-focus` ONLY — bare `:focus` would leave a permanent ring after mouse clicks; never call `.focus()` on tab nodes (steals typing from page content).
- Backups: `backups/*.v0.11.0-kbdfocus.bak`. Checklist: `docs/checklists/checklist_tab_polish_v2.md`.

## 2026-08-12 Tab interaction polish (0.11.0)
- `src/ui/webview.rs`: Ctrl+Tab/Ctrl+Shift+Tab `cycleTab(dir)`, Ctrl+1..9 `jumpTab(k)` (9=last); tab `onauxclick` middle-close; `tabs-container` dblclick on empty space = new tab; `tabDisplayLabel` truncates via `Array.from` (surrogate-safe).
- `src/ui/chrome.rs`: `truncate` now char-based (byte slice panicked on multibyte titles); Ctrl+Tab cycling via tabs_json in `handle_keyboard`.
- Backups: `backups/{{webview,chrome}}.rs.v0.11.0-tabkeys.bak`. Checklist: `docs/checklists/checklist_tab_polish_v1.md`. cargo check clean both default + servo-engine; release binary launch-verified.

## 2026-08-12 Release ship gate (0.11.3)
- Public truth is **0.11.3**: crate version, chrome canary `0.11.3-theme-tokens`, GitHub Latest = `v0.11.3` asset `amni-browse-v0.11.3-win64.zip` (zip carries fresh exe + hot-load toolbar.html — both must be freshened, exe alone is not enough), site tags. Severity chips/safebar are theme-native (`color-mix` over palette `p`; `warning` token threaded Rust->JS). Older tags stay as history.

## 2026-08-12 Release ship gate (0.11.1)
- Public truth is **0.11.1** only: crate `CARGO_PKG_VERSION`, README, chrome canary `0.11.1-settings`, site tags (product/index/about/faq), GitHub Latest = `v0.11.1` asset `amni-browse-v0.11.1-win64.zip`. Leave `v0.11.0` / older tags as history (0.11.0 zip is pre-polish).
- WebView chrome: `tokens` TAB38+NAV44+BOOK28 = TOTAL 110 / push 114; shadow DOM toolbar mounts on external http(s); SPA home keeps tab/nav/bookmarks bars; OS decorations on.
- Amni Apps → `https://amni-scient.com` (IPC AmniAppList/LaunchApp navigate; list_apps_json empty).
- Theme: `__AMNI_SYNC_THEME` + boot ThemeConfig seed. Tabs: host TabManager + `__AMNI_TAB_SEED` / get_tabs resync.
- Package: `target/release/amni-browse-v0.11.3-win64.zip` (toolbar on disk hot-loads; chromeRev must match tag).
## Chrome UI
- `assets/chrome/toolbar.html` — entire browser chrome (tab strip + nav bar + progress). CSS tokens in `:root`, 66px shell (32 tab + 32 nav + 2 progress). Interactive targets 28px (26px in-pill). Nav is a **left cluster**: `#nav-start` | `#url-wrap{flex:0 1 960px}` | `#nav-end` (no margin-left:auto). Free ultrawide space is after the menu. Tab strip: `#tab-list` horizontal scroll + `scrollIntoView` on active/roving. Keyboard: roving tabs; `:focus`/`:focus-visible`; URL via `#url-wrap:focus-within`. Lock via `setLock` + `.secure`/`.insecure`/`.local` (incl. `data:` local). Progress: loading → 72%, complete → 100% then fade (`finishing`). Canary: `window.__amni.chromeRev` (`0.11.5-chrome-polish`). Full theme fidelity: `applyTheme` maps all state tokens incl. `font_family`→`--font`, `accent_glow`→`--glow` (url-wrap focus ring), `tab_active`/`tab_inactive`→`.tab` fills (v0.11.3). Tab favicons are hue-hashed monograms; shield/bookmark buttons reflect live state (`state.shield`, `state.bookmarked`).
- Menu opens the real Settings page (data URL from `settings_page_html()`): search engine, homepage, shield, default zoom, UA override, bookmarks. Mutations go through `amnibrowse://cmd/setting_set` gated by a per-boot `cmd_token`; persisted via `BrowserConfig::save()` (AppData/amni-browse). Blank homepage serves the built-in start page (`newtab_html()`, bookmark tiles).
- Servo composites chrome over content; body transparent/pointer-events none so content hits fall through.
- **Critical:** `load_toolbar_html()` in `src/platform/servo_real.rs` reads disk first (cwd then exe-dir), falls back to `include_str!` embed. Chrome HTML hot-loads on launch — **relaunch, don’t rebuild**, after toolbar edits. About/Shield HTML is compiled into the binary — needs rebuild. `CHROME_HEIGHT_CSS` must stay 66 matching `#shell` (both changed together in v0.11.0). `cargo check` without `servo-real` does not compile this file.
## Launch
- `run.bat` is the single entrypoint: mtime-skips cargo when `target\release\amni-browse.exe` is newer than `src/`, `build.rs`, `Cargo.toml`, `Cargo.lock`. `run-fast.bat` remains as a no-build alias. GStreamer probe: `C:\gstreamer\1.0\msvc_x86_64` then Program Files; gate on `bin\gstreamer-1.0-0.dll`. Real Servo binary is ~100MB+; a ~9MB exe is the feature-stripped default build — do not use it to verify chrome.
## Engine
- Servo-primary hybrid; media/DRM routes to WebView2 via media_engine.
## WebView chrome (default `webview` feature)
- `src/ui/tokens.rs`: TAB 38 + NAV 44 + BOOKMARKS 28 = `TOTAL_CHROME_H` 110; content push 114.
- `src/platform/webview.rs` injects shadow-DOM chrome on external http(s) pages: tab strip + nav + bookmarks, themed from live `ThemeConfig` via `__AMNI_SYNC_THEME`. Tab state survives leave-home via `__AMNI_TAB_SEED` + page-load resync + `get_tabs`. OS frame: `WindowBuilder.with_decorations(true)`.
- Amni Apps menu/ctx/palette/IPC (`amni_app_list`, `launch_app`) → `https://amni-scient.com` only (no local inventory panel or launch list).
- Crate / UA / site download tag aligned at **0.11.1** (single source: `Cargo.toml` → `CARGO_PKG_VERSION`). Site product/index/about/faq tags must match Latest release.
## Recent chrome UI
- v0.11.1 / distribution: polished chrome shipped as Latest; canary `0.11.1-settings`; site about+faq version strings closed (were still 0.11.0 while download was 0.11.1)
- v0.11.0 / webview parity: theme sync home↔external, multi-tab seed on nav, chrome host height matches tokens (was 82px clip), decorations forced on
- v0.10.10 / a3ddb5b: dark About+Shield, progress finish-and-fade, tab scroll-into-view (Grok/Claude)
- v0.11.0 / ff18986: real Settings + start page, live shield toggle (config.block_ads gates adblock), real bookmarks (BookmarkManager wired), search-engine prefix honored by URL bar, default zoom, UA override via Preferences, amnibrowse:// cmd/state locked to chrome webview or token, 66px chrome
- v0.10.8: URL flex-grow 0 so left cluster actually sticks on ultrawide (Grok)
- v0.10.7: unpin nav-end, scheme lock, ghost-close, bookmark 26px, zoom .off (Claude)
- v0.10.6: (superseded) right-edge pin attempt
- v0.10.5: roving tabs + nav clusters
- e8aa647: runtime disk load of toolbar.html
- v0.10.4 / 2c725e1: 32px targets + focus rings
- f6e1b60: tokenized dark surfaces
