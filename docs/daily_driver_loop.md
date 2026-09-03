# Daily-driver loop QA

## 2026-08-17T00:09Z loopqa (v0.12.4 release, GPU still exhausted)

**Exe:** reuse `target\release\amni-browse.exe` 123510784 B @ 19:59:41. Src older (servo_real 19:42). No rebuild. No leftover `amni-browse` at start.

**Unit tests** (`amni_browse-87b30e71fe8ac621.exe`): media_for_drm_domains, mse_hosts_stay_on_servo, content_bounds_sit_under_chrome, parses_watch_short_embed, pdf_detects_query_and_hash, navigate_clears_stale_media_engine — **6/6 PASS**.

**Launch 1** `AMNI_PROFILE=loopqa` `https://html.duckduckgo.com/html/` (~12s):

| check | result |
|---|---|
| Backend: Real Servo | PASS (logged then died) |
| restore/route Servo | **FAIL** never reached — panic at init |
| title DuckDuckGo | **FAIL** process dead, HWND enum n/a |
| spawn_media_window | ABSENT |
| panic | **FAIL** surfman `SwapChain11::reset` HRESULT **0x80070057** assert 0x0 (`surfman-0.12.0` surface.rs:266) |

Crash recovery noted previous session; CLI url logged. No second launch (Netflix/YT skipped — ANGLE still toast; one Stop-Process would not help).

**Files:** `docs/loop_run.log`, this section. No rust.

**Next:** leave GPU idle a long time; one launch only after cool-down. Do not kill-relaunch storm.

## 2026-08-16T23:54Z / 2026-08-17T00:00Z engine-follow rebuild

**Pre-rebuild exe** (19:38): DDG/example/PDF/YT all restored as **Media** + in-tab DRM pane (stale session engine). Netflix 1 HWND + real DRM. **FAIL** Servo-primary.

**Shipped:** `cargo test navigate_clears_stale_media_engine` **PASS**. `cargo build --release --features servo-real` → exe 123510784 B @ 19:59:41.

**Post-rebuild:**

| check | result |
|---|---|
| DDG | PASS `restore … Servo` title DuckDuckGo HTML. No DRM pane |
| example.com | PASS Servo |
| spawn_media_window | ABSENT |
| YT then Netflix (rapid relaunch) | **FAIL** panic surfman `SwapChain11::reset` HRESULT 0x80070057 / assert 0x0. Retry Netflix +3s same. ANGLE swap-chain exhaustion after kill/relaunch storm |
| DRM same HWND | not re-proven after panic |

**Files:** rebuilt exe; `docs/daily_driver_loop.md`. No new rust this fire (prior engine_for_url).

**Next:** cool GPU, single Netflix + YT progressive check; don’t sequential-kill-relaunch.

## 2026-08-16T23:39Z loopqa (v0.12.4 release exe)

**Exe:** rebuilt by someone else — 123536384 B @ 2026-08-16 19:38. Logs **v0.12.4**. No leftovers.

**Unit tests** (existing debug test exe): media_for_drm, mse_hosts, content_bounds, parses_watch, pdf_detects — **5/5 PASS**.

**Launch** `AMNI_PROFILE=loopqa`:

| check | result |
|---|---|
| Backend: Real Servo | PASS |
| spawn_media_window / panic | ABSENT PASS |
| DDG | PASS title DuckDuckGo HTML; `servo restore tab 0 … Servo` |
| example.com | PASS Example Domain |
| W3C PDF | PASS Servo URL; title Amni Browse |
| YouTube | PASS Servo page `Me at the zoo`; **no** progressive-player line |
| Netflix | PASS title Netflix…; `restore … Media` + `in-tab DRM pane`; **1** Amni HWND |
| amnibrowse://settings | **FAIL** `restore tab 0 amnibrowse://settings Media` — stale TabEngine from prior Netflix session |

**Fix shipped (source, not this exe):** `tabs.rs` engine_for_url on new/nav/back/forward; `servo_real` always `route(url)`. Test `navigate_clears_stale_media_engine`. CHANGELOG + architecture_map.

**Next:** vcvars release rebuild so loopqa runs the engine-follow fix; progressive extract still missing on YT.

## 2026-08-16T23:24Z loopqa (v0.12.3 release exe)

**Exe:** still `target\release\amni-browse.exe` 123493888 B @ 2026-08-16 16:36. Tree is 0.12.4; binary logs 0.12.3. No rebuild (exe present). No leftover user session.

**Unit tests** (debug test exe `amni_browse-87b30e71fe8ac621.exe`): media_for_drm, mse_hosts, content_bounds, parses_watch, pdf_detects — **5/5 PASS**.

**Launch** `AMNI_PROFILE=loopqa` (seen_onboarding now true):

| check | result |
|---|---|
| Backend: Real Servo | PASS all launches |
| spawn_media_window / panic | ABSENT PASS |
| DDG CLI | PASS title `DuckDuckGo HTML…` initial url html.duckduckgo.com/html/ |
| example.com | PASS `Example Domain - Amni Browse` |
| W3C dummy PDF | PASS Servo loaded URL (title stayed `Amni Browse`) |
| YouTube watch | PASS Servo page `Me at the zoo`; no progressive-player log; no hatch |
| Netflix | PASS `in-tab DRM pane … WebView2`; **1** Amni-titled HWND (`New Tab - Amni Browse`); content initial was newtab data URL + child pane |
| amnibrowse://settings | PARTIAL — opened newtab data URL, title New Tab |

Each proc: 1 Amni-titled + 2 extra HWNDs (blank, exe path). Not a second winit DRM window.

**Files:** `docs/daily_driver_loop.md` only. No rust change.

**Next:** ship 0.12.4 release exe (`vcvars` + `cargo build --release --features servo-real`); prove progressive extract log on YT; settings scheme.

## 2026-08-16T23:13Z loopqa (v0.12.3 release exe)

**Exe:** `target\release\amni-browse.exe` present (123493888 bytes, 2026-08-16 16:36). Cargo.toml is **0.12.4**; shipped binary still logs **v0.12.3**. No rebuild this fire (exe existed).

**Unit tests** (existing `target\debug\deps\amni_browse-87b30e71fe8ac621.exe`, not a cargo rebuild):

| test | result |
|---|---|
| media_for_drm_domains | PASS |
| mse_hosts_stay_on_servo | PASS |
| content_bounds_sit_under_chrome | PASS |
| parses_watch_short_embed | PASS |
| pdf_detects_query_and_hash | PASS |

`cargo test` with a piped filter matched **0** tests. A later `cargo test` **without vcvars64** started a full rebuild and **mozangle** failed (`LIB`/`INCLUDE` empty). Always vcvars for compile.

**Launch 1** `AMNI_PROFILE=loopqa` `https://html.duckduckgo.com/html/` (~8s alive):

| check | result |
|---|---|
| Backend: Real Servo | PASS |
| panic / spawn_media_window | ABSENT (PASS) |
| in-tab DRM / progressive player / child webview | not expected on DDG |
| CLI DDG as content | **FAIL** — `servo content initial url` was first-run tutorial data: URL (`seen_onboarding` false). 0.12.4 source already skips tutorial when `cli_http`. |

Process later gone (killed/exited). Did not touch a personal session.

**Launch 2–5** sequential loopqa (8s each, then Stop-Process):

| URL | alive | titled Amni HWND | notes |
|---|---|---|---|
| https://example.com | yes | Example Domain - Amni Browse | Servo nav PASS |
| w3.org dummy.pdf | yes | Amni Browse | Servo; title not PDF-specific |
| youtube.com/watch?v=jNQXAC9IVRw | yes | Me at the zoo - YouTube | Servo page (200). No `progressive player` line. No WebView hatch. PASS per law |
| https://www.netflix.com/ | yes | New Tab - Amni Browse | `media_engine: in-tab DRM pane ... WebView2`. **No** `spawn_media_window`. One Amni-titled top-level. PASS |

Each process also had 2 extra visible HWNDs (empty title + exe-path) — not a second winit “Amni Browse” DRM window.

**Files changed:** this log only. No rust fix shipped (0.12.4 already has CLI-skip-tutorial).

**Next swing:** `vcvars64` + `cargo build --release --features servo-real` so loopqa runs **0.12.4**; re-prove DDG CLI skip + progressive extract path; cargo test only under MSVC env.
