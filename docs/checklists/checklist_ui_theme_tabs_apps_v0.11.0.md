# Checklist — UI theme/tabs/apps/version parity v0.11.0

- [x] Scan architecture_map.md
- [x] Backup platform/webview.rs, media_engine.rs, app_launcher.rs, config.rs, tokens.rs, toolbar.html
- [ ] Unify shadow-chrome CSS vars to SPA tokens (`--bg-primary`, `--surface`/`--bg-secondary`, `--text-primary`, `--accent`, …)
- [ ] MEDIA_UA + USER_AGENT pin to `CARGO_PKG_VERSION` (0.11.0)
- [ ] Amni Apps: no local inventory UI; launch always → https://amni-scient.com
- [ ] Tab seed survives leave-home (init + page-load + TabsUpdated)
- [ ] TOTAL_CHROME_H 110 webview path; servo toolbar 66 kept as host-owned path
- [ ] Home claims honest (telemetry/blocker/cookies/DDG/vault/local)
- [ ] cargo build --release (default webview)
- [ ] Cold-launch smoke + gate notes
- [ ] architecture_map + CHANGELOG
- [ ] commit + push main + tag v0.11.0
