# Checklist — v0.12.7 SVG paint + prefers-color-scheme compat

Cause of "formatting and buttons broken across many sites" (2026-09-02 screenshots:
speedtest.net, x.com, developers.cloudflare.com, downdetector.com).

## Root causes found

- [x] **A. Inline SVG is rasterised standalone.** Servo XML-serialises only the
      `<svg>` subtree into a `data:` URL (`svgsvgelement.rs::serialize_and_cache_subtree`)
      and hands it to resvg. Anything outside the subtree is lost.
- [x] **B. `fill` / `stroke` are `engine = "gecko"` in Stylo.** Servo never computes
      them, so author CSS paint rules are dropped and `fill` falls back to initial
      black. `currentColor` collapses to black for the same reason.
- [x] **C. viewBox sizing.** At pin `68ca280`, `layout/replaced.rs:571` carried
      `// TODO: This is incorrect if the SVG has a viewBox` and unconditionally
      overwrote the CSS-computed box with the SVG natural size → giant Cloudflare
      logo, clipped SPEEDTEST wordmark. Upstream `main` fixed this
      (`if !has_viewbox { base.set_rect(...) }`).
- [x] **D. `prefers-color-scheme` never set.** `notify_theme_change` was called
      nowhere in `src/`, so Servo stayed on its `Theme::Light` default while the
      Amni chrome rendered dark.
- [x] **E. Cloudflare Turnstile.** Not a rendering bug; Servo is an unsupported
      browser. Already surfaced by `challenge_notice_script`. Out of scope.

## Work

- [x] Backup originals to `backups/*.v0.12.6.bak` (servo_compat, servo_real, Cargo.toml, Cargo.lock)
- [x] `Theme::is_dark()` via bg_primary relative luminance; `ThemeConfig::active_is_dark()`
- [x] `AppState::servo_theme` / `apply_servo_theme` / `broadcast_servo_theme`
- [x] Wire theme at all 4 webview creation sites + on settings theme change
- [x] `servo_compat::svg_repair_script()` — localise `<use>` and `url(#id)` refs
      under fresh ids, stamp author CSS paint as presentation attributes, resolve
      `currentColor` against computed `color` (which Servo *does* support)
- [x] Inject repair on load-complete and URL change
- [x] Bump Servo pin `68ca280` → `c91fc17` for cause C
- [x] Resolve bump fallout: `core.longpaths`, `rusqlite` 0.37→0.38, `cargo update`,
      pin `kstring` 2.0.2 (2.0.4 needs rustc 1.96, box has 1.95)
- [x] `cargo check --release --features servo-real` green
- [x] Release build `amni-browse` 0.12.7 with pin `c91fc17` (MouseButton Primary/Secondary/Auxiliary map)
- [ ] On-screen verification against the reported sites
- [ ] Update `architecture_map.md` + `changelog.md`

## Evidence

Baseline vs repaired, measured by replicating Servo's pipeline in Chrome
(serialize each `<svg>` alone → rasterise → mean luminance of covered pixels):

| speedtest.net | before | after |
|---|---|---|
| inline svgs | 31 | 31 |
| rendered pure black (lum < 40) | **22** | **0** |
| `currentColor` occurrences | 29 | 0 |
| dangling `<use>` | 0 | 0 |

Note: an early probe counted 43 "CSS fill" cases; 54 of those inherit `fill` from
an ancestor *presentation attribute*, which usvg resolves natively. Only 1 case on
speedtest.net truly comes from an author CSS rule. `currentColor` was the driver.
