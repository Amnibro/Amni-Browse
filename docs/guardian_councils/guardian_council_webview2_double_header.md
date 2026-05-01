# Guardian Council — WebView2 Double Header + Broken Home

**Date:** 2026-04-17
**Target:** `src/platform/webview.rs`

## Problem Statement
Windows WebView2 rewrites the registered `amnibrowse://` custom scheme to an internal `http://amnibrowse.<host>/` origin. This leaks in two ways:
1. `chrome_init_js()` sees `location.protocol === 'http:'` on newtab and injects the shadow toolbar on top of the SPA chrome → double header.
2. `Act::Nav` → `webview.load_url("amnibrowse://newtab")` from a real http/https page does not round-trip through the remap, so home navigation silently no-ops.

## Guardian Proposals

### 1. Architect — "Match the platform's reality"
Detect WebView2's internal origin by hostname (`amnibrowse.*`) in the init script AND pre-remap `amnibrowse://` URLs to `http://amnibrowse.<host>/<path>` before `load_url`. Both layers then speak the same language the browser process actually uses. **Vote: FIX.**

### 2. Sentinel — "Guard at the boundary"
Shadow toolbar injection is a safety/UX boundary — strengthen it. Check hostname, not just protocol. Also guard against `window.location.hostname === 'amnibrowse.newtab'` exact-match for safety. Keep the home rewrite scoped so external `http://` URLs that legitimately happen to have `amnibrowse.` prefix don't collide. **Vote: FIX with strict prefix.**

### 3. Scholar — "Is there a crate-level fix?"
wry 0.47+ exposes `with_custom_protocol_handlers` with better URL mapping, but upgrading is out of scope for a hotfix. Document the WebView2 remap in ARCHITECTURE.md so future contributors don't re-hit this. **Vote: FIX + document.**

### 4. Engineer — "Smallest diff, verifiable"
Two surgical edits in one file. No new deps, no re-plumbing. Preserve existing behavior on every non-amnibrowse URL. Verify via `cargo build --release`. **Vote: FIX.**

### 5. Pathfinder — "Future-proof the scheme"
If we ever add more internal pages (`amnibrowse://history`, `amnibrowse://settings`), the remap helper should handle any host, not just `newtab`. Generalize to `http://amnibrowse.<host>/<path>` form. **Vote: FIX generalized.**

## Verdict
**5-0 FIX.** Consensus: two-surface fix in `src/platform/webview.rs`:
- init-script guard checks hostname prefix `amnibrowse.`
- `Act::Nav` rewrites `amnibrowse://<host>/<path>` → `http://amnibrowse.<host>/<path>` before `webview.load_url`
- document the WebView2 remap in `ARCHITECTURE.md`
