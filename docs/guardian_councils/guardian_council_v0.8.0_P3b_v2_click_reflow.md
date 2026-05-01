---
name: Guardian Council — v0.8.0 P3b-v2: Click Coords + Resize Reflow
date: 2026-04-17
scope: src/platform/servo.rs
verdict: 5-0 APPROVE (egui-response click + render-only reflow with script-skip)
---

# Question
Two known gaps from P3b: (1) image clicks never reach `dispatch_click` because egui consumes them, (2) window resize rescales the old texture instead of reflowing at the new width. How do we fix both without a second paint pipeline or a major refactor?

# Guardian verdicts

## 1. Architect — APPROVE
The fixes live entirely in `src/platform/servo.rs`. The pipeline already exposes `render_to_pixels(html, css, vw, vh)` — a pure, cache-free, network-free function — which is exactly the primitive a reflow needs. The click path should route through egui's `Response` API because that's how egui's event model works; fighting it with raw `WindowEvent` is the reason the bug exists. Clean separation: event capture in egui, coordinate translation in the view code, hit-testing in `Interactor`.

## 2. Sentinel — APPROVE (watch the reflow storm)
Resize events fire per-pixel of drag. Without a guard, a user dragging the window edge for 2 seconds could queue hundreds of renders. The proposed `render_pending` single-flight flag handles this: while one reflow is in progress, no new ones queue. Threshold `(cur_vw - rendered_vw).abs() > 8.0` adds hysteresis so a 1-pixel jitter doesn't trigger. Also: `reflow: true` flag must skip `run_page_scripts` — otherwise every resize re-runs every `<script>`, which is both wasteful and could cause double-fires of anything non-idempotent.

## 3. Scholar — APPROVE
This is exactly how Gecko has reflowed since the '90s: layout is a pure function of (DOM, CSS, available_width), and changing any input means rerunning from the last cache layer. We already have the DOM and CSS in memory; we just need to invoke the layout+paint. The only thing real browsers do on top is incremental/dirty-rect reflow, which is a month of work — pure reflow is fine for 2026-04 Amni.

## 4. Engineer — APPROVE
Small diff, no new types. Two struct fields, one IPC field, one branch in the render loop, one branch in `check_rendered_pages`, one closure edit for click capture. Should be <60 LoC total. Click coordinate math is well-understood: `(pointer_pos - img_rect.min) / display_scale`. No thread-safety concerns — all new state is `Arc<Mutex>`-free because it lives on the single-threaded `AmniApp`.

## 5. Pathfinder — APPROVE (flag the deferred)
Ship this. Explicitly defer: scroll-position preservation across reflow (texture ID changes, `ScrollArea` state resets — user will notice a jump, but it's way better than "frozen at old width"), hover cursor styling, GPU port. Write these into the checklist so they don't quietly disappear.

# Verdict
5-0 APPROVE. No dissenting voice. Ship the two-branch servo.rs edit.
