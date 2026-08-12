# Checklist — tab strip mouse polish v0.11.12

- [x] Scan architecture_map.md
- [x] Backup assets/chrome/toolbar.html -> backups/toolbar.html.v0.11.11.bak
- [x] Wheel over tab strip scrolls it horizontally (scrollbar is hidden; overflow was unreachable by mouse)
- [x] Double-click empty strip area -> new tab
- [x] Middle-click empty strip area -> new tab (tab close behavior preserved; + button excluded)
- [x] node --check on inline JS (JS_OK)
- [x] Bump Cargo.toml 0.11.11 -> 0.11.12
- [ ] servo-real rebuild (build_servo_real.bat) completes EXITCODE=0
- [ ] Repack zip via ship-guard packer (refuses webview exe)
- [ ] Cold-extract smoke: 5 gates + 3 new mouse affordances
- [ ] Publish v0.11.12 as sole Latest; flip site chips
- [ ] Update architecture_map.md + changelog.md
- [ ] Anthony confirms
