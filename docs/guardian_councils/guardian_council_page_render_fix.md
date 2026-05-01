# Guardian Council — Page Render Fix (v0.8.2)

**Question under debate:** Servo backend paints every page as a tiny dark rectangle (~150×80 px) instead of a real layout. Evidence: live screenshot of `crunchyroll at DuckDuckGo` tab shows no rendered content. Root cause identified in `src/engine/layout.rs::to_taffy_style`: the `_` fallback arm maps every non-Flex / non-Grid / non-None CSS display (i.e. `Block`, `Inline`, `InlineBlock`, `Contents` — the vast majority of elements on every real webpage) to `taffy::Display::Flex` with default `flex_direction: Row` and no wrap. Every webpage is laid out as one horizontal row of squashed children.

**Secondary defects in scope for the same patch:**
- `platform/servo.rs` never syncs `chrome.url_input` from the active tab on tab switch — URL bar reads empty.
- `platform/servo.rs` submits wgpu command buffers every frame but never polls the device — wgpu logs spam `Device::maintain: waiting for submission index` at INFO.

---

## Proposals

### Architect — "Use taffy's native Block"
Taffy 0.7.7 has first-class `Display::Block`. Map `CssDisplay::Block` (and the `_` fallback for Inline/InlineBlock/Contents) directly to `taffy::Display::Block`. Zero new machinery, semantics match what the DOM expects, no flex-direction gymnastics. Width-auto = fill parent content area, height-auto = fit content, children stack vertically. This is what CSS normal flow is. **Vote: Block.**

### Sentinel — "Don't simulate what the engine already does"
Emulating block flow with `Flex + FlexDirection::Column` is a trap. Flex items have different sizing defaults (main-axis = content, cross-axis = stretch) from block items (main-axis = parent-width, cross-axis = content). You'd get subtle width mismatches on every page and spend the next six patches chasing them. **Vote: Block.**

### Scholar — "Check the CssDisplay enum coverage"
`engine/style.rs:87` declares `Display { Block, Inline, Flex, Grid, InlineBlock, InlineFlex, None, Contents }`. Current code handles Flex (both forms), Grid, None. Missing: Block, Inline, InlineBlock, Contents. Mapping all four to `taffy::Display::Block` is the minimum-surprise treatment for the current MVP renderer. `Inline` / `InlineBlock` wanting true horizontal inline flow is a later concern for the inline-layout engine (`engine/inline_layout.rs` already exists). **Vote: Block.**

### Engineer — "Fix it in one arm; don't touch flex_direction"
Taffy ignores `flex_direction`, `flex_grow`, `flex_shrink`, and `gap` on non-Flex displays. We do not need to special-case these fields for Block — leave them populated as-is. The single behavioral change is the display discriminant. This keeps the patch to a one-token replacement in the match arm. **Vote: Block, minimal diff.**

### Pathfinder — "Ship adjacent fixes in the same patch"
The URL-bar-empty and the wgpu-submission-spam bugs were both caught in the same repro screenshot. They are small, obvious, and shipping them together avoids a second round-trip. Tag the release v0.8.2. **Vote: Block + url sync + device.poll.**

---

## Verdict

**Unanimous:** `CssDisplay::Block | Inline | InlineBlock | Contents | _` → `taffy::Display::Block`. Ship the URL-bar sync and `device.poll(Maintain::Poll)` in the same patch as v0.8.2. No changes to `flex_direction` / `flex_grow` / `flex_shrink` / `gap` — those fields are inert under Block display in taffy 0.7.7.

## Deferred

- True CSS Inline flow via `engine/inline_layout.rs` integration — currently inline elements are treated as block, which is wrong for mixed text+inline content (e.g., `<p>text <em>word</em> text</p>`). Schedule for v0.9.x.
- `Display::Contents` semantics (no box, only children lay out) — taffy has no native equivalent; current Block treatment produces a real box. Low impact, rare in the wild.
- Replace per-frame `window.request_redraw()` with an event-driven redraw loop — currently we burn GPU at full refresh rate with nothing changing. Schedule for v0.8.3.
