# Checklist: theme + tabs on external pages v0.10.11

- [x] Backup `platform/webview.rs`
- [x] `chrome_init_js` consumes live ThemeConfig colors (CSS vars)
- [x] Shadow DOM tab strip + `__AMNI_SYNC_TABS` from host TabManager
- [x] `__AMNI_SYNC_THEME` for runtime theme apply without reload
- [x] Tab new/switch/close on external chrome navigates active tab URL
- [x] amni-apps → amni-scient.com (already fixed; re-verify)
- [ ] Build release / run.bat
- [ ] Dogfood: 3 domains × theme accents match
- [ ] Dogfood: multi-tab survives leave-home nav
- [ ] Update architecture_map.md + CHANGELOG
