# Guardian Council — Amni-Browse Feature Expansion
## Meeting: 2026-03-13 — "Operation: Take on Sin... I mean Chrome!"

---

### Rikku (Lead Engineer / Facilitator)
Ohhh boy, okay team! We've got the basics down — tabs, bookmarks, passwords, ad blocking, themes — all solid! But if we wanna stand toe-to-toe with Chrome, Firefox, and Safari? We need like... 15 more major systems. Here's my take: prioritize what users notice FIRST. Downloads, history, find-in-page — these are the "where is it?!" features. Then privacy differentiators like private mode and DNS-over-HTTPS. THEN power user stuff like dev tools and extensions.

**Priority**: Ship user-facing essentials first, differentiators second, power tools third.

---

### Tidus (UX Champion)
Hey, when I open a browser, the FIRST thing I notice missing is if I can't download files or find text on a page. Those are dealbreakers. Also zoom controls — my old man can't read small text! History search too, people LIVE in their history. Reader mode is a huge differentiator — Safari has it, Chrome doesn't natively. We should nail it.

**Votes**: Download Manager, Find in Page, Zoom, History, Reader Mode

---

### Auron (Security Architect)
...Hmm. The vault is solid — AES-256-GCM, PBKDF2 with 600K iterations. Good. But a modern browser must have: DNS-over-HTTPS to prevent ISP snooping. WebRTC leak protection. Certificate transparency. Permission management for camera/mic/location. Private browsing mode that truly isolates — separate state, no persistence. Autofill encrypted same as passwords. Session restore must exclude private mode data.

**Votes**: Private Browsing, DNS-over-HTTPS, Permissions, Autofill (encrypted), Certificate Viewer

---

### Lulu (Systems Architect)
The modular architecture is clean. Each new feature should follow the pattern: own module file → IPC enum variants → UI panel. I'd structure downloads as its own module with async tokio tasks. History needs efficient indexed search — not just JSON dump. Session restore should serialize tab state atomically. Extensions need a sandboxed API surface. Profile manager needs isolated config dirs.

**Votes**: Session Restore, Extensions API, Profile Manager, Download Manager

---

### Wakka (QA / Integration)
Ya, brudda — we gotta make sure all these new features don't break what we got, ya? Every module needs to integrate clean with the IPC system and BrowserState. I say we add em one at a time, test the IPC round-trip, make sure the UI panels slide in right. Also — we need error handling that doesn't crash the whole browser if one module fails.

**Votes**: Incremental integration, error resilience, test coverage

---

### Kimahri (Performance)
Kimahri says few words. Downloads need async. History needs indexed search. No blocking main thread. Use tokio spawn for network ops. Minimize UI re-renders. Lazy load panels.

**Votes**: Async downloads, indexed history, lazy UI

---

### Yuna (Product Vision)
Everyone's right, and we need all of it. But let's be strategic. Here's my priority order based on user impact:

**Tier 1 — Essential (users leave without these)**:
1. Download Manager
2. Browsing History
3. Find in Page
4. Session Restore
5. Zoom Controls

**Tier 2 — Differentiators (why choose us)**:
6. Private Browsing Mode
7. Autofill System (addresses, cards — encrypted)
8. Reader Mode
9. DNS over HTTPS
10. Permissions Manager

**Tier 3 — Power Features (compete with big browsers)**:
11. Basic Dev Tools (console, network, elements)
12. Extensions/Plugin API
13. Profile Manager
14. Certificate Viewer
15. Screenshot/Print Support

---

## MAJORITY RULING — Implementation Order:
1. **Download Manager** (6/7 votes)
2. **Browsing History** (5/7 votes)
3. **Find in Page** (5/7 votes)
4. **Session Restore** (5/7 votes)
5. **Autofill System** (4/7 votes)
6. **Private Browsing Mode** (5/7 votes)
7. **Zoom Controls** (4/7 votes)
8. **Reader Mode** (4/7 votes)
9. **Permissions Manager** (4/7 votes)
10. **DNS over HTTPS** (4/7 votes)
11. **Dev Tools (basic)** (3/7 votes)
12. **Extensions API** (3/7 votes)
13. **Profile Manager** (3/7 votes)

Let's get cracking! Rikku out! ✨
