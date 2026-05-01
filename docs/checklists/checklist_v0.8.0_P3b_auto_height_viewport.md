# Checklist — v0.8.0 P3b: Native Auto-Height Render + Real Viewport

**Date:** 2026-04-17
**Scope:** `src/engine/pipeline.rs`, `src/platform/servo.rs`
**Roadmap phase:** v0.5 guardian council — Phase 3 (paint) ergonomic step before GPU port

## Why
Native backend currently renders every page to a hardcoded 1280×2048 RGBA bitmap. Anything past 2048 CSS px is clipped; anything narrower than 1280 is scaled down; the viewport never matches the actual window. Scroll already works (egui `ScrollArea::both` wraps the image) but has nothing to scroll *to* past 2048.

## Changes
- [x] Backup `pipeline.rs` → `backups/pipeline.v0.7.2.bak`
- [x] Backup `platform/servo.rs` → `backups/platform_servo.v0.7.2.bak`
- [x] Guardian council convened (`docs/guardian_councils/guardian_council_v0.8.0_P3b_auto_height.md`)
- [x] `RenderPipeline::render_to_pixels` — after layout, compute `content_h = max(rect.y + rect.h)` across all layout rects, clamp to `[vh, 16384]`, allocate pixels at `(vw, content_h)` instead of `(vw, vh)`
- [x] `platform/servo.rs` — pass `ui.available_width()` / `ui.available_height()` to `fetch_and_render` instead of hardcoded 1280/2048
- [x] Rebuild: `cargo build --release --no-default-features --features servo-engine` — 0 errors, 418 warnings, 3m 21s
- [x] Update `ARCHITECTURE.md` — note dynamic viewport + content-height-driven painting
- [x] Update `CHANGELOG.md` — v0.8.0 P3b entry
- [ ] User confirms: long pages (e.g. wikipedia article) scroll the full content, window resize reflows on next nav

## Deferred (NOT in this sprint)
- Click coordinate correction through egui scroll offset + image display scale (currently image clicks are consumed by egui, raw-window handler never fires). → separate P3b-v2 sprint.
- Re-render on window resize without re-fetching. → P3b-v2.
- GPU paint port (`SoftwareRenderer` → wgpu RenderPipeline). → P3a sprint.
