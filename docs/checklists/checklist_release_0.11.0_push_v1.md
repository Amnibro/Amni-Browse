# Checklist — Amni-Browse release 0.11.0 push (2026-08-12)

- [x] Confirm Cargo.toml / README / chrome canary 0.11.0
- [x] Chrome: TOTAL_CHROME_H 110 + shadow toolbar + SPA tab/nav/bookmarks
- [x] amni-apps → https://amni-scient.com only
- [x] Theme sync home ↔ external (__AMNI_SYNC_THEME)
- [x] Host tab model survives nav (seed + get_tabs)
- [x] cargo build --release (amni-browse 0.11.0 binary; embeds __AMNI_SYNC_THEME + __amni_push_style + masthead push)
- [x] Commit + push main (`e5351a1` applyContentPush; `b09ee35` theme/apps/tabs)
- [x] GitHub release Latest = v0.11.0 + `amni-browse-v0.11.0-win64.zip` (v0.10.3 remains historical, not Latest)
- [x] Site index/browse/about/faq 0.11.0 commit + push (`e43da418`)
