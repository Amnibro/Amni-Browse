## 0.13.0 - 2026-09-03 — Chromium (WebView2) is the Windows engine; Servo parked
- **Default lane flipped:** `default = ["webview"]`. `cargo build --release` produces the 6.6 MB Chromium lane (`src/platform/chromium.rs`). Servo stays buildable as `--no-default-features --features servo-real` (`build_servo_real.bat`) with all the 0.12.8 engine work kept under `vendor/`.
- **Architecture matches the Android app:** native chrome bar (`assets/chrome/toolbar.html`, same look as before) in its own WebView2 child, one WebView2 child per tab (page state survives tab switches), private tabs in an in-memory WebView2 profile, internal pages (new tab, settings, tutorial) served from `src/ui/internal_pages.rs`.
- **Verified on screen:** speedtest (logo, colors, icons), GitHub login with "Continue with Google" reaching the real Google Accounts sign-in, Hacker News, Wikipedia, long-token wrap, textareas; tab clicks, `+`, Ctrl+1..9, Ctrl+T/W/L/Tab, session restore with window position.
- **Privacy:** request-level shield through WebView2 `WebResourceRequested` (every subresource runs through the ad/tracker lists; speedtest renders ad-free), DNT + Sec-GPC headers, DoH via browser args when enabled, telemetry/sync/SmartScreen/Topics flags off, separate user-data folder. Auth popups (Google, Microsoft, Apple, GitHub OAuth) are left to WebView2 so `window.opener` flows work; other `target=_blank` links open as tabs.
- **Config:** `block_ads`/`block_trackers`/`enable_do_not_track` turned on in the local profile (were off). Defaults in code already true.
- **Parity lap (same day):** frameless window with the toolbar's own minimize/maximize/close, drag on the empty tab strip, native edge resize (5px theme-colored frame; maximized/fullscreen drop it); favicons in the tab strip (`FaviconChanged`); real `CanGoBack`/`CanGoForward`, `GoBack`/`GoForward`/`Reload`/`Stop` from the engine; HTML5 fullscreen (`ContainsFullScreenElementChanged`) hides chrome and goes borderless; tab audio state; Chromium password autosave + form autofill (toggle in Settings); clear cookies/cache/all on exit through `ClearBrowsingData`; download progress into the downloads panel (`DownloadStarting` + `BytesReceived` + `StateChanged`, open-in-Explorer, remove, clear); full key map: Ctrl+T/W/L/D/F/P/R/H/J/U/Tab/1-9/+/-/0, Ctrl+Shift+T/N/K/I, F5/F11/F12, Esc, Alt+Left/Right/Home; DevTools via F12; settings toggles for DNT/GPC, DoH, autofill, clear-on-exit.
- **Parity lap 2:** find-in-page with highlight-all (CSS Custom Highlight API: yellow matches, orange current, Enter/Shift+Enter cycle, Esc clears); pinned tabs (right-click a tab: favicon-only chips kept at the front, persisted); tab groups with collapse (right-click: add/move/remove, colored group chip, click the chip to collapse/expand, persisted); tab context menu (pin, group, duplicate, close); Amni content scripts/CSS from `extensions/` injected on every page load; Ctrl+N opens a second window; Linux/macOS keep the plain wry backend so `cargo build` still works there.
- **Known:** Chrome Web Store extensions need a custom WebView2 environment (Amni content scripts cover the injection use case); no tab search/vertical tabs; Amni-OS Linux lane runs on WebKitGTK through wry without the COM extras.

## 0.12.8 - 2026-09-03 — engine fixes in-tree: icons, contrast, text wrap, form boxes
- **Servo + Stylo are now vendored and patched** (`vendor/servo`, `vendor/stylo`, wired with `[patch]` in Cargo.toml; base revs in `UPSTREAM_REV`). Site problems get fixed in the engine from here on.
- **Dropped CSS:** `:has()` enabled (223 rules were dropped on a 6-site run, now 0); `@media` features prefers-reduced-motion / prefers-reduced-transparency / prefers-contrast / forced-colors / inverted-colors / dynamic-range / -ms-high-contrast evaluate (507 dropped media blocks -> 0); `text-wrap`, `word-break: break-word`, `display: -webkit-box` parse.
- **Icons:** `mask-image: url()` paints (mask image tinted with the background colour through a WebRender colour matrix; gradient masks paint nothing instead of a solid box); inline SVG `fill="color(display-p3 …)"` from the repair script is converted to `rgb()` so wordmarks like speedtest's show; WOFF2 icon fonts verified (Material Icons, Font Awesome).
- **Contrast:** `-webkit-text-fill-color` paints; `background-clip: text` no longer draws the gradient as a box and transparent-fill text takes the first gradient stop.
- **Text wrap:** `overflow-wrap: break-word|anywhere` and `word-break: break-word` break long tokens (URLs) only when the word is wider than the line, Chrome-style.
- **Form boxes:** `<textarea rows>` automatic height is at least rows x line-height, computed in layout (`text_area_rows`); the old presentational `height: rows x 1em` hint cropped the second line and fought `box-sizing`.
- **Site JS:** document-start polyfill via Servo user scripts: `requestIdleCallback`/`cancelIdleCallback` (GitHub's React partials threw and rendered "Looks like something went wrong"), `scrollIntoViewIfNeeded`.
- **Desktop:** window position + maximized state persist and restore clamped to the monitor work area (was landing under the taskbar / half off-screen); window title follows tab switches; key release after a consumed shortcut (Ctrl+digit, Ctrl+Tab) no longer leaks into the page; local `file://` pages can load their own subresources.
- Debug: `AMNI_TRACE_INPUT=1`, `AMNI_PROBE_JS=<file>`, `scripts/probe_sites.ps1`, `test/engine/features.html`.
- Known: `-webkit-line-clamp` still gecko-only; gradient masks (gradient border rings) paint nothing; 2 pre-existing unit tests fail on the uncommitted 0.12.7 chrome (anchor 119px chrome, compat sheet font-stretch).

## 0.12.7 - 2026-09-02 — SVG paint + prefers-color-scheme + Servo viewBox
- **Cause of black/missing icons** across speedtest, Cloudflare docs, X: Servo rasterises each inline `<svg>` alone, drops CSS `fill`/`stroke` (gecko-only in Stylo), and `currentColor` went black. `servo_compat::svg_repair_script` localises `<use>`/`url(#id)`, stamps paint as presentation attrs, resolves `currentColor`.
- **Giant Cloudflare / clipped logos:** bumped Servo `68ca280` → `c91fc17` so replaced layout respects SVG viewBox.
- **Dark sites looking light:** `notify_theme_change` from Amni theme luminance → Servo `prefers-color-scheme`.
- **Pin fallout:** map winit mouse buttons to Servo `Primary`/`Secondary`/`Auxiliary`.
- Cloudflare Turnstile still unsupported (notice banner). Downdetector-style interstitials need Chrome/Edge.

## desktop: omnibox keys were going to the page - 2026-08-23
- Clicking the URL bar waited on `fetch(amnibrowse://cmd/kbd)` before `kbd_in_chrome` flipped. Keys typed in that gap (or if the fetch lagged) went to the content webview, so the omnibox looked dead.
- Mouse-down in the chrome band now claims the keyboard immediately. Content mouse-down still releases it. `#url-wrap` mousedown focuses `#url`.
- Test: `chrome_mousedown_claims_keyboard_before_any_fetch`. Toolbar gate 14/14. Installed copy updated (`%LOCALAPPDATA%\AmniBrowse`).

## 0.16.0-android.0 - 2026-08-23 - private by default + remaining Chrome/Firefox chrome
- New tabs are private (separate WebView profile, not in session). Menu "New open tab" is the saved lane.
- Tab strip long-press drag-reorder; All-tabs grid uses last-seen thumbnails.
- Omnibox network suggestions from the picked engine (DDG / Google / Bing / Startpage / Wikipedia). DDG default.
- Bookmarks bar groups by folder; long-press a bookmark to edit title/url/folder or delete.
- Per-site cookie block (wipe on start); third-party cookies off. Tracker params stripped on navigate.
- HTML5 fullscreen + PiP (home-button while a custom view is up, or Page → Picture-in-picture).
- Signed release APK (`assembleRelease`, CN=Amniscient LLC). Unit tests 28 passed. Site beta: `amni-scient-site/downloads/amni-browse.apk` + `browse/android-latest.json`.
- Backups: `android/backups/*.v016-pre.bak`. Keystore is gitignored (`android/amni-browse-release.jks` + `keystore.properties`) — back those up off-box.

## desktop: favicon stash is token-keyed; gone re-primes - 2026-08-21
- `window.__amniFav` was a same-origin writable global. Stash is now `window['__amniFav_'+cmd_token]`; poll reads the same key. A page that sets `__amniFav` cannot write the tab icon cache.
- Poll tag `'gone'` (document replaced mid-poll) no longer calls `finish_origin_favicon`. The origin is marked for re-prime on the next chrome-state tick, still bounded by `FAVICON_POLL_MAX`.
- Tests: `stash_key_is_token_scoped_not_page_global`, `only_started_pending_and_gone_keep_the_origin_polling`.

## desktop: favicon prime reads bytes through x-user-defined - 2026-08-21
- `scripts\check.cmd` was red: `E0597` at `servo_real.rs:1347`, the `evaluate_javascript` callback bound `tag` to `s.as_str()` from an owned match arm. `tag` is now `String`.
- The prime script set `responseType='arraybuffer'` on a synchronous XHR. Servo's `SetResponseType` returns `InvalidAccess` whenever `sync_in_window()`, so the assignment threw into the script's own `catch`, `x.response` fell to the text branch, `new Uint8Array(aString)` produced zero bytes, and every origin without a `<link rel=icon>` logged `embedder-cache decode-fail`.
- Prime now calls `overrideMimeType('text/plain; charset=x-user-defined')` between `open` and `send` and reads `responseText` with `charCodeAt(i)&255`. `final_charset()` prefers the override, and `encoding_rs` maps `0x80-0xFF` to `U+F780-U+F7FF`, so the octets round-trip.
- Tests: `prime_script_reads_bytes_via_x_user_defined_not_response_type`. Suite 185 passed, 0 failed.

## desktop: window blur clears modifiers and ends live composition - 2026-08-21
- `WindowEvent::Focused` had no arm; blur fell through `_ => {}`. `state.modifiers` is written only by `ModifiersChanged`, so a modifier released while the window was unfocused stayed latched — hold Ctrl, alt-tab, release it elsewhere, come back, and every keystroke carried a phantom Ctrl through `keyboard_event_from_winit`.
- `WindowEvent::Focused(false)` now resets `ModifiersState::empty()` and calls `ime_blur()`, which sends `CompositionState::End` only when a composition is live and clears `IME_TARGET`. Blur never sends `ImeEvent::Dismissed`, because Servo answers that with `FocusOperation::Focus(FocusableArea::Viewport)` and would blur the page's focused input on every alt-tab.
- Tests: `window_blur_ends_live_composition_without_dismissing_focus`, `window_blur_while_idle_sends_nothing`. Suite 182 passed, 0 failed. The modifier reset itself is compile-verified only.

## desktop: IME blur dedup, favicon XHR eval callback - 2026-08-21
- Google Red: duplicate `End` on app-switch when Windows already sent `WM_IME_ENDCOMPOSITION`; dropped result string when `IME_TARGET` cleared on blur; `<link>` priming mutates document head.
- `IME_END_DELIVERED` skips second `End` on `Focused(false)`. `IME_TARGET` kept across blur until `ime_events` finishes composition. `Ime::Disabled` → `Dismissed` only while window focused.
- Favicon priming: sync XHR `/favicon.ico` returns `b64:` through `evaluate_javascript` callback into `origin_favicons` — no head mutation, no `amnibrowse://` postback.
- Tests: `blur_skips_end_when_system_already_terminated`, `disabled_while_unfocused_never_dismisses`, `prime_script_fetches_without_dom_mutation`. `ime_tests`: 10 passed.

- Glass probe showed `favicon.ico servo-net … ok` but zero `embedder-cache` lines: page `fetch('amnibrowse://cmd/favicon_cache')` is blocked by document `connect-src` on custom schemes.
- `favicon_prime_script()` injects `<link rel=icon href=/favicon.ico>` when no icon link exists. Servo subresource stack handles the GET; strip reads `WebView::favicon()` via `notify_favicon_changed`. Test: `prime_script_uses_link_subresource_not_page_fetch`.
- `scripts/run_qc_bundle.cmd` runs build, check, test, toolbar, and a 30s log-only favicon gate (no mouse/keyboard).

## desktop: QC bundle cannot report green on a tree that fails cargo check - 2026-08-21
- `scripts\run_qc_bundle.cmd` runs `check.cmd` first and exits non-zero if it fails, skipping build/test/probe.
- x-user-defined decode of all 256 octets has UTF-16 length 256, so `t.length>900000` matches byte size.

## desktop: window blur modifier reset is unit-tested - 2026-08-21
- `Focused(false)` calls `on_blur(&state.modifiers, composing)`: empty modifiers, `End("")` iff composing, never `Dismissed`, `IME_TARGET` cleared.
- Tests drive Ctrl|Shift and Alt cells to empty without glass. `scripts\test.cmd ime_tests`: 8 passed, 0 failed.

## desktop: IME target is WebViewId, not Debug text - 2026-08-21
- Claude keyed flush on `format!("{:?}", wv.id())`. Servo’s `WebViewId` is `Copy + Eq + Hash` (`PainterId` + `BrowsingContextId`). Ownership is now `Cell<Option<WebViewId>>`; flush compares `o.id() == stale`.
- Tests drive `ime_retarget_into` on `u64` stand-ins plus a compile-time `Copy + Eq` check on `Option<WebViewId>`.

## desktop: IME composition is owned per webview - 2026-08-21
- `IME_COMPOSING` was one process-wide flag while the IME target flips between the chrome webview (omnibar focus) and the active content webview (tab switch). Compose CJK in the page, click the omnibar, and the page webview never got `CompositionState::End` while the omnibar got an `Update` with no `Start`. Servo's per-webview composition state desynced from the embedder's.
- `IME_TARGET` now records the owning webview. `ime_retarget()` returns the stale key when focus moves mid-composition; the handler sends a terminating `End("")` to that webview and clears the flag so the new target starts with `Start`. A webview destroyed mid-composition just gets the reset.
- Tests: `focus_change_mid_composition_flushes_old_and_restarts_new`, `same_target_never_flushes_and_keeps_composing`. Suite 179 passed, 0 failed.

## desktop: favicon fetch without DOM mutation - 2026-08-21
- Origin fallback no longer appends `<link rel=icon>` into the live document. `favicon_prime_script()` uses `fetch('/favicon.ico')` through Servo's subresource stack, then posts bytes to `amnibrowse://cmd/favicon_cache` for embedder-side storage in `origin_favicons`. Avoids CSP link injection, mutation-observer conflicts, and window error noise on 404.
- Tab strip still prefers `WebView::favicon()`; embedder cache is the fallback. Test: `prime_script_fetches_without_dom_mutation`.
- Tab uid drag: `duplicate_url_order_unchanged_after_tab_move` proves URL-order poll would false-clear; `check_toolbar.js` asserts uid sync wiring.

## desktop: origin favicon fallback uses Servo's network stack - 2026-08-21
- `/favicon.ico` fallback no longer calls `reqwest`. When `WebView::favicon()` is empty, the content webview runs `favicon_fallback_script()` which appends `<link rel=icon href=/favicon.ico>` if the document has no icon link. Servo then fetches that URL through the same load path as page subresources (`load_web_resource`, cookies, proxy, rustls roots).
- One prime per origin per session (`origin_favicon_primed`). No detached threads, so mass restore cannot spawn a fetch pool. The tab strip still reads the bitmap from `WebView::favicon()` after `notify_favicon_changed`.
- Test: `favicon_tests::fallback_script_injects_origin_icon_through_document`.

## desktop: color popover lands under the input, embedder anchors stop dividing by DPI - 2026-08-21
- The color popover, context menus, `<select>` lists, and the IME candidate box all positioned themselves with `r.min.x / scale` and `(r.min.y + chrome_device) / scale`. Servo hands those rects back in CSS px, so every anchor was divided by the device scale a second time. At 1.25 DPI the color popover logged `32,300` for an input whose bottom-left corner is `40,354`, which put it on top of the element instead of under it. One shared `embedder_anchor_css(x, y)` now adds the chrome height in CSS and does not divide.
- `show_select` anchored on the top edge of the `<select>`, dropping the option list over the element. It anchors on the bottom edge now.
- The overlay stash rect was hardcoded `236x300`, so the restore blit ran past the popover and painted a white strip of empty chrome background over the page. `showColor` measures `getBoundingClientRect()` after layout and reports it through a new `overlay_rect` command; the hardcoded size stays only as the pre-measurement guess.
- Shot 04 renders. Grok's lap-8 stash/restore was correct; the binary on disk was three laps behind the source it was being judged against.
- `scripts/probe_hit.ps1` added. It clicks a button that sets `document.title`, so the tab strip answers "did the click reach content" before anyone debugs further downstream.
- Suite: 175 passed, 0 failed.

## desktop: stable tab UIDs fix duplicate-URL drag scramble - 2026-08-21
- Lap-8 `movePending` keyed on URL order, so three tabs at the same address (`about:blank`, same domain, etc.) produced identical before/after URL sequences and cleared the poll gate before `move_tab` finished — the strip scrambled on the next paint. Each tab now gets a stable `uid` at creation (`tab_uids` rotates with `move_tab`, drops on close). Chrome sync waits on uid order via `applyMoveIds`, not URLs. Positional `t{i}` ids unchanged for switch/close commands.
- Origin favicon fallback capped at 8 concurrent detached threads (`favicon_fetch_inflight`) so mass session restore cannot exhaust the pool. Still out-of-band `reqwest` — proxy/CA gap open until moved into Servo's stack.

## desktop: tabs show real favicons, top-edge resize verified on a live window - 2026-08-21
- Tabs fell back to a letter box whenever Servo's `WebView::favicon()` returned nothing, so most sites showed a placeholder. Added an `origin/favicon.ico` fallback: `favicon_origin` builds `scheme://host[:port]` for http/https only, `fetch_origin_favicon` does a 6s-timeout GET with a 1MB cap, and the bytes go through the same 32px PNG encoder the Servo path uses. Servo's answer still wins; the fetch only runs when Servo has nothing.
- One request per origin per session. `origin_favicons` inserts `None` before spawning the thread, so a 404 is negative-cached instead of retried on every 250ms poll. Verified live: github.com and en.wikipedia.org resolve their marks into the tab strip, example.com stays a placeholder because it serves no favicon.
- The fallback uses `reqwest` and therefore bypasses Servo's network stack, the adblocker, and the cookie jar. It sends no cookies and no referrer and only contacts the origin already open in that tab.
- `scripts/probe_lap9.ps1` added. It seeds a three-tab session, screenshots the strip, then presses at `top + 3px` and drags down 120px to read the window rect back. Result `dT=120 dB=0`, so the top perimeter resizes instead of moving the window, which closes the top-border defect on a real window rather than in a unit test alone.
- Suite: 172 passed, 0 failed.

## desktop: tab drag sync + top-edge resize - 2026-08-21
- `movePending` matched tab **titles**, so three "New Tab" tabs cleared the poll gate before `move_tab` finished, and a title change mid-drag held the strip frozen until the 5s valve. Sync now waits on **URL order**: snapshot `lastBackendUrls`, expect `applyMoveUrls(from,to)`, unfreeze when backend matches.
- `resize_edge()` bailed on all `p.y < chrome_px`, which killed north resize on the top 6px perimeter. Top edge (`p.y <= m`) is evaluated first; interior chrome band still blocks side resize.

## desktop: color picker was painted then blitted over - 2026-08-21
- Shot 04: `scripts/probe_color.ps1` opened `file://` `input type=color`, logged `embedder color picker at 32,180 (#C89B4E)`, and the screenshot showed only the gold swatch. The popover ran in the chrome document; the content blit then covered every pixel below the toolbar band.
- While `chrome_overlay_px > 0`, paint order is now content → blit → chrome, so `#colorpop` sits on the page. Idle frames keep content → chrome → blit so the toolbar band stays the untouched GL top.
- `push_embedder_json` requests a redraw the same tick the overlay height is set.
- Probe now reads stderr. Hex parser has a unit test (`#C89B4E`, `#abc`).

## desktop: IME composition wired, file:// gate rebuilt on origin - 2026-08-21
- `WindowEvent::Ime` was falling through `_`, and winit 0.30 never emits IME events until the window opts in. Added `window.set_ime_allowed(true)` plus an `Ime` arm that routes composition to the same chrome-vs-content target as the keyboard arm, and `set_ime_cursor_area` so the candidate window follows the pointer instead of pinning to the window origin.
- `ime_transition` maps winit `Preedit`/`Commit`/`Disabled` to Servo `ImeEvent::Composition(Start|Update|End)` and `Dismissed`. A bare `Commit` with no preceding preedit now synthesizes a `Start` first, otherwise Servo drops the dead-key accent.
- The lap-6 `file://` traversal guard tested `req_url.path()` for `..`, which the `url` crate normalizes away during `parse` (measured: both `../..` and `%2e%2e/%2e%2e` resolve to the same path before we see them). The branch was unreachable and its test `file_traversal_rejected` failed on this tree. Deleted both.
- Replaced with the rule that protects the user: `file_load_allowed(is_for_main_frame, document_url)` allows a `file://` load only for a top-level navigation or a document that is itself `file://`. An `https://` page can no longer pull local files as subresources. The chrome webview is exempted by webview id because it is a `data:` document that loads its own woff2 fonts over `file://`.
- `scripts/test.cmd` added; a plain `cargo test` cannot find `gstreamer-audio-1.0` without check.cmd's gstreamer-root probe, so the suite had never been run. It was `162 passed; 1 failed`; it is now `166 passed; 0 failed`.
- `scripts/probe_color.ps1` added. It proves `show_color_picker` fires with the right position and color and the control then goes unhandled, so the missing color popover is on the `toolbar.html` side, not in the Rust plumbing.

## desktop: file:// tabs actually load - 2026-08-21
- Session restore filtered schemes to http/https/data, so a `file:///` smoke tab became the home page with no error. Restore now admits `file` and `about`.
- Servo has no on-disk file protocol in this embedder, so `load_web_resource` now intercepts `file://`, reads the path, and returns the bytes with a mime from the extension. Missing files are 404; unreadable files are 403.
- Omnibox: an existing local path (and any `file:` URL) navigates instead of being stuffed into search.
- IME composition is still unwired (`WindowEvent::Ime` falls through `_`).

## desktop: it runs on screen now - frameless window, tab reorder verified, content/tab mismatch fixed - 2026-08-21
- **Nobody had launched the build in five laps.** `scripts/smoke_lap5.ps1` launches the release exe, seeds a
  two-tab session, screenshots first paint, drives a real pointer drag across the tab strip, and captures the
  app's stderr. Every claim below came off a screenshot or an `INFO` line, not a `cargo check`.
- **`build_servo_real.bat` never called `scripts/vsenv.cmd`.** Without the MSVC environment `mozangle`'s build
  script dies at exit 101, so the release exe silently stayed weeks old while the log said `Finished`. The bat
  now calls `vsenv.cmd` first and fails loudly.
- **`show_simple_dialog` did not compile** (E0505: `dlg_type` borrowed `payload`, then `payload` moved into
  `push_embedder_json`). Whole tree was red. `dlg_type` is owned now.
- **The window had an OS titlebar** while the chrome carried its own `#win-btns` and `#tab-drag`. Decorations
  are off by default (`AMNI_DECORATIONS=1` restores them), and `resize_edge()` gives the 6px frame back through
  `drag_resize_window` on all eight directions, so a frameless window is still resizable.
- **Tab drag-reorder snapped to the wrong order.** The optimistic `pendingTabMove` layer keyed off tab ids that
  are positional (`t0`..`tN`), so it re-applied the move to every poll payload forever - the backend logged
  `move_tab 0 -> 3, active now 2` while the strip showed the move applied twice. Replaced with a direct DOM
  reorder on drop plus a 500ms render hold. Verified on screen: drag tab 0 to the end, strip and backend agree.
- **The active tab painted a different tab's page.** Content webviews were never hidden, so the compositor kept
  painting a background one. `apply_media_visibility` now shows the active content webview and hides the rest.
  Verified on screen.
- **Color picker** (`<input type=color>`) has a UI: HSV square, hue strip, 20 swatches, hex field, wired to
  `ColorPicker::select`/`submit` through a new `color_pick` command. Compile-gated, not yet on screen.
- **View source opened a blank tab.** `open_html_tab` spawned the webview on `about:blank` and then loaded the
  data URL, so servo dropped the second navigation mid-flight. It spawns directly on the data URL now. The
  dump itself works (logged 1744 bytes of the newtab document). Compile-gated, not yet on screen.
- Not verified on screen: color picker, the view-source render fix, file picker, alert/confirm/prompt.
  `powershell -File scripts\smoke_lap5.ps1` after a rebuild is the gate.

## desktop: tab drag-reorder and a real view-source - 2026-08-21
- **Tabs could not be reordered.** Chrome has had drag-to-reorder since 2008; we had a click handler and
  nothing else. Pointer-based drag in `assets/chrome/toolbar.html`: 5px threshold before a drag starts,
  the grabbed tab follows the cursor with `translateX`, the tabs it passes slide out of the way by one tab
  width, and drop sends `move_tab{from,to}`. HTML5 drag-and-drop was avoided on purpose — the servo build
  does not deliver `dragstart`/`drop`, so nothing would have fired.
- **`renderTabs` is suppressed while a drag is live.** The 250ms state poll rewrites `#tab-list.innerHTML`
  wholesale; without the guard the dragged element is destroyed mid-gesture.
- **Click suppression after a drop.** `mouseup` sets a one-tick flag so the synthesized `click` does not
  fire `switch_tab` on whatever tab landed under the cursor.
- **`move_tab` in `servo_real.rs`** rotates the three index-parallel vectors together
  (`content_webviews`, `tab_zoom`, `media_panes`) and remaps `active_content_index` for all four cases
  (the moved tab is active, a tab moved past the active one from the left, from the right, or neither).
  Dropping any one of those vectors out of sync is how a media pane ends up bound to the wrong tab.
- **View source rendered nothing.** `amni_viewsrc` ran `document.documentElement.outerHTML` and threw the
  result away in a `|_| {}` callback. The result now rides a `Weak<AppState>` into `pending_source`, drains
  on the next event-loop spin next to `pending_media_urls`, and opens a themed, line-numbered source tab.
  Capped at 512 KB on a char boundary with a visible truncation note.
- **Ctrl+U** bound to `view_source`.
- `cargo check --release --features servo-real`: 0 errors, 40.08s. Toolbar JS `node --check` clean.

## desktop: the chrome band renders again — no Y-flip, paint order fixed - 2026-08-21
- **The shipping backend had no browser UI at all.** Built and launched the tree: window opened with the page
  filling 100% of the client area — no tab strip, no omnibox, no nav. Every chrome feature landed this round
  (context menu, `<select>`, dialogs, favicons, home/stop) was compositing to nothing.
- **Root cause 1 — paint order.** `paint_and_present` painted the chrome webview first, then rendered the
  content painter, which wiped the window framebuffer. Order is now content.paint() → chrome.paint() →
  offscreen blit, so the blit (which only clears its own scissor rect) lands under an intact chrome band.
  `AMNI_PAINT_ORDER=legacy` restores the old order for bisecting.
- **Root cause 2 — there is no WebRender Y-flip.** A ruler probe page (CSS y=0..800 stripes plus a
  flex-end shell) rendered top-to-bottom in order: CSS top at the visual top, the `flex-end` shell at the
  visual BOTTOM. So the v0.12.5 `justify-content:flex-end` workaround was putting the toolbar at the window
  floor while hit-testing (`p.y < chrome_px`) aimed at the ceiling — the exact symptom that fix was chasing.
  Chrome now lays out at CSS top (`flex-start`), `#suggest`/`#panel` anchor `top:84px`, the context menu
  anchors `top:${y}px`, and `content_blit_rect` blits at GL origin (0,0) so the free band is the visual top.
- **Window grew 25% per launch.** `persist_session` wrote `window.inner_size()` (physical px) and startup fed
  it back as `LogicalSize`; at 125% DPI every run multiplied by 1.25. Found it at 3419x2198 on a 3440x1440
  screen. Saves logical size now and clamps the restore to 95%/92% of the primary monitor.
- **Archivo never loaded in the chrome.** `file_url()` let Windows' `\?\` verbatim prefix into the URL
  (`file://?/C:/Users/...`), so @font-face always failed and the UI fell back to Segoe UI. Prefix stripped.
- **Chrome HTML hot-loads in release** (`AMNI_CHROME_HTML` env → cwd → exe dir → embedded). Iterating on
  `toolbar.html` no longer costs a 13-minute LTO rebuild.
- Harness: `test/run_logged.cmd` (RUST_LOG=info → test/run_err.log), `test/ui_probe.ps1` (shot/click/rclick),
  `test/chrome_probe.html` (does the chrome layer composite at all), `scripts/check.cmd`, `scripts/build_release.cmd`.
- Verified on screen: tab strip with favicon, omnibox with live URL, back/forward/stop/home, zoom, shield,
  find, menu — with the page rendering underneath. Screenshot: `test/shot_final.png`.
- Still unverified interactively: right-click produced no menu in a synthetic-click test and no
  `embedder context menu at` log line. Next seat gets that.

## 0.15.0-android.2 - 2026-08-21 - http:// on phone (cleartext)
- `net::ERR_CLEARTEXT_NOT_PERMITTED` on Amni-Connect / LAN / grok-remote: the APK had
  `usesCleartextTraffic=false`. A browser has to load `http://`. Network security config now
  permits cleartext; WebView mixed-content set to compatibility mode.

## desktop servo-real — context menu + select - 2026-08-20
- Right-click and `<select>` no longer die in an empty `show_embedder_control`. Overlay menu anchored with `bottom` (Webrender chrome flip), full-window overlay hit-band while open, Esc/scrim dismiss.
- Custom items: Open link in new tab, Save image as. Servo actions (back/forward/reload/copy/cut/paste/select-all) forwarded via `ContextMenu::select`.
- Gates: `scripts/check.cmd` 4.73s warm, 0 errors; toolbar JS `node --check` clean. Not launched on screen this turn — first window should right-click DDG and open a `<select>` on a form page and screenshot the menu vs cursor.

## 0.15.0-android.0 - 2026-08-20 - true private tabs + hands-free import (PROVEN on device)
- Private tabs now ride a REAL WebView profile ("private" via ProfileStore/MULTI_PROFILE):
  own cookie jar, storage and cache in a second WebView instance, swapped per tab; on the
  last private close the profile cookies + view cache/history are wiped. Falls back to the
  old behavior only if the device WebView lacks MULTI_PROFILE. Clients are shared fields
  with active-view guards so background view callbacks cannot touch the live tab state.
- Automated import, no user files: tools/export_browser_data.py reads Chrome/Edge/Brave
  profiles (Bookmarks JSON + History sqlite via temp copy) and Firefox places.sqlite,
  merges to one v1 JSON, adb-pushes into the app's external import/ dir; the app scans
  that dir on every launch with zero permissions. Verified live end-to-end:
  "imported bm=43 hist=5114 from=pc_export.json" in logcat from his real Chrome x2 + Edge.
- SAF-watched folder lane and share-into-app remain for phone-side exports.

## desktop servo-real parity — input routing, real stop, home, favicons - 2026-08-20
- **Keyboard followed the mouse.** `WindowEvent::KeyboardInput` picked its target from the cursor
  position (`p.y < chrome_px`), so typing in the omnibox with the pointer parked over the page sent
  every keystroke to the page instead. Routing now uses a focus flag (`kbd_in_chrome`) fed by the
  chrome overlay's `focusin`/`focusout`; clicking the page clears it and blurs the omnibox, and
  Ctrl+L / Ctrl+F set it in Rust before the JS focus round-trip so the first keystroke can't race it.
- **Stop reloaded the page.** `"stop"` was `c.reload()` with a `(reload as proxy)` log line. It is now
  `window.stop()`, which Servo implements. The reload button becomes ✕ while loading and Esc stops the
  load — unless chrome holds keyboard focus, so Esc still blurs the omnibox first.
- Ctrl+wheel zoom over content (was missing). Home button + Alt+Home → `home` command.
- **Real favicons in the tab strip** instead of the letter monogram: `WebView::favicon()` → PixelFormat
  swizzle to RGBA → icons over 32px downscaled → PNG → base64 data URL in the state JSON, cached by tab
  URL and dropped on `notify_favicon_changed`. Monogram stays as the fallback.
- Gates added: `scripts/check.cmd` (cargo check under vsenv + GStreamer; bash `cargo check` dies in
  mozangle bindgen without MSVC `INCLUDE`) and `scripts/build_release.cmd`.
- Backups: `backups/servo_real.rs.v0.12.5.bak`, `backups/toolbar.html.v0.12.5.bak`.
- Found and left for the next seat: `WebViewDelegate::show_embedder_control` is unimplemented, so
  right-click menus, `<select>` dropdowns, file pickers, `alert`/`confirm`/`prompt`, the color picker
  and IME are all no-ops. Full API map in `docs/checklists/checklist_desktop_parity_v0.12.6.md`.

## 0.14.0-android.0 - 2026-08-20 - Chrome-parity wave 2: the rest of the ledger
- Tab switcher dialog, private tabs (badge, zero history, session-excluded, cache wipe on
  last close; cookies still shared - documented), omnibox suggestions from bookmarks+history,
  pull-to-refresh, long-press link/image context menu, bookmarks manager + bookmarks bar,
  history with search + clear-all, clear-browsing-data dialog, per-site JS block, text size,
  algorithmic dark pages, print/save-PDF, translate, reader mode.
- Menu regrouped: top-level tabs, then Page / Library / Settings submenus.
- DAO gained search/clear/delete/top queries (no schema change). Deps: +swiperefreshlayout.

## 0.13.0-android.0 - 2026-08-20 - Chrome-parity wave 1 + auto-import from other browsers
- Tabs: per-chip close x, favicons in chips (host-keyed), fixed close-index bookkeeping.
- Find in page, Desktop/Mobile site toggle, Share page, Bookmark this page, real downloads
  via DownloadManager (cookies+UA carried), external schemes (market/intent/tel/mailto)
  hand off to the OS.
- Import understands every mainstream export now: Chrome/Edge/Brave desktop Bookmarks JSON
  (WebKit-epoch dates), Firefox JSON backup, Netscape bookmarks HTML, plus the original v1
  JSON with history. Folder paths preserved; refuses files with credential keys.
- AUTO-import two ways: share an export file from any app straight into AmniBrowse
  (SEND/VIEW filters), or set "Auto-import folder" once (SAF, persisted) and every launch
  quietly imports new bookmark exports it finds there (per-file mtime dedupe).
- Parity ledger: docs/checklists/checklist_android_chrome_parity_v0.13.md.

## 0.12.6-android.0 - 2026-08-20 - Chrome-shaped chrome, amni-scient skin
- Android app relaid to match Chrome on a Fold: tab strip moved to the TOP (tablet Chrome
  order) with rounded-top active/inactive tab chips, toolbar below it (back, forward, new
  reload button, pill omnibox, home, menu), 2dp brass progress line under the toolbar.
- Styling: rounded 20dp omnibox pill (focus ring in accent), proper Material vector icons
  replacing the ic_media_* placeholders, neutral icon tint with brass reserved for accents -
  the existing amni-scient palette (ink #08090B / brass #C89B4E) carries the identity.
- All view ids preserved (btnBack/btnForward/btnHome/btnMenu/urlBar/tabStrip/...);
  new btnRefresh wired to web.reload(). Installed to SM-F966U over adb (debug signature
  matches the installed build), launch verified, versionName confirmed on device.

## v0.12.5-android.0 daily driver APK — 2026-08-20
- Kotlin app in `android/` (WebView + Amni chrome, Room bookmarks/history, http/https default-browser intents).
- Windows `scripts/export-chrome-amni.ps1` → `amni-chrome-import.json` from Chrome Default (no passwords).
- Autofill uses the system Google Password Manager. Servo on Android is later.
## v0.12.5 chrome not upside-down — 2026-08-16
- Toolbar was painting at the bottom of the window; clicks were aimed at the top. Same bug made on-page video (amni-scient.com/braid) look dead (00:00 / missed play).
- Chrome HTML is flex-end (WR Y-flip → visual top). Content blit starts at y=chrome_px.
## v0.12.4 session restore — 2026-08-16
- Tabs and window size persist across launches. Crash lock recovers the last dirty session.
- Duplicate tab (Ctrl+Shift+K). Settings toggle for restore. window.open DRM stays in-tab.
- Tab engine follows the URL: CLI/session navigate off Netflix no longer keeps Media on `amnibrowse://settings`. `route()` wins over stale `TabEngine`.
## chrome strip at window-top (not floor) - 2026-08-16
- Chrome webview was resized to 84px on a full-window GL context; ANGLE presented that sliver at the bottom (tabs on the floor). Chrome now resizes to the full window; content blit is GL (0,0,w,h-chrome). toolbar.html body is transparent again so the blit shows through.
## v0.12.3 in-tab DRM (no extra window) - 2026-08-16
- Netflix/etc. open in the same Amni window under the Servo chrome, as a normal tab. Child WebView2 fills the content band only.
## v0.12.2 Servo-primary media + in-engine PDF - 2026-08-16
- WebView hatch is DRM/CDM only. YouTube and other MSE hosts stay on Servo.
- Amni extracts YouTube progressive (muxed) streams and plays them in a Servo `<video>` page.
- PDF opens in-tab via pdf.js on fetched bytes; system viewer is fallback.
## v0.12.1 click-install + updates + BYO vault - 2026-08-12
- One-click Windows install from site feed then GitHub Latest (`scripts/install.ps1`, `AmniBrowse-Setup.cmd`) into `%LOCALAPPDATA%\AmniBrowse`.
- Auto-update check (site JSON → GitHub) in Settings + toolbar badge; Install replaces the installed copy and relaunches.
- Bring-your-own password manager: Amni / Bitwarden / 1Password / KeePassXC. Key icon fills the focused login like Chrome. Autofill-on-load when exactly one match.
- Publish `docs/latest.json` to `https://amni-scient.com/browse/latest.json` when you cut a release.
## v0.12.0 servo-core daily driver - 2026-08-12
- Default Cargo feature is `servo-real` (libservo). WebView is the opt-in stub.
- Downloads fire on archive/office/PDF navigations; PDF gets an in-tab opener page plus Downloads copy.
- Find in page (Ctrl+F), print (Ctrl+P), save/download (Ctrl+S), history/downloads flyouts (Ctrl+H/J), private tab (Ctrl+Shift+N).
- Omnibox suggestions from history + bookmarks. Vault unlock in Settings autofills matching logins. Local extension scripts inject on load.
- Windows “Set as default…” registers AmniBrowseHTML + StartMenuInternet and opens Settings.
- Profiles create/switch (relaunch with `AMNI_PROFILE`). CLI URL/`%1` opens in the first tab. `window.open` becomes a tab.
- Backups: `backups/*.v0.11.13.bak`.
## v0.11.14-dev webview stub hit parity - 2026-08-12
- Webview stub .tab-close 18px -> 28px matching CLOSE_HITBOX=28 (tab 32px, same geometry as shipping toolbar). Non-shipping backend (packer refuses it); src hygiene only. Backup: backups/webview.rs.v0.11.13.bak.

## v0.11.13 content blit vs chrome band - 2026-08-12
- **Header bar occluded by content (Anthony's fix):** paint_and_present blitted the content layer at GL target_rect origin (0, chrome_px) — GL is bottom-left origin, so the offset pushed content UP over the top chrome band instead of below it. Origin now (0,0) with height win.height-chrome_px: content owns the bottom, chrome overlay owns the top band. This was the remaining "launches with no header bar" pixel cause on the servo-real backend.
- README: shipping backend documented as servo-real; plain `cargo build --release` called out as the webview stub trap (clobber class from v0.11.11).
- Rebuilt --no-default-features --features servo-real after a default-feature build clobbered target/release at 10:02 (guard held: packer refused the webview exe).
- Tab close hitbox 20px -> **28px** (`CLOSE_HITBOX=28`, toolbar `.close` 28×28, font 14px) — Fitts-friendly without growing the 32px tab.
- URL bar hides internal page URLs (`data:`, `about:blank`, `amnibrowse://newtab`) — blank bar + placeholder on home instead of a data-URI dump.
- **Glass polish (UI seat):** Servo chrome contract **84px** (40 tab + 42 nav + 2 progress) in `tokens.rs` + `toolbar.html`; nav hits **36×36**; tab min/max 120–240; strip scrollbars thin not hidden-only. Theme root emits dual aliases (`--bg`/`--bg-primary`, `--dim`/`--text-muted`, `--chrome`, tab tokens) so home content + chrome poll paint one palette. Newtab footer shows `v{CARGO_PKG_VERSION} · Real Servo`. Webview shell matches tab 40 / nav 36 hit targets; engine badge is feature-honest (`Real Servo` vs `WebView2 · Chromium`).

## v0.11.12 tab strip mouse polish - 2026-08-12
- **Wheel-scrolls the tab strip:** #tab-list is overflow-x:auto with the scrollbar hidden (scrollbar-width:none), so once tabs overflowed there was NO mouse path to off-screen tabs (keyboard roving only). Vertical wheel over the strip now pans it horizontally (passive:false, deltaX passthrough untouched).
- **Double-click empty strip -> new tab** (Chrome/Firefox affordance; cuts travel to the 26px + button). Excludes tabs and the + button itself.
- **Middle-click empty strip -> new tab** (Firefox affordance), moved the auxclick handler from #tab-list to #tabs so the dead space right of the last tab counts; middle-click-on-tab still closes it.
- node --check on inline JS clean. Backup: backups/toolbar.html.v0.11.11.bak.
- **Poll-loop rubber band fixed (would have killed wheel pan):** renderTabs' unchanged-HTML early return called scrollIntoView(active) EVERY 250ms poll — any wheel pan snapped back to the active tab within a quarter second. Now scrollIntoView fires only when the active tab id actually changes (covers scroll-into-view on create/activate).
- **Close-neighborhood scroll stability:** innerHTML swap reset scrollLeft to 0 on every strip change (close, title update, loading flag). scrollLeft now preserved across the swap, so the tab under the cursor stays put after a close.

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

## 0.15.0-android.1 (2026-08-20)
Android: status-bar fix + home polish. targetSdk 35 forces edge-to-edge, so the tab strip sat
under the status bar - root now pads systemBars insets top and bottom. Home panel redesigned:
short copy, Import bookmarks / Make default buttons, BOOKMARKS section with two-line rows
(title + host) replacing the raw title\nurl dump that read like an import log, empty-state
hint. Tab chips and bars converted from raw px to dp. Import toast shortened; private-tab
toast now matches the real per-profile isolation.

## v0.12.5 - 2026-08-21 - favicon prime no longer freezes the page
Sync XHR to /favicon.ico blocked script and input on the document thread for the whole
round trip; on a slow origin that was a visible hang. The prime script now fires an async
XHR with an 8s timeout, stashes the result on window.__amniFav, and returns immediately.
The embedder pulls the stash on later state polls (max 30, then poll-timeout). BOM strip,
x-user-defined plane scan and the 900 KB cap are unchanged, just moved into onload.
191 tests pass.
