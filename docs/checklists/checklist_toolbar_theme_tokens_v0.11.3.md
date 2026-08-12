# Checklist — toolbar.html full theme-token fidelity (v0.11.3)

- [x] Backup assets/chrome/toolbar.html -> backups/toolbar.html.v0.11.2.bak
- [x] `--font` var + html/body use it; applyTheme maps th.font_family
- [x] `--glow` var; url-wrap focus ring uses accent_glow instead of solid accent-dim
- [x] `--tab-active` / `--tab-inactive` vars wired into .tab / .tab.active; applyTheme maps them
- [x] chromeRev bump -> 0.11.3-theme-tokens
- [x] cargo check clean; release build clean; boot smoke v0.11.3 (session restore + navigate, 0 leaked processes)
- [x] Cargo.toml 0.11.3 + changelog + architecture_map
- [x] Commit 75e28f6 + tag v0.11.3 + push (Amnibro identity)
