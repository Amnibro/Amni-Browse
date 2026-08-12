# Checklist: close successor = visual strip neighbor (0.11.0)
- [x] `close_tab` uses `ordered_tabs()` for successor (next, else prev)
- [x] Unit: mid-group close lands on strip neighbor
- [x] Unit: last-in-strip close lands on prev neighbor
- [x] Active paint: fill + bottom accent (not group fill)
- [x] Group paint: label + left rail only
- [x] kbd-focus: `:focus-visible` + `.kbd-focus` only (no bare `:focus`)
- [x] Backup: `backups/tabs.rs.v0.11.0-close-successor.bak`
- [x] cargo test tabs; cargo check; release rebuild; zip repack
