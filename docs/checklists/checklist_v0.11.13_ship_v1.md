# Checklist: v0.11.13 ship (Anthony polish pass included)

- [x] Confirm Anthony's improvements postdate 10:45 exe (toolbar 28px close, URL-hide regex, version footer, theme var aliases, paint_and_present y=0 fix) — zip from 10:48 is STALE, do not ship
- [x] faq.html v0.11.12 -> v0.11.13 (missed 4th site file; other three already flipped)
- [ ] Rebuild: cargo build --release --no-default-features --features servo-real
- [ ] Repack: scripts/package_release.sh (libservo guard must pass)
- [ ] Glass gates: scripts/run_glass_gates.ps1
- [ ] Grok 6-step: 84px chrome cold launch / theme flip x4 tabs / tab stress no vanish / amni-apps redirect / honest badge / close 28px hit
- [ ] Commit + push amni-browse as Amnibro (no Claude co-sign)
- [ ] gh release create v0.11.13 (Latest) with fresh zip + staged changelog notes
- [ ] Site: commit/deploy about.html, amni-browse.html, index.html, faq.html AFTER release asset live; verify deployed URL not local
- Deferred: webview stub .tab-close still 18px vs CLOSE_HITBOX=28 (non-shipping backend, packer refuses it; align next src window)
