# Checklist — v0.8.0 P3b-v2: Click Coords + Resize Reflow

**Date:** 2026-04-17
**Scope:** `src/platform/servo.rs` only (pipeline already has the right APIs)
**Roadmap phase:** v0.5 guardian council — Phase 3 (paint) polish

## Why
P3b shipped auto-height + live viewport, but two known gaps remained:
- **Clicks through the image are dead.** `WindowEvent::MouseInput` is consumed by egui on the egui `Image` widget before our raw handler sees it. Forms, links, buttons inside rendered pages can't be clicked.
- **Resize doesn't reflow.** We only re-render when the URL changes. Making the window wider/narrower keeps the old texture and just rescales it, so text wraps at the *original* width and the content looks compressed or pillarboxed.

## Design
- **Click path:** drop the raw `WindowEvent::MouseInput` dispatch. Replace `ui.image(...)` with `ui.add(egui::Image::…sense(Sense::click()))`, capture the `Response`, and on `clicked()` compute `(pointer_pos - resp.rect.min) / display_scale` → that's the content-space coordinate we already computed layout in. Dispatch through `interactor.dispatch_click(x, y)`.
- **Reflow path:** on every frame, if the painted page's `rendered_vw` differs from `ui.available_width()` by >8 px AND no render is pending AND the url matches, spawn a render-only task that calls `pipeline.render_to_pixels(html, css, vw, vh)` directly (no network fetch). Emit the same `page_painted` IPC JSON with a `reflow: true` flag so `check_rendered_pages` skips `run_page_scripts` (scripts already ran).
- **State additions on `AmniApp`:** `rendered_css: Vec<String>`, `rendered_vw: f32`.
- **IPC response:** add `css_sources` array + `reflow` bool.

## Changes
- [x] Backup `src/platform/servo.rs` → `backups/platform_servo.v0.8.0-P3b.bak`
- [x] Guardian council convened (`docs/guardian_councils/guardian_council_v0.8.0_P3b_v2_click_reflow.md`)
- [x] Add `rendered_css: Vec<String>` and `rendered_vw: f32` to `AmniApp`; init in `run()`
- [x] Extend fetch-task IPC JSON: include `css_sources: page.css_sources` and `reflow: false`
- [x] Add resize-reflow branch that spawns a `render_to_pixels`-only task, emits IPC JSON with `reflow: true`
- [x] Remove `WindowEvent::MouseInput` raw-click handler (lines 95-105)
- [x] Swap `ui.image(SizedTexture::new(tex.id(), display_size))` → `ui.add(egui::Image::from_texture(...).sense(Sense::click()))`, capture response, collect `pending_click: Option<(f32,f32)>` scaled back to content coords
- [x] After the central panel closure: if `pending_click.is_some()`, call `self.state.interactor.dispatch_click(x, y)` + `focus_node` on hit
- [x] `check_rendered_pages`: unpack `css_sources` → `self.rendered_css`, set `self.rendered_vw = w as f32`; skip `run_page_scripts` when `reflow == true`
- [x] Rebuild: `cargo build --release --no-default-features --features servo-engine` — 0 errors, 418 warnings, 2m 44s
- [x] Update `ARCHITECTURE.md` — note click-via-egui-response + render-only reflow
- [x] Update `CHANGELOG.md` — v0.8.0 P3b-v2 entry
- [ ] User confirms: clicks on page links fire, window resize reflows wrapping at new width

## Deferred (NOT in this sprint)
- Scroll-position preservation across reflow (egui ScrollArea resets on texture-size change). → nice-to-have.
- GPU paint port (`SoftwareRenderer` → wgpu RenderPipeline). → P3a sprint.
- Hover cursor feedback (egui `Image` with `Sense::click` shows pointer cursor — usually OK; if not, `.on_hover_cursor`). → polish.
