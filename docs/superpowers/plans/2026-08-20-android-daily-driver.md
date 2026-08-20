# AmniBrowse Android v0 Implementation Plan

> **For agentic workers:** Inline execution in this session (user: autonomously build).

**Goal:** Sideloadable Android APK with Amni chrome, system WebView, Chrome PC import, Autofill.

**Architecture:** `Amni-Browse/android/` Kotlin app; `scripts/export-chrome-amni.ps1` writes `amni-chrome-import.json`.

**Tech Stack:** AGP 8.7, Kotlin 2.0, Room, WebView, minSdk 26.

## Global Constraints

- No password fields in import JSON; reject files that include them.
- No Servo on Android in v0.
- Zero comments / no blank lines in source (project rule).
- Package `com.amniscient.browse`.
