# Checklist — UI + version parity v0.11.0

- [x] Scan architecture_map.md
- [x] Backup platform/webview.rs + tokens.rs → backups/
- [x] Theme parity: chrome_init_js seeds live ThemeConfig colors + danger; __AMNI_SYNC_THEME
- [x] Tab strip survives leave-home: __AMNI_TAB_SEED + page-load resync + get_tabs
- [x] amni-apps → https://amni-scient.com (menu/ctx/command palette; panel list removed)
- [x] Host chrome height uses TOTAL_CHROME_H (was hardcoded 82, clipped bookmarks)
- [x] WindowBuilder.with_decorations(true) for OS title bar
- [x] Cargo.toml / APP_VERSION / UA / README / chromeRev → 0.11.0
- [x] cargo check --features webview (Finished; warnings only)
- [ ] release rebuild target/release/amni-browse.exe
- [x] amni-scient-site version strings + download link → 0.11.0 (index+faq were still 0.10.3; product page 0.11.0)
- [x] architecture_map.md + CHANGELOG update
- [ ] commit + push Amni-Browse; tag v0.11.0
- [x] home claims truth: third-party cookies (default) + AES-256-GCM
- [x] README Amni Apps → amni-scient.com (no local inventory table)
- [x] IPC AmniAppList/LaunchApp → NavigateTo https://amni-scient.com (no local inventory)
