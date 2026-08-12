# Checklist v0.11.10 — servo internal-page theme parity + theme picker

- [x] Backups: servo_real.rs / toolbar.html / Cargo.toml → backups/*.v0.11.9.bak
- [x] NEWTAB_TPL + SETTINGS_TPL palettes driven by active theme (`__THEME__` root vars), no hardcoded gold/dark hexes
- [x] Settings page gains Theme section (radio chips, all built-in + custom themes, active checked)
- [x] `setting_set` handles `theme` key → ThemeConfig::set_theme + instant settings-page re-render; toolbar follows via 250ms state poll
- [x] toolbar.html `chromeRev` → `__CHROMEREV__` placeholder, injected with CARGO_PKG_VERSION in chrome_data_url()
- [x] Cargo.toml → 0.11.10
- [x] cargo build --release --no-default-features --features servo-real (0 errors)
- [x] Derivation canaries on built pages: 0 hardcoded `#C89B4E`/`#08090B` in rendered newtab/settings when non-default theme active
- [x] Zip repack (forward-slash entries), tag v0.11.10, gh release sole Latest, site chips + download href
- [x] architecture_map.md + changelog.md updated
