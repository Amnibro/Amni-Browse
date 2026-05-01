---
name: Guardian Council — v0.8.0 P3b: Auto-Height Render + Live Viewport
date: 2026-04-17
scope: src/engine/pipeline.rs, src/platform/servo.rs
verdict: 5-0 APPROVE (auto-height + ui.available_size)
---

# Question
Native backend renders every page to a hardcoded 1280x2048 RGBA bitmap. Clips content past 2048 CSS px, scales sub-1280 windows, and never reflows on window resize. How do we size the canvas from layout + window, cheaply and without destabilising paint?

# Guardian verdicts

## 1. Architect — APPROVE
Pipeline already separates `fetch_and_parse` -> `parse_and_layout` -> `render_to_pixels`. Layout is computed *before* allocation. The canvas size should be a *derived* value, not a parameter dictated by the caller. `vw` stays the content-box width (from window); `vh` becomes a floor, not a ceiling. Content height = `max(rect.y + rect.h)` over all layout rects, clamped `[vh, 16384]`. Clean, local, matches how real browsers treat document height.

## 2. Sentinel — APPROVE (with clamp)
Hard clamp at 16384 px is mandatory. A malicious or buggy stylesheet can blow `content_h` into gigabyte allocations. 16384 * 2000 * 4 bytes ~ 125 MiB worst case — survivable, not silent. Also: floor at `vh` so short pages still fill the viewport (prevents white gutter below `<body>`).

## 3. Scholar — APPROVE
This is exactly how Firefox/Blink's "canvas sizing" has worked since the Gecko 1.0 era: layout produces an unconstrained *document height*, paint surface matches that, scroll containers crop. The only reason we hardcoded 2048 was the v0.5 "first paint" sprint just wanted *something* on screen. That debt is now billable.

## 4. Engineer — APPROVE
Two edits, both <15 LoC. No new types, no trait changes, no renderer API churn. `SoftwareRenderer::new(w, h)` already accepts arbitrary dimensions. egui `Image` sized by `RenderedPage.width/height` already — no UI code needs touching. `ScrollArea::both` in servo.rs will just *work* once the image is taller.

## 5. Pathfinder — APPROVE (flag the deferred)
Green-light this sprint but write the deferred list into the checklist *now*: (a) image-click coordinate translation through scroll offset + image scale is still broken, (b) window-resize triggers re-layout/re-paint without re-fetch, (c) GPU port. P3b-v2 sprint owns (a)+(b); P3a sprint owns (c). Do not let this sprint grow to eat them.

# Verdict
5-0 APPROVE. Ship the two-file edit. Keep deferred items on the checklist.
