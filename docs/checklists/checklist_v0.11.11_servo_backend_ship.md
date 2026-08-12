# Checklist — v0.11.11 servo backend ship (root-cause: webview exe shipped in 0.11.10 zip)

Root cause: `default = ["webview"]` + shared `target/release/amni-browse.exe` path.
A plain `cargo build --release` at 07:06 clobbered the servo-real exe; the 0.11.10 zip
packed the 9.3MB webview binary → `Backend: WebView (wry/tao)` → no chrome toolbar on
cold launch (Anthony's "no header bar"). Reproduced from live zip, smoke log proof.

- [x] Reproduce on cold-pulled live v0.11.10 zip (md5 match local; log shows WebView backend, zero toolbar-mount lines)
- [x] Root-cause backend selection (main.rs cfg gates; Cargo.toml default features)
- [x] Backup Cargo.toml → backups/Cargo.toml.v0.11.10.bak; bump 0.11.11
- [x] Rebuild --no-default-features --features servo-real (121836032 bytes, 2026-08-12 08:04)
- [x] Packer guard: scripts/package_release.sh asserts exe embeds "Real Servo (libservo)" and NOT "WebView (wry/tao)" before zipping
- [x] Repack v0.11.11 zip (DLL layout from v0.11.9 base, swapped 116MB servo-real exe + fresh assets) → amni-browse-v0.11.11-win64.zip ~102MB
- [x] Cold-launch packed exe from staging: Backend: Real Servo + toolbar mount (15601 bytes cwd assets) + chromeRev 0.11.11
- [ ] Tag v0.11.11, release sole Latest, demote 0.11.10
- [x] Site chip + download href → 0.11.11 (amni-scient-site already flipped)
- [ ] Anthony confirms header bar on his launch chain

## Five Anthony gates (cold zip, 2026-08-12)
1. Header bar paints (toolbar overlay mounted) — PASS
2. chromeRev / public version = 0.11.11 only in this binary — PASS
3. Amni Apps → https://amni-scient.com (IPC NavigateTo) — source/contract PASS
4. Theme: multi-tab data: HTML reload on setting_set theme — source PASS (in linked exe)
5. Tabs persist across new tab (renderTabs from state.tabs) — source PASS
