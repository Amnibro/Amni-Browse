# Checklist: Async Response Delivery Fix (v0.6.1)

## Problem
3 async IPC handlers (FetchPage, PageMetaReq, ReaderFetch) spawn tokio tasks that compute results but DROP them — no mechanism to deliver responses back to WebView SPA. Additionally, no tokio runtime exists so tokio::spawn would panic if triggered.

## Changes

- [x] **main.rs** — Create `tokio::runtime::Runtime`, call `rt.enter()` for spawn context
- [x] **app.rs struct** — Add `async_tx: Option<std::sync::mpsc::Sender<String>>` and `async_notify: Option<Arc<dyn Fn() + Send + Sync>>`
- [x] **app.rs constructor** — Init both fields to None (set by platform layer)
- [x] **app.rs FetchPage** — Clone tx+notify, send PageRendered response + wake proxy
- [x] **app.rs PageMetaReq** — Clone tx+notify, send PageMetaResp response + wake proxy
- [x] **app.rs ReaderFetch** — Clone tx+notify, send ReaderHtml response + wake proxy
- [x] **platform/webview.rs** — Create mpsc channel, set tx+notify on BrowserState, drain rx in UserEvent handler
- [x] **pipeline.rs** — Add CSS resource fetching (fetch linked stylesheets during parse), scoped DOM drop
- [x] **Build** — 0 errors, 59 warnings (dead code)
- [x] **Test** — Browser launched successfully (PID confirmed), tokio runtime active
- [x] **Docs** — CHANGELOG.md v0.6.1 entry, ARCHITECTURE.md data flow + box diagram updated
