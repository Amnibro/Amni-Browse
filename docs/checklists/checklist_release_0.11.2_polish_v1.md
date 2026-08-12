# Checklist: v0.11.2 polish release (secchip/safebar theme tokens)

- [x] Backup originals (webview.rs pre-change, Cargo.toml, README.md, toolbar.html)
- [x] Verify uncommitted secchip/safebar color-mix patch compiles (cargo check)
- [x] Bump Cargo.toml -> 0.11.2
- [x] Bump README.md headings -> 0.11.2
- [x] Bump toolbar.html chromeRev -> 0.11.2-settings
- [x] cargo build --release
- [x] Repack amni-browse-v0.11.2-win64.zip (new exe + toolbar over v0.11.1 DLL set)
- [x] Update changelog.md + architecture_map.md
- [x] Commit as Amnibro, tag v0.11.2, push
- [x] gh release create v0.11.2 --latest with zip
- [x] amni-scient-site: bump version strings + download links -> v0.11.2, push
