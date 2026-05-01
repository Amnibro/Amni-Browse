# Checklist — Page Render Fix v0.8.2

**Scope:** Three fixes in one patch, all rooted in the Servo backend render/chrome layer.

## Tasks

- [x] Read CLAUDE.md workflow rules (zero comments, no empty lines, ternary, minimize line count)
- [x] Read ARCHITECTURE.md current state (v0.8.1.1)
- [x] Read CHANGELOG.md recent entries
- [x] Read src/main.rs (entry point, feature gating)
- [x] Read src/platform/servo.rs (renderer, window/event loop, texture upload)
- [x] Read src/engine/pipeline.rs (fetch_and_render, render_to_pixels, build_tree)
- [x] Read src/engine/paint.rs (RenderTree::build_from_dom, build_display_list, SoftwareRenderer)
- [x] Read src/engine/layout.rs (to_taffy_style — **root cause found here**)
- [x] Grep CssDisplay enum variants in src/engine/style.rs
- [x] Grep taffy version in Cargo.lock (0.7.7)
- [x] Convene guardian council (docs/guardian_councils/guardian_council_page_render_fix.md)
- [x] Get the maintainer's green-light (manager mode approval)
- [x] Backup src/engine/layout.rs → backups/layout.v0.8.1.bak
- [x] Backup src/platform/servo.rs → backups/servo.v0.8.1.bak
- [x] Edit src/engine/layout.rs — `_` fallback arm now `taffy::Display::Block`
- [x] Edit src/platform/servo.rs — `last_tab_url` field added, sync at top of `render()`, init in `run()`
- [x] Edit src/platform/servo.rs — `gpu.device.poll(wgpu::Maintain::Poll)` after `queue.submit` + `frame.present`
- [x] Update ARCHITECTURE.md with v0.8.2 header
- [x] Update CHANGELOG.md with v0.8.2 entry
- [x] the maintainer: `cargo run --no-default-features --features servo-engine` — pastes result
- [x] Verify URL bar shows active tab URL after tab switch (WORKING)
- [~] Verify page renders with content — page sized correctly but text invisible; root cause: default `Color.a = 0.0`. Fix shipped in v0.8.2.1.
- [~] Verify terminal is quiet — `Maintain::Poll` alone insufficient; `wgpu_core` INFO log filter needed. Fix shipped in v0.8.2.1.

## v0.8.2.1 follow-up patch

- [x] Backup src/main.rs → backups/main.v0.8.2.bak
- [x] Backup src/engine/paint.rs → backups/paint.v0.8.2.bak
- [x] Backup src/engine/pipeline.rs → backups/pipeline.v0.8.2.bak
- [x] Edit src/engine/paint.rs — `cs.color = Color { r:0, g:0, b:0, a:1.0 }` in `walk()`
- [x] Edit src/engine/pipeline.rs — same init in `build_tree()`
- [x] Edit src/main.rs — env_logger filter `info,wgpu_core=warn,wgpu_hal=warn,naga=warn,egui_wgpu=warn`
- [x] Update ARCHITECTURE.md with v0.8.2.1 header
- [x] Update CHANGELOG.md with v0.8.2.1 entry
- [~] the maintainer re-ran — "same issue". Deeper diagnosis: not the default text color (that fix is still correct and still in). Real bug is `parse_color` returning opaque black for every non-hex value. Fix shipped in v0.8.2.2.

## v0.8.2.2 follow-up patch (real color parser)

- [x] Backup src/engine/style.rs → backups/style.v0.8.2.1.bak
- [x] Rewrite `parse_color` as dispatcher (transparent/inherit/currentcolor/none keywords, shorthand tokenization)
- [x] Add `parse_color_one` (hex 3/4/6/8 + rgb()/rgba() with comma or space/slash separators, % or 0–255 components)
- [x] Add `parse_named_color` (~130 CSS3 named colors in one dense match)
- [x] Unknown values return `Color { a: 0.0 }` (transparent), NOT opaque black
- [x] Apply patch via Python script (Edit tool truncated on CRLF-heavy replacement); style.rs now 592 lines
- [x] Update ARCHITECTURE.md with v0.8.2.2 header
- [x] Update CHANGELOG.md with v0.8.2.2 entry
- [ ] the maintainer: re-run `cargo run --no-default-features --features servo-engine`
- [ ] Verify DuckDuckGo text is visible on rendered page
- [ ] Verify terminal is quiet (no `Device::maintain` spam)

## Non-goals (deferred)

- Inline layout integration (v0.9.x)
- Event-driven redraw (v0.8.3)
- Consolidating layout + render tree traversals into one pass (v0.8.3)
- WebView backend bugs — separate patch, separate session
