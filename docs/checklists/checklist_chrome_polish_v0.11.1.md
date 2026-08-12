# Checklist: chrome polish sweep v0.11.1 (2026-08-12)

Source: full-chrome audit (webview.rs + toolbar.html + theme.rs), 17 findings.

- [x] 1. Split View dead — add #split-content + #split-resize to DOM
- [x] 2. Command palette clicks close panels instantly — exclude #cmd-palette from doc-click guard
- [x] 3. Engine frame top:48px/z999 covers nav+bookmarks+panels — top:110px bottom:22px z:5
- [x] 4. Settings toggles keyboard-unreachable — role=switch, tabindex, Enter/Space
- [x] 5. Toggle knob hardcoded white, invisible in light themes — var(--text-primary)
- [x] 6. Ctrl+Shift+E advertised but unbound — bind fetch_page
- [x] 7. Menu can render off-screen — max-height + measured clamp
- [x] 8. No focus-visible on nav/ctx/bookmark/close/vault/find buttons — one shared rule
- [x] 9. BG image/opacity theme controls dead — wire into applyTheme + newtab ::before
- [x] 10. toolbar.html default palette mismatches amni_dark — seed :root with real tokens
- [x] 11. toolbar.html radii literals ignore --radius — derive from token
- [x] 12. Hardcoded white/#000 on themed surfaces — var(--bg-primary)
- [x] 13. Heavy black shadows on light themes — soften opacities
- [x] 14. Bookmark star never un-fills — true toggle via bookmark_remove + url set
- [x] 15. Ctrl+W throws on null tab record — guard
- [ ] 16. emoji CUSTOM registry inert — noted, skipped (zero user impact)
- [x] 17. Copy/dead-token cleanup: baked ddg URL, Settings→Menu title, unused emoji lets, tabs-container inline style dup
- [x] cargo build --release clean
- [x] changelog + architecture_map updated
- [x] commit + push (Amnibro identity, no co-sign)
