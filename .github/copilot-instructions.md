# Amni Browse — Copilot Instructions

- This is a Rust project using `wry` (webview) and `tao` (windowing).
- Privacy-first design: no telemetry, no tracking, no external analytics.
- Modular architecture: tabs, ad-blocking, bookmarks, networking, UI are separate modules.
- Use `cargo build` / `cargo run` to build and launch.
- All user data (bookmarks, settings) stored locally in the user's config directory.
- Ad/tracker blocking uses a built-in filter list (no external dependencies at runtime).
