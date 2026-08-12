# Checklist: Settings + chrome 0.11.0 (ff18986)

- [x] Wire BrowserConfig + BookmarkManager into Servo path (were constructed then dropped)
- [x] menu -> real Settings page (search engine, homepage, shield, zoom, UA override, bookmarks)
- [x] shield cmd toggles config.block_ads (+trackers), persists, gates ad_blocker at request time
- [x] bookmark cmd toggles current URL (Ctrl+D too); star reflects state.bookmarked
- [x] new_tab/initial tab honor home_page; blank -> built-in start page w/ bookmark tiles
- [x] URL bar search honors config.search_engine prefix
- [x] default_zoom applied to new tabs
- [x] custom_user_agent -> ServoBuilder Preferences (restart applies)
- [x] SECURITY: amnibrowse:// cmd+state now require chrome webview id or per-boot token (was: any website could drive the browser + read tab state)
- [x] chrome 66px redesign (monogram favicons, live shield/star, tab strip scrolls without growing)
- [x] scripts/check_toolbar.js 10/10 at 0.11.0-settings
- [x] cargo check clean; commit ff18986 pushed (ls-remote verified)
- [ ] Release build + relaunch + live verify (settings save, shield flip, bookmark star, newtab)
- [ ] Scroll-at-bottom repro on DDG (task #4)
