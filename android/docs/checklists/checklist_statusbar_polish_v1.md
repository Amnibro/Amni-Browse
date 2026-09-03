# Checklist — status-bar overlap + home polish v1

targetSdk 35 forces edge-to-edge on Android 15+: root layout starts at y=0, 40dp tab strip
hides under the status bar. "Import log" = home ListView dumping title\nurl system rows after
a Chrome import. Concurrent-editor note: another session edited until 23:24; working via Edit
with fresh reads.

- [x] Backups (BrowseActivity.kt, activity_browse.xml)
- [x] Insets: root id + ViewCompat listener pads systemBars top/bottom
- [x] Home panel: reworded copy, Import/Default buttons tidied, BOOKMARKS section label,
      styled two-line rows (title + host), empty-state hint
- [x] paintTabs/paintBmBar/history px -> dp() (raw ints are PIXELS; Fold density ~3x)
- [x] Import toast shortened; stale private-tab toast corrected (profiles ARE separate now)
- [x] gradlew assembleDebug (JBR) -> adb install on phone -> visual check via screenshot
- [x] changelog + memory
