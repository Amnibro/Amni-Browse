## v0.11.11 servo backend ship guard - 2026-08-12
- **Root cause of "no header bar" (round N of the class, now structural):** `default = ["webview"]` + shared target/release path — a plain `cargo build --release` clobbered the servo-real exe at 07:06 and the v0.11.10 zip packed the 9.3MB webview binary. Cold-pulled live zip proof: log says `Backend: WebView (wry/tao)`, zero toolbar-mount lines — the webview backend has no chrome overlay, so no header, no theme radios, none of the 0.11.10 features.
- **Packer guard shipped:** scripts/package_release.sh refuses any exe not embedding "Real Servo (libservo)" (and refuses any embedding "WebView (wry/tao)") before zipping; zips via python zipfile (forward-slash entries, per v0.11.8 lesson), staging from the previous zip's DLL layout with exe + assets swapped.
- Rebuilt --no-default-features --features servo-real; repacked; cold-launched from staging before tagging.
- **Theme multi-tab recolor:** `setting_set theme` reloads every open `data:text/html` content tab (settings → settings HTML, newtab/home → newtab HTML) so home + settings + toolbar (250ms state poll) flip together; was only reloading the active settings view, leaving sibling newtabs on the old palette.
- Dropped Servo-unknown `text-overflow` on tab titles / newtab tiles (overflow:hidden + nowrap still clips).
- Dropped Servo-invalid `#tab-list::-webkit-scrollbar` (kept `scrollbar-width:none`) — kills cold-launch CSS parse warn on chrome mount.
## v0.11.10 servo internal-page theme parity + theme picker - 2026-08-12
- **Theme parity gap on the servo backend closed:** NEWTAB_TPL and SETTINGS_TPL were hardcoded to the dark-gold palette while the toolbar re-themes live from amnibrowse://state -> pick any non-default theme and home/settings stayed dark gold (Anthony's original "themes change inconsistently on home vs browser tab" class, alive on internal pages). Both templates now take a `__THEME__` :root block (bg/elev/stroke/text/dim/accent/accent-dim) from ThemeConfig::active_theme() at render time.
- **Theme picker shipped:** the servo build had NO way to switch themes (setting_set had no theme key; settings page no theme section). Settings now lists all built-in + custom themes as radio chips; `setting_set theme` -> ThemeConfig::set_theme (persisted) + instant settings re-render; toolbar follows within one 250ms state poll.
- **chromeRev version drift killed as a class:** toolbar.html now carries `__CHROMEREV__`, injected with CARGO_PKG_VERSION in chrome_data_url() — the shipped overlay can never report a stale rev again (0.11.9 zip shipped chromeRev '0.11.8-textsec-parity').
- Settings shortcuts line: added Ctrl+Tab (real, was undocumented). Backups: backups/*.v0.11.9.bak.

## v0.11.9 keyboard travel + hash routing - 2026-08-12
- **Shipped-path shortcuts completed:** injected overlay handled only Ctrl+W/T/Tab/1-9/K on external pages. Added Ctrl+L (focus+select URL bar — the single largest keyboard-travel saver), Ctrl+D (bookmark current page via existing bookmark_add IPC), Ctrl+H / Ctrl+J (history / downloads panels).
- **Fragment routing bug (root cause, pre-existing):** internal target builder fused fragments into the hostname (`amnibrowse://newtab#history` -> `http://amnibrowse.newtab#history/`), so location.hash arrived as `#history/` and the Developer menu deep-links (#themes/#ext/#bug) silently missed their data-p selector, landing on the default tab. Builder now splits `#frag` off before host/path and reattaches after: `http://amnibrowse.newtab/#history`.
- **Menu History/Downloads dead-end fixed:** both routed to bare newtab (home, no affordance). Now deep-link `#history`/`#downloads`; home SPA opens the matching panel from location.hash (history/downloads/vault/devtools).
- Overlay's hostname guard confirmed: internal pages never load the overlay script, so no double keydown handling with the SPA's own map. cargo check clean. Backups: backups/*_webview.rs.v0.11.8-kbdtravel.bak.

## v0.11.8 asset repack + claims audit - 2026-08-12
- **Zip packaging fix (asset clobbered in place, same tag):** Compress-Archive wrote one entry (assets/chrome/toolbar.html) with backslash separators - Info-ZIP/7-zip extraction dropped it as a flat root file, so the toolbar never hot-loaded = the "no header bar" failure class. Rewrote all 170 entries with forward slashes, gh release upload --clobber, live asset re-pulled + Info-ZIP cold-extracted + smoke-launched clean (header mounts, 0 strays). Future zips: python zipfile, never Compress-Archive.
- **Home-page claims audited TRUE against source:** no telemetry/analytics SDK or phone-home (only outbound Amni URL is navigation to amni-scient.com); adblocker.rs domain+pattern stripping; DEFAULT_SEARCH_ENGINE=DDG (storage/config.rs); Aes256Gcm + PBKDF2-HMAC-SHA256 600_000 iters matches vault panel copy; no profile sync/upload paths; cookie line correctly hedged to system WebView policy.
- UA plug domain fixed amniscient.dev -> amni-scient.com (net/http.rs); stray "src/crypto/* (copy).rs" cruft moved to backups/.

## v0.11.8 - 2026-08-12 (text_secondary parity: dev hub dim == home SPA dim)
- **Last derivation mismatch killed:** hub built text_secondary via shade(tx,-40) (additive -102/channel, clamps to 0, hue-crushes saturated text) while home SPA uses dim(text,40) (multiplicative x0.6, hue-preserving). Identical only for pure-white text; e.g. gold #E8C55A -> hub #825f00 vs SPA #8b7636. Hub now carries the SPA's dim() verbatim.
- Proven on the cold-pulled shipped v0.11.7 zip (not source): all other fields already parity (positive shade == lighten), glow ac+'26' both surfaces, cyan canary 0.
- chromeRev 0.11.8-textsec-parity. Backups: backups/*.v0.11.8-pre.bak.

## v0.11.7 - 2026-08-12 (custom-theme parity: dev hub == home SPA)
- **Cyan glow bug killed:** Developer hub "Save & apply" hardcoded accent_glow rgba(0,212,255,.15) plus navy secondaries into every custom theme (gold/green theme -> cyan focus rings). Now derives bg_secondary/tertiary/hover/border/tab fills from the chosen BG and glow from the chosen accent, byte-for-byte the same recipe as the home SPA editor.
- Both editors now seed their color pickers from the ACTIVE theme (dev hub live via active_theme IPC; static defaults flipped from old navy #0a0e14/#e0e6f0 to shell tokens #08090B/#EDEFF2).
- Same inputs -> same theme object in both surfaces: gradient stops (accent/text/bg) and font stack aligned.
- chromeRev 0.11.7-theme-parity. Backups: backups/*.v0.11.7-pre.bak.

## v0.11.6 - 2026-08-12 (WebView chrome parity + brand gold residual)
- **Home SPA + injected shadow chrome** now kill active-tab reflow the same way as Servo toolbar: accent is inset box-shadow, not a 2px border-bottom that reflows the strip (was only fixed on toolbar.html in the prior 0.11.5 pass).
- Injected URL focus ring matches toolbar: 3px accent_glow (was 2px).
- **Brand residual:** media DRM bar, Developer page fallbacks, Servo settings/new-tab, custom-theme accent default, profile avatar default — cyan #00d4ff swapped for Amni Scient gold #C89B4E / shell tokens.
- README H1 was still v0.11.2 after architecture header bump; now current. License badge color flipped to gold.
- **App icon de-cyaned:** amni-browse.svg gradients moved to gold; amni-browse.ico recolored (cyan hues remapped to gold, shape untouched) so the exe/taskbar icon matches the shell.
- Cargo -> 0.11.6 (v0.11.5 zip already shipped; behavior/brand changes get their own tag - no duplicate-version headings). Backups: backups/*v0.11.6-pre*.

## v0.11.5 - 2026-08-12 (chrome polish + changelog encoding repair)
- **Active-tab 2px layout jiggle killed:** all tabs now carry a transparent 1px border; activation only recolors it (was: border added on .active, shifting every tab 2px on switch).
- **URL bar Escape reverts instantly** to the live page URL (tracked lastUrl) instead of leaving typed text on screen until the next 250ms poll; Enter on empty/whitespace input no longer fires navigate.
- Close-tab glyph gets title + aria-label (was unlabeled role=presentation); menu button aria-haspopup.
- **changelog.md encoding repaired:** PowerShell `>>` had appended UTF-16LE blocks into the UTF-8 file (grep saw binary); tail transcoded byte-wise, v0.7.1/v0.11.0/v0.11.4 entries recovered.
- chromeRev 0.11.5-chrome-polish; Cargo 0.11.5. Backups: backups/toolbar.html.v0.11.5.bak, Cargo.toml.v0.11.5.bak, changelog.md.v0.11.5.bak.

## v0.11.3 - 2026-08-12 (toolbar chrome = full theme-token fidelity)
- **`applyTheme` in `assets/chrome/toolbar.html` dropped four tokens the state endpoint already ships:** `font_family` (Paper Sunset serif / Mint Matrix mono changed pages but never the chrome), `accent_glow` (focus ring was a loud 2px solid `accent_hover`; now the designed 3px glow matching internal pages), `tab_active`/`tab_inactive` (custom themes can now style tab fills; builtins render identically).
- New `:root` vars `--font`/`--glow`/`--tab-active`/`--tab-inactive` seeded with amni-dark values (no flash if poll fails); chromeRev `0.11.3-theme-tokens`.
- Toolbar hot-loads from disk — live on relaunch; release rebuild keeps `include_str!` fallback in parity. cargo check clean.
- Backup: `backups/toolbar.html.v0.11.2.bak`.

## v0.11.2 - 2026-08-12 (severity palette goes theme-native)
- **secchip/safebar theme tokens:** `.secchip.safe/low/medium/high`, `#_safebar`, and `.ab` badge were hardcoded dark-theme hex (`#143d28`, `#3d3a1a`, `#2a1518`, `#ff4757`, `#04140a`) injected regardless of active theme — dark lozenges on light chrome. Now derived via `color-mix` over `p.ok`/`p.warn`/`p.danger`/`p.bg`/`p.border`; single rule set serves dark + light.
- **`warning` token threaded** Rust `Theme` -> JS palette (`p.warn`, fallback `#E8B04B`); tab-close hover text also mixed off `p.danger` instead of fixed `#fff`.
- Public truth = 0.11.2: Cargo.toml, README, chromeRev `0.11.2-settings`, GitHub Latest `amni-browse-v0.11.2-win64.zip`, site tags.
- Backup: `backups/webview.rs.v0.11.1-pre-secchip-theme.bak`. Launch-verified: v0.11.2 boots, chrome host up, tab restore + navigate clean, zero leaked processes.

## v0.11.1 - 2026-08-12 (version single-source + polished chrome ship)
- **Public truth = 0.11.1:** Cargo.toml, chromeRev `0.11.1-settings`, README, GitHub Latest `amni-browse-v0.11.1-win64.zip`, site download + index tag. v0.11.0 zip left as pre-polish history (no clobber).
- **End-user path verified:** fresh GH download → toolbar canary present; exe boots with window title + responding chrome host.
- **Site residual:** about.html + faq.html Browse version strings aligned to 0.11.1 (product page already was).
- Ships the full-chrome polish already on the 04:01 binary (header bar, split view, star/aria-pressed, light-theme contrast, theme sync, tabs persist, apps→amni-scient.com).

## v0.11.0 - 2026-08-12 (full-chrome polish sweep, 17-finding audit)
- **Split View resurrected:** `#split-content` + `#split-resize` never existed in the DOM — every entry point threw. Elements added, real drag-resize (5px handle, 13px hit target via ::after, hover accent, `.dragging` state), `split-on` flex layout.
- **Command palette fixed:** palette item clicks bubbled to the document click-guard and closed the panel they just opened — every open-panel command was a no-op. `#cmd-palette` added to the guard exclusions.
- **Engine frame geometry:** `top:48px;z-index:999` painted the DRM/engine iframe over the nav bar, bookmarks bar, and every panel/toast. Now `top:110px;bottom:22px;z-index:5` — under all overlays, clear of chrome.
- **Bookmark star is a true toggle:** was add-only, never un-filled, never reset on navigation. Now `bookmarkIds` url→id map drives `bookmark_add`/`bookmark_remove` and the star refreshes on tab switch + navigation.
- **Toggle switches keyboard-reachable:** `role=switch`, `tabindex=0`, `aria-checked`, Enter/Space; knob was hardcoded white (invisible on light themes) — now `var(--text-primary)`.
- **Focus-visible everywhere:** one shared accent-ring rule for nav/ctx/bookmark/close/vault/find/dt/theme/cmd controls (previously only `.tab` had one).
- **Menu clamps to viewport:** measured-size clamp + max-height/scroll — bottom items were unreachable on short windows.
- **Ctrl+Shift+E bound** (was advertised in the palette, never wired); **Ctrl+W guarded** against null tab records.
- **Theme honesty:** custom-theme BG image/opacity controls now actually apply (newtab ::before layer); hardcoded white/#000 on themed surfaces → `var(--bg-primary)`; shadows softened for light themes.
- **Servo overlay parity:** default palette now seeds amni-dark gold tokens (no cyan flash / permanent mismatch if the state poll fails); radii literals honor `--radius`; baked duckduckgo URL removed; hamburger titled Menu; aria-pressed states on star/shield.
- Release zip repacked from this binary; 133/133 tests pass.

## v0.11.0 - 2026-08-12 (selected tab contrast lock)
- **Light themes:** Amni Light + Paper Sunset strip was lighter/same as active (tab_active approx strip), so inactive chips read louder than selected. Classic hierarchy restored: strip darker (bg_secondary), active fill = content (tab_active = bg_primary), inactive = strip.
- **Selected elevation:** home SPA + injected chrome .tab.active get lift shadow + z-index so selected always beats group rail and strip wash on dark and light; kbd ring still :focus-visible/.kbd-focus only.
- **Servo toolbar.html:** bare .tab:focus permanent ring removed (mouse-click no longer double-encodes selected); close affordance matches.
﻿## v0.11.0 — 2026-08-12 (close successor = strip neighbor)
- **Bug:** `TabManager::close_tab` activated the next raw-Vec sibling after remove, so mid-group close jumped past group-sorted strip neighbors. Now successor is next strip tab (else previous) via `ordered_tabs()` — same map as Ctrl+Tab / paint.
- **Selected vs group paint:** group is label + left accent rail only; active tab keeps fill + bottom accent bar (no full-fill group hue). Release zip repacked from this binary (`amni-browse-v0.11.0-win64.zip`).
## v0.11.0 — 2026-08-12 (tab group + private strip polish)
- **Canonical tab order:** `TabManager::to_json` ships group-sorted (visual strip) order; cycle/jump in home chrome, injected chrome, and egui chrome all use the same `orderedTabs` sequence — strip index = cycle index = jump index. Session save unaffected (iterates raw tabs).
- **Kbd-focus ring:** transient 900ms `.kbd-focus` accent ring on Ctrl+Tab/Ctrl+N target; `:focus-visible` only (no mouse-click ring); no DOM focus steal from page content.
- **Injected chrome parity:** middle-click close, dblclick-strip new tab, Ctrl+1..9 jump, surrogate-safe tab/group labels; egui chrome gets Ctrl+1..9, middle-click close, highlighted active tab, gold group labels; ARIA `role=tab`/`aria-selected` on home strip.
- **Tab keyboard travel:** Ctrl+Tab / Ctrl+Shift+Tab cycle tabs; Ctrl+1..8 jump to tab N; Ctrl+9 jumps to last — wired in both WebView chrome and egui servo chrome.
- **Mouse travel:** middle-click a tab to close it (no 18px close-button hunt); double-click empty tab-strip space opens a new tab.
- **Crash fix (servo chrome):** `truncate` byte-sliced titles at index 20 — panicked on emoji/CJK at the boundary; now char-based.
- **Glyph fix:** `tabDisplayLabel` uses `Array.from` so 20-char truncation can't split a surrogate pair into a broken glyph.
- **Group strip:** stable within-group order; gold labels ellipsize at ~12 chars; reset label boundary across ungrouped runs.
- **Private tabs (external chrome):** title fallback matches home (`Private` / host / Home); gold `P` pill + `.priv` inset; fingerprint tracks `is_private`.
- **Radius parity:** bookmark chips use theme radius on home and injected chrome.
- **Home strip:** active tab scrolls into view on repaint.
## v0.11.0 — 2026-08-12 (DRM chrome + honest privacy copy)
- **DRM/media window chrome:** `media_engine` injects a persistent top bar (Back to Amni / Home / close) + OS decorations; IPC `amni_media_close` drains the media window so tabs remain reachable.
- **External chrome watchdog:** shadow toolbar re-attaches if sites strip the host; 400ms interval + MutationObserver (no 80-try bail).
- **Truthful privacy claims:** home/settings/README no longer claim “third-party cookies blocked by default” on WebView — cookies follow system WebView policy; Amni telemetry remains none; URL-bar stripping is what we ship on navigate.
## v0.11.0 — 2026-08-12 (UI parity + release tag)
- **Version single-source:** `Cargo.toml` 0.11.0; UA `AmniBrowse/0.11`; GitHub tag `v0.11.0`; site download strings match (retires mixed 0.10.3 / 0.7.0 docs noise).
- **Theme home ↔ external:** shadow chrome seeds live ThemeConfig colors (`danger` included); `__AMNI_SYNC_THEME` applies without reload.
- **Tabs survive leave-home:** host TabManager → `__AMNI_TAB_SEED` + page-load resync; new/switch/close re-navigates active tab URL.
- **Chrome geometry:** `tokens::TOTAL_CHROME_H=110` (tabs+nav+bookmarks); host height no longer hardcoded 82 (was clipping bookmarks). Content push 114.
- **OS title bar:** `WindowBuilder.with_decorations(true)` so release binary keeps the standard frame.
- **Amni Apps:** menu/ctx/command palette navigate to `https://amni-scient.com` (local app list panel removed).
- Servo chrome path still 66px settings shell (ff18986): real Settings + start page, live shield/bookmark, token-gated cmd channel.
# Changelog

## Unreleased — 2026-08-11
### Chrome palette + progress + tab strip (a3ddb5b) — v0.10.10
- **About/Shield pages** now use chrome dark tokens (`--bg:#0a0e1a`, accent cyan) instead of light `#f7f9fc`; shield was a silent unknown cmd, now loads a privacy status data-URL.
- **Progress bar** finishes to 100% then fades (`finishing`) instead of snapping off at 72%.
- **Tab strip:** `#tab-list` `overflow-x:auto` + `scrollIntoView` on active/roving; loading tabs pulse the favicon chip. Canary `0.10.10-palette`.
- **Lock:** `data:` scheme treated as local (About/Shield won't flash insecure).
### Chrome tight left cluster (toolbar.html) — v0.10.8
- **Root cause:** `#url-wrap{flex:1 1 auto}` kept eating the row on ultrawide even after `#nav-end` lost `margin-left:auto` (Servo flex + max-width still left zoom/menu far from the pill).
- **Fix:** `#url-wrap{flex:0 1 960px}` + `#nav{justify-content:flex-start}` so nav-start | URL | nav-end stay one left cluster; free space sits *after* the menu, not between URL and menu. Canary `0.10.8-cluster`.
- **Lock:** scheme-driven classes (`.secure` / `.insecure` / `.local`); `setLock(url.value)` on boot so https pages don’t flash a dim/local glyph; default color is dim until classed.
### Chrome cluster + lock + ghost-close (toolbar.html) — v0.10.7
- **`#nav-end` un-pinned** from frame edge (travel fix vs v0.10.6 auto-margin pin).
- **Lock indicator:** `setLock()` from URL scheme (was hardcoded green forever).
- **Ghost close:** `.tab .close` is `pointer-events:none` until hover/focus so background-tab X hits switch the tab instead of closing.
- **Bookmark star:** 26px inside the 30px pill content box (was overflowing).
- **Zoom hover:** `.off` class instead of inline `style.color` so hover isn’t dead.
### Chrome ultrawide shell pin (toolbar.html) — v0.10.6 (superseded by 0.10.7/0.10.8)
- Had `#nav-end{margin-left:auto}` right-edge pin — wrong travel on 3440px. Replaced by left cluster.
- **URL flex:** `#url{min-width:0}`; tab close 16→22px; `setRoving(el, focus)` poll-safe.

### Chrome roving + ultrawide travel (toolbar.html)
- **Nav clusters:** `#nav-start` (back/forward/reload) + `#nav-end` (zoom/shield/menu) so the URL pill caps at `max-width:960px` (end cluster pin completed in v0.10.6).
- **Tab roving:** single tab stop (`tabindex` 0/-1), ArrowLeft/Right + Home/End move focus and switch; poll `renderTabs` restores focus without `CSS.escape` (Servo-safe); click promotes the clicked tab into the roving set.
- **Focus rings:** `:focus` + `:focus-visible` on chrome controls (Servo is flaky on `:focus-visible` alone). URL still rings via `#url-wrap:focus-within`.
- **Loader log:** `load_toolbar_html()` logs cwd / exe-relative / embedded so a Jul-22 prebuilt can't silently fake a "disk UI" pass.

### Chrome keyboard/mouse travel (toolbar.html)
- **Hit targets:** `.nav-btn` / `#new-tab` / `#url-wrap` / `#zoom-level` raised 28→32px inside the 36px nav/tab rows (4px gutter, not 8). Nav gap tightened 4→2px so back/forward/reload sit as one cluster.
- **Focus rings:** `#url` still has `outline:none` (native ring looks broken in the pill) but `#url-wrap:focus-within` + `.focused` now draw accent border + `box-shadow` with `--accent-dim`. `:focus-visible` rings on `.nav-btn`, `#new-tab`, `#zoom-level`, `.tab`.
- **Keyboard:** zoom reset is `tabindex=0` + Enter/Space; tabs are `role=tab`/`tabindex=0` with Enter/Space switch and Delete/Backspace close (delegated on `#tab-list`).
- Dropped duplicate `#zoom-level:hover` rule.

### Launchers no longer hard-fail when GStreamer isn't in Program Files
- **Bug:** `run.bat` / `run-fast.bat` hardcoded `C:\Program Files\gstreamer\1.0\msvc_x86_64` and bailed with "GStreamer not found"; bypassing the check produced a Windows loader dialog because `gstreamer-1.0-0.dll` was missing everywhere on the machine.
- **Fix:** both launchers now probe `C:\gstreamer\1.0\msvc_x86_64` first, fall back to the Program Files path, and test for `bin\gstreamer-1.0-0.dll` (not just `bin\`) so a symbols-only devel extract can't pass the check.
- Verified: prebuilt `amni-browse.exe` launches, Servo renders duckduckgo.com, window title tracks the live page title.
- Licence holder corrected to Amniscient, LLC; `Cargo.lock` synced to 0.10.3.

## v0.10.3 — 2026-07-22
### Window resize finally works (never had, since v0.10.0)
- **Bug:** stretching the window left chrome, content, and GL surface frozen at launch size with white void beyond — reproduced via Win32 `MoveWindow` probe + screenshots.
- **Root cause:** Servo's `Painter::resize_rendering_context` (paint/painter.rs:1272, rev 68ca280) early-returns when `rendering_context.size() == new_size`. Our `resize_all` resized `rendering_context` + `offscreen_context` directly *before* calling `WebView::resize`, so the painter saw sizes already matching and skipped the entire viewport-rect/WebRender-document/relayout/repaint chain.
- **Fix:** deleted the manual context resizes; `chrome.resize(window)` + `content.resize(content_area)` now drive Servo's full resize path, same as servoshell. Verified by probe: content reflows and fills 1560×980.
- **Diagnostics kept:** `resize_all` entry log + paint-time ctx-vs-window mismatch detector (one log per size change).
- **Known-limitation notes:** rendering glitches on complex sites (speedtest.net) and general speed are Servo-engine-level at rev 68ca280 (`position: fixed` falls back to static in the taffy path; perf gap vs Chromium is engine maturity, build already opt-level 3 + fat LTO). Next lever: bump the Servo pin to current main.

## v0.10.2 — 2026-07-21
### Servo-primary hybrid routing hardened (Chromium only when required)
- **Bug:** URL-bar `navigate` / `new_tab` / `reopen_tab` always loaded into Servo via `WebView::load`, bypassing the media engine. Netflix/YouTube typed in the address bar never opened WebView2.
- **Fix:** `execute_command` now uses `media_engine::wants_media_window(url)` and queues `pending_media_urls` (same path as link navigation).
- **Bug:** `MEDIA_PATTERNS` only matched path suffixes like `netflix.com/watch`, so browse/login URLs stayed on Servo while `drm_fallback` already knew full DRM domains.
- **Fix:** `route()` = MSE patterns ∪ `drm_fallback::is_drm_required()` so Netflix, Disney+, Max, Spotify, etc. always leave Servo; normal sites (DDG, Wikipedia, GitHub) stay Servo.
- **Bug:** embed filter used `player.` / `/player/` and blocked legitimate top-level media hosts (e.g. `player.vimeo.com`).
- **Fix:** `is_embed_url` only treats `/embed/` (and youtube embed) as non-window; avoids double-spawn for iframes without starving real media tabs.
- **Bug:** chrome state listed media tabs as empty URL/`Media` title; `close_tab`/`switch_tab` ignored `mN` ids.
- **Fix:** state JSON carries media URL + host title; switch focuses media window; close drops media window entry.
- **Tests:** `media_engine` unit tests lock Servo-vs-Media classification.
- **Docs:** checklist + guardian council under `docs/checklists|guardian_councils/*v0.10.2*`.
- **Build env (2026-07-22):** release rebuild had been blocked because only the GStreamer *runtime* MSI was installed — no `lib\pkgconfig`, so `gobject-sys`/`gio-sys` failed. Fixed by administrative extract (`msiexec /a`, no elevation) of `gstreamer-1.0-devel-msvc-x86_64-1.26.11.msi` to `C:\gstreamer\1.0\msvc_x86_64` (a path `scripts/install_build_deps.ps1` already recognizes; `.pc` files are `pcfiledir`-relative so no prefix patching). Runtime DLLs still load from `C:\Program Files\gstreamer` (same 1.26.11). 131/131 tests pass; exe rebuilt + launch-verified (DDG on Servo, title sync OK).
- **Version:** `Cargo.toml` bumped 0.7.0 → 0.10.2; About page now renders `env!("CARGO_PKG_VERSION")` instead of a hardcoded string.

## v0.10.1 — 2026-05-01
### Chrome strip no longer paints over content (the "big black section" bug)
- **Root cause** — Servo's `OffscreenRenderingContext::render_to_parent_callback` (in `components/shared/paint/rendering_context.rs`) does a scissored `gl.clear()` *before* binding the target framebuffer. After `content.paint()` the offscreen FB is the currently-bound DRAW target, so the clear scissor corrupts pixels in the *source* FB at the same `target_rect` region. The subsequent `glBlitFramebuffer` then reads those just-cleared (black) pixels and writes them to the on-screen FB. Symptom on the maintainer's machine: a giant black band below the chrome strip regardless of which page was loaded; earlier attempts to shift `target_rect.y` only changed which slice of the source got corrupted (partial DDG visible at `y=chrome_px`, full wipe at `y=0`).
- **Fix (Rust)** — `paint_and_present` now calls `state.rendering_context.prepare_for_rendering()` before invoking the blit callback. That re-binds the rendering-context FB as DRAW, so Servo's internal clear hits the target instead of trampling the source. With that in place, `target_rect = (0, 0, W, content_h)` puts the blit cleanly below the chrome strip in GL coords (= window y=chrome_px..H after WR's flip-projection).
- **Fix (HTML/CSS)** — `assets/chrome/toolbar.html`: `body` is now `background: transparent; pointer-events: none`, and `#shell` is `height: 74px; pointer-events: auto`. Servo doesn't support `position: fixed` yet, so the static-flow fallback puts `#shell` at body top, which after WR's flip-projection lands at window-top — exactly where we want the chrome strip. Body's transparent fill + `pointer-events:none` lets the content blit show through below the strip and lets clicks fall through to the content webview, which also kills the `Empty hit test result for input event, ignoring` flood from prior versions.
- **Verified** — DDG homepage and amni-scient.com both render fully (search box, headline, cards, CTAs); chrome strip stays at top across navigation; tab title and URL bar update correctly; no black band; hit-test warnings near zero.
- **Build** — `cargo build --release --features servo-real` clean (483 pre-existing warnings, 0 errors).

## v0.10.0 — 2026-04-19
### Chrome-parity batch (phase 4e)
- **Window title sync** — `WebViewDelegate::notify_page_title_changed` updates `Window::set_title("{title} — Amni Browse")` when the active tab's title changes (ignored for background tabs and for the chrome data-URL webview).
- **Per-tab zoom** — `AppState::tab_zoom: RefCell<Vec<f32>>` parallel to `content_webviews`. Commands `zoom_in` / `zoom_out` step by ×1.1 clamped to `[0.25, 5.0]`; `zoom_reset` → 1.0. Applied via `WebView::set_page_zoom(f32)`. Chrome exposes `zoom` in state JSON; toolbar renders a click-to-reset zoom % pill (accent-colored when ≠ 100%).
- **Shortcuts** — `Ctrl+=` / `Ctrl+-` / `Ctrl+0` zoom, `Ctrl+Shift+T` reopen-closed-tab, `F11` fullscreen toggle, `Esc` exits fullscreen, `Ctrl+1..8` jump to tab N, `Ctrl+9` jump to last tab (Chrome convention).
- **Reopen closed tab** — `closed_tabs: RefCell<Vec<Url>>` stack; `close_tab` pushes the URL before removal, `reopen_tab` pops and spawns a fresh content webview at the end. Stack persists across the process lifetime (cleared on exit; no disk state).
- **F11 fullscreen** — `is_fullscreen: Cell<bool>` toggles `window.set_fullscreen(Some(Fullscreen::Borderless(None)))`; chrome JS reads `fullscreen` from state JSON (future: hide chrome strip entirely in fullscreen).
- **Middle-click close** — chrome JS `auxclick` handler fires `close_tab` on middle-button press over a tab (Chrome UX).
- **`new_tab` accepts `url` arg** — future-ready for link-opens-in-new-tab; default stays DDG home.
- **BUGFIX: input routing used `.last()` cloned webview** (stale since Phase 4a). Now uses `active_content()`, so mouse/keyboard input actually follows the selected tab.
- **Build** — `cargo check --no-default-features --features servo-real` clean (~9s full, 0 errors).
- **Still deferred** — favicons (4b), menu panel + bookmark wiring (4d), find-in-page (Servo has no embedder API for this yet).

## v0.10.0-pre — 2026-04-19
### Servo-rendered browser chrome (Option C — offscreen framebuffer blit)
- **assets/chrome/toolbar.html** — single-file HTML/CSS/JS browser chrome. 36px tab strip + 36px nav bar + 2px progress hairline. Dark theme with `--bg:#0a0e1a` / `--accent:#00d4ff`. Inline JS:
  - `cmd(name, args)` → `fetch('amnibrowse://cmd/<name>?<args>', {mode:'no-cors'})` dispatches to Rust handler. `no-cors` sidesteps the data-URL/scheme cross-origin rejection since commands are fire-and-forget.
  - `poll()` → `fetch('amnibrowse://state', {cache:'no-store'})` every 250ms, syncs URL input (only when not focused), progress bar, back/forward disabled class, tab DOM. Initial poll on load.
  - Tab list is rendered from server state (HTML no longer hardcodes tabs). Close button bubbles up via `.closest('.tab')`.
- **src/platform/servo_real.rs** — dual-webview composition. Chrome webview paints directly into main `WindowRenderingContext` (full window; bottom region gets overwritten); each content webview (one per tab, shared `OffscreenRenderingContext` sized `(width, height - chrome_px)`, only the active one painted per frame). Per-frame order: `chrome.paint()` → `active_content.paint()` → `offscreen_context.render_to_parent_callback()(gl, Rect(0, chrome_px, width, height - chrome_px))` → `rendering_context.present()`. Callback is Servo's built-in `glBlitFramebuffer` helper — no hand-rolled shaders. Chrome is 74 CSS px (scales with DPI via `scale_factor.get() * 74.0`).
- **`AppState` extended** — `chrome_webview: RefCell<Option<WebView>>`, `offscreen_context: Rc<OffscreenRenderingContext>`, `active_content_index: Cell<usize>`, `scale_factor: Cell<f32>`, `self_weak: Weak<AppState>`. Built via `Rc::new_cyclic` so `spawn_content_webview` can get a fresh `Rc<AppState>` for new delegates.
- **Command bus (Phase 2)** — `WebViewDelegate::load_web_resource` intercepts `amnibrowse://` scheme. Host=`cmd` → `execute_command(name, args)` acting on active content webview. Host=`state` → returns JSON. Host=unknown → 404. All responses include `Access-Control-Allow-Origin: *` headers. Commands: `back`, `forward`, `reload`, `navigate` (URL resolver: bare domain → `https://`, plain text → DDG search), `new_tab`, `close_tab`, `switch_tab`, `bookmark` (stub), `menu` (stub).
- **State push (Phase 3)** — `amnibrowse://state` returns `{url, title, loading, canBack, canForward, tabs:[{id, url, title, active, loading, engine}]}`. Includes media windows as `engine:"media"` tabs. `Content-Type: application/json`, `Cache-Control: no-store`.
- **Multi-tab (Phase 4a)** — `new_tab` appends a content webview on the offscreen context, sets active to new index. `switch_tab` parses `t<i>` ID and bounds-checks before assigning `active_content_index`. `close_tab` removes at index, adjusts active_index, refuses to close the last tab. `resize_all` now iterates all content webviews (was last-only) so switching to a tab doesn't show stale size.
- **Keyboard shortcuts (Phase 4c)** — `handle_shortcut` intercepts KeyboardInput before webview dispatch. Bindings: `Ctrl/Cmd+T` (new tab), `Ctrl/Cmd+W` (close active), `Ctrl/Cmd+R` + `F5` (reload), `Ctrl/Cmd+L` (focus URL bar via `chrome.evaluate_javascript("document.getElementById('url').focus();…select()")`), `Ctrl/Cmd+Tab` / `Ctrl+Shift+Tab` (cycle tabs), `Alt+Left` / `Alt+Right` (back/forward). `Ctrl||Super` gate lets Mac `Cmd+*` work too.
- **Input routing** — `WindowEvent::CursorMoved` / `MouseInput` / `MouseWheel` / `KeyboardInput` dispatch to chrome webview when pointer y < chrome_px (absolute coords), else to active content webview with y translated by `-chrome_px`. Pointer crossing the seam sends `MouseLeftViewportEvent` to the webview being exited.
- **Data URL bootstrap** — chrome loads via `data:text/html;charset=utf-8,<urlencoded TOOLBAR_HTML>`. Opaque origin, but `fetch` with `no-cors` (commands) or explicit CORS response headers (state) works cleanly.
- **Cargo.toml** — added `http = "1"` (dependency already transitive via hyper; direct dep needed to name `StatusCode`, `HeaderMap`, `HeaderValue`).
- **Build** — `cargo check --no-default-features --features servo-real` clean through all four phases (0 errors, 541 pre-existing warnings, ~3–5s incremental).
- **Deferred to next session** — Phase 4b (favicons via `notify_page_favicon_changed`), Phase 4d (menu panel + bookmark wiring — needs `Rc<RefCell<BrowserState>>` threaded into `AppState`). Runtime smoke test unblocks end-user feedback loop.

## v0.9.0 — 2026-04-18
### Hybrid media engine (Servo primary + wry media fallback)
- **Servo continues as primary engine** for general browsing. Real libservo at rev `68ca280` renders HTML/CSS/JS via ANGLE+D3D11 on Windows, with GStreamer media backend for simple `<video>` playback.
- **New module `src/platform/media_engine.rs`** — cross-platform media-mode dispatch. Routes URLs matching known streaming patterns (YouTube, Twitch, Vimeo, Netflix, Disney+, Hulu, HBO Max / Max, Prime Video, Paramount+, Crunchyroll, Apple TV+, Spotify embed, Tidal, SoundCloud, Discovery+, ESPN+) through the system native WebView via `wry 0.46`. On Windows that's WebView2 (Chromium/Edge, includes Widevine CDM + full MSE). On macOS, WKWebView (Safari/WebKit, FairPlay native + Safari Widevine CDM). On Linux, WebKitGTK (full MSE, opt-in Widevine via `~/.config/amni-browse/widevine/libwidevinecdm.so`). This sidesteps the Servo MSE/DRM limitation that previously blocked YouTube and all paid streaming.
- **Privacy-hardened WebView2 on Windows** — `configure_privacy_env()` sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` to disable SmartScreen, mixed-content auto-upgrade, optimization-hints, background networking, sync, breakpad, default-browser check, and first-run. `WEBVIEW2_USER_DATA_FOLDER` is pointed at `%APPDATA%/amni-browse/webview2-data/` so media-mode state is isolated from both Servo and the system Edge profile. This preserves the "ALWAYS ON / zero telemetry" promise on media tabs.
- **Linux Widevine opt-in** — `install_widevine()` stub documents the manual path for users who want DRM streams on Linux. Widevine binary is not shipped by default (Google TOS). `widevine_installed()` returns true on Windows/macOS (system-provided) and actually probes `libwidevinecdm.so` on Linux. `WEBKIT_FORCE_WIDEVINE_ENABLED=1` is exported when the binary is present.
- **engine/tabs.rs** — `Tab` gains `engine: TabEngine` field (`Servo` / `Media`), `#[serde(default)]` so existing sessions restore cleanly. Enables future UI indicator + session persistence of engine choice per tab.
- **platform/servo_real.rs** — `AppState` tracks `media_windows: HashMap<WindowId, MediaWindow>`. At startup, restored tabs with URLs matching `MEDIA_PATTERNS` spawn their own winit+wry window alongside the main Servo window. `window_event` routes by `WindowId` to the correct window (Servo main vs media). Close handling shuts down just the closed window; event loop exits only when all windows are gone.
- **Cargo.toml** — `servo-real` feature now pulls in `wry` as well. Both `wry` (media) and `servo` (primary) build into the same binary. `winit 0.30` hosts both via `raw-window-handle 0.6`; `tao` is no longer required on the `servo-real` path.

## v0.8.2.2 — 2026-04-17
### Native backend: CSS color parser was nuking every page to black
- **engine/style.rs** — Full rewrite of `parse_color`. v0.8.2.1 fixed the default-text-color transparency bug but the page still rendered as a large opaque-black rectangle. Root cause: the old parser only handled `#hex`; **every non-hex value fell through to `Color { r: 0, g: 0, b: 0, a: 1.0 }` — opaque black.** DDG (and every real page) uses rgb(), rgba(), named colors (`white`, `lightgray`, `#fff` shorthand, etc.), CSS variables, and shorthand values like `background: #fff url(...) center`. All of those were returning opaque black, so every background-painted FillRect was coat-of-black-paint over the text. New parser: (1) dispatcher `parse_color` early-returns transparent for `transparent` / `inherit` / `initial` / `unset` / `currentcolor` / `none` / empty and tokenizes `background`-shorthand values by whitespace (excluding `rgb(` / `hsl(` which contain their own whitespace), returning the first parseable colour; (2) `parse_color_one` handles `#rgb` / `#rgba` / `#rrggbb` / `#rrggbbaa` and `rgb(...)` / `rgba(...)` with comma or space/slash separators and percentage or number components; (3) `parse_named_color` covers ~130 CSS3 named colors in a single dense match. **Unknown values now return `Color { a: 0.0 }` (transparent), NOT opaque black.** Compositing over the white canvas now actually shows the text.

## v0.8.2.1 — 2026-04-17
### Native backend: invisible text + wgpu log spam
- **engine/paint.rs** + **engine/pipeline.rs** — `RenderTree::walk` and `RenderPipeline::build_tree` now initialise `cs.color = Color { r: 0, g: 0, b: 0, a: 1.0 }` before applying CSS. `ComputedStyle::default()` derives `Color` via `#[derive(Default)]`, which yields `a = 0.0` (fully transparent). Any element whose `color` property was not explicitly set by CSS had its text rasterized with alpha 0 — the glyphs were correctly laid out and `draw_text` was correctly called, but every pixel was blended at zero coverage and the page came through as an empty canvas with only CSS-styled elements visible. Opaque-black default matches browser UA stylesheet behaviour and makes unstyled text show up.
- **main.rs** — `env_logger` default filter widened from `"info"` to `"info,wgpu_core=warn,wgpu_hal=warn,naga=warn,egui_wgpu=warn"`. v0.8.2's `Maintain::Poll` drains submissions but does not stop the `Device::maintain: waiting for submission index <N>` log line — that's emitted by `wgpu_core::device::resource` at INFO level regardless of Poll vs Wait. Filtering wgpu sub-crates to WARN leaves our own `info!()` calls intact while burying the per-frame drainer log. Application-level logs (`log::info!`, `log::warn!`, `log::error!`) still print at INFO.

## v0.8.2 — 2026-04-17
### Native backend: block layout + URL-bar sync + wgpu queue drain
- **engine/layout.rs** — `to_taffy_style` fallback arm now maps `CssDisplay::Block` (and `Inline` / `InlineBlock` / `Contents` / any other non-Flex/Grid/None) to `taffy::Display::Block` instead of `taffy::Display::Flex`. Before this, every normal webpage (the entire `<body>` tree of divs, paragraphs, headings) was laid out as a single horizontal flex row with no wrap — all children shrinking to slivers, content_h collapsing to the viewport minimum, producing a tiny dark rectangle in place of the page. Root cause: taffy 0.7 has native block-flow support but the `_` arm shipped as `taffy::Display::Flex`. `flex_direction` / `flex_grow` / `flex_shrink` / `gap` fields are inert under Block display in taffy and left unchanged.
- **platform/servo.rs** — `AmniApp` now tracks `last_tab_url: String`; `render()` hoists `active_url` above `chrome.render()` and syncs `chrome.url_input` on tab-switch when `last_tab_url != active_url`. Before this, the URL bar showed empty on every tab after the first, even though the tab's URL was present in state. Stomp-on-user-typing is avoided by gating on the tab-change edge rather than every frame.
- **platform/servo.rs** — `gpu.device.poll(wgpu::Maintain::Poll)` inserted after `queue.submit` + `frame.present`. Before this, every `request_redraw`-driven frame submitted a command buffer but the driver never got a chance to release completed submissions. `wgpu_core::device::resource` logged `Device::maintain: waiting for submission index <N>` at INFO every frame (index passed 29,000 in the repro), burying real logs. Non-blocking `Poll` variant — no stall, just drains completed work.

## v0.8.1.1 — 2026-04-17
### Native backend: Haven-clean Visuals pass
- **platform/servo.rs** — `apply_theme_to_egui` no longer sets `override_text_color` (RichText `.color()` calls were getting stomped) and no longer sets `widgets.noninteractive.bg_fill`/`weak_bg_fill` (every label was being boxed in `bg_tertiary`). Labels now render as flat text on `panel_fill`, matching the clean Amni-Haven aesthetic. Inactive button stroke dropped to `Stroke::NONE` so buttons read as filled tiles instead of bordered rectangles. `extreme_bg_color` moved from `bg_primary` to `bg_secondary` so text-edit fields sit distinct from the surrounding panel.

## v0.8.1 — 2026-04-17
### Native backend: theme applied to egui + event-driven reflow
- **platform/servo.rs** — `apply_theme_to_egui(ctx, theme)` maps our `Theme` struct (bg_primary/secondary/tertiary, border, text_primary/secondary, accent) into `egui::Visuals` and calls `ctx.set_visuals(...)`. Luma check on `bg_primary` picks `Visuals::light()` vs `dark()` base. Applied in `render()` when `applied_theme_id` changes. Before this, egui was rendering in its default light mode — the whole chrome was white regardless of which "theme" the user clicked.
- **platform/servo.rs** — wgpu clear color now derived from `active_theme.bg_primary` instead of hardcoded `(0.06, 0.06, 0.09)`, so theme switch affects the window background, not just widgets.
- **platform/servo.rs** — Resize reflow is now event-driven. `WindowEvent::Resized` sets `pending_reflow = true`; the render loop only re-renders when the flag is set (and we have painted content + no render in flight), then clears it. Replaces the per-frame width-diff poll which had a `>8px` hysteresis window that could occasionally storm on scrollbar appearance.
- **ui/chrome.rs** — Theme panel buttons now dispatch real theme IDs (`amni-dark`, `amni-cosmos`, `amni-emerald`, `amni-light`, `amni-crimson`, `amni-solarflare`, `amni-mint-matrix`, `amni-paper-sunset`, `amni-deep-space`) instead of `theme_0`..`theme_4`. Before this, `ThemeSet { theme_id: "theme_0" }` didn't match any built-in or custom theme, so `active_theme()` silently fell back to `amni-dark` on every button press — clicking any button was a no-op.

## v0.8.0 P3b-v2 — 2026-04-17
### Native backend: click-through-egui + resize reflow
- **platform/servo.rs** — Image clicks now route through egui's `Response` API. Replaced `ui.image(...)` with `ui.add(egui::Image::from_texture(...).sense(egui::Sense::click()))`; on `clicked()`, computes content-space coordinate `(pointer_pos - resp.rect.min) / display_scale` and dispatches through `Interactor::dispatch_click` after the central-panel closure. Removed the dead raw `WindowEvent::MouseInput` handler (egui consumed the events before it fired).
- **platform/servo.rs** — Window resize now reflows at the new width without re-fetching the network. `AmniApp` tracks `rendered_css: Vec<String>` + `rendered_vw: f32`; when `ui.available_width()` drifts >8 px from `rendered_vw` (and no render is pending, and url still matches), spawns a task that calls `pipeline.render_to_pixels(html, css, vw, vh)` directly. Reflow IPC carries `reflow: true` so `check_rendered_pages` skips `run_page_scripts` on reflow (scripts already ran on the original paint). Fetch-path IPC now also carries `css_sources` so reflow has the stylesheets to re-layout against.

## v0.8.0 P3b — 2026-04-17
### Native backend: auto-height render + live viewport
- **engine/pipeline.rs** — `render_to_pixels` now computes output height from layout rects (`content_h = max(r.y + r.h)`), floored at `vh` and clamped to 16384 px, instead of using the caller-supplied `vh` as the canvas height. Long pages (Wikipedia articles, docs) are painted in full; short pages still fill the viewport.
- **platform/servo.rs** — `fetch_and_render` call site captures `ui.available_width()` / `ui.available_height()` (floored at 640×400) on the UI thread and passes them into the async render task. First paint after each navigation matches the real window; hardcoded `1280.0, 2048.0` removed.
- **Deferred to P3b-v2:** re-render on window resize without re-fetch; click-coordinate translation through scroll offset + image scale. **Deferred to P3a:** wgpu GPU paint port.

## v0.7.2 — 2026-04-17
- **Fix: double header on newtab.** Windows WebView2 rewrites `amnibrowse://newtab/` to the internal origin `http://amnibrowse.newtab/`, causing the `chrome_init_js` protocol guard to treat it as a regular http page and inject the shadow toolbar on top of the SPA chrome. Guard now also bails on `location.hostname` starting with `amnibrowse.`.
- **Fix: Home button no-op.** `webview.load_url("amnibrowse://newtab")` did not round-trip through the WebView2 custom-protocol remap on subsequent loads. `Act::Nav` now pre-remaps any `amnibrowse://<host>/<path>` to `http://amnibrowse.<host>/<path>` before calling `load_url`.

## v0.7.0 — 2026-03-15
### Amni Apps Launcher + Desktop Shortcut & Icon
- **engine/app_launcher.rs** — NEW: Hardcoded `AMNI_APPS` registry (10 apps) with `AmniApp` struct (id, name, desc, emoji, LaunchType, AppCategory); `launch_app()` validates against allowlist then spawns via `std::process::Command` (Bat→cmd /C start, Cargo→cmd /K cargo run --release, Web→navigate); `list_apps_json()` serializes for IPC
- **engine/mod.rs** — Added `pub mod app_launcher` re-export
- **net/ipc.rs** — Added `AmniAppList` and `LaunchApp{id}` IPC messages; Added `AmniApps{data}`, `AppLaunched{message}`, `AppNavigate{url}` IPC responses
- **app.rs** — Wired `AmniAppList` → returns app registry JSON; `LaunchApp` → validates + spawns process or returns NavigateTo for web apps
- **ui/webview.rs** — NEW "Amni Apps" slide panel with grouped cards (Local Apps / Web Apps), emoji icons, name, description, Launch/Open buttons; context menu entry; command palette entry; JS `renderAmniApps()` renderer; response handlers for `amni_apps`, `app_launched`, `app_navigate`; panel auto-requests `amni_app_list` on open
- **ui/emoji.rs** — `bolt`, `diamond`, `crown` emojis already in atlas (used by app cards)
- **assets/amni-browse.ico** — NEW: Multi-resolution Windows icon (16/32/48/64/128/256px) — dark navy shield with cyan "A" and privacy dot
- **assets/amni-browse.svg** — NEW: Vector source for the icon
- **assets/windows_app.rc** — NEW: Windows resource file linking ICO to binary
- **build.rs** — NEW: Uses `embed-resource` to compile .ico into .exe for native taskbar icon
- **scripts/create_shortcut.ps1** — NEW: Creates pinnable desktop shortcut with icon
- **run.bat** — NEW: Simple launcher builds release if needed then starts browser
- **assets/amni-browse.png** — User-provided PNG icon now treated as canonical source; `assets/amni-browse.ico` regenerated from this PNG for shortcut/taskbar use
- **engine/app_launcher.rs** — `list_apps_json()` now emits optional `icon_src` for app cards; Amni Mail card auto-loads an image from `assets/` (`amni-mail.png`, `amni-mail.jpg`, `amni-mail.jpeg`, or best image fallback)
- **ui/webview.rs** — `renderAmniApps()` now renders per-app PNG icons when available, with emoji fallback
- **app.rs** — `ThemeSet` now returns `ActiveTheme` response so theme changes apply instantly without requiring a second IPC round-trip
- **ui/webview.rs** — Hardened `__amni_receive` with try/catch and defensive JSON handling; tab rendering now validates payload shape and guards against malformed entries to prevent tab UI crashes
- **platform/webview.rs** — Replaced fragile injected site toolbar with a shadow-DOM self-healing toolbar; it now re-injects after SPA/body rerenders and preserves Home/Back/Forward controls instead of disappearing on subsequent navigations
- **platform/webview.rs** — Fixed WebView2 crash when returning to Home (`wry` panic: `InvalidUri(Empty)` in IPC source URI path) by replacing `data:` home navigation with `webview.load_html(...)` and adding empty-URL navigation guards
- **platform/webview.rs + ui/webview.rs** — Navigation now stays inside the Amni shell (in-page `loadUrl(...)`) instead of replacing the root WebView document; external pages render in `#web-content` iframe so tab bar, themes, and settings panels remain available while browsing
- **platform/webview.rs** — Prevented injected page toolbar from running inside iframe content (`window.self !== window.top` guard), removing duplicate header bars when browsing sites inside the shell
- **ui/webview.rs** — Fixed hamburger/context-menu actions by not immediately closing panels on menu-item clicks; menu options now open Themes/Settings/Downloads/etc. correctly
- **ui/webview.rs + ui/theme.rs** — Theme panel now highlights active theme, supports deleting custom themes directly from cards, and includes 4 new creative presets: Solarflare, Mint Matrix, Paper Sunset, and Deep Space
- **engine/paint.rs** — Fixed pre-existing non-exhaustive match on PaintCommand (added wildcard arm)
### Apps Available
| App | Type | Launch |
|-----|------|--------|
| Amni AI | Local | run.bat (Gradio @ :7700) |
| Azno v2 | Local | Run.bat (Trading @ :8050) |
| Amni Mail | Local | run.bat (FastAPI+React) |
| Amni Gen | Local | run.bat (Gradio @ :7860) |
| Amni Calc | Local | run.bat (WASM @ :8090) |
| Amni Explore | Local | run.bat (Ursina 3D) |
| Amni Miner | Local | run_dashboard.bat @ :8080 |
| Amni Game | Local | cargo run --release |
| Amni Coder | Web | amni-scient.com/coder |
| Amni-Scient | Web | amni-scient.com |

### Fixes
- **platform/webview.rs** — Corrected navigation handler to use the runtime ad-block toggle state (`state.ad_blocker.enabled`) so sites are not blocked when ad blocking is disabled.

## v0.6.1 — 2026-03-14
### Async Response Delivery — Engine Pipeline Now Fully Wired
- **main.rs** — Created `tokio::runtime::Runtime` with `rt.enter()` guard; `tokio::spawn()` now has proper async context for task execution on worker threads
- **app.rs** — Added `async_tx: Option<std::sync::mpsc::Sender<String>>` and `async_notify: Option<Arc<dyn Fn() + Send + Sync>>` to BrowserState; all 3 async IPC handlers (FetchPage, PageMetaReq, ReaderFetch) now clone tx+notify, send `IpcResponse::to_js_call()` through channel, and wake event loop via notify callback; previously responses were created and silently dropped
- **platform/webview.rs** — Created `std::sync::mpsc::channel::<String>()` for async response delivery; sender+notify callback set on BrowserState after construction; `async_rx.try_recv()` drained in UserEvent handler before sync act queue, feeding responses to `webview.evaluate_script()`; imported `Arc` for notify callback
- **engine/pipeline.rs** — `fetch_and_parse()` now resolves relative CSS `<link>` hrefs against page base URL and fetches stylesheet content via AmniClient; CSS text stored in `PageResult::css_sources`; DOM scoped to block for early drop (Rc<Node> not Send across await points); added `fetch_full_layout()` convenience method for end-to-end fetch+parse+layout; `fetch_reader()` refactored to drop DOM before returning
### Data Flow (Fixed)
- Engine Fetch: Cmd Palette → sendIpc(fetch_page) → handle_command → tokio::spawn → pipeline.fetch_and_parse() → async_tx.send(PageRendered.to_js_call()) → async_notify() → UserEvent → async_rx.try_recv() → webview.evaluate_script() → window.__amni_receive({type:'page_rendered'}) → engine-viewer overlay displayed
- Page Meta: Same flow → PageMetaResp → status bar update
- Reader Fetch: Same flow → ReaderHtml → reader overlay displayed

## v0.6.0 — 2026-03-14
### Engine Independence — Custom Network, DOM, CSS, Layout & Pipeline Integration
- **ui/emoji.rs** (NEW) — Centralized emoji atlas with 65+ static mappings, dynamic `register()`, `e()`/`eh()` accessors for raw/HTML entity output
- **ui/webview.rs** — All hardcoded HTML entities replaced with format variables from emoji atlas; Command Palette (Ctrl+K) with 22 commands (including Engine Fetch, Reader Fetch, Page Meta), fuzzy search, keyboard nav; `page_rendered` and `page_meta` response handlers with engine-viewer overlay
- **net/http.rs** (NEW) — Custom HTTPS client via hyper 1 + hyper-rustls 0.27 + rustls 0.23; response caching (cache-control aware, max 3600s TTL); DNT + Sec-GPC privacy headers; custom user agent; GET/POST with redirect detection
- **net/cookies.rs** (NEW) — Privacy-controlled cookie jar; third-party cookie blocking; domain matching; Set-Cookie header parsing; allow/deny lists; JSON persistence via serde
- **engine/dom.rs** (NEW) — Custom DOM parser wrapping html5ever 0.38 + markup5ever_rcdom 0.38; `parse()` from HTML string; `extract_meta()` (title, description, charset, lang, links, scripts, stylesheets, images, headings, text_content, meta_tags); `extract_reader_content()` for article extraction with nav/header/footer/script filtering; `query_by_tag()`, `query_by_id()`
- **engine/style.rs** (NEW) — CSS parser wrapping cssparser 0.34; `StyleSheet::parse()` tokenizes CSS into rules/declarations; `ComputedStyle` with 25+ properties (display, position, flex, color, font, margin, padding, border, opacity, z-index, etc.); color parsing (#hex), dimension parsing (px/em/rem/vh/vw/%), font-weight keywords
- **engine/layout.rs** (NEW) — Layout engine wrapping taffy 0.7; `LayoutEngine` manages node tree with `add_node()`/`add_leaf()`; `compute()` runs flexbox/grid layout against viewport dimensions; CSS-to-taffy style conversion (display, position, sizing, margins, padding, borders, flex properties, overflow, gap)
- **engine/pipeline.rs** (NEW) — Render pipeline orchestrator connecting HTTP→DOM→CSS→Layout; `fetch_and_parse()` fetches URL via AmniClient, parses with AmniDom, extracts PageMeta; `fetch_reader()` for reader mode; `parse_and_layout()` full CSS cascade + taffy layout computation; selector matching (tag, #id, .class); inline style support
- **app.rs** — BrowserState now holds `Arc<TokioMutex<RenderPipeline>>`; new IPC handlers: `FetchPage` (async engine fetch), `PageMetaReq` (metadata extraction), `ReaderFetch` (server-side reader content via AmniDom); `ReaderContent` now uses AmniDom for server-side article extraction instead of just wrapping raw HTML
- **net/ipc.rs** — Added 3 new IPC messages (`FetchPage`, `PageMetaReq`, `ReaderFetch`) and 2 new responses (`PageRendered`, `PageMetaResp`)
- **main.rs** — Added `rustls::crypto::ring::default_provider().install_default()` for TLS initialization
### Dependencies Added
- hyper 1 (client, http1, http2), hyper-util 0.1, hyper-rustls 0.27, http-body-util 0.1
- rustls 0.23, webpki-roots 0.26, bytes 1
- html5ever 0.38, markup5ever_rcdom 0.38
- cssparser 0.34, selectors 0.26, taffy 0.7
### Infrastructure
- Version bump 0.5.0 → 0.6.0
- Backups at backups/v0.6.0-pre/, v0.6.0-phase34/, v0.6.0-wired/
- Build: 0 errors, browser launches and runs successfully
- Rustls crypto provider (ring) initialized at startup

## v0.5.0 — 2026-03-14
### Functional Browser — Navigation Pipeline
- **platform/webview.rs** — Complete IPC response dispatching via action queue pattern (Act::Nav/Js/Title), EventLoopProxy for cross-callback signaling, initialization script toolbar injection on external pages, base64 data URI for home navigation, navigation handler for ad blocking at domain level
- **Navigation flow**: SPA URL bar → IPC → handle_command → NavigateTo → Act::Nav → webview.load_url → real page with injected toolbar → IPC back to Rust for Back/Forward/Home/Bookmark
- **Back/Forward**: Now return NavigateTo responses from internal tab history (no longer use JS history API), ensuring proper navigation through tab-managed URL history
- **Tab switching**: SPA updateTabs triggers actual WebView navigation for tabs with real URLs via IPC navigate
- **Ad blocking (navigation level)**: with_navigation_handler blocks main-frame navigations to 60+ known ad/tracker domains
- **URL dedup**: Tab::navigate skips duplicate history entries for same-URL navigations
- **Internal URL guard**: amnibrowse:// URLs excluded from browsing history recording
- **Toolbar (chrome_init_js)**: Floating dark toolbar on http/https pages with Back/Forward/Reload/Home/URL input/Bookmark/Shield, auto-updates URL bar and ad-blocked count via IPC
- **Home navigation**: base64 data URI approach loads SPA HTML (with_html for initial load panics on file:// URLs in wry 0.46)

## v0.4.1 — 2025-07-18
### 7-Pillar Modular Restructure
- **UI Pillar** (ui/) — chrome.rs, webview.rs, theme.rs, reader.rs
- **Communication Pillar** (net/) — ipc.rs, dns.rs
- **Storage Pillar** (storage/) — config.rs, bookmarks.rs, history.rs, session.rs, downloads.rs, profiles.rs
- **Encryption Pillar** (crypto/) — vault.rs, autofill.rs
- **Media Pillar** (media/) — placeholder for v0.5+
- **Platform Pillar** (platform/) — webview.rs (was browser.rs), servo.rs (was servo_backend.rs)
- **Engine Pillar** (engine/) — tabs.rs (was tab_manager.rs), adblocker.rs (was ad_blocker.rs), extensions.rs, permissions.rs, devtools.rs
### File Renames
- browser.rs → platform/webview.rs
- servo_backend.rs → platform/servo.rs
- ui.rs → ui/webview.rs
- chrome.rs → ui/chrome.rs
- tab_manager.rs → engine/tabs.rs
- ad_blocker.rs → engine/adblocker.rs
- download_manager.rs → storage/downloads.rs
- password_manager.rs → crypto/vault.rs
### Infrastructure
- All imports updated from flat crate:: paths to pillar-qualified paths
- 7 mod.rs re-export files with feature-gated visibility
- main.rs rewritten for module hierarchy (7 top-level mod declarations)
- Both backends compile clean (0 errors, warnings only)
- v0.4.0-flat backed up to backups/ directory
- ARCHITECTURE.md updated for pillar topology
- GUARDIAN_COUNCIL_MODULARIZE.md — council proposals per pillar with Rust-native approaches
- 30 files (23 source + 7 mod.rs), ~5,900 LOC total

## v0.4.0 — 2026-03-14
### Dual-Backend Architecture (Servo Integration)
- **app.rs** (NEW) — Extracted shared BrowserState from browser.rs; central handle_command() dispatcher used by both backends
- **chrome.rs** (NEW) — Native egui browser chrome for Servo backend: tab bar, nav bar, status bar, find bar, 10 side panels (vault, themes, settings, downloads, history, devtools, extensions, profiles, autofill, permissions), keyboard shortcuts
- **servo_backend.rs** (NEW) — winit event loop + wgpu GPU compositor + egui rendering pipeline; ApplicationHandler implementation with GPU state management; forget_lifetime() pattern for wgpu 22 render pass arcanization
- **browser.rs** — Refactored to WebView-only backend; feature-gated under `#[cfg(feature = "webview")]`; delegates all state to app.rs
- **main.rs** — Feature-gated module declarations and backend selection: `webview` (default) or `servo-engine`
- **Cargo.toml** — Dual feature flags: `webview = ["dep:wry", "dep:tao"]` and `servo-engine = ["dep:winit", "dep:wgpu", "dep:egui", "dep:egui-winit", "dep:egui-wgpu", "dep:raw-window-handle", "dep:pollster"]`; wgpu pinned to v22 for egui-wgpu 0.29 compatibility
### Build Commands
- `cargo build` — WebView backend (default, uses system WebView)
- `cargo build --no-default-features --features servo-engine` — Servo backend (custom wgpu rendering)
### Design Documents
- GUARDIAN_COUNCIL_v0.5.md — Guardian council proposals for v0.5 AmniShunt vision
- AMNISHUNT_DESIGN.md — Technical design for WebKit-Servo translation shunt layer, septidecimal IR encoding, process-isolated sandbox
### Infrastructure
- Version bump 0.3.0 → 0.4.0
- All v0.3.0 and v0.4.0 files backed up to backups/ directory
- Architecture map updated for dual-backend topology
- 23 source files, ~5,900 LOC total

## v0.3.0 — 2025-07-15
### New Modules (13 features)
- **download_manager.rs** — Async file downloads with progress tracking, cancel/remove/clear
- **history.rs** — Browsing history with search, date grouping, visit count deduplication
- **session.rs** — Session save/restore on startup, crash recovery via lock-file detection
- **autofill.rs** — Address profiles + AES-256-GCM encrypted payment cards, vault key sharing
- **permissions.rs** — Per-site permission management (Camera/Mic/Location/Notifications/Clipboard/Fullscreen/Autoplay/Popups)
- **dns.rs** — DNS-over-HTTPS resolver with TTL cache (Cloudflare/Google/Quad9/Custom providers)
- **devtools.rs** — Console + network logging with 1000-entry ring buffer
- **extensions.rs** — Manifest-based extension system, content script injection, URL matching
- **profiles.rs** — Multi-profile support with isolated data directories
- **reader.rs** — Reader mode with content extraction, Light/Dark/Sepia themes
### Core Changes
- **ipc.rs** — Completely rewritten with 60+ IpcMessage variants and 25+ IpcResponse variants
- **browser.rs** — Rewritten with BrowserState holding 15 subsystem managers, full dispatch
- **tab_manager.rs** — Added private browsing tabs (is_private, no history) and zoom controls (0.25x-5.0x)
- **config.rs** — Added restore_session, enable_doh, doh_provider, default_zoom, enable_reader_mode, downloads_dir
- **password_manager.rs** — Added public key accessor for vault key sharing with autofill
### UI Updates (ui.rs)
- Downloads panel with list/cancel/remove/clear
- History panel with search and delete
- Find-in-page bar (Ctrl+F)
- Zoom controls with visual indicator (Ctrl+=/-/0)
- Private tab badge on tab bar
- DevTools panel (Console + Network tabs)
- Extensions manager panel
- Profiles manager panel
- Autofill addresses and cards panel
- Permissions management panel
- DNS-over-HTTPS toggle in settings
- Session restore toggle in settings
- Reader mode button
- Expanded context menu with all features
- 10 new keyboard shortcuts
### Infrastructure
- Version bump 0.2.0 → 0.3.0
- All v0.2.0 files backed up to backups/ directory
- Architecture map updated
- README fully rewritten with all features documented
## v0.2.0
- Initial release with tabs, bookmarks, ad blocker, password vault, themes, split view
Fix: Navigation handler blocked websites.

2026-03-17 v0.7.1 - Fixed logic issue in adblocker's Wry navigation handler that caused all websites to be blocked.

## Date: 2026-03-18
- Rerouted HTTP navigation to use the internal RenderPipeline over Webview's native loader to prevent UI override and bypass X-Frame-Options
- Upgraded the JS hydration layer to properly inject and sandbox PageRendered payloads using element manipulation on the #engine-viewer.

## Date: 2026-08-12 (v0.11.0 polish)
- Tab groups: right-click any tab (home SPA or injected chrome) to name a group; strip sorts grouped tabs together with gold group labels; new `tab_set_group` IPC + `TabManager::set_tab_group`.
- Tab titles derived from URL host (`Tab::title_from_url`) instead of persistent "New Tab"; page `document.title` reported back on load via `update_title`; titles capped at 80 chars.
- Default theme retoned to Amni-Scient gold-on-dark tokens (#C89B4E accent, 4px radius, Segoe UI Variable); shadow-chrome palette fallbacks match.
- Home SPA tab sizing unified with external chrome (fixed 148px), engine badge in status bar, private-tab label fallback fixed.
- Verified: release build clean, binary launches with header chrome, pushed as 8586602.

## Date: 2026-08-12 (v0.11.4 font fidelity)
- Fixed invalid `font:<size> inherit` shorthand (silently dropped by CSS parsers) in URL bar (Anthony's toolbar.html fix, now committed) and DevTools panel (tabs/inputs/buttons in developer.rs) - these elements were rendering in UA default font instead of theme font.
- chromeRev 0.11.4-font-fidelity; Cargo 0.11.4; release zip rebuilt from v0.11.3 payload with fresh exe + toolbar.
