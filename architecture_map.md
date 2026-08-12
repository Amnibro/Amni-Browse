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
