# Checklist: v0.11.2 polish release (secchip/safebar theme tokens)

- [ ] Backup originals (webview.rs pre-change, Cargo.toml, README.md, toolbar.html)
- [ ] Verify uncommitted secchip/safebar color-mix patch compiles (cargo check)
- [ ] Bump Cargo.toml -> 0.11.2
- [ ] Bump README.md headings -> 0.11.2
- [ ] Bump toolbar.html chromeRev -> 0.11.2-settings
- [ ] cargo build --release
- [ ] Repack amni-browse-v0.11.2-win64.zip (new exe + toolbar over v0.11.1 DLL set)
- [ ] Update changelog.md + architecture_map.md
- [ ] Commit as Amnibro, tag v0.11.2, push
- [ ] gh release create v0.11.2 --latest with zip
- [ ] amni-scient-site: bump version strings + download links -> v0.11.2, push
