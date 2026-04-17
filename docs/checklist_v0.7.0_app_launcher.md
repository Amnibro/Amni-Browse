# Checklist v0.7.0 — Amni Apps Launcher + Desktop Shortcut

## Files to Modify
- [x] `src/net/ipc.rs` — Add `LaunchApp` and `AmniAppList` IPC messages
- [x] `src/app.rs` — Add hardcoded app registry + `handle_command` for `LaunchApp`
- [x] `src/ui/webview.rs` — Add "Amni Apps" panel, context menu entry, JS renderer
- [x] `src/ui/emoji.rs` — Add rocket/app emoji if needed

## Files to Create
- [x] `src/engine/app_launcher.rs` — App registry struct + process spawner (allowlist only)
- [x] `src/engine/mod.rs` — Re-export app_launcher module
- [x] `assets/amni-browse.ico` — Windows icon for taskbar/shortcut
- [x] `assets/amni-browse.svg` — SVG icon source
- [x] `create_shortcut.ps1` — PowerShell script to create desktop shortcut with icon

## Changes Summary
1. New `AmniApp` struct: id, name, description, emoji, launch_type (Bat/Cargo/Web)
2. `AMNI_APPS` static registry with all 10+ apps
3. `launch_app` IPC → validates against allowlist → `std::process::Command::new("cmd").args(["/C", bat_path])` 
4. New slide panel "Amni Apps" with app cards (emoji, name, desc, Launch button)
5. Context menu entry "Amni Apps" between Extensions and Profiles
6. Desktop shortcut via PowerShell with .ico pinnable icon
7. Zero user-supplied paths (Auron's security ruling)

## Testing
- [ ] `cargo check` passes
- [ ] `cargo build --release` succeeds
- [ ] App panel renders correctly
- [ ] Launch buttons spawn correct processes
- [ ] Web links navigate browser
- [ ] Desktop shortcut works
- [ ] Icon appears in taskbar when pinned
