# AmniBrowse Android v0 — daily driver (WebView)

Date: 2026-08-20  
Status: implemented v0 (2026-08-20) — session persist, import merge, Autofill, WebView-missing + retry; Servo still later  
Desktop: Amni-Browse 0.12.x (Rust, Servo + media WebView) stays as-is. Servo on Android is out of this spec.

## Goal

Ship a sideloadable Android APK that replaces Chrome as the phone browser: Amni-owned chrome (toolbar, tabs, home, theme), Android System WebView for pages, DuckDuckGo search, local bookmarks/history, one-shot import from this PC’s Chrome profile, passwords via Android Autofill + Google Password Manager. Google can change WebView; they cannot move our toolbar.

## Non-goals (v0)

- Servo / GeckoView / Chromium fork
- Extensions
- Copying or decrypting Google Password Manager or Chrome `Login Data`
- Chrome cookies, sessions, or open-tab steal
- Live sync with desktop AmniBrowse
- Play Store listing
- Rooted access to `com.android.chrome` on the phone

## Architecture

New module `Amni-Browse/android/` (Kotlin, Gradle, minSdk 26, targetSdk current stable). One process, one main activity.

```
BrowseActivity
  ├── ChromeBar (back, forward, URL, home, menu)
  ├── TabStrip or TabSheet (thumb-reachable)
  ├── WebView (system)
  └── AutofillManager (OS → Google Password Manager)
Local store: Room (bookmarks, history, settings, import cursor)
Import file: amni-chrome-import.json (no password fields)
Windows helper: scripts/export-chrome-amni.ps1
```

Intent filters: `http`, `https`, `VIEW`. Menu action opens Android default-apps / “open links” settings so the user can set AmniBrowse default.

## Components

| Piece | Role |
|---|---|
| `BrowseActivity` | Owns WebView, chrome, tab model |
| `TabStore` | In-memory tabs + last URL; persist last session list |
| `NavResolver` | Bare host → `https://`; else DDG `https://duckduckgo.com/?q=` |
| `Room` | Bookmarks tree, history rows, settings |
| `ImportParser` | Reads `amni-chrome-import.json`, merge by URL |
| `export-chrome-amni.ps1` | Reads Chrome `Default` Bookmarks + History on Windows |
| Themes | Desktop gold/dark tokens as Android XML / Compose colors |

Reuse desktop visual tokens (accent gold, dark bg) from `src/ui/theme.rs` / CSS vars, transcribed once into Android resources. Do not load the desktop `toolbar.html` inside WebView as the phone chrome; native chrome stays under our layout so Google cannot restyle it.

## Chrome import (Windows)

Source (this machine, Chrome profile **Default** unless the script is pointed at another profile dir):

- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\Bookmarks` (JSON)
- `%LOCALAPPDATA%\Google\Chrome\User Data\Default\History` (SQLite: `urls`, `visits`)

Copy History to a temp file before querying (Chrome locks the live DB).

Output `amni-chrome-import.json`:

```json
{
  "version": 1,
  "source": "chrome-windows",
  "profile": "Default",
  "exportedAt": "ISO-8601",
  "bookmarks": [{ "title": "", "url": "", "path": ["Bookmarks bar", "..."], "added": 0 }],
  "history": [{ "url": "", "title": "", "lastVisit": 0, "visitCount": 0 }]
}
```

`added` / `lastVisit` are Unix ms. History export cap: 50,000 most-recent `urls` rows by `last_visit_time`. No `password`, `cookie`, or `token` keys. Parser must reject files that contain those keys.

Phone: first-run card “Import from Chrome (PC)” → system file picker. Settings → Import can run again. Merge key = normalized URL; keep existing title if already present, update `lastVisit` if imported is newer. Never wipe local data on import.

## Passwords

WebView important-for-autofill + `AutofillManager`. User’s Android autofill service remains Google Password Manager (or whatever they already use). AmniBrowse does not store Google logins. Optional later: Amni vault as an Autofill service; not v0.

## Data flow

1. User runs `scripts/export-chrome-amni.ps1` on the PC → JSON on disk / USB / Drive.
2. Install APK (sideload).
3. First run: Amni home, empty local profile, import CTA.
4. Pick JSON → Room merge → bookmarks and history lists populate.
5. Navigate: URL bar → `NavResolver` → `WebView.loadUrl`.
6. History: record committed navigations (http/https only), skip `about:` and error pages.
7. Login fields: OS autofill sheet (Google).
8. Set default: menu → system default-browser screen.

## Error handling

- Chrome History locked / missing: script copies DB; if copy fails, export bookmarks only and print which part failed.
- Corrupt or wrong JSON: show the parse error, do not write Room.
- Reject import `version` ≠ 1 until a migrator exists.
- WebView missing / disabled: blocking screen with link to install/update Android System WebView.
- Import file huge: stream parse; history insert in batches; keep UI responsive.
- No network: chrome and local home still work; WebView shows the usual error page plus a retry in our bar.

## Security

- No backup of WebView cookies to cloud.
- Import JSON is user-held; do not upload it.
- `cleartextTrafficPermitted` false.
- Custom URL schemes from desktop (`amnibrowse://`) are not registered on Android in v0; home is an in-app destination.
- File picker for import only (`ACTION_OPEN_DOCUMENT`), no broad storage permission.

## Testing

- Unit: `NavResolver`, JSON parser (happy, extra password key rejected, merge-by-URL).
- Instrumented: one tab load `example.com`, back/forward, new tab, history row written.
- Manual: set as default, open an https link from Messages, autofill on a real Google-saved login, import a fixture JSON with a known bookmark.

## Ship shape

- Package `com.amniscient.browse` (confirm at build if the site already uses another id).
- Icon from `assets/amni-browse-icon.png`.
- Version aligned with desktop where practical (label `0.12.5-android.0` or `0.13.0-android` at first APK).
- Install notes in README: sideload, export script, Autofill = Google, set default.

## Later (explicitly after v0)

Servo (or GeckoView) as an engine behind the same chrome. Desktop profile sync. Amni vault autofill provider. Play distribution.
