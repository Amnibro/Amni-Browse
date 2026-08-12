# Checklist v0.11.7 - custom-theme parity (dev hub == home SPA)
- [x] Backup developer.rs/webview.rs/Cargo.toml to backups/*.v0.11.7-pre.bak
- [x] Dev hub saveTheme: derive secondaries from BG via shade(), glow from accent (kills hardcoded cyan rgba(0,212,255,.15))
- [x] Both editors seed pickers from active theme; static defaults -> shell tokens #08090B/#EDEFF2
- [x] Gradient stops + font stack identical across both save paths
- [x] Cargo 0.11.7, chromeRev 0.11.7-theme-parity, cargo build --release clean
- [x] Smoke launch v0.11.7, zero strays
- [x] Zip packaged (170 entries), gh release Latest, v0.11.6 demoted to Pre-release
- [x] Site literals flipped AFTER asset live, live pages verified
