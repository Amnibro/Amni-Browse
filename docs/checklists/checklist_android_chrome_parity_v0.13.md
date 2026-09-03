# Checklist: Android Chrome parity (2026-08-20, waves 1+2 shipped as 0.13.0 / 0.14.0)

## Shipped
- [x] Tab chips: close x, favicons, Chrome-tablet strip; private-tab badge
- [x] Tab switcher: "All tabs…" list (tap switch, long-press close)
- [x] Private tabs: no history recorded, never persisted to session, form-save off,
      cache+form data wiped when the last one closes. LIMIT: cookies still shared with
      normal tabs (true isolation needs a multi-process WebView profile - next)
- [x] Omnibox suggestions: live dropdown from bookmarks (starred) + history while typing
- [x] Pull-to-refresh (brass spinner, only at scrollTop)
- [x] Long-press context menu: open in new tab / copy link / share link / download image
- [x] Find in page, Desktop site toggle, Share page, Bookmark this page
- [x] Bookmarks manager (folder paths shown, tap open, long-press delete) + bookmarks bar
      (menu toggle, newest 20)
- [x] History: search box, tap to open, Clear all with confirm
- [x] Clear browsing data: history / cookies / cache multi-choice
- [x] Per-site JavaScript block (menu toggle, host set, applied at page start)
- [x] Text size 75/100/125/150, persisted
- [x] Dark pages (algorithmic darkening when WebView supports it), persisted
- [x] Print / Save as PDF (system print dialog)
- [x] Translate (Google Translate proxy of current page)
- [x] Reader mode (article/main text extraction into an amni-styled page)
- [x] Downloads via DownloadManager; external schemes handed to the OS
- [x] Import: Chrome/Edge/Brave Bookmarks JSON, Firefox JSON, Netscape HTML, v1 JSON
- [x] Auto-import: share-into-app + watched folder (SAF, per-file mtime dedupe)
- [x] VERIFIED on device: automated PC pipeline imported bm=43 hist=5114 (Chrome x2 + Edge)

## Still open for exact Chrome parity
- [x] True incognito cookie isolation (WebView MULTI_PROFILE, own cookie jar, wiped on last close)
- [x] Tab drag-reorder on the strip (long-press + drop); tab grid with last-seen thumbnails
- [x] Search-engine suggestions in omnibox (network autocomplete), engine picker (DDG default)
- [x] Bookmarks bar folders; edit bookmark dialog (title/url/folder, delete)
- [x] Per-site cookie controls (host set, wipe on page start; third-party cookies off always). LIMIT: WebView still applies Set-Cookie on the first request of a blocked host before wipe
- [x] Fullscreen video callbacks + picture-in-picture (custom view + onUserLeaveHint)

## 0.16.0-android.0
- [x] Private by default: new tabs are private; "New open tab" is the persisted lane
- [x] Tracker query strip on navigate (UTM, fbclid, gclid, …)
- [x] Signed release APK (not debug); site feed `browse/android-latest.json`
