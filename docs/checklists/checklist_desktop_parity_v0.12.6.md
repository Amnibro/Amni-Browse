# v0.12.6 desktop chrome parity — input, stop, home, favicons

- [x] Backup originals → `backups/servo_real.rs.v0.12.5.bak`, `backups/toolbar.html.v0.12.5.bak`
- [x] Keyboard routing by focus, not cursor position (`kbd_in_chrome` + chrome `focusin/focusout` → `kbd` cmd)
- [x] Click on page clears chrome keyboard focus and blurs the omnibox
- [x] Ctrl+L / Ctrl+F claim keyboard focus synchronously (no race with the JS focus round-trip)
- [x] `stop` = `window.stop()` (was `reload()`); reload button ↔ ✕ while loading; Esc stops the load
- [x] Ctrl+wheel zoom over content
- [x] Home button + Alt+Home → `home` cmd → configured home page
- [x] Real favicons: `WebView::favicon()` → RGBA swizzle → ≤32px PNG → base64 data URL, cached per tab URL, invalidated on `notify_favicon_changed`
- [x] `scripts/check.cmd` compile gate (vsenv + GStreamer env) and `scripts/build_release.cmd`
- [x] `cargo check --release --features servo-real` → 0 errors
- [x] toolbar inline JS `node --check` clean
- [ ] Launch and eyeball: favicons on DDG/GitHub, Esc mid-load, omnibox typing with the pointer over the page, Ctrl+wheel, Home
- [ ] CHANGELOG + architecture_map entries
- [ ] Anthony confirms working
