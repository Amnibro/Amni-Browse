# Checklist — Resize + Rendering + Perf v0.10.3

**Reported (2026-07-22):** stretching the window does not resize contents; rendering glitches (speedtest.net and others); slower than expected.

## Reproduced
- [x] Programmatic MoveWindow probe + screenshots: chrome strip, content, AND GL surface all stay at launch size; new window area is unpainted white. Resize has never worked.
- [x] No surfman resize warnings in RUST_LOG=warn run → suspect `WindowEvent::Resized` never reaches handler, or GL ops in resize path silently fail.

## In flight
- [x] Instrument `resize_all` (entry log) + paint-time ctx-vs-window size mismatch detector
- [x] Candidate fix: `make_current()` before resize GL work (offscreen `Framebuffer::new` no-ops on non-current context)
- [x] Rebuild, re-probe, read logs → root cause: painter's `resize_rendering_context` early-returns when `rendering_context.size() == new_size`; our `resize_all` pre-resized both contexts manually, so `WebView::resize` became a no-op (viewport/relayout/repaint chain skipped)
- [x] Fix: removed direct `rendering_context.resize` + `offscreen_context.resize` calls; `webview.resize()` drives the whole chain (matches servoshell)
- [x] Fix confirmed by screenshot probe 2026-07-22: chrome + content fill 1560x980 after resize, reflow correct
- [ ] User confirms interactive drag-resize feels right

## Rendering / perf context (not embedder bugs)
- Servo rev 68ca280 has incomplete site compat (e.g. `position: fixed` falls back to static in taffy path) → layout glitches on complex sites are engine-level
- Release profile already opt-level=3 + fat LTO; speed gap vs Chromium is engine maturity
- [ ] Evaluate Servo pin bump to current main (perf + compat gains vs API churn) — separate task

## Non-goals
- Media window resize behavior (wry handles its own)
