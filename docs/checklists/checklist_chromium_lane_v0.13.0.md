# Checklist — v0.13.0 flip to the Chromium (WebView2) lane

Anthony 2026-09-03: "do the flip because the servo formatting is evidently incredibly fragile. need my own private browser that looks good, runs better, and is secure and private."

- [x] Backups: backups/Cargo.toml.v0.12.7.bak, webview.rs.v0.12.7.bak, ui_webview.rs.v0.12.7.bak
- [x] `default = ["webview"]`; build/package scripts repointed; servo-real stays buildable
- [x] Shared internal pages (`ui/internal_pages.rs`) moved out of servo_real.rs
- [x] `platform/chromium.rs`: chrome child + per-tab WebView2 children, toolbar contract (state/cmd/suggest/history/downloads), session restore + window position, keyboard shortcuts (chrome ipc + content ipc + window fallback), private tabs (incognito), popups, downloads, zoom, find, print, view-source, settings/newtab/tutorial pages
- [x] Request-level shield (WebResourceRequested) + DNT/GPC + DoH args + privacy browser args
- [x] On-screen: speedtest ad-free with logo/colors; GitHub login -> Google Accounts sign-in page; HN; Wikipedia; tab click/+/Ctrl+1 work; titles sync
- [x] Local config: block_ads/block_trackers/enable_do_not_track on (backup config.json.bak-v0128)
- [x] Installed copy refreshed (%LOCALAPPDATA%\AmniBrowse, old exe kept as .servo-0.12.bak)
- [x] Frameless window with the toolbar's own min/max/close; edge resize (1768 -> 1920 in the rig) and drag-move (32,32 -> 232,235) verified
- [x] Favicons in the tab strip (FaviconChanged)
- [x] Download progress into the downloads panel; [x] find bar with highlight-all (CSS Custom Highlight API)
- [x] Engine back/forward/reload/stop, HTML5 fullscreen, audio state, password autosave + autofill, clear-on-exit, DevTools, full key map
- [x] Amni-OS (Linux) lane: same code on WebKitGTK via wry (verified in the Amni OS VM, speedtest via CLI URL); CEF later if needed
- [x] Android tab groups collapse (0.16.9-android.0, on the Fold + site)

## Parity lap 2 (2026-09-03)
- [x] Pinned tabs (right-click menu), persisted
- [x] Tab groups with collapse chip, persisted
- [x] Find highlights (verified via no-input probe)
- [x] Content scripts/CSS from extensions/ on load
- [x] Ctrl+N new window (--new-window, no session persist)
- [x] Linux/macOS fallback to plain wry backend (cfg gating)
- [x] Packager rewritten: exe + toolbar + docs, versioned + unversioned zip
- [x] Site: amni-browse.html Windows download block + browse/latest.json 0.13.0
- [x] GitHub release v0.13.0 (assets re-uploaded after the Linux refactor) + site push + live md5 check

## Linux lane (2026-09-04)
- [x] cfg(windows) gating of COM imports/handlers; `Core = ()` on Linux; numeric download-state consts
- [x] Native `amnibrowse://` scheme on Linux, `http://amnibrowse.<host>/` on Windows; `fetch_shim()` empty off Windows
- [x] Decorated window off Windows; overlay-inset content rect; favicon.ico fallback
- [x] CLI URL argument opens as a tab; `WEBKIT_DISABLE_DMABUF_RENDERER=1`
- [x] Built in the Amni Bake chroot (`C:/amni-bake/incoming/linux-build.sh`), ran in the Amni OS VM (`run-ab.sh`), screenshots os2_ab4/os2_ab5
- [x] PKGBUILD moved to the WebKitGTK lane (backup `amni-os/backups/PKGBUILD.servo-lane.v0.13.0.bak`)
- [x] WebKitGTK content-rule-list shield for subresources + css-display-none for blocked ad iframes (patch_linux_shield_v0131.py, built in WSL Ubuntu-24.04 since the bake VM was in use)
- [ ] Fresh ISO bake with the new PKGBUILD
