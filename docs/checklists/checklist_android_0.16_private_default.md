# Checklist: Android 0.16.0 private-default + remaining parity

- [x] Backups `android/backups/*.v016-pre.bak`
- [x] Failing unit tests first (SearchEngine, TabOrder, BookmarkFolders, CookieHosts, SessionCodec, tracker strip)
- [x] Helpers + NavResolver + SessionCodec
- [x] BrowseActivity: private default, drag tabs, grid thumbs, suggest, folders, cookies, FS/PiP
- [x] `testDebugUnitTest` 28 passed
- [x] Signed `assembleRelease` (not debug)
- [x] Site APK + `browse/android-latest.json` + amni-browse.html CTA
- [x] CHANGELOG + architecture_map
- [ ] Phone sideload (no adb device this turn)
- [ ] Site deploy / Pages push
- [ ] Anthony confirms
