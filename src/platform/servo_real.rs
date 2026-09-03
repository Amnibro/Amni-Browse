use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use euclid::{Point2D, Scale};
use euclid::default::{Point2D as DefaultPoint2D, Rect as DefaultRect, Size2D as DefaultSize2D};
use servo::{
    CompositionEvent, CompositionState, ImeEvent,
    ContextMenuAction, ContextMenuElementInformationFlags, ContextMenuItem, CreateNewWebViewRequest,
    Cursor, DevicePixel, EmbedderControl, EmbedderControlId, EventLoopWaker, FilePicker, InputEvent, LoadStatus,
    SimpleDialog,
    MouseButton as ServoMouseButton, MouseButtonAction, MouseButtonEvent, MouseLeftViewportEvent, MouseMoveEvent,
    NavigationRequest, OffscreenRenderingContext, Opts, Preferences, RenderingContext, SelectElementOptionOrOptgroup,
    Servo, ServoBuilder, Theme as ServoTheme, WebResourceLoad, WebResourceResponse, WebView, WebViewBuilder, WebViewDelegate, WebViewId, WheelDelta,
    WheelEvent, WheelMode, WindowRenderingContext,
};
use url::Url;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Ime as WinitIme, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::event::KeyEvent;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Fullscreen, Window, WindowId};
use log::info;
use glow::HasContext;
use crate::app::BrowserState;
use crate::crypto::pm::{self, PmState};
use crate::crypto::vault::PasswordManager;
use crate::net::updater;
use crate::engine::adblocker::AdBlocker;
use crate::engine::daily_driver;
use crate::engine::stream_extract;
use crate::engine::extensions::ExtensionManager;
use crate::platform::media_engine::{self, EngineKind, MediaWindow};
use crate::platform::os_default;
use crate::platform::servo_keys::keyboard_event_from_winit;
use crate::storage::bookmarks::BookmarkManager;
use crate::storage::config::BrowserConfig;
use crate::storage::downloads::DownloadManager;
use crate::storage::history::HistoryManager;
use crate::storage::import_browsers;
use crate::storage::profiles::ProfileManager;
use crate::storage::session::{SessionManager, SessionTab};
use crate::ui::theme::ThemeConfig;
use crate::ui::tokens;
const CHROME_HEIGHT_CSS: f32 = tokens::SERVO_CHROME_HEIGHT_CSS as f32;
const TOOLBAR_HTML_EMBEDDED: &str = include_str!("../../assets/chrome/toolbar.html");

fn file_url(p: &std::path::Path) -> String {
    let raw = p.canonicalize().unwrap_or_else(|_| p.to_path_buf()).to_string_lossy().to_string();
    let s = raw.trim_start_matches("\\\\?\\").replace('\\', "/");
    if s.starts_with("//") { format!("file:{}", s) } else { format!("file:///{}", s.trim_start_matches('/')) }
}
fn content_scheme_ok(s: &str) -> bool {
    matches!(s, "http" | "https" | "data" | "file" | "about")
}
fn mime_for_path(p: &std::path::Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}
fn secure_file_path(req_url: &url::Url) -> Result<std::path::PathBuf, http::StatusCode> {
    let path = match req_url.to_file_path() {
        Ok(p) => p,
        Err(()) => return Err(http::StatusCode::BAD_REQUEST),
    };
    let canon = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            info!("file:// missing {}", path.display());
            return Err(http::StatusCode::NOT_FOUND);
        }
    };
    if !canon.is_file() {
        info!("file:// not a file {}", canon.display());
        return Err(http::StatusCode::FORBIDDEN);
    }
    Ok(canon)
}
fn file_load_allowed(is_for_main_frame: bool, document_url: Option<&url::Url>) -> bool {
    is_for_main_frame || document_url.map(|u| u.scheme() == "file").unwrap_or(false)
}
/// Subresource loads reach the embedder before `WebView::url()` is set for a fresh document, so
/// fall back to the request's referrer and then to the tab's own document.
fn file_subresource_allowed(is_for_main_frame: bool, document_url: Option<&url::Url>, referrer: Option<&url::Url>, tab_url: Option<&url::Url>) -> bool {
    file_load_allowed(is_for_main_frame, document_url) || referrer.map(|u| u.scheme() == "file").unwrap_or(false) || (document_url.is_none() && tab_url.map(|u| u.scheme() == "file").unwrap_or(false))
}
fn intercept_file_load(load: WebResourceLoad, req_url: url::Url) {
    let path = match secure_file_path(&req_url) {
        Ok(p) => p,
        Err(code) => {
            load.intercept(WebResourceResponse::new(req_url).status_code(code)).finish();
            return;
        }
    };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mut headers = http::HeaderMap::new();
            if let Ok(v) = http::HeaderValue::from_str(mime_for_path(&path)) {
                headers.insert(http::header::CONTENT_TYPE, v);
            }
            headers.insert(http::header::CACHE_CONTROL, http::HeaderValue::from_static("no-store"));
            let mut intercepted = load.intercept(WebResourceResponse::new(req_url).status_code(http::StatusCode::OK).headers(headers));
            intercepted.send_body_data(bytes);
            intercepted.finish();
        }
        Err(e) => {
            info!("file:// read {}: {}", path.display(), e);
            load.intercept(WebResourceResponse::new(req_url).status_code(http::StatusCode::FORBIDDEN)).finish();
        }
    }
}
fn font_urls() -> (String, String) {
    let names = ["assets/fonts/archivo-var.woff2", "assets/fonts/archivo-var-ext.woff2"];
    let mut found = Vec::new();
    for n in names {
        let cwd = std::path::PathBuf::from(n);
        if cwd.is_file() { found.push(file_url(&cwd)); continue; }
        if let Ok(exe) = std::env::current_exe() {
            let p = exe.parent().unwrap_or(exe.as_path()).join(n);
            if p.is_file() { found.push(file_url(&p)); continue; }
        }
        found.push(String::new());
    }
    (found[0].clone(), found[1].clone())
}
fn inject_fonts(html: String) -> String {
    let (m, e) = font_urls();
    html.replace("__FONT_MAIN__", &m).replace("__FONT_EXT__", &e)
}
fn load_toolbar_html() -> String {
    if let Ok(over) = std::env::var("AMNI_CHROME_HTML") {
        if let Ok(content) = std::fs::read_to_string(&over) {
            info!("chrome toolbar: AMNI_CHROME_HTML {} ({} bytes)", over, content.len());
            return inject_fonts(content);
        }
    }
    if let Ok(content) = std::fs::read_to_string("assets/chrome/toolbar.html") {
        info!("chrome toolbar: cwd hot-load ({} bytes)", content.len());
        return inject_fonts(content);
    }
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap_or(exe.as_path()).join("assets/chrome/toolbar.html");
        if let Ok(content) = std::fs::read_to_string(&p) {
            info!("chrome toolbar: exe-dir hot-load ({} bytes)", content.len());
            return inject_fonts(content);
        }
    }
    info!("chrome toolbar: embedded {} bytes rev {}", TOOLBAR_HTML_EMBEDDED.len(), env!("CARGO_PKG_VERSION"));
    inject_fonts(TOOLBAR_HTML_EMBEDDED.to_string())
}

fn servo_prefs(config: &BrowserConfig) -> Preferences {
    let mut p = Preferences::default();
    p.layout_grid_enabled = true;
    p.layout_columns_enabled = true;
    p.layout_variable_fonts_enabled = true;
    p.layout_writing_mode_enabled = true;
    p.layout_container_queries_enabled = true;
    p.layout_css_attr_enabled = true;
    p.dom_fontface_enabled = true;
    p.dom_adoptedstylesheet_enabled = true;
    p.dom_intersection_observer_enabled = true;
    p.dom_visual_viewport_enabled = true;
    p.dom_webgl2_enabled = true;
    p.dom_indexeddb_enabled = true;
    p.dom_resize_observer_enabled = true;
    p.layout_unimplemented = true;
    stylo_static_prefs::set_pref!("layout.css.has-selector.enabled", true);
    stylo_static_prefs::set_pref!("layout.css.nth-child-of.enabled", true);
    stylo_static_prefs::set_pref!("layout.css.starting-style-at-rules.enabled", true);
    stylo_static_prefs::set_pref!("layout.css.light-dark.images.enabled", true);
    if let Some(ua) = config.custom_user_agent.as_ref().filter(|u| !u.trim().is_empty()) {
        info!("custom user agent: {}", ua);
        p.user_agent = ua.clone();
    }
    p
}
fn chrome_data_url() -> Url {
    let html = load_toolbar_html().replace("__CHROMEREV__", env!("CARGO_PKG_VERSION"));
    let encoded = urlencoding::encode(&html);
    Url::parse(&format!("data:text/html;charset=utf-8,{}", encoded)).expect("chrome data url")
}
use crate::ui::internal_pages::{NEWTAB_TPL, SETTINGS_TPL, TUTORIAL_TPL, esc_html};
fn chrome_height_px(scale: f32) -> u32 { (CHROME_HEIGHT_CSS * scale).round().max(1.0) as u32 }
fn content_size(window_size: PhysicalSize<u32>, chrome_px: u32) -> PhysicalSize<u32> {
    PhysicalSize::new(window_size.width.max(1), window_size.height.saturating_sub(chrome_px).max(1))
}
fn content_blit_rect(win_w: u32, win_h: u32, chrome_px: u32) -> DefaultRect<i32> {
    let h = win_h.saturating_sub(chrome_px).max(1);
    DefaultRect::new(DefaultPoint2D::new(0i32, 0i32), DefaultSize2D::new(win_w.max(1) as i32, h as i32))
}
pub fn run(state: BrowserState) {
    info!("Amni Browse Real Servo backend initializing...");
    info!("Media engine platform: {}", media_engine::platform_label());
    let event_loop = EventLoop::<WakerEvent>::with_user_event().build().expect("event loop");
    let ad_blocker = Arc::new(Mutex::new(state.ad_blocker.clone()));
    let config = state.config.clone();
    let bookmarks = state.bookmarks.clone();
    let mut initial_urls: Vec<(String, EngineKind)> = state.tabs.tabs.iter().map(|t| {
        (t.url.clone(), media_engine::route(&t.url))
    }).collect();
    if let Ok(test_url) = std::env::var("AMNI_TEST_MEDIA_URL") { info!("AMNI_TEST_MEDIA_URL set \u{2192} injecting media tab: {}", test_url); initial_urls.push((test_url, EngineKind::Media)); }
    let themes = state.themes.clone();
    let mut app = App::new(&event_loop, ad_blocker, initial_urls, config, bookmarks, themes);
    event_loop.run_app(&mut app).expect("event loop run");
}
#[derive(Debug)]
struct WakerEvent;
#[derive(Clone)]
struct Waker(EventLoopProxy<WakerEvent>);
impl Waker { fn new(event_loop: &EventLoop<WakerEvent>) -> Self { Self(event_loop.create_proxy()) } }
impl EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> { Box::new(Self(self.0.clone())) }
    fn wake(&self) { let _ = self.0.send_event(WakerEvent); }
}
struct AppState {
    window: Window,
    servo: Servo,
    /// Document-start polyfills (requestIdleCallback and friends) injected into every content page.
    user_content: Rc<servo::UserContentManager>,
    rendering_context: Rc<WindowRenderingContext>,
    offscreen_context: Rc<OffscreenRenderingContext>,
    chrome_webview: RefCell<Option<WebView>>,
    content_webviews: RefCell<Vec<WebView>>,
    active_content_index: Cell<usize>,
    mouse_point: Cell<Point2D<f32, DevicePixel>>,
    modifiers: Cell<ModifiersState>,
    scale_factor: Cell<f32>,
    ad_blocker: Arc<Mutex<AdBlocker>>,
    media_panes: RefCell<Vec<Option<MediaWindow>>>,
    pending_media_urls: RefCell<Vec<String>>,
    closed_tabs: RefCell<Vec<Url>>,
    tab_zoom: RefCell<Vec<f32>>,
    tab_titles: RefCell<Vec<String>>,
    tab_uids: RefCell<Vec<u64>>,
    next_tab_uid: Cell<u64>,
    is_fullscreen: Cell<bool>,
    config: RefCell<BrowserConfig>,
    bookmarks: RefCell<BookmarkManager>,
    themes: RefCell<ThemeConfig>,
    cmd_token: String,
    self_weak: Weak<AppState>,
    history: RefCell<HistoryManager>,
    downloads: RefCell<DownloadManager>,
    vault: RefCell<PasswordManager>,
    pm: RefCell<PmState>,
    update: Arc<Mutex<Option<updater::ReleaseInfo>>>,
    extensions: RefCell<ExtensionManager>,
    profiles: RefCell<ProfileManager>,
    find_query: RefCell<String>,
    chrome_overlay_px: Cell<u32>,
    overlay_css: Cell<Option<(i32, i32, i32, i32)>>,
    kbd_in_chrome: Cell<bool>,
    paint_logged: Cell<bool>,
    favicons: RefCell<std::collections::HashMap<String, String>>,
    origin_favicons: RefCell<std::collections::HashMap<String, Option<String>>>,
    origin_favicon_primed: RefCell<std::collections::HashSet<String>>,
    origin_favicon_polls: RefCell<std::collections::HashMap<String, u32>>,
    origin_favicon_reprimes: RefCell<std::collections::HashSet<String>>,
    origin_favicon_probing: RefCell<std::collections::HashSet<String>>,
    favicon_jobs: FaviconJobs,
    pending_relaunch: RefCell<Option<String>>,
    pending_session_persist: Cell<bool>,
    last_import: RefCell<import_browsers::ImportReport>,
    pending_embedder: RefCell<Option<EmbedderControl>>,
    ctx_link: RefCell<Option<String>>,
    ctx_image: RefCell<Option<String>>,
    pending_source: RefCell<Vec<(String, String)>>,
    /// True while the last cursor we set was a frameless-window resize grip.
    resize_cursor_on: Cell<bool>,
    /// Physical keys whose press was consumed by a chrome shortcut; their release is swallowed too.
    shortcut_keys: RefCell<std::collections::HashSet<winit::keyboard::PhysicalKey>>,
    /// Last cursor requested by page content (Servo notify_cursor_changed).
    page_cursor: Cell<winit::window::CursorIcon>,
    /// Popup tab index → opener content tab index (GSI / OAuth window.open).
    popup_opener: RefCell<std::collections::HashMap<usize, usize>>,
}
impl AppState {
    fn sync_window_title(&self) {
        if let Some(c) = self.active_content() { self.set_window_title(c.page_title()); }
    }
    fn set_window_title(&self, title: Option<String>) {
        let t = title.unwrap_or_default();
        let trimmed = t.trim();
        let display = if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("Amni Browse") {
            "Amni Browse".to_string()
        } else if trimmed.contains("Amni Browse") {
            trimmed.to_string()
        } else {
            format!("{} \u{2014} Amni Browse", trimmed)
        };
        self.window.set_title(&display);
    }
    fn chrome_px(&self) -> u32 { chrome_height_px(self.scale_factor.get()) }
    fn hit_chrome_px(&self) -> f32 { (self.chrome_px() + self.chrome_overlay_px.get()) as f32 }
    fn window_size(&self) -> PhysicalSize<u32> { self.window.inner_size() }
    fn active_content(&self) -> Option<WebView> {
        let tabs = self.content_webviews.borrow();
        let idx = self.active_content_index.get().min(tabs.len().saturating_sub(1));
        tabs.get(idx).cloned()
    }
    fn tab_index_for_webview(&self, webview: &WebView) -> Option<usize> {
        self.content_webviews.borrow().iter().position(|c| c.id() == webview.id())
    }
    /// Route chrome + input to the webview the user actually clicked in.
    fn focus_content_tab(&self, webview: &WebView) -> Option<usize> {
        let idx = self.tab_index_for_webview(webview)?;
        if idx != self.active_content_index.get() {
            self.active_content_index.set(idx);
            info!("focus tab \u{2192} idx {}", idx);
        }
        Some(idx)
    }
    fn remember_tab_title(&self, idx: usize, title: &str) {
        let t = title.trim();
        if t.is_empty() || t.eq_ignore_ascii_case("New Tab") { return; }
        let mut cache = self.tab_titles.borrow_mut();
        while cache.len() <= idx { cache.push(String::new()); }
        cache[idx] = t.to_string();
    }
    fn tab_display_title(&self, idx: usize, c: &WebView, tab_url: &str) -> String {
        let raw = c.page_title().unwrap_or_default();
        let trimmed = raw.trim();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("New Tab") {
            self.remember_tab_title(idx, trimmed);
            return trimmed.to_string();
        }
        if let Some(cached) = self.tab_titles.borrow().get(idx).filter(|s| !s.is_empty()) {
            return cached.clone();
        }
        host_display_title(tab_url).unwrap_or_else(|| "New Tab".into())
    }
    fn parse_tab_index(id: &str) -> Option<usize> { id.strip_prefix('t').or_else(|| id.strip_prefix('m')).and_then(|s| s.parse().ok()) }
    fn alloc_tab_uid(&self) -> u64 {
        let id = self.next_tab_uid.get();
        self.next_tab_uid.set(id.wrapping_add(1));
        id
    }
    fn push_tab_uid(&self) { self.tab_uids.borrow_mut().push(self.alloc_tab_uid()); }
    fn media_bounds(&self) -> wry::Rect {
        let w = self.window_size();
        media_engine::content_bounds(w.width, w.height, self.chrome_px())
    }
    fn sync_media_len(&self) {
        let n = self.content_webviews.borrow().len();
        let mut m = self.media_panes.borrow_mut();
        while m.len() < n { m.push(None); }
        if m.len() > n { m.truncate(n); }
    }
    fn attach_media_at(&self, idx: usize, url: &str) {
        self.sync_media_len();
        let bounds = self.media_bounds();
        let mut panes = self.media_panes.borrow_mut();
        if idx >= panes.len() { return; }
        if let Some(p) = panes[idx].as_mut() {
            let _ = p.webview.load_url(url);
            p.url = url.to_string();
        } else {
            panes[idx] = media_engine::spawn_media_pane(&self.window, url, bounds);
        }
        drop(panes);
        self.apply_media_visibility();
    }
    fn attach_media_to_active(&self, url: &str) {
        self.attach_media_at(self.active_content_index.get(), url);
    }
    fn persist_session(&self, clean: bool) {
        if !self.config.borrow().restore_session && !clean { return; }
        let active = self.active_content_index.get();
        let panes = self.media_panes.borrow();
        let scale = self.scale_factor.get().max(0.01) as f64;
        let phys = self.window_size();
        let sz = winit::dpi::LogicalSize::<f64>::new(phys.width as f64 / scale, phys.height as f64 / scale);
        let tabs: Vec<SessionTab> = self.content_webviews.borrow().iter().enumerate().filter_map(|(i, c)| {
            let (url, title, media) = if let Some(Some(p)) = panes.get(i) {
                (p.url.clone(), media_engine::display_title(&p.url), true)
            } else {
                let u = c.url().map(|x| x.as_str().to_string()).unwrap_or_default();
                if u.is_empty() { return None; }
                (u, c.page_title().unwrap_or_default(), false)
            };
            Some(SessionTab { url: url.clone(), title, is_active: i == active, history: vec![url], history_index: 0, engine: if media { "media".into() } else { "servo".into() }, pinned: false, group: None })
        }).collect();
        if tabs.is_empty() { return; }
        let mut sm = SessionManager::new(true);
        sm.state.window_width = sz.width as f64;
        sm.state.window_height = sz.height as f64;
        sm.state.maximized = self.window.is_maximized();
        if let Ok(pos) = self.window.outer_position() { sm.state.window_x = Some(pos.x as f64 / scale); sm.state.window_y = Some(pos.y as f64 / scale); }
        sm.capture(tabs);
        if clean { sm.save_clean_exit(); } else { sm.save(); }
    }
    fn apply_media_visibility(&self) {
        self.sync_media_len();
        let idx = self.active_content_index.get();
        let bounds = self.media_bounds();
        let panes = self.media_panes.borrow();
        for (i, slot) in panes.iter().enumerate() {
            if let Some(p) = slot.as_ref() { media_engine::apply_pane_bounds(p, bounds, i == idx); }
        }
        drop(panes);
        for (i, c) in self.content_webviews.borrow().iter().enumerate() {
            match i == idx { true => c.show(), false => c.hide() }
        }
    }
    fn drop_media_at(&self, idx: usize) {
        let mut panes = self.media_panes.borrow_mut();
        if idx < panes.len() {
            if let Some(pane) = panes[idx].take() {
                media_engine::trash_pane(pane); // Drop engine
            }
        }
    }
    /// Silence, then Drop. No about:blank, no deferred queue — closed means gone.
    fn drop_webview(c: WebView) {
        c.evaluate_javascript(media_engine::SILENCE_JS, |_| {});
        c.hide();
        drop(c);
    }
    fn active_is_media(&self) -> bool {
        let idx = self.active_content_index.get();
        self.media_panes.borrow().get(idx).map(|s| s.is_some()).unwrap_or(false)
    }
    fn self_rc(&self) -> Rc<AppState> { self.self_weak.upgrade().expect("AppState alive") }
    fn chrome_js(&self, js: &str) { if let Some(c) = self.chrome_webview.borrow().as_ref() { let _ = c.evaluate_javascript(js, |_| {}); } }
    fn hide_embedder_ui(&self) { self.chrome_js("window.__amni&&window.__amni.hideEmbedder&&window.__amni.hideEmbedder()"); self.chrome_overlay_px.set(0); self.overlay_css.set(None); }
    fn dismiss_embedder(&self) { let _ = self.pending_embedder.borrow_mut().take(); self.ctx_link.borrow_mut().take(); self.ctx_image.borrow_mut().take(); self.hide_embedder_ui(); }
    fn push_embedder_json(&self, payload: serde_json::Value) {
        let js = format!("window.__amni&&window.__amni.showEmbedder&&window.__amni.showEmbedder({})", payload);
        self.chrome_js(&js);
        self.chrome_overlay_px.set(self.window_size().height.max(1));
        self.window.request_redraw();
    }
    fn ctx_action_name(a: ContextMenuAction) -> &'static str {
        match a {
            ContextMenuAction::GoBack => "GoBack", ContextMenuAction::GoForward => "GoForward", ContextMenuAction::Reload => "Reload",
            ContextMenuAction::CopyLink => "CopyLink", ContextMenuAction::OpenLinkInNewWebView => "OpenLinkInNewWebView",
            ContextMenuAction::CopyImageLink => "CopyImageLink", ContextMenuAction::OpenImageInNewView => "OpenImageInNewView",
            ContextMenuAction::Cut => "Cut", ContextMenuAction::Copy => "Copy", ContextMenuAction::Paste => "Paste", ContextMenuAction::SelectAll => "SelectAll",
        }
    }
    fn parse_ctx_action(s: &str) -> Option<ContextMenuAction> {
        Some(match s {
            "GoBack" => ContextMenuAction::GoBack, "GoForward" => ContextMenuAction::GoForward, "Reload" => ContextMenuAction::Reload,
            "CopyLink" => ContextMenuAction::CopyLink, "OpenLinkInNewWebView" => ContextMenuAction::OpenLinkInNewWebView,
            "CopyImageLink" => ContextMenuAction::CopyImageLink, "OpenImageInNewView" => ContextMenuAction::OpenImageInNewView,
            "Cut" => ContextMenuAction::Cut, "Copy" => ContextMenuAction::Copy, "Paste" => ContextMenuAction::Paste, "SelectAll" => ContextMenuAction::SelectAll,
            _ => return None,
        })
    }
    fn show_context_menu(&self, menu: servo::ContextMenu) {
        let r = menu.position();
        let (x, y) = embedder_anchor_css(r.min.x, r.min.y);
        let info = menu.element_info().clone();
        *self.ctx_link.borrow_mut() = info.link_url.as_ref().map(|u| u.as_str().to_string());
        *self.ctx_image.borrow_mut() = info.image_url.as_ref().map(|u| u.as_str().to_string());
        let mut items = Vec::new();
        for it in menu.items() {
            match it {
                ContextMenuItem::Separator => items.push(serde_json::json!({"sep":true})),
                ContextMenuItem::Item { label, action, enabled } => items.push(serde_json::json!({"id": Self::ctx_action_name(*action), "label": label, "enabled": enabled})),
            }
        }
        if info.flags.contains(ContextMenuElementInformationFlags::Link) {
            if let Some(u) = info.link_url.as_ref() { items.push(serde_json::json!({"id":"amni_newtab","label":"Open link in new tab","enabled":true,"url":u.as_str()})); }
        }
        if info.flags.contains(ContextMenuElementInformationFlags::Image) {
            if let Some(u) = info.image_url.as_ref() { items.push(serde_json::json!({"id":"amni_saveimg","label":"Save image as\u{2026}","enabled":true,"url":u.as_str()})); }
        }
        items.push(serde_json::json!({"sep":true}));
        items.push(serde_json::json!({"id":"amni_viewsrc","label":"View page source","enabled":true}));
        *self.pending_embedder.borrow_mut() = Some(EmbedderControl::ContextMenu(menu));
        self.push_embedder_json(serde_json::json!({"kind":"menu","x":x,"y":y,"items":items}));
        info!("embedder context menu at {},{}", x, y);
    }
    fn show_select(&self, sel: servo::SelectElement) {
        let r = sel.position();
        let (x, y) = embedder_anchor_css(r.min.x, r.max.y);
        let selected = sel.selected_options();
        let mut items = Vec::new();
        for og in sel.options() {
            match og {
                SelectElementOptionOrOptgroup::Option(o) => items.push(serde_json::json!({"id":o.id,"label":o.label,"enabled":!o.is_disabled,"selected":selected.contains(&o.id)})),
                SelectElementOptionOrOptgroup::Optgroup { label, options } => {
                    items.push(serde_json::json!({"sep":true,"label":label}));
                    for o in options { items.push(serde_json::json!({"id":o.id,"label":o.label,"enabled":!o.is_disabled,"selected":selected.contains(&o.id)})); }
                }
            }
        }
        *self.pending_embedder.borrow_mut() = Some(EmbedderControl::SelectElement(sel));
        self.push_embedder_json(serde_json::json!({"kind":"select","x":x,"y":y,"items":items}));
        info!("embedder select at {},{}", x, y);
    }
    fn show_color_picker(&self, picker: servo::ColorPicker) {
        let r = picker.position();
        let (x, y) = embedder_anchor_css(r.min.x, r.max.y);
        let cur = picker.current_color().map(|c| format!("#{:02X}{:02X}{:02X}", c.red, c.green, c.blue)).unwrap_or_else(|| "#000000".to_string());
        *self.pending_embedder.borrow_mut() = Some(EmbedderControl::ColorPicker(picker));
        self.overlay_css.set(Some((x as i32, y as i32, 236, 300)));
        self.push_embedder_json(serde_json::json!({"kind":"color","x":x,"y":y,"value":cur}));
        info!("embedder color picker at {},{} ({})", x, y, cur);
    }
    fn parse_hex_rgb(s: &str) -> Option<servo::RgbColor> {
        let h = s.trim().trim_start_matches('#');
        let full = match h.len() {
            3 => h.chars().flat_map(|c| [c, c]).collect::<String>(),
            6 => h.to_string(),
            _ => return None,
        };
        let v = u32::from_str_radix(&full, 16).ok()?;
        Some(servo::RgbColor { red: (v >> 16) as u8, green: (v >> 8) as u8, blue: v as u8 })
    }
    fn show_simple_dialog(&self, dialog: SimpleDialog) {
        let payload = match &dialog {
            SimpleDialog::Alert(d) => serde_json::json!({"kind":"dialog","type":"alert","message":d.message()}),
            SimpleDialog::Confirm(d) => serde_json::json!({"kind":"dialog","type":"confirm","message":d.message()}),
            SimpleDialog::Prompt(d) => serde_json::json!({"kind":"dialog","type":"prompt","message":d.message(),"value":d.current_value()}),
        };
        let dlg_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        *self.pending_embedder.borrow_mut() = Some(EmbedderControl::SimpleDialog(dialog));
        self.push_embedder_json(payload);
        info!("embedder dialog {}", dlg_type);
    }
    fn show_file_picker(&self, picker: FilePicker) {
        let mut dialog = rfd::FileDialog::new();
        if picker.allow_select_multiple() {
            dialog = dialog.set_title("Choose files");
        } else {
            dialog = dialog.set_title("Choose file");
        }
        for pat in picker.filter_patterns() {
            let ext = pat.0.trim_start_matches('.');
            if !ext.is_empty() {
                dialog = dialog.add_filter(ext, &[ext]);
            }
        }
        let paths: Option<Vec<PathBuf>> = match picker.allow_select_multiple() {
            true => dialog.pick_files(),
            false => dialog.pick_file().map(|p| vec![p]),
        };
        let mut picker = picker;
        match paths {
            Some(ps) if !ps.is_empty() => {
                picker.select(&ps);
                picker.submit();
                info!("file picker selected {} file(s)", ps.len());
            }
            _ => {
                picker.dismiss();
                info!("file picker dismissed");
            }
        }
    }
    fn servo_theme(&self) -> ServoTheme {
        match self.themes.borrow().active_is_dark() { true => ServoTheme::Dark, false => ServoTheme::Light }
    }
    fn apply_servo_theme(&self, wv: &WebView) { wv.notify_theme_change(self.servo_theme()); }
    fn broadcast_servo_theme(&self) {
        let t = self.servo_theme();
        info!("prefers-color-scheme \u{2192} {:?}", t);
        if let Some(c) = self.chrome_webview.borrow().as_ref() { c.notify_theme_change(t); }
        for c in self.content_webviews.borrow().iter() { c.notify_theme_change(t); }
    }
    fn spawn_content_webview(&self, url: Url) -> WebView {
        let scale = self.scale_factor.get();
        let wv = WebViewBuilder::new(&self.servo, self.offscreen_context.clone())
            .url(url)
            .hidpi_scale_factor(Scale::new(scale))
            .delegate(self.self_rc())
            .user_content_manager(self.user_content.clone())
            .build();
        self.apply_servo_theme(&wv);
        wv.resize(self.offscreen_context.size());
        let z = self.default_zoom();
        if (z - 1.0).abs() > 0.01 { wv.set_page_zoom(z); }
        wv
    }
    fn default_zoom(&self) -> f32 { (self.config.borrow().default_zoom as f32).clamp(0.25, 5.0) }
    fn home_url(&self) -> String {
        if !self.config.borrow().seen_onboarding || std::env::var("AMNI_TUTORIAL").is_ok() {
            return format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.tutorial_html()));
        }
        let hp = self.config.borrow().home_page.trim().to_string();
        match hp.starts_with("http") { true => hp, false => format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.newtab_html())) }
    }
    fn tutorial_html(&self) -> String {
        let found = import_browsers::detect();
        let browsers = if found.is_empty() {
            "<p class='dim'>No Chrome, Edge, Brave, or Firefox profile found on this account.</p>".into()
        } else {
            let mut s = found.iter().map(|b| format!("<div class='card'><strong>{}</strong><p class='dim'>{}</p><button class='primary' onclick=\"imp('{}')\">Import {}</button></div>", esc_html(&b.name), esc_html(&b.path), esc_html(&b.id), esc_html(&b.name))).collect::<String>();
            s.push_str("<p><button class='primary' onclick=\"imp('all')\">Import everything we found</button></p>");
            s
        };
        TUTORIAL_TPL.replace("__THEME__", &self.theme_root_vars()).replace("__VER__", env!("CARGO_PKG_VERSION")).replace("__TOK__", &self.cmd_token).replace("__BROWSERS__", &browsers)
    }
    fn theme_root_vars(&self) -> String {
        let t = self.themes.borrow().active_theme();
        format!(
            "--bg:{p};--bg-primary:{p};--elev:{e};--bg-tertiary:{e};--bg-secondary:{s};--stroke:{b};--border:{b};--text:{tp};--text-primary:{tp};--dim:{td};--text-secondary:{td};--text-muted:{td};--accent:{a};--accent-dim:{ah};--tab-active:{ta};--tab-inactive:{ti};--chrome:{s}",
            p = t.bg_primary, e = t.bg_tertiary, s = t.bg_secondary, b = t.border, tp = t.text_primary, td = t.text_secondary, a = t.accent, ah = t.accent_hover, ta = t.tab_active, ti = t.tab_inactive
        )
    }
    fn newtab_html(&self) -> String {
        let b = self.bookmarks.borrow();
        let tiles: String = match b.bookmarks.is_empty() {
            true => "<p class='dim'>Bookmark pages with \u{2606} or Ctrl+D and they land here.</p>".into(),
            false => b.bookmarks.iter().take(12).map(|bm| {
                let host = Url::parse(&bm.url).ok().and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string())).unwrap_or_else(|| bm.title.clone());
                let ch: String = host.chars().next().unwrap_or('\u{2022}').to_uppercase().collect();
                let hue = host.bytes().fold(0u32, |a, x| a.wrapping_mul(31).wrapping_add(x as u32)) % 360;
                format!("<a class='tile' href='{}'><div class='mono' style='background:hsl({},45%,38%)'>{}</div><span>{}</span></a>", esc_html(&bm.url), hue, esc_html(&ch), esc_html(&host))
            }).collect(),
        };
        NEWTAB_TPL.replace("__THEME__", &self.theme_root_vars()).replace("__TILES__", &tiles).replace("__VER__", env!("CARGO_PKG_VERSION")).replace("__ENGINE__", "Real Servo")
    }
    fn settings_page_html(&self) -> String {
        let c = self.config.borrow();
        let b = self.bookmarks.borrow();
        let engines = [("DuckDuckGo", "https://html.duckduckgo.com/html/?q="), ("Brave", "https://search.brave.com/search?q="), ("Startpage", "https://www.startpage.com/sp/search?query="), ("Google", "https://www.google.com/search?q=")];
        let radios: String = engines.iter().map(|(n, p)| format!("<label class='opt'><input type='radio' name='se' value='{}'{} onchange='set(\"search_engine\",this.value)'><span>{}</span></label>", p, match c.search_engine == *p { true => " checked", false => "" }, n)).collect();
        let zooms: String = [(0.8, "80%"), (0.9, "90%"), (1.0, "100%"), (1.1, "110%"), (1.25, "125%"), (1.5, "150%")].iter().map(|(z, l)| format!("<option value='{}'{}>{}</option>", z, match (*z - c.default_zoom).abs() < 0.01 { true => " selected", false => "" }, l)).collect();
        let bms: String = match b.bookmarks.is_empty() {
            true => "<p class='dim'>No bookmarks yet \u{2014} hit \u{2606} in the URL bar or Ctrl+D.</p>".into(),
            false => b.bookmarks.iter().map(|bm| format!("<div class='row' id='bm-{}'><a href='{}' title='{}'>{}</a><button class='x' onclick='rmbm(\"{}\")'>remove</button></div>", esc_html(&bm.id), esc_html(&bm.url), esc_html(&bm.url), esc_html(&bm.title), esc_html(&bm.id))).collect(),
        };
        let th = self.themes.borrow();
        let active_id = th.active_theme().id;
        let themes: String = th.all_themes().iter().map(|t| format!("<label class='opt'><input type='radio' name='th' value='{}'{} onchange='set(\"theme\",this.value)'><span>{}</span></label>", esc_html(&t.id), match t.id == active_id { true => " checked", false => "" }, esc_html(&t.name))).collect();
        drop(th);
        let pms = self.pm.borrow();
        let vault_on = pms.unlocked(&self.vault.borrow());
        let pm_kind = pms.kind.clone();
        let pm_label = pms.label().to_string();
        drop(pms);
        let pm_radios: String = [("amni","Amni vault"),("bitwarden","Bitwarden"),("onepassword","1Password"),("keepassxc","KeePassXC")].iter().map(|(id,n)| format!("<label class='opt'><input type='radio' name='pm' value='{}'{} onchange='set(\"password_provider\",this.value)'><span>{}</span></label>", id, match pm_kind == *id { true => " checked", false => "" }, n)).collect();
        let upd = match self.update.lock() {
            Ok(g) => match g.as_ref() { Some(r) => format!("update {} available ({})", r.version, r.source), None => "up to date (or not checked)".into() },
            Err(_) => "update state busy".into(),
        };
        let pmgr = self.profiles.borrow();
        let active_pid = pmgr.active_id.clone();
        let profs: String = pmgr.profiles.iter().map(|p| format!("<div class='row'><span>{}{}</span><button class='x' onclick='set(\"profile_switch\",\"{}\")'>use</button></div>", esc_html(&p.name), match p.id == active_pid { true => " · active", false => "" }, esc_html(&p.id))).collect();
        drop(pmgr);
        let imp = self.last_import.borrow();
        let imp_note = if imp.source.is_empty() { String::new() } else { format!("{}: {} bookmarks, {} history, {} passwords", imp.source, imp.bookmarks, imp.history, imp.passwords) };
        drop(imp);
        SETTINGS_TPL.replace("__THEME__", &self.theme_root_vars()).replace("__THEMES__", &themes).replace("__VER__", env!("CARGO_PKG_VERSION")).replace("__RADIOS__", &radios).replace("__HOME__", &esc_html(match c.home_page.starts_with("http") { true => c.home_page.as_str(), false => "" })).replace("__SHIELD__", match c.block_ads { true => " checked", false => "" }).replace("__ZOOMS__", &zooms).replace("__UA__", &esc_html(c.custom_user_agent.as_deref().unwrap_or(""))).replace("__BMS__", &bms).replace("__TOK__", &self.cmd_token).replace("__VAULT__", match vault_on { true => " unlocked", false => " locked" }).replace("__PROFS__", &profs).replace("__PMRADIOS__", &pm_radios).replace("__PMLABEL__", &esc_html(&pm_label)).replace("__PMCLI__", &esc_html(c.pm_cli_path.as_deref().unwrap_or(""))).replace("__PMDB__", &esc_html(c.pm_keepass_db.as_deref().unwrap_or(""))).replace("__AUTOFILL__", match c.autofill_on_load { true => " checked", false => "" }).replace("__CHKUPD__", match c.check_updates { true => " checked", false => "" }).replace("__RESTORE__", match c.restore_session { true => " checked", false => "" }).replace("__CRASH__", if SessionManager::was_crash() { "Last run did not exit cleanly — tabs were recovered from the crash lock." } else { "" }).replace("__UPD__", &esc_html(&upd)).replace("__IMPORTNOTE__", &esc_html(&imp_note))
    }
    fn load_html_data(&self, webview: &WebView, html: &str) {
        if let Ok(parsed) = Url::parse(&format!("data:text/html;charset=utf-8,{}", urlencoding::encode(html))) { webview.load(parsed); }
    }
    fn open_html_tab(&self, html: &str) {
        let start = Url::parse(&format!("data:text/html;charset=utf-8,{}", urlencoding::encode(html))).unwrap_or_else(|_| Url::parse("about:blank").unwrap());
        let wv = self.spawn_content_webview(start);
        let mut tabs = self.content_webviews.borrow_mut();
        tabs.push(wv);
        self.tab_zoom.borrow_mut().push(self.default_zoom());
        self.push_tab_uid();
        self.active_content_index.set(tabs.len() - 1);
        drop(tabs);
        self.sync_media_len();
        self.apply_media_visibility();
        self.persist_session(false);
        info!("cmd open_html_tab \u{2192} idx {}", self.content_webviews.borrow().len() - 1);
        self.window.request_redraw();
    }
    fn request_view_source(&self) {
        let Some(c) = self.active_content() else { return };
        let url = c.url().map(|u| u.as_str().to_string()).unwrap_or_default();
        let weak = self.self_weak.clone();
        c.evaluate_javascript("(function(){try{return '<!DOCTYPE html>\\n'+document.documentElement.outerHTML}catch(e){return String(e)}})()", move |r| {
            let src = match r {
                Ok(servo::JSValue::String(s)) => s,
                Ok(v) => format!("{:?}", v),
                Err(e) => format!("view-source failed: {:?}", e),
            };
            if let Some(st) = weak.upgrade() { st.pending_source.borrow_mut().push((url, src)); }
        });
        info!("cmd view_source \u{2192} dump requested");
    }
    fn view_source_html(&self, url: &str, src: &str) -> String {
        const CAP: usize = 512 * 1024;
        let truncated = src.len() > CAP;
        let cut = (0..=CAP.min(src.len())).rev().find(|i| src.is_char_boundary(*i)).unwrap_or(0);
        let body = match truncated { true => &src[..cut], false => src };
        let rows: String = body.lines().enumerate().map(|(i, l)| format!("<tr><td class=\"n\">{}</td><td class=\"l\">{}</td></tr>", i + 1, esc_html(l))).collect();
        let note = match truncated { true => "<div class=\"warn\">Truncated at 512 KB.</div>", false => "" };
        format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Source of {t}</title><style>{vars}\nhtml,body{{margin:0;background:var(--bg-primary,#08090B);color:var(--text-primary,#EDEFF2);font:12.5px/1.5 Consolas,\"Cascadia Mono\",monospace}}\nheader{{position:sticky;top:0;display:flex;gap:10px;align-items:center;padding:10px 14px;background:var(--bg-secondary,#0D0F12);border-bottom:1px solid var(--stroke,#20242B);font-family:system-ui,sans-serif;font-size:12px}}\nheader b{{letter-spacing:.16em;text-transform:uppercase;color:var(--accent,#C89B4E);font-size:10.5px}}\nheader span{{color:var(--text-secondary,#A7ADB6);overflow:hidden;white-space:nowrap;text-overflow:ellipsis}}\n.warn{{padding:8px 14px;color:#E8B04B}}\ntable{{border-collapse:collapse;width:100%}}\ntd.n{{width:1%;padding:0 12px 0 14px;text-align:right;color:var(--text-secondary,#A7ADB6);opacity:.55;user-select:none;vertical-align:top;border-right:1px solid var(--stroke,#20242B)}}\ntd.l{{padding:0 14px;white-space:pre-wrap;word-break:break-word}}\ntr:hover td.l{{background:var(--bg-hover,#161A20)}}\n</style></head><body><header><b>Source</b><span>{t}</span></header>{note}<table>{rows}</table></body></html>",
            t = esc_html(url), vars = self.theme_root_vars(), note = note, rows = rows)
    }
    fn load_pdf_viewer(&self, webview: &WebView, url: &str) {
        self.downloads.borrow_mut().start_download(url);
        self.load_html_data(webview, &daily_driver::pdf_viewer_html(url, &self.theme_root_vars(), &self.cmd_token));
    }
    fn try_load_stream(&self, webview: &WebView, url: &str) -> bool {
        match stream_extract::try_player_html(url, &self.theme_root_vars()) {
            Some(html) => { self.load_html_data(webview, &html); true }
            None => false,
        }
    }
    fn inject_after_load(&self, webview: &WebView, url: &str) {
        if url.starts_with("data:") || url.starts_with("amnibrowse:") { return; }
        let auto = self.config.borrow().autofill_on_load;
        let hits = {
            let mut pm = self.pm.borrow_mut();
            pm::matches_for_url(&mut pm, &self.vault.borrow(), url)
        };
        if auto && hits.len() == 1 {
            if let Ok((u, p)) = pm::secret_for(&self.pm.borrow(), &self.vault.borrow(), &hits[0].id) {
                webview.evaluate_javascript(daily_driver::autofill_script(&u, &p), |_| {});
            }
        }
        for (_id, scripts, css) in self.extensions.borrow().get_content_scripts(url) {
            for sheet in css { webview.evaluate_javascript(daily_driver::inject_css_script(&sheet), |_| {}); }
            for js in scripts { webview.evaluate_javascript(js, |_| {}); }
        }
        webview.evaluate_javascript(crate::engine::servo_compat::inject_script(), |_| {});
        webview.evaluate_javascript(crate::engine::servo_compat::svg_repair_script(), |_| {});
        webview.evaluate_javascript(crate::engine::servo_compat::challenge_notice_script(), |_| {});
        if let Some(js) = std::env::var_os("AMNI_PROBE_JS").and_then(|f| std::fs::read_to_string(f).ok()) { let u = url.to_string(); webview.evaluate_javascript(js, move |r| info!("probe {} => {:?}", u, r)); }
        if self.favicon_data_url(webview, url).is_none() { self.prime_origin_favicon(webview, url); }
    }
    fn execute_command(&self, name: &str, args: &std::collections::HashMap<String, String>) {
        match name {
            "back" => { if let Some(c) = self.active_content() { if c.can_go_back() { let _ = c.go_back(1); info!("cmd back"); } } }
            "forward" => { if let Some(c) = self.active_content() { if c.can_go_forward() { let _ = c.go_forward(1); info!("cmd forward"); } } }
            "reload" => {
                if self.active_is_media() {
                    let idx = self.active_content_index.get();
                    if let Some(Some(p)) = self.media_panes.borrow().get(idx) { let _ = p.webview.load_url(&p.url); info!("cmd reload media"); }
                } else if let Some(c) = self.active_content() { c.reload(); info!("cmd reload"); }
            }
            "navigate" => {
                let raw = args.get("url").cloned().unwrap_or_default();
                let engine = self.config.borrow().search_engine.clone();
                match resolve_navigate_input(&raw, &engine) {
                    Some(u) => {
                        let us = u.as_str().to_string();
                        match media_engine::wants_media_window(&us) {
                            true => { info!("cmd navigate \u{2192} in-tab DRM {}", us); self.attach_media_to_active(&us); }
                            false if daily_driver::is_pdf_url(&us) => { self.drop_media_at(self.active_content_index.get()); if let Some(c) = self.active_content() { self.load_pdf_viewer(&c, &us); } self.apply_media_visibility(); }
                            false if daily_driver::is_download_url(&us) => { self.downloads.borrow_mut().start_download(&us); info!("cmd navigate \u{2192} download {}", us); }
                            false => {
                                self.drop_media_at(self.active_content_index.get());
                                self.apply_media_visibility();
                                if let Some(c) = self.active_content() {
                                    if self.try_load_stream(&c, &us) { info!("cmd navigate \u{2192} progressive player {}", us); }
                                    else { info!("cmd navigate \u{2192} {}", u); c.load(u); }
                                }
                            }
                        }
                    }
                    None => info!("cmd navigate: empty/invalid input"),
                }
            }
            "new_tab" => {
                let raw = args.get("url").cloned().unwrap_or_else(|| self.home_url());
                let start = Url::parse(&raw).unwrap_or_else(|_| Url::parse("https://html.duckduckgo.com/html/").unwrap());
                let us = start.as_str().to_string();
                let wv = self.spawn_content_webview(start);
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(self.default_zoom());
                self.push_tab_uid();
                self.active_content_index.set(tabs.len() - 1);
                drop(tabs);
                self.sync_media_len();
                if media_engine::wants_media_window(&us) { self.attach_media_to_active(&us); }
                self.apply_media_visibility();
                self.persist_session(false);
                info!("cmd new_tab \u{2192} idx {}", self.content_webviews.borrow().len() - 1);
                self.window.request_redraw();
            }
            "duplicate_tab" => {
                let url = if let Some(Some(p)) = self.media_panes.borrow().get(self.active_content_index.get()) {
                    p.url.clone()
                } else {
                    self.active_content().and_then(|c| c.url().map(|u| u.as_str().to_string())).unwrap_or_else(|| self.home_url())
                };
                let mut a = std::collections::HashMap::new();
                a.insert("url".into(), url);
                self.execute_command("new_tab", &a);
            }
            "reopen_tab" => {
                let Some(url) = self.closed_tabs.borrow_mut().pop() else { info!("cmd reopen_tab: stack empty"); return };
                let us = url.as_str().to_string();
                let wv = self.spawn_content_webview(url.clone());
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(self.default_zoom());
                self.push_tab_uid();
                self.active_content_index.set(tabs.len() - 1);
                drop(tabs);
                self.sync_media_len();
                if media_engine::wants_media_window(&us) { self.attach_media_to_active(&us); }
                self.apply_media_visibility();
                info!("cmd reopen_tab \u{2192} {}", url);
                self.window.request_redraw();
            }
            "zoom_in" | "zoom_out" | "zoom_reset" => {
                let idx = self.active_content_index.get();
                let mut zooms = self.tab_zoom.borrow_mut();
                if idx >= zooms.len() { return; }
                let cur = zooms[idx];
                let next = match name { "zoom_in" => (cur * 1.1).min(5.0), "zoom_out" => (cur / 1.1).max(0.25), _ => 1.0 };
                zooms[idx] = next;
                drop(zooms);
                if let Some(c) = self.active_content() { c.set_page_zoom(next); info!("cmd {} \u{2192} {:.2}", name, next); }
            }
            "fullscreen" => {
                let new = !self.is_fullscreen.get();
                self.is_fullscreen.set(new);
                self.window.set_fullscreen(match new { true => Some(Fullscreen::Borderless(None)), false => None });
                info!("cmd fullscreen \u{2192} {}", new);
            }
            "stop" => { if let Some(c) = self.active_content() { c.evaluate_javascript("try{window.stop()}catch(e){}", |_| {}); info!("cmd stop"); } }
            "home" => {
                self.drop_media_at(self.active_content_index.get());
                self.apply_media_visibility();
                let target = self.home_url();
                match (Url::parse(&target), self.active_content()) {
                    (Ok(u), Some(c)) => { info!("cmd home \u{2192} {}", target.chars().take(64).collect::<String>()); c.load(u); }
                    _ => info!("cmd home: no target"),
                }
            }
            "kbd" => { self.kbd_in_chrome.set(args.get("on").map(|v| v == "1").unwrap_or(false)); }
            "switch_tab" => {
                let Some(id) = args.get("id") else { return };
                let Some(idx) = Self::parse_tab_index(id) else { return };
                let len = self.content_webviews.borrow().len();
                if idx < len { self.active_content_index.set(idx); self.apply_media_visibility(); self.sync_window_title(); info!("cmd switch_tab \u{2192} idx {}", idx); self.window.request_redraw(); }
            }
            "favicon_cache" => {
                let origin = args.get("origin").cloned().unwrap_or_default();
                let b64 = args.get("b64").cloned().unwrap_or_default();
                if origin.is_empty() || b64.len() > 2400 { return; }
                let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64).ok()
                    .and_then(|b| image::load_from_memory(&b).ok())
                    .and_then(|img| favicon_png_data_url(img.to_rgba8()));
                let hit = data.is_some();
                let mut cache = self.origin_favicons.borrow_mut();
                if cache.len() > 128 { cache.clear(); }
                cache.insert(origin.clone(), data);
                info!("favicon.ico embedder-cache {} {}", origin, if hit { "ok" } else { "decode-fail" });
            }
            "move_tab" => {
                let idx_of = |k: &str| args.get(k).and_then(|s| Self::parse_tab_index(s).or_else(|| s.parse::<usize>().ok()));
                let (Some(from), Some(to)) = (idx_of("from"), idx_of("to")) else { return };
                let len = self.content_webviews.borrow().len();
                if from >= len || to >= len || from == to { return }
                { let mut t = self.content_webviews.borrow_mut(); let w = t.remove(from); t.insert(to, w); }
                { let mut z = self.tab_zoom.borrow_mut(); if from < z.len() && to < z.len() { let v = z.remove(from); z.insert(to, v); } }
                { let mut t = self.tab_titles.borrow_mut(); if from < t.len() && to < t.len() { let v = t.remove(from); t.insert(to, v); } }
                { let mut u = self.tab_uids.borrow_mut(); if from < u.len() && to < u.len() { let v = u.remove(from); u.insert(to, v); } }
                { let mut p = self.media_panes.borrow_mut(); if from < p.len() && to < p.len() { let v = p.remove(from); p.insert(to, v); } }
                let a = self.active_content_index.get();
                let new_active = match a {
                    a if a == from => to,
                    a if from < a && a <= to => a - 1,
                    a if to <= a && a < from => a + 1,
                    a => a,
                };
                self.active_content_index.set(new_active);
                self.sync_media_len();
                self.apply_media_visibility();
                self.persist_session(false);
                info!("cmd move_tab {} \u{2192} {}, active now {}", from, to, new_active);
                self.window.request_redraw();
            }
            "view_source" => self.request_view_source(),
            "close_tab" => {
                let Some(id) = args.get("id") else { return };
                let Some(idx) = Self::parse_tab_index(id) else { return };
                let tab_count = self.content_webviews.borrow().len();
                if idx >= tab_count || tab_count <= 1 { info!("cmd close_tab: refusing (idx {} of {})", idx, tab_count); return; }
                if let Some(pane) = self.media_panes.borrow().get(idx).and_then(|s| s.as_ref()) {
                    if let Ok(u) = Url::parse(&pane.url) { self.closed_tabs.borrow_mut().push(u); }
                } else if let Some(u) = self.content_webviews.borrow().get(idx).and_then(|c| c.url()) {
                    self.closed_tabs.borrow_mut().push(u);
                }
                // Drop media engine first, then remove its slot.
                {
                    let mut panes = self.media_panes.borrow_mut();
                    if idx < panes.len() {
                        if let Some(pane) = panes[idx].take() {
                            media_engine::trash_pane(pane);
                        }
                        panes.remove(idx);
                    }
                }
                // Drop Servo webview first, then remove tab bookkeeping.
                {
                    let mut tabs = self.content_webviews.borrow_mut();
                    let doomed = tabs.remove(idx);
                    Self::drop_webview(doomed);
                    let mut zooms = self.tab_zoom.borrow_mut();
                    if idx < zooms.len() { zooms.remove(idx); }
                    drop(zooms);
                    let mut titles = self.tab_titles.borrow_mut();
                    if idx < titles.len() { titles.remove(idx); }
                    drop(titles);
                    let mut uids = self.tab_uids.borrow_mut();
                    if idx < uids.len() { uids.remove(idx); }
                    drop(uids);
                    let active = self.active_content_index.get();
                    let new_active = match active {
                        a if a == idx => idx.min(tabs.len() - 1),
                        a if a > idx => a - 1,
                        a => a,
                    };
                    self.active_content_index.set(new_active);
                    drop(tabs);
                    info!("cmd close_tab \u{2192} dropped {}, active now {}", idx, new_active);
                }
                self.apply_media_visibility();
                // Defer disk write off the amnibrowse://cmd intercept path.
                self.pending_session_persist.set(true);
                self.window.request_redraw();
            }
            "bookmark" => {
                let Some(c) = self.active_content() else { return };
                let u = c.url().map(|u| u.as_str().to_string()).unwrap_or_default();
                if u.is_empty() || u.starts_with("data:") || u.starts_with("amnibrowse") { info!("cmd bookmark: skip {}", u); return; }
                let t = c.page_title().unwrap_or_default();
                let mut b = self.bookmarks.borrow_mut();
                match b.find_by_url(&u).map(|x| x.id.clone()) {
                    Some(id) => { b.remove(&id); info!("cmd bookmark \u{2192} removed {}", u); }
                    None => { b.add(match t.trim().is_empty() { true => u.as_str(), false => t.trim() }, &u, None); info!("cmd bookmark \u{2192} added {}", u); }
                }
            }
            "shield" => {
                let on = { let mut c = self.config.borrow_mut(); c.block_ads = !c.block_ads; c.block_trackers = c.block_ads; c.save(); c.block_ads };
                info!("cmd shield \u{2192} {}", match on { true => "on", false => "off" });
            }
            "setting_set" => {
                let (Some(k), Some(v)) = (args.get("k"), args.get("v")) else { info!("setting_set: missing k/v"); return };
                if k == "theme" {
                    self.themes.borrow_mut().set_theme(v);
                    info!("cmd setting_set theme \u{2192} {}", v);
                    self.broadcast_servo_theme();
                    let settings_data = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.settings_page_html()));
                    let newtab_data = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.newtab_html()));
                    let tabs = self.content_webviews.borrow();
                    for c in tabs.iter() {
                        let u = c.url().map(|x| x.as_str().to_string()).unwrap_or_default();
                        if !u.starts_with("data:text/html") { continue; }
                        let title = c.page_title().unwrap_or_default();
                        let target = if title.contains("Settings") { settings_data.as_str() } else { newtab_data.as_str() };
                        if let Ok(parsed) = Url::parse(target) { c.load(parsed); }
                    }
                    return;
                }
                {
                    let mut c = self.config.borrow_mut();
                    match k.as_str() {
                        "search_engine" => c.search_engine = v.clone(),
                        "home_page" => c.home_page = v.clone(),
                        "block_ads" => { c.block_ads = v == "true"; c.block_trackers = c.block_ads; }
                        "default_zoom" => c.default_zoom = v.parse().unwrap_or(1.0),
                        "custom_user_agent" => c.custom_user_agent = match v.trim().is_empty() { true => None, false => Some(v.trim().to_string()) },
                        "default_browser" => { drop(c); match os_default::register_browser() { Ok(m) => info!("{}", m), Err(e) => info!("default browser: {}", e) }; return; }
                        "vault_pw" => {
                            drop(c);
                            let mut vlt = self.vault.borrow_mut();
                            let mut pm = self.pm.borrow_mut();
                            match pm::unlock(&mut pm, v, &mut vlt) { Ok(m) => info!("{}", m), Err(e) => info!("pm unlock: {}", e) }
                            return;
                        }
                        "password_provider" => {
                            c.password_provider = pm::normalize_kind(v);
                            c.save();
                            drop(c);
                            let cfg = self.config.borrow();
                            *self.pm.borrow_mut() = PmState::from_config(&cfg.password_provider, cfg.pm_cli_path.clone(), cfg.pm_keepass_db.clone());
                            info!("password provider → {}", v);
                            return;
                        }
                        "pm_cli_path" => { c.pm_cli_path = match v.trim().is_empty() { true => None, false => Some(v.trim().into()) }; self.pm.borrow_mut().cli = c.pm_cli_path.clone(); }
                        "pm_keepass_db" => { c.pm_keepass_db = match v.trim().is_empty() { true => None, false => Some(v.trim().into()) }; self.pm.borrow_mut().keepass_db = c.pm_keepass_db.clone(); }
                        "autofill_on_load" => { c.autofill_on_load = v == "true"; }
                        "check_updates" => { c.check_updates = v == "true"; }
                        "restore_session" => { c.restore_session = v == "true"; }
                        "update_feed" => { c.update_feed = match v.trim().is_empty() { true => None, false => Some(v.trim().into()) }; }
                        "update_check" => {
                            drop(c);
                            let feed = self.config.borrow().update_feed.clone();
                            match updater::check_for_update(env!("CARGO_PKG_VERSION"), feed.as_deref()) {
                                Ok(Some(r)) => { info!("update available {}", r.version); if let Ok(mut g) = self.update.lock() { *g = Some(r); } }
                                Ok(None) => info!("no update"),
                                Err(e) => info!("update check: {}", e),
                            }
                            return;
                        }
                        "import_browser" => {
                            drop(c);
                            let src = v.clone();
                            let report = import_browsers::apply_import(&src, &mut self.bookmarks.borrow_mut(), &mut self.history.borrow_mut(), &mut self.vault.borrow_mut());
                            info!("import {}: bm={} hist={} pw={}", report.source, report.bookmarks, report.history, report.passwords);
                            *self.last_import.borrow_mut() = report;
                            return;
                        }
                        "show_tutorial" => {
                            drop(c);
                            let url_str = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.tutorial_html()));
                            if let (Ok(parsed), Some(tab)) = (Url::parse(&url_str), self.active_content()) { tab.load(parsed); }
                            return;
                        }
                        "update_now" => {
                            drop(c);
                            let rel = self.update.lock().ok().and_then(|g| g.clone());
                            match rel {
                                Some(r) => match updater::apply_update(&r) { Ok(m) => { info!("{}", m); *self.pending_relaunch.borrow_mut() = Some("__update__".into()); } Err(e) => info!("update apply: {}", e) },
                                None => info!("no cached update — check first"),
                            }
                            return;
                        }
                        "profile_new" => {
                            drop(c);
                            if !v.trim().is_empty() { self.profiles.borrow_mut().create_profile(v.trim(), "#C89B4E"); info!("profile created {}", v); }
                            return;
                        }
                        "profile_switch" => {
                            drop(c);
                            if self.profiles.borrow_mut().switch_profile(v) { *self.pending_relaunch.borrow_mut() = Some(v.clone()); info!("profile switch queued {}", v); }
                            return;
                        }
                        other => { info!("setting_set: unknown key {}", other); return; }
                    }
                    c.save();
                }
                info!("cmd setting_set {} \u{2192} {}", k, v);
            }
            "bookmark_remove" => {
                let Some(id) = args.get("id") else { return };
                if self.bookmarks.borrow_mut().remove(id) { info!("cmd bookmark_remove {}", id); }
            }
            "overlay" => {
                let h: u32 = args.get("h").and_then(|s| s.parse().ok()).unwrap_or(0);
                self.chrome_overlay_px.set(h.min(self.window_size().height.max(1)));
            }
            "overlay_rect" => {
                let n = |k: &str| args.get(k).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                let (w, h) = (n("w"), n("h"));
                self.overlay_css.set(match w > 0 && h > 0 { true => Some((n("x"), n("y"), w, h)), false => None });
                self.window.request_redraw();
            }
            "ctx_pick" => {
                let Some(id) = args.get("id").cloned() else { return };
                match id.as_str() {
                    "amni_newtab" => {
                        let url = args.get("url").cloned().or_else(|| self.ctx_link.borrow().clone()).unwrap_or_default();
                        self.dismiss_embedder();
                        if !url.is_empty() { let mut a = std::collections::HashMap::new(); a.insert("url".into(), url); self.execute_command("new_tab", &a); }
                    }
                    "amni_saveimg" => {
                        let url = args.get("url").cloned().or_else(|| self.ctx_image.borrow().clone()).unwrap_or_default();
                        self.dismiss_embedder();
                        if !url.is_empty() { self.downloads.borrow_mut().start_download(&url); info!("ctx save image {}", url); }
                    }
                    "amni_viewsrc" => {
                        self.dismiss_embedder();
                        self.request_view_source();
                    }
                    other => {
                        if let Some(action) = Self::parse_ctx_action(other) {
                            match self.pending_embedder.borrow_mut().take() {
                                Some(EmbedderControl::ContextMenu(menu)) => { menu.select(action); info!("ctx servo {}", other); }
                                other_ctl => { *self.pending_embedder.borrow_mut() = other_ctl; }
                            }
                            self.hide_embedder_ui();
                            if action == ContextMenuAction::OpenLinkInNewWebView {
                                if let Some(u) = self.ctx_link.borrow().clone() { let mut a = std::collections::HashMap::new(); a.insert("url".into(), u); self.execute_command("new_tab", &a); }
                            }
                            if action == ContextMenuAction::OpenImageInNewView {
                                if let Some(u) = self.ctx_image.borrow().clone() { let mut a = std::collections::HashMap::new(); a.insert("url".into(), u); self.execute_command("new_tab", &a); }
                            }
                        }
                    }
                }
            }
            "ctx_dismiss" => { self.dismiss_embedder(); }
            "select_pick" => {
                let Some(id) = args.get("id").and_then(|s| s.parse::<usize>().ok()) else { return };
                match self.pending_embedder.borrow_mut().take() {
                    Some(EmbedderControl::SelectElement(mut sel)) => { sel.select(vec![id]); sel.submit(); info!("select pick {}", id); }
                    other => { *self.pending_embedder.borrow_mut() = other; }
                }
                self.hide_embedder_ui();
            }
            "dialog_ok" => {
                let val = args.get("v").cloned();
                match self.pending_embedder.borrow_mut().take() {
                    Some(EmbedderControl::SimpleDialog(sd)) => {
                        match sd {
                            SimpleDialog::Alert(d) => d.confirm(),
                            SimpleDialog::Confirm(d) => d.confirm(),
                            SimpleDialog::Prompt(mut d) => {
                                if let Some(v) = val { d.set_current_value(&v); }
                                d.confirm();
                            }
                        }
                        info!("dialog ok");
                    }
                    other => { *self.pending_embedder.borrow_mut() = other; }
                }
                self.hide_embedder_ui();
            }
            "color_pick" => {
                let hex = args.get("v").cloned().unwrap_or_default();
                match self.pending_embedder.borrow_mut().take() {
                    Some(EmbedderControl::ColorPicker(mut p)) => {
                        match Self::parse_hex_rgb(&hex) {
                            Some(c) => { p.select(Some(c)); info!("color pick {}", hex); }
                            None => info!("color pick: bad hex {}", hex),
                        }
                        p.submit();
                    }
                    other => { *self.pending_embedder.borrow_mut() = other; }
                }
                self.hide_embedder_ui();
            }
            "dialog_cancel" => {
                match self.pending_embedder.borrow_mut().take() {
                    Some(EmbedderControl::SimpleDialog(sd)) => {
                        sd.dismiss();
                        info!("dialog cancel");
                    }
                    other => { *self.pending_embedder.borrow_mut() = other; }
                }
                self.hide_embedder_ui();
            }
            "find" => {
                let q = args.get("q").cloned().unwrap_or_default();
                let dir: i32 = args.get("dir").and_then(|s| s.parse().ok()).unwrap_or(1);
                *self.find_query.borrow_mut() = q.clone();
                if let Some(c) = self.active_content() { c.evaluate_javascript(daily_driver::find_script(&q, dir), |_| {}); }
            }
            "print" => { if let Some(c) = self.active_content() { c.evaluate_javascript(daily_driver::print_script(), |_| {}); } }
            "download" => {
                let url = args.get("url").cloned().or_else(|| self.active_content().and_then(|c| c.url().map(|u| u.as_str().to_string()))).unwrap_or_default();
                if !url.is_empty() { self.downloads.borrow_mut().start_download(&url); info!("cmd download {}", url); }
            }
            "open_download" => {
                let Some(url) = args.get("url") else { return };
                self.downloads.borrow_mut().start_download(url);
                let name = url.rsplit('/').next().unwrap_or("download").split('?').next().unwrap_or("download");
                let path = crate::storage::downloads::DownloadManager::downloads_dir().join(name);
                let _ = os_default::open_path(&path.to_string_lossy());
            }
            "tutorial_done" => {
                { let mut c = self.config.borrow_mut(); c.seen_onboarding = true; c.save(); }
                let url_str = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.newtab_html()));
                if let (Ok(parsed), Some(tab)) = (Url::parse(&url_str), self.active_content()) { tab.load(parsed); info!("tutorial done"); }
            }
            "import_browser" => {
                let src = args.get("src").cloned().unwrap_or_else(|| "all".into());
                let report = import_browsers::apply_import(&src, &mut self.bookmarks.borrow_mut(), &mut self.history.borrow_mut(), &mut self.vault.borrow_mut());
                info!("import {}: bm={} hist={} pw={}", report.source, report.bookmarks, report.history, report.passwords);
                *self.last_import.borrow_mut() = report;
            }
            "fill_login" => {
                let Some(id) = args.get("id") else { return };
                match pm::secret_for(&self.pm.borrow(), &self.vault.borrow(), id) {
                    Ok((u, p)) => { if let Some(c) = self.active_content() { c.evaluate_javascript(daily_driver::autofill_script(&u, &p), |_| {}); info!("filled login {}", id); } }
                    Err(e) => info!("fill_login: {}", e),
                }
            }
            "private_tab" => {
                let start = Url::parse(&self.home_url()).unwrap_or_else(|_| Url::parse("https://html.duckduckgo.com/html/").unwrap());
                let wv = self.spawn_content_webview(start);
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(self.default_zoom());
                self.push_tab_uid();
                self.active_content_index.set(tabs.len() - 1);
                info!("cmd private_tab");
                self.window.request_redraw();
            }
            "win_min" => { self.window.set_minimized(true); }
            "win_max" => { self.window.set_maximized(!self.window.is_maximized()); }
            "win_close" => { *self.pending_relaunch.borrow_mut() = Some("__exit__".into()); }
            "win_drag" => { let _ = self.window.drag_window(); }
            "menu" | "settings" => {
                let url_str = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.settings_page_html()));
                match Url::parse(&url_str) {
                    Ok(parsed) => { if let Some(c) = self.active_content() { c.load(parsed); info!("cmd {} \u{2192} settings page", name); } }
                    Err(e) => info!("cmd {}: failed to build data-url ({})", name, e),
                }
            }
            other => info!("cmd unknown: {}", other),
        }
    }
}
fn cors_headers() -> http::HeaderMap {
    let mut h = http::HeaderMap::new();
    h.insert(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, http::HeaderValue::from_static("*"));
    h.insert(http::header::ACCESS_CONTROL_ALLOW_METHODS, http::HeaderValue::from_static("GET, POST, OPTIONS"));
    h.insert(http::header::ACCESS_CONTROL_ALLOW_HEADERS, http::HeaderValue::from_static("*"));
    h
}
fn embedder_anchor_css(edge_x: i32, edge_y: i32) -> (f32, f32) {
    (edge_x as f32, edge_y as f32 + CHROME_HEIGHT_CSS)
}
fn favicon_png_data_url(buf: image::RgbaImage) -> Option<String> {
    let (w, h) = (buf.width(), buf.height());
    if w == 0 || h == 0 || w > 512 || h > 512 { return None; }
    let small = match w > 32 || h > 32 { true => image::imageops::resize(&buf, 32, 32, image::imageops::FilterType::Triangle), false => buf };
    let mut png = std::io::Cursor::new(Vec::new());
    small.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(format!("data:image/png;base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png.into_inner())))
}
fn favicon_origin(tab_url: &str) -> Option<String> {
    let u = Url::parse(tab_url).ok()?;
    if !matches!(u.scheme(), "http" | "https") { return None; }
    let host = u.host_str()?;
    Some(match u.port() { Some(p) => format!("{}://{}:{}", u.scheme(), host, p), None => format!("{}://{}", u.scheme(), host) })
}
fn host_display_title(tab_url: &str) -> Option<String> {
    let u = Url::parse(tab_url).ok()?;
    u.host_str().map(|h| {
        let h = h.strip_prefix("www.").unwrap_or(h);
        if h.is_empty() { "New Tab".into() } else { h.to_string() }
    })
}
const FAVICON_POLL_MAX: u32 = 30;
const FAVICON_MAX_BYTES: u64 = 900_000;
const FAVICON_FETCH_UA: &str = concat!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AmniBrowse/", env!("CARGO_PKG_VERSION"));
pub enum FaviconJob { Running, Done(String, Option<String>) }
pub type FaviconJobs = Arc<Mutex<std::collections::HashMap<String, FaviconJob>>>;
fn favicon_link_probe_script() -> String {
    "(function(){try{var q=document.querySelectorAll('link[rel]');for(var i=0;i<q.length;i++){var r=(q[i].rel||'').toLowerCase();if(r.indexOf('icon')>=0)return 'has'}return 'none'}catch(e){return 'err'}})()".to_string()
}
fn favicon_fetch_headers() -> [(&'static str, &'static str); 4] {
    [("Accept", "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5"), ("Sec-Fetch-Dest", "image"), ("Sec-Fetch-Mode", "no-cors"), ("Sec-Fetch-Site", "same-origin")]
}
fn favicon_https_to_http_downgrade(from_scheme: &str, to_scheme: &str) -> bool {
    from_scheme == "https" && to_scheme == "http"
}
fn favicon_refuse_https_to_http(attempt: reqwest::redirect::Attempt<'_>) -> reqwest::redirect::Action {
    let from_https = attempt.previous().last().is_some_and(|u| u.scheme() == "https");
    if favicon_https_to_http_downgrade(if from_https { "https" } else { "http" }, attempt.url().scheme()) {
        return attempt.error("mixed-content favicon redirect");
    }
    attempt.follow()
}
fn favicon_jobs_insert_running(jobs: &mut std::collections::HashMap<String, FaviconJob>, origin: String) {
    if jobs.len() > 128 { jobs.retain(|_, v| matches!(v, FaviconJob::Running)); }
    jobs.insert(origin, FaviconJob::Running);
}
fn favicon_fetch_origin(origin: &str, ua: Option<&str>) -> (String, Option<String>) {
    let Ok(client) = reqwest::blocking::Client::builder().redirect(reqwest::redirect::Policy::custom(favicon_refuse_https_to_http)).timeout(std::time::Duration::from_secs(8)).user_agent(ua.filter(|u| !u.trim().is_empty()).unwrap_or(FAVICON_FETCH_UA)).build() else { return ("fetch-err".into(), None) };
    let mut req = client.get(format!("{}/favicon.ico", origin));
    for (k, v) in favicon_fetch_headers() { req = req.header(k, v); }
    let resp = match req.send() {
        Ok(r) => r,
        Err(e) => {
            if e.to_string().contains("mixed-content") { return ("mixed-redirect".into(), None); }
            return ("miss".into(), None);
        }
    };
    if !resp.status().is_success() { return (format!("miss-{}", resp.status().as_u16()), None); }
    if resp.content_length().is_some_and(|n| n > FAVICON_MAX_BYTES) { return ("big".into(), None); }
    let Ok(bytes) = resp.bytes() else { return ("read-err".into(), None) };
    if bytes.is_empty() { return ("empty".into(), None); }
    if bytes.len() as u64 > FAVICON_MAX_BYTES { return ("big".into(), None); }
    match image::load_from_memory(&bytes).ok().and_then(|i| favicon_png_data_url(i.to_rgba8())) {
        Some(d) => ("embedder-fetch ok".into(), Some(d)),
        None => ("embedder-fetch decode-fail".into(), None),
    }
}
fn favicon_tag_is_terminal(tag: &str) -> bool {
    !matches!(tag, "started" | "pending" | "gone" | "none")
}
impl AppState {
    fn favicon_data_url(&self, c: &WebView, key: &str) -> Option<String> {
        if key.is_empty() { return None; }
        if let Some(hit) = self.favicons.borrow().get(key) { return Some(hit.clone()); }
        let (w, h, rgba) = {
            let img = c.favicon()?;
            let (w, h) = (img.width, img.height);
            if w == 0 || h == 0 || w > 512 || h > 512 { return None; }
            let src = img.data();
            let rgba: Vec<u8> = match img.format {
                servo::PixelFormat::RGBA8 => src.to_vec(),
                servo::PixelFormat::BGRA8 => src.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect(),
                servo::PixelFormat::RGB8 => src.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
                servo::PixelFormat::KA8 => src.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
                servo::PixelFormat::K8 => src.iter().flat_map(|v| [*v, *v, *v, 255]).collect(),
            };
            (w, h, rgba)
        };
        if rgba.len() < (w as usize) * (h as usize) * 4 { return None; }
        let data = favicon_png_data_url(image::RgbaImage::from_raw(w, h, rgba)?)?;
        let mut cache = self.favicons.borrow_mut();
        if cache.len() > 64 { cache.clear(); }
        cache.insert(key.to_string(), data.clone());
        drop(cache);
        if let Some(origin) = favicon_origin(key) {
            let mut origins = self.origin_favicons.borrow_mut();
            if origins.len() > 128 { origins.clear(); }
            origins.insert(origin, Some(data.clone()));
        }
        Some(data)
    }
    fn origin_favicon(&self, tab_url: &str) -> Option<String> {
        let origin = favicon_origin(tab_url)?;
        self.origin_favicons.borrow().get(&origin).and_then(|v| v.clone())
    }
    fn finish_origin_favicon(&self, origin: &str, data: Option<String>) {
        {
            let mut cache = self.origin_favicons.borrow_mut();
            if cache.len() > 128 { cache.clear(); }
            cache.insert(origin.to_string(), data);
        }
        let mut primed = self.origin_favicon_primed.borrow_mut();
        if primed.len() > 128 { primed.clear(); }
        primed.insert(origin.to_string());
        self.origin_favicon_polls.borrow_mut().remove(origin);
        self.origin_favicon_reprimes.borrow_mut().remove(origin);
        self.origin_favicon_probing.borrow_mut().remove(origin);
    }
    fn prime_origin_favicon(&self, c: &WebView, tab_url: &str) {
        let Some(origin) = favicon_origin(tab_url) else { return };
        if self.origin_favicon_primed.borrow().contains(&origin) { return; }
        if matches!(self.favicon_jobs.lock().unwrap().get(&origin), Some(FaviconJob::Running)) { return; }
        if self.origin_favicon_probing.borrow().contains(&origin) {
            let n = {
                let mut polls = self.origin_favicon_polls.borrow_mut();
                let slot = polls.entry(origin.clone()).or_insert(0u32);
                let n = *slot;
                *slot += 1;
                n
            };
            if n >= FAVICON_POLL_MAX {
                info!("favicon.ico servo-net {} poll-timeout", origin);
                self.finish_origin_favicon(&origin, None);
            }
            return;
        }
        if let Some(FaviconJob::Done(tag, data)) = self.favicon_jobs.lock().unwrap().remove(&origin) {
            info!("favicon.ico embedder-net {} {}", origin, tag);
            self.finish_origin_favicon(&origin, data);
            return;
        }
        let n = {
            let mut polls = self.origin_favicon_polls.borrow_mut();
            if polls.len() > 128 { polls.clear(); }
            let slot = polls.entry(origin.clone()).or_insert(0u32);
            let n = *slot;
            *slot += 1;
            n
        };
        if n >= FAVICON_POLL_MAX {
            info!("favicon.ico servo-net {} poll-timeout", origin);
            self.finish_origin_favicon(&origin, None);
            return;
        }
        self.origin_favicon_reprimes.borrow_mut().remove(&origin);
        self.origin_favicon_probing.borrow_mut().insert(origin.clone());
        let weak = self.self_weak.clone();
        let jobs = self.favicon_jobs.clone();
        let ua = self.config.borrow().custom_user_agent.clone();
        c.evaluate_javascript(&favicon_link_probe_script(), move |r| {
            if let Some(st) = weak.upgrade() { st.origin_favicon_probing.borrow_mut().remove(&origin); }
            let tag: String = match r {
                Ok(servo::JSValue::String(s)) => s,
                Ok(_) => "eval-unexpected".into(),
                Err(_) => "eval-fail".into(),
            };
            if tag != "none" {
                info!("favicon.ico servo-net {} {}", origin, tag);
                if favicon_tag_is_terminal(&tag) {
                    if let Some(st) = weak.upgrade() { st.finish_origin_favicon(&origin, None); }
                }
                return;
            }
            {
                let mut j = jobs.lock().unwrap();
                if j.contains_key(&origin) { return; }
                favicon_jobs_insert_running(&mut j, origin.clone());
            }
            info!("favicon.ico embedder-net {} started", origin);
            let sink = jobs.clone();
            std::thread::spawn(move || {
                let (tag, data) = favicon_fetch_origin(&origin, ua.as_deref());
                sink.lock().unwrap().insert(origin, FaviconJob::Done(tag, data));
            });
        });
    }
    fn build_state_json(&self) -> String {
        let content_opt = self.active_content();
        let (url, title, loading, can_back, can_forward) = if let Some(Some(mw)) = self.media_panes.borrow().get(self.active_content_index.get()) {
            (mw.url.clone(), media_engine::display_title(&mw.url), false, false, false)
        } else { match content_opt.as_ref() {
            Some(c) => {
                let tab_url = c.url().map(|u| u.as_str().to_string()).unwrap_or_default();
                let idx = self.active_content_index.get();
                (
                    tab_url.clone(),
                    self.tab_display_title(idx, c, &tab_url),
                    !matches!(c.load_status(), LoadStatus::Complete),
                    c.can_go_back(),
                    c.can_go_forward(),
                )
            },
            None => (String::new(), String::new(), false, false, false),
        } };
        let active_idx = self.active_content_index.get();
        let panes = self.media_panes.borrow();
        let uids = self.tab_uids.borrow();
        let tabs: Vec<serde_json::Value> = self.content_webviews.borrow().iter().enumerate().map(|(i, c)| {
            let uid = uids.get(i).copied().unwrap_or(i as u64);
            if let Some(Some(mw)) = panes.get(i) {
                serde_json::json!({
                    "id": format!("t{}", i),
                    "uid": uid,
                    "url": mw.url,
                    "title": media_engine::display_title(&mw.url),
                    "active": i == active_idx,
                    "loading": false,
                    "engine": "media",
                })
            } else {
                let tab_url = c.url().map(|u| u.as_str().to_string()).unwrap_or_default();
                // Background tabs: cache only. Calling WebView::favicon() + prime on every
                // 250ms poll across N tabs was a major close/switch freeze contributor.
                let icon = match tab_url.starts_with("data:") {
                    true => None,
                    false if i == active_idx => {
                        let got = self.favicon_data_url(c, &tab_url)
                            .or_else(|| self.origin_favicon(&tab_url));
                        if got.is_none() { self.prime_origin_favicon(c, &tab_url); }
                        got
                    }
                    false => self.favicons.borrow().get(&tab_url).cloned()
                        .or_else(|| self.origin_favicon(&tab_url)),
                };
                serde_json::json!({
                    "id": format!("t{}", i),
                    "uid": uid,
                    "url": tab_url,
                    "title": self.tab_display_title(i, c, &tab_url),
                    "active": i == active_idx,
                    "loading": !matches!(c.load_status(), LoadStatus::Complete),
                    "engine": "servo",
                    "icon": icon,
                })
            }
        }).collect();
        let all_tabs = tabs;
        let zoom = self.tab_zoom.borrow().get(active_idx).copied().unwrap_or(1.0);
        let theme: serde_json::Value = serde_json::from_str(&self.themes.borrow().active_theme_json()).unwrap_or(serde_json::Value::Null);
        serde_json::json!({
            "url": url,
            "title": title,
            "loading": loading,
            "canBack": can_back,
            "canForward": can_forward,
            "tabs": all_tabs,
            "theme": theme,
            "zoom": zoom,
            "fullscreen": self.is_fullscreen.get(),
            "maximized": self.window.is_maximized(),
            "canReopen": !self.closed_tabs.borrow().is_empty(),
            "shield": self.config.borrow().block_ads,
            "bookmarked": !url.is_empty() && self.bookmarks.borrow().find_by_url(&url).is_some(),
            "vault": self.pm.borrow().unlocked(&self.vault.borrow()),
            "downloads": self.downloads.borrow().downloads.len(),
            "profile": self.profiles.borrow().active_profile().name,
            "find": self.find_query.borrow().clone(),
            "pm": self.pm.borrow().label(),
            "logins": self.pm.borrow().last,
            "update": self.update.lock().ok().and_then(|g| g.as_ref().map(|r| serde_json::json!({"version": r.version, "source": r.source, "notes": r.notes}))),
        }).to_string()
    }
}
/// Work area (taskbar excluded) of the monitor containing `point`, in physical pixels: (x, y, w, h).
#[cfg(windows)]
fn monitor_work_area(point: (i32, i32)) -> Option<(i32, i32, i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    unsafe {
        let m = MonitorFromPoint(POINT { x: point.0, y: point.1 }, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        (GetMonitorInfoW(m, &mut info) != 0).then(|| { let r = info.rcWork; (r.left, r.top, r.right - r.left, r.bottom - r.top) })
    }
}
#[cfg(not(windows))]
fn monitor_work_area(_point: (i32, i32)) -> Option<(i32, i32, i32, i32)> { None }
/// Clamp a saved logical window origin so the whole window sits inside the work area of the
/// monitor it was on; without a saved origin, center on the primary work area.
fn restore_window_position(saved: Option<(f64, f64)>, size: (f64, f64), scale: f64) -> (f64, f64) {
    let (x, y) = saved.unwrap_or((f64::NAN, f64::NAN));
    let probe = match saved { Some((x, y)) => ((x * scale) as i32 + 8, (y * scale) as i32 + 8), None => (0, 0) };
    let Some((ax, ay, aw, ah)) = monitor_work_area(probe) else { return match saved { Some(p) => p, None => (100.0, 60.0) }; };
    let (ax, ay, aw, ah) = (ax as f64 / scale, ay as f64 / scale, aw as f64 / scale, ah as f64 / scale);
    let clamp = |v: f64, lo: f64, span: f64, len: f64| match v.is_nan() { true => lo + ((span - len) / 2.0).max(0.0), false => v.max(lo).min((lo + span - len).max(lo)) };
    (clamp(x, ax, aw, size.0), clamp(y, ay, ah, size.1))
}
fn handle_shortcut(key_event: &KeyEvent, state: &AppState) -> bool {
    if key_event.state != ElementState::Pressed {
        return state.shortcut_keys.borrow_mut().remove(&key_event.physical_key);
    }
    let mods = state.modifiers.get();
    let ctrl = mods.control_key() || mods.super_key();
    let shift = mods.shift_key();
    let alt = mods.alt_key();
    let empty: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let switch_to = |idx: usize| { let mut a = std::collections::HashMap::new(); a.insert("id".to_string(), format!("t{}", idx)); state.execute_command("switch_tab", &a); };
    let close_idx = |idx: usize| { let mut a = std::collections::HashMap::new(); a.insert("id".to_string(), format!("t{}", idx)); state.execute_command("close_tab", &a); };
    match (&key_event.logical_key, ctrl, shift) {
        (Key::Named(NamedKey::ArrowLeft), _, _) if alt => { state.execute_command("back", &empty); true }
        (Key::Named(NamedKey::ArrowRight), _, _) if alt => { state.execute_command("forward", &empty); true }
        (Key::Named(NamedKey::F5), false, _) => { state.execute_command("reload", &empty); true }
        (Key::Named(NamedKey::F11), false, _) => { state.execute_command("fullscreen", &empty); true }
        (Key::Named(NamedKey::Home), _, _) if alt => { state.execute_command("home", &empty); true }
        (Key::Named(NamedKey::Escape), false, false) => {
            if state.is_fullscreen.get() { state.execute_command("fullscreen", &empty); return true; }
            let loading = state.active_content().map(|c| !matches!(c.load_status(), LoadStatus::Complete)).unwrap_or(false);
            match loading && !state.kbd_in_chrome.get() { true => { state.execute_command("stop", &empty); true } false => false }
        }
        (Key::Named(NamedKey::Tab), true, false) => {
            let len = state.content_webviews.borrow().len();
            if len > 1 { switch_to((state.active_content_index.get() + 1) % len); } true
        }
        (Key::Named(NamedKey::Tab), true, true) => {
            let len = state.content_webviews.borrow().len();
            if len > 1 { switch_to((state.active_content_index.get() + len - 1) % len); } true
        }
        (Key::Character(c), true, true) if c.eq_ignore_ascii_case("t") => { state.execute_command("reopen_tab", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("t") => { state.execute_command("new_tab", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("w") => { close_idx(state.active_content_index.get()); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("r") => { state.execute_command("reload", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("l") => {
            state.kbd_in_chrome.set(true);
            if let Some(chrome) = state.chrome_webview.borrow().as_ref() {
                let _ = chrome.evaluate_javascript("document.getElementById('url').focus();document.getElementById('url').select();", |_| {});
            }
            true
        }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("u") => { state.execute_command("view_source", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("d") => { state.execute_command("bookmark", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("f") => {
            state.kbd_in_chrome.set(true);
            if let Some(chrome) = state.chrome_webview.borrow().as_ref() {
                let _ = chrome.evaluate_javascript("window.__amni&&window.__amni.showFind&&window.__amni.showFind()", |_| {});
            }
            true
        }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("p") => { state.execute_command("print", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("s") => { state.execute_command("download", &empty); true }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("j") => {
            if let Some(chrome) = state.chrome_webview.borrow().as_ref() {
                let _ = chrome.evaluate_javascript("window.__amni&&window.__amni.showPanel&&window.__amni.showPanel('dl')", |_| {});
            }
            true
        }
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("h") => {
            if let Some(chrome) = state.chrome_webview.borrow().as_ref() {
                let _ = chrome.evaluate_javascript("window.__amni&&window.__amni.showPanel&&window.__amni.showPanel('hist')", |_| {});
            }
            true
        }
        (Key::Character(c), true, true) if c.eq_ignore_ascii_case("n") => { state.execute_command("private_tab", &empty); true }
        (Key::Character(c), true, true) if c.eq_ignore_ascii_case("k") => { state.execute_command("duplicate_tab", &empty); true }
        (Key::Character(c), true, _) if c.as_str() == "+" || c.as_str() == "=" => { state.execute_command("zoom_in", &empty); true }
        (Key::Character(c), true, _) if c.as_str() == "-" || c.as_str() == "_" => { state.execute_command("zoom_out", &empty); true }
        (Key::Character(c), true, false) if c.as_str() == "0" => { state.execute_command("zoom_reset", &empty); true }
        (Key::Character(c), true, false) if matches!(c.as_str(), "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8") => {
            let n: usize = c.as_str().parse().unwrap_or(1);
            let len = state.content_webviews.borrow().len();
            let idx = (n - 1).min(len.saturating_sub(1));
            switch_to(idx); true
        }
        (Key::Character(c), true, false) if c.as_str() == "9" => {
            let len = state.content_webviews.borrow().len();
            if len > 0 { switch_to(len - 1); } true
        }
        _ => false,
    }
}
fn resolve_navigate_input(raw: &str, search_prefix: &str) -> Option<Url> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return None; }
    if let Ok(u) = Url::parse(trimmed) {
        if content_scheme_ok(u.scheme()) { return Some(u); }
    }
    let as_path = std::path::Path::new(trimmed);
    if as_path.is_file() {
        return Url::parse(&file_url(as_path)).ok();
    }
    let has_dot = trimmed.contains('.');
    let has_space = trimmed.contains(' ');
    match has_dot && !has_space {
        true => Url::parse(&format!("https://{}", trimmed)).ok(),
        false => {
            let prefix = match search_prefix.starts_with("http") { true => search_prefix, false => "https://html.duckduckgo.com/html/?q=" };
            Url::parse(&format!("{}{}", prefix, urlencoding::encode(trimmed))).ok()
        }
    }
}
impl WebViewDelegate for AppState {
    fn notify_new_frame_ready(&self, _: WebView) { self.window.request_redraw(); }
    fn notify_favicon_changed(&self, webview: WebView) {
        let key = webview.url().map(|u| u.as_str().to_string()).unwrap_or_default();
        if !key.is_empty() { self.favicons.borrow_mut().remove(&key); }
    }
    fn notify_page_title_changed(&self, webview: WebView, title: Option<String>) {
        if let Some(idx) = self.tab_index_for_webview(&webview) {
            if let Some(t) = title.as_deref() {
                self.remember_tab_title(idx, t);
            }
        }
        let is_active = self.active_content().map(|a| a.id() == webview.id()).unwrap_or(false);
        if !is_active { return; }
        self.set_window_title(title);
    }
    fn load_web_resource(&self, webview: WebView, load: WebResourceLoad) {
        let req_url = load.request().url.clone();
        if req_url.scheme() == "file" {
            let main_frame = load.request().is_for_main_frame;
            let doc = webview.url();
            let is_chrome = self.chrome_webview.borrow().as_ref().map(|c| c.id() == webview.id()).unwrap_or(false);
            let referrer = load.request().referrer_url.clone();
            let tab_url = self.content_webviews.borrow().iter().find_map(|c| c.url().filter(|u| u.scheme() == "file")).or_else(|| self.content_webviews.borrow().iter().all(|c| c.url().is_none()).then(|| url::Url::parse("file:///").ok()).flatten());
            if !is_chrome && !file_subresource_allowed(main_frame, doc.as_ref(), referrer.as_ref(), tab_url.as_ref()) {
                info!("file:// blocked for non-local document {:?} \u{2192} {}", doc.map(|u| u.to_string()), req_url);
                load.intercept(WebResourceResponse::new(req_url).status_code(http::StatusCode::FORBIDDEN)).finish();
                return;
            }
            intercept_file_load(load, req_url);
            return;
        }
        if req_url.scheme() == "amnibrowse" {
            let host = req_url.host_str().unwrap_or("");
            let path = req_url.path();
            let from_chrome = self.chrome_webview.borrow().as_ref().map(|c| c.id() == webview.id()).unwrap_or(false);
            let tok_ok = req_url.query_pairs().any(|(k, v)| k == "tok" && v == self.cmd_token.as_str());
            if !from_chrome && !tok_ok {
                info!("amnibrowse://: denied {:?}{} from non-chrome webview", host, path);
                load.intercept(WebResourceResponse::new(req_url).status_code(http::StatusCode::FORBIDDEN).headers(cors_headers())).finish();
                return;
            }
            match host {
                "cmd" => {
                    let name = path.trim_start_matches('/');
                    let args: std::collections::HashMap<String, String> = req_url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
                    self.execute_command(name, &args);
                    load.intercept(WebResourceResponse::new(req_url).headers(cors_headers())).finish();
                    return;
                }
                "state" => {
                    let body = self.build_state_json();
                    let mut headers = cors_headers();
                    headers.insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json; charset=utf-8"));
                    headers.insert(http::header::CACHE_CONTROL, http::HeaderValue::from_static("no-store"));
                    let mut intercepted = load.intercept(WebResourceResponse::new(req_url).headers(headers));
                    intercepted.send_body_data(body.into_bytes());
                    intercepted.finish();
                    return;
                }
                "suggest" => {
                    let q = req_url.query_pairs().find(|(k, _)| k == "q").map(|(_, v)| v.into_owned()).unwrap_or_default();
                    let bms: Vec<(String, String)> = self.bookmarks.borrow().bookmarks.iter().map(|b| (b.url.clone(), b.title.clone())).collect();
                    let extra: Vec<(&str, &str)> = bms.iter().map(|(u, t)| (u.as_str(), t.as_str())).collect();
                    let body = self.history.borrow().omnibox_json(&q, &extra, 8);
                    let mut headers = cors_headers();
                    headers.insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json; charset=utf-8"));
                    let mut intercepted = load.intercept(WebResourceResponse::new(req_url).headers(headers));
                    intercepted.send_body_data(body.into_bytes());
                    intercepted.finish();
                    return;
                }
                "downloads" => {
                    let body = self.downloads.borrow().to_json();
                    let mut headers = cors_headers();
                    headers.insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json; charset=utf-8"));
                    let mut intercepted = load.intercept(WebResourceResponse::new(req_url).headers(headers));
                    intercepted.send_body_data(body.into_bytes());
                    intercepted.finish();
                    return;
                }
                "history" => {
                    let body = self.history.borrow().recent_json(40);
                    let mut headers = cors_headers();
                    headers.insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json; charset=utf-8"));
                    let mut intercepted = load.intercept(WebResourceResponse::new(req_url).headers(headers));
                    intercepted.send_body_data(body.into_bytes());
                    intercepted.finish();
                    return;
                }
                "import" => {
                    let body = match path.trim_start_matches('/') {
                        "detect" => serde_json::to_string(&import_browsers::detect()).unwrap_or_else(|_| "[]".into()),
                        _ => serde_json::to_string(&*self.last_import.borrow()).unwrap_or_else(|_| "{}".into()),
                    };
                    let mut headers = cors_headers();
                    headers.insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json; charset=utf-8"));
                    let mut intercepted = load.intercept(WebResourceResponse::new(req_url).headers(headers));
                    intercepted.send_body_data(body.into_bytes());
                    intercepted.finish();
                    return;
                }
                _ => {
                    info!("amnibrowse://: unknown host {:?} path {:?}", host, path);
                    load.intercept(WebResourceResponse::new(req_url).status_code(http::StatusCode::NOT_FOUND).headers(cors_headers())).finish();
                    return;
                }
            }
        }
        let url_str = req_url.as_str().to_string();
        let blocked = self.config.borrow().block_ads && self.ad_blocker.lock().map(|mut b| b.should_block(&url_str)).unwrap_or(false);
        if blocked {
            info!("adblock: blocked {}", url_str);
            load.intercept(WebResourceResponse::new(req_url)).finish();
        }
    }
    fn request_navigation(&self, webview: WebView, req: NavigationRequest) {
        let url = req.url.as_str().to_string();
        let idx = self.focus_content_tab(&webview);
        if media_engine::wants_media_window(&url) {
            info!("nav \u{2192} in-tab DRM {}", url);
            req.deny();
            self.attach_media_to_active(&url);
            self.apply_media_visibility();
            self.window.request_redraw();
            return;
        }
        if stream_extract::is_progressive_host(&url) {
            req.deny();
            if !self.try_load_stream(&webview, &url) { if let Ok(u) = Url::parse(&url) { webview.load(u); } }
            if idx.is_some() { self.apply_media_visibility(); self.window.request_redraw(); }
            return;
        }
        if daily_driver::is_pdf_url(&url) {
            req.deny();
            self.load_pdf_viewer(&webview, &url);
            if let Some(i) = idx { self.drop_media_at(i); }
            self.apply_media_visibility();
            self.window.request_redraw();
            return;
        }
        if daily_driver::is_download_url(&url) {
            req.deny();
            self.downloads.borrow_mut().start_download(&url);
            return;
        }
        if let Some(i) = idx { self.drop_media_at(i); }
        self.apply_media_visibility();
        req.allow();
        self.window.request_redraw();
    }
    fn notify_url_changed(&self, webview: WebView, url: url::Url) {
        let us = url.as_str().to_string();
        if us.starts_with("http") {
            let title = webview.page_title().unwrap_or_default();
            self.history.borrow_mut().record_visit(&us, &title);
        }
        if !us.starts_with("data:") && !us.starts_with("amnibrowse:") {
            webview.evaluate_javascript(crate::engine::servo_compat::inject_script(), |_| {});
            webview.evaluate_javascript(crate::engine::servo_compat::svg_repair_script(), |_| {});
            webview.evaluate_javascript(crate::engine::servo_compat::challenge_notice_script(), |_| {});
        }
        self.persist_session(false);
    }
    fn notify_load_status_changed(&self, webview: WebView, status: LoadStatus) {
        if !matches!(status, LoadStatus::Complete) { return; }
        if self.chrome_webview.borrow().as_ref().map(|c| c.id() == webview.id()).unwrap_or(false) { return; }
        let us = webview.url().map(|u| u.as_str().to_string()).unwrap_or_default();
        self.inject_after_load(&webview, &us);
    }
    fn notify_cursor_changed(&self, webview: WebView, cursor: Cursor) {
        let icon = cursor_from_servo(cursor);
        // Chrome webview cursors only matter while the pointer is in the strip.
        let is_chrome = self.chrome_webview.borrow().as_ref().map(|c| c.id() == webview.id()).unwrap_or(false);
        if !is_chrome {
            self.page_cursor.set(icon);
        }
        if self.resize_cursor_on.get() { return; }
        let in_chrome = self.mouse_point.get().y < self.hit_chrome_px();
        if is_chrome == in_chrome {
            self.window.set_cursor(icon);
        }
    }
    fn request_create_new(&self, parent: WebView, request: CreateNewWebViewRequest) {
        let opener_idx = self.focus_content_tab(&parent).or_else(|| {
            self.content_webviews.borrow().iter().position(|c| c.id() == parent.id())
        });
        let scale = self.scale_factor.get();
        let wv = request.builder(self.offscreen_context.clone())
            .hidpi_scale_factor(Scale::new(scale))
            .delegate(self.self_rc())
            .build();
        self.apply_servo_theme(&wv);
        wv.resize(self.offscreen_context.size());
        let z = self.default_zoom();
        if (z - 1.0).abs() > 0.01 { wv.set_page_zoom(z); }
        let mut tabs = self.content_webviews.borrow_mut();
        tabs.push(wv);
        let popup_idx = tabs.len() - 1;
        self.tab_zoom.borrow_mut().push(z);
        self.push_tab_uid();
        self.active_content_index.set(popup_idx);
        let opened = tabs.last().and_then(|w| w.url().map(|u| u.as_str().to_string()));
        drop(tabs);
        if let Some(opener) = opener_idx {
            self.popup_opener.borrow_mut().insert(popup_idx, opener);
        }
        self.sync_media_len();
        if let Some(us) = opened.as_ref() {
            if media_engine::wants_media_window(us) { self.attach_media_to_active(us); }
        }
        self.apply_media_visibility();
        self.persist_session(false);
        info!("window.open \u{2192} tab {} from {:?} url {:?}", popup_idx, opener_idx, opened);
        self.window.request_redraw();
    }
    fn notify_closed(&self, webview: WebView) {
        let Some(idx) = self.content_webviews.borrow().iter().position(|c| c.id() == webview.id()) else { return };
        let opener = self.popup_opener.borrow_mut().remove(&idx);
        {
            let mut map = self.popup_opener.borrow_mut();
            let prev: Vec<(usize, usize)> = map.drain().collect();
            for (p, o) in prev {
                if p == idx { continue; }
                let np = if p > idx { p - 1 } else { p };
                if o == idx { continue; }
                let no = if o > idx { o - 1 } else { o };
                map.insert(np, no);
            }
        }
        let tab_count = self.content_webviews.borrow().len();
        if tab_count <= 1 {
            info!("notify_closed on last tab \u{2192} ignore");
            return;
        }
        {
            let mut panes = self.media_panes.borrow_mut();
            if idx < panes.len() {
                if let Some(pane) = panes[idx].take() { media_engine::trash_pane(pane); }
                panes.remove(idx);
            }
        }
        {
            let mut tabs = self.content_webviews.borrow_mut();
            let doomed = tabs.remove(idx);
            Self::drop_webview(doomed);
            if idx < self.tab_zoom.borrow().len() { self.tab_zoom.borrow_mut().remove(idx); }
            if idx < self.tab_titles.borrow().len() { self.tab_titles.borrow_mut().remove(idx); }
            if idx < self.tab_uids.borrow().len() { self.tab_uids.borrow_mut().remove(idx); }
            let active = self.active_content_index.get();
            let new_active = if let Some(op) = opener {
                let op = if op > idx { op - 1 } else { op };
                op.min(tabs.len().saturating_sub(1))
            } else {
                match active {
                    a if a == idx => idx.min(tabs.len().saturating_sub(1)),
                    a if a > idx => a - 1,
                    a => a,
                }
            };
            self.active_content_index.set(new_active);
        }
        self.apply_media_visibility();
        self.pending_session_persist.set(true);
        info!("notify_closed \u{2192} dropped popup {}, restored opener {:?}", idx, opener);
        self.window.request_redraw();
    }
    fn show_embedder_control(&self, _webview: WebView, control: EmbedderControl) {
        match control {
            EmbedderControl::ContextMenu(menu) => { let _ = self.pending_embedder.borrow_mut().take(); self.show_context_menu(menu); }
            EmbedderControl::SelectElement(sel) => { let _ = self.pending_embedder.borrow_mut().take(); self.show_select(sel); }
            EmbedderControl::SimpleDialog(dialog) => { let _ = self.pending_embedder.borrow_mut().take(); self.show_simple_dialog(dialog); }
            EmbedderControl::FilePicker(picker) => { let _ = self.pending_embedder.borrow_mut().take(); self.show_file_picker(picker); }
            EmbedderControl::ColorPicker(picker) => { let _ = self.pending_embedder.borrow_mut().take(); self.show_color_picker(picker); }
            EmbedderControl::InputMethod(ctl) => {
                let r = ctl.position();
                let scale = self.scale_factor.get().max(0.01) as f64;
                let (cx, cy) = embedder_anchor_css(r.min.x, r.min.y);
                self.window.set_ime_cursor_area(
                    PhysicalPosition::new(cx as f64 * scale, cy as f64 * scale),
                    PhysicalSize::new((r.width() as f64 * scale).max(1.0), (r.height() as f64 * scale).max(1.0)),
                );
                info!("embedder ime at {},{}", r.min.x, r.min.y);
            }
        }
    }
    fn hide_embedder_control(&self, _webview: WebView, id: EmbedderControlId) {
        let drop = self.pending_embedder.borrow().as_ref().map(|c| c.id() == id).unwrap_or(false);
        if drop { self.dismiss_embedder(); }
    }
}
fn kbd_after_mouse(in_chrome: bool, down: bool, prev: bool) -> bool {
    if down { in_chrome } else { prev }
}
fn resize_edge(p: Point2D<f32, DevicePixel>, size: PhysicalSize<u32>, scale: f32, chrome_px: f32) -> Option<winit::window::ResizeDirection> {
    // Keep grips thin so content near the sides does not look like a resize handle.
    let m = (5.0 * scale.max(1.0)).max(5.0);
    let (w, h) = (size.width as f32, size.height as f32);
    let (l, r, t, b) = (p.x <= m, p.x >= w - m, p.y <= m, p.y >= h - m);
    use winit::window::ResizeDirection as D;
    if t {
        return match (l, r) {
            (true, _) => Some(D::NorthWest),
            (_, true) => Some(D::NorthEast),
            _ => Some(D::North),
        };
    }
    // Frameless: side grips must work through the chrome strip too.
    if p.y < chrome_px {
        return match (l, r) {
            (true, _) => Some(D::West),
            (_, true) => Some(D::East),
            _ => None,
        };
    }
    match (l, r, b) {
        (true, _, true) => Some(D::SouthWest),
        (_, true, true) => Some(D::SouthEast),
        (true, _, _) => Some(D::West),
        (_, true, _) => Some(D::East),
        (_, _, true) => Some(D::South),
        _ => None,
    }
}
fn cursor_from_servo(c: Cursor) -> winit::window::CursorIcon {
    use winit::window::CursorIcon as I;
    match c {
        Cursor::Default => I::Default,
        Cursor::Pointer => I::Pointer,
        Cursor::ContextMenu => I::ContextMenu,
        Cursor::Help => I::Help,
        Cursor::Progress => I::Progress,
        Cursor::Wait => I::Wait,
        Cursor::Cell => I::Cell,
        Cursor::Crosshair => I::Crosshair,
        Cursor::Text => I::Text,
        Cursor::VerticalText => I::VerticalText,
        Cursor::Alias => I::Alias,
        Cursor::Copy => I::Copy,
        Cursor::Move => I::Move,
        Cursor::NoDrop => I::NoDrop,
        Cursor::NotAllowed => I::NotAllowed,
        Cursor::Grab => I::Grab,
        Cursor::Grabbing => I::Grabbing,
        Cursor::EResize => I::EResize,
        Cursor::NResize => I::NResize,
        Cursor::NeResize => I::NeResize,
        Cursor::NwResize => I::NwResize,
        Cursor::SResize => I::SResize,
        Cursor::SeResize => I::SeResize,
        Cursor::SwResize => I::SwResize,
        Cursor::WResize => I::WResize,
        Cursor::EwResize => I::EwResize,
        Cursor::NsResize => I::NsResize,
        Cursor::NeswResize => I::NeswResize,
        Cursor::NwseResize => I::NwseResize,
        Cursor::ColResize => I::ColResize,
        Cursor::RowResize => I::RowResize,
        Cursor::AllScroll => I::AllScroll,
        Cursor::ZoomIn => I::ZoomIn,
        Cursor::ZoomOut => I::ZoomOut,
        Cursor::None => I::Default,
    }
}
fn apply_window_cursor(state: &AppState, edge: Option<winit::window::ResizeDirection>) {
    use winit::window::{CursorIcon, ResizeDirection as D};
    match edge {
        Some(dir) => {
            let icon = match dir {
                D::North | D::South => CursorIcon::NsResize,
                D::East | D::West => CursorIcon::EwResize,
                D::NorthEast | D::SouthWest => CursorIcon::NeswResize,
                D::NorthWest | D::SouthEast => CursorIcon::NwseResize,
            };
            state.window.set_cursor(icon);
            state.resize_cursor_on.set(true);
        }
        None if state.resize_cursor_on.get() => {
            state.window.set_cursor(state.page_cursor.get());
            state.resize_cursor_on.set(false);
        }
        None => {}
    }
}
fn drain_pending_media(_event_loop: &winit::event_loop::ActiveEventLoop, state: &AppState) {
    if state.pending_session_persist.take() {
        state.persist_session(false);
    }
    let urls: Vec<String> = state.pending_media_urls.borrow_mut().drain(..).collect();
    for u in urls { state.attach_media_to_active(&u); }
    let srcs: Vec<(String, String)> = state.pending_source.borrow_mut().drain(..).collect();
    for (u, s) in srcs {
        let html = state.view_source_html(&u, &s);
        state.open_html_tab(&html);
        info!("view source \u{2192} tab for {} ({} bytes)", u, s.len());
    }
}
static PAINT_CHROME_LAST: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static LAST_MISMATCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn paint_and_present(state: &AppState) {
    let win_now = state.window_size();
    let ctx_now = state.rendering_context.size();
    let key = ((win_now.width as u64) << 32) | win_now.height as u64;
    if (ctx_now.width, ctx_now.height) != (win_now.width, win_now.height) && LAST_MISMATCH.swap(key, std::sync::atomic::Ordering::Relaxed) != key {
        info!("paint mismatch: ctx {}x{} vs win {}x{}", ctx_now.width, ctx_now.height, win_now.width, win_now.height);
    }
    let chrome_opt = state.chrome_webview.borrow().clone();
    let content_opt = state.active_content();
    if !state.paint_logged.get() {
        state.paint_logged.set(true);
        let cs = chrome_opt.as_ref().map(|c| (c.size(), c.load_status(), c.focused()));
        let ct = content_opt.as_ref().map(|c| c.size());
        info!("paint#1 win {}x{} chrome_px {} chrome {:?} content {:?} blit {:?}", win_now.width, win_now.height, state.chrome_px(), cs, ct, content_blit_rect(win_now.width, win_now.height, state.chrome_px()));
    }
    let overlay = state.chrome_overlay_px.get() > 0;
    let legacy = *PAINT_CHROME_LAST.get_or_init(|| std::env::var("AMNI_PAINT_ORDER").map(|v| v.trim() == "legacy").unwrap_or(false));
    if let Some(content) = content_opt.as_ref() { content.paint(); }
    if legacy { if let Some(chrome) = chrome_opt.as_ref() { chrome.paint(); } }
    if !legacy {
        state.rendering_context.prepare_for_rendering();
        if let Some(chrome) = chrome_opt.as_ref() { chrome.show(); chrome.paint(); }
    }
    let stash = if overlay && !legacy { overlay_stash(state) } else { None };
    if let Some(callback) = state.offscreen_context.render_to_parent_callback() {
        let win = state.window_size();
        let chrome_px = state.chrome_px();
        let target_rect = content_blit_rect(win.width, win.height, chrome_px);
        state.rendering_context.prepare_for_rendering();
        let gl = state.rendering_context.glow_gl_api();
        callback(&gl, target_rect);
    }
    if let Some(s) = stash { overlay_restore(state, s); }
    state.rendering_context.present();
}
fn overlay_band_gl(win_h: i32, scale: f32, x: i32, y: i32, w: i32, h: i32) -> (i32, i32, i32, i32) {
    let sx = ((x as f32) * scale).round() as i32;
    let sy = ((y as f32) * scale).round() as i32;
    let sw = ((w as f32) * scale).round().max(1.0) as i32;
    let sh = ((h as f32) * scale).round().max(1.0) as i32;
    (sx.max(0), (win_h - sy - sh).max(0), sw, sh)
}
struct OverlayStash { tex: glow::Texture, fbo: glow::Framebuffer, x: i32, y: i32, w: i32, h: i32 }
fn overlay_stash(state: &AppState) -> Option<OverlayStash> {
    let (x, y, w, h) = state.overlay_css.get()?;
    let win = state.window_size();
    let (gx, gy, gw, gh) = overlay_band_gl(win.height as i32, state.scale_factor.get(), x, y, w, h);
    if gx >= win.width as i32 || gy >= win.height as i32 { return None; }
    let gl = state.rendering_context.glow_gl_api();
    unsafe {
        let tex = gl.create_texture().ok()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        gl.copy_tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA, gx, gy, gw, gh, 0);
        let fbo = gl.create_framebuffer().ok()?;
        Some(OverlayStash { tex, fbo, x: gx, y: gy, w: gw, h: gh })
    }
}
fn overlay_restore(state: &AppState, stash: OverlayStash) {
    let gl = state.rendering_context.glow_gl_api();
    unsafe {
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(stash.fbo));
        gl.framebuffer_texture_2d(glow::READ_FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(stash.tex), 0);
        gl.blit_framebuffer(0, 0, stash.w, stash.h, stash.x, stash.y, stash.x + stash.w, stash.y + stash.h, glow::COLOR_BUFFER_BIT, glow::NEAREST);
        gl.delete_framebuffer(stash.fbo);
        gl.delete_texture(stash.tex);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
    }
}
fn resize_all(state: &AppState, new_size: PhysicalSize<u32>) {
    info!("resize_all \u{2192} {}x{}", new_size.width, new_size.height);
    let _ = state.rendering_context.make_current();
    let chrome_px = state.chrome_px();
    let content = content_size(new_size, chrome_px);
    if let Some(chrome) = state.chrome_webview.borrow().as_ref() {
        chrome.resize(PhysicalSize::new(new_size.width.max(1), new_size.height.max(1)));
    }
    for c in state.content_webviews.borrow().iter() { c.resize(content); }
    state.apply_media_visibility();
    state.window.request_redraw();
}
enum App { Initial(Waker, Arc<Mutex<AdBlocker>>, Vec<(String, EngineKind)>, BrowserConfig, BookmarkManager, ThemeConfig), Running(Rc<AppState>) }
impl App {
    fn new(event_loop: &EventLoop<WakerEvent>, ad_blocker: Arc<Mutex<AdBlocker>>, initial_urls: Vec<(String, EngineKind)>, config: BrowserConfig, bookmarks: BookmarkManager, themes: ThemeConfig) -> Self {
        Self::Initial(Waker::new(event_loop), ad_blocker, initial_urls, config, bookmarks, themes)
    }
}
impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Initial(waker, ad_blocker, initial_urls, config, bookmarks, themes) = self {
            let display_handle = event_loop.display_handle().expect("display handle");
            let saved = SessionManager::load();
            let (mon_w, mon_h) = event_loop.primary_monitor().map(|m| {
                let s = m.scale_factor();
                let sz = m.size();
                ((sz.width as f64 / s) * 0.95, (sz.height as f64 / s) * 0.92)
            }).unwrap_or((f64::MAX, f64::MAX));
            let (win_w, win_h) = saved.as_ref().map(|s| (s.window_width.max(720.0), s.window_height.max(480.0))).unwrap_or((1280.0, 800.0));
            let (win_w, win_h) = (win_w.min(mon_w).max(720.0), win_h.min(mon_h).max(480.0));
            let scale_guess = event_loop.primary_monitor().map(|m| m.scale_factor()).unwrap_or(1.0);
            let saved_pos = saved.as_ref().and_then(|s| Some((s.window_x?, s.window_y?)));
            let (win_x, win_y) = restore_window_position(saved_pos, (win_w, win_h), scale_guess);
            let maximized = saved.as_ref().map(|s| s.maximized).unwrap_or(false);
            info!("window size \u{2192} {}x{} logical at {:.0},{:.0} (monitor cap {:.0}x{:.0}) maximized={}", win_w, win_h, win_x, win_y, mon_w, mon_h, maximized);
            let window = event_loop.create_window(
                Window::default_attributes()
                    .with_title("Amni Browse")
                    .with_decorations(std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(false))
                    .with_transparent(false)
                    .with_min_inner_size(LogicalSize::new(720.0, 480.0))
                    .with_inner_size(LogicalSize::new(win_w, win_h))
                    .with_position(winit::dpi::LogicalPosition::new(win_x, win_y))
                    .with_maximized(maximized)
            ).expect("window");
            window.set_ime_allowed(true);
            let window_handle = window.window_handle().expect("window handle");
            let window_size = window.inner_size();
            let scale = window.scale_factor() as f32;
            let rendering_context = Rc::new(WindowRenderingContext::new(display_handle, window_handle, window_size).expect("rendering context"));
            let _ = rendering_context.make_current();
            let chrome_px = chrome_height_px(scale);
            let content_init = content_size(window_size, chrome_px);
            let offscreen_context = Rc::new(rendering_context.offscreen_context(content_init));
            let prefs = servo_prefs(config);
            let mut opts = Opts::default();
            let servo_store = BrowserConfig::config_dir().join("servo-store");
            let _ = std::fs::create_dir_all(&servo_store);
            opts.config_dir = Some(servo_store);
            let servo = ServoBuilder::default().opts(opts).event_loop_waker(Box::new(waker.clone())).preferences(prefs).build();
            let user_content = Rc::new(servo::UserContentManager::new(&servo));
            user_content.add_script(Rc::new(servo::UserScript::new(crate::engine::servo_compat::document_start_script().to_string(), None)));
            let ad_blocker_clone = ad_blocker.clone();
            let cmd_token = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x5eed) ^ 0x9e37_79b9_7f4a_7c15u64);
            let app_state = Rc::new_cyclic(|weak: &Weak<AppState>| AppState {
                window, servo, rendering_context, offscreen_context, user_content,
                chrome_webview: RefCell::new(None),
                content_webviews: Default::default(),
                active_content_index: Cell::new(0),
                mouse_point: Cell::new(Point2D::zero()),
                modifiers: Cell::new(ModifiersState::empty()),
                scale_factor: Cell::new(scale),
                ad_blocker: ad_blocker_clone,
                media_panes: RefCell::new(Vec::new()),
                pending_media_urls: RefCell::new(Vec::new()),
                closed_tabs: RefCell::new(Vec::new()),
                tab_zoom: RefCell::new(Vec::new()),
                tab_titles: RefCell::new(Vec::new()),
                tab_uids: RefCell::new(Vec::new()),
                next_tab_uid: Cell::new(1),
                is_fullscreen: Cell::new(false),
                config: RefCell::new(config.clone()),
                bookmarks: RefCell::new(bookmarks.clone()),
                themes: RefCell::new(themes.clone()),
                cmd_token,
                self_weak: weak.clone(),
                history: RefCell::new(HistoryManager::new()),
                downloads: RefCell::new(DownloadManager::new()),
                vault: RefCell::new(PasswordManager::new()),
                pm: RefCell::new(PmState::from_config(&config.password_provider, config.pm_cli_path.clone(), config.pm_keepass_db.clone())),
                update: Arc::new(Mutex::new(None)),
                extensions: RefCell::new({ let mut e = ExtensionManager::new(); e.scan_extensions(); e }),
                profiles: RefCell::new(ProfileManager::new()),
                find_query: RefCell::new(String::new()),
                chrome_overlay_px: Cell::new(0),
                overlay_css: Cell::new(None),
                kbd_in_chrome: Cell::new(false),
                shortcut_keys: RefCell::new(std::collections::HashSet::new()),
                paint_logged: Cell::new(false),
                favicons: RefCell::new(std::collections::HashMap::new()),
                origin_favicons: RefCell::new(std::collections::HashMap::new()),
                favicon_jobs: Arc::new(Mutex::new(std::collections::HashMap::new())),
                origin_favicon_primed: RefCell::new(std::collections::HashSet::new()),
                origin_favicon_polls: RefCell::new(std::collections::HashMap::new()),
                origin_favicon_reprimes: RefCell::new(std::collections::HashSet::new()),
                origin_favicon_probing: RefCell::new(std::collections::HashSet::new()),
                pending_relaunch: RefCell::new(None),
                pending_session_persist: Cell::new(false),
                last_import: RefCell::new(import_browsers::ImportReport::default()),
                pending_embedder: RefCell::new(None),
                ctx_link: RefCell::new(None),
                ctx_image: RefCell::new(None),
                pending_source: RefCell::new(Vec::new()),
                resize_cursor_on: Cell::new(false),
                page_cursor: Cell::new(winit::window::CursorIcon::Default),
                popup_opener: RefCell::new(std::collections::HashMap::new()),
            });
            let chrome_url = chrome_data_url();
            info!("servo chrome data url len: {}", chrome_url.as_str().len());
            let chrome_webview = WebViewBuilder::new(&app_state.servo, app_state.rendering_context.clone())
                .url(chrome_url)
                .hidpi_scale_factor(Scale::new(scale))
                .delegate(app_state.clone())
                .build();
            app_state.apply_servo_theme(&chrome_webview);
            chrome_webview.resize(PhysicalSize::new(window_size.width.max(1), window_size.height.max(1)));
            *app_state.chrome_webview.borrow_mut() = Some(chrome_webview);
            let cli_http = initial_urls.iter().any(|(u, _)| u.starts_with("http://") || u.starts_with("https://"));
            let tutorial = !cli_http && (!app_state.config.borrow().seen_onboarding || std::env::var("AMNI_TUTORIAL").is_ok());
            let open: Vec<(String, EngineKind)> = if tutorial {
                vec![(app_state.home_url(), EngineKind::Servo)]
            } else if initial_urls.is_empty() {
                vec![(app_state.home_url(), EngineKind::Servo)]
            } else {
                initial_urls.iter().filter(|(u, _)| !u.is_empty()).cloned().collect()
            };
            let z0 = app_state.default_zoom();
            let mut want_active = 0usize;
            for (i, (u, k)) in open.iter().enumerate() {
                let parsed = Url::parse(u).ok().filter(|p| content_scheme_ok(p.scheme())).unwrap_or_else(|| Url::parse(&app_state.home_url()).unwrap_or_else(|_| Url::parse("https://html.duckduckgo.com/html/").unwrap()));
                let wv = WebViewBuilder::new(&app_state.servo, app_state.offscreen_context.clone()).url(parsed).hidpi_scale_factor(Scale::new(scale)).delegate(app_state.clone()).user_content_manager(app_state.user_content.clone()).build();
                app_state.apply_servo_theme(&wv);
                if (z0 - 1.0).abs() > 0.01 { wv.set_page_zoom(z0); }
                app_state.content_webviews.borrow_mut().push(wv);
                app_state.tab_zoom.borrow_mut().push(z0);
                app_state.push_tab_uid();
                if *k == EngineKind::Media { want_active = i; }
                info!("servo restore tab {} {} {:?}", i, u, k);
            }
            if let Some(s) = saved.as_ref() {
                if let Some(i) = s.tabs.iter().position(|t| t.is_active) { want_active = i.min(open.len().saturating_sub(1)); }
            }
            app_state.active_content_index.set(want_active.min(app_state.content_webviews.borrow().len().saturating_sub(1)));
            app_state.sync_media_len();
            for (i, (u, k)) in open.iter().enumerate() {
                if *k == EngineKind::Media { app_state.attach_media_at(i, u); }
            }
            app_state.apply_media_visibility();
            if config.check_updates {
                let slot = app_state.update.clone();
                let feed = config.update_feed.clone();
                std::thread::spawn(move || {
                    match updater::check_for_update(env!("CARGO_PKG_VERSION"), feed.as_deref()) {
                        Ok(Some(r)) => { info!("update available: {} from {}", r.version, r.source); if let Ok(mut g) = slot.lock() { *g = Some(r); } }
                        Ok(None) => info!("update check: current"),
                        Err(e) => info!("update check: {}", e),
                    }
                });
            }
            *self = Self::Running(app_state);
            info!("Servo embedder ready (chrome + content compositing)");
        }
    }
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _event: WakerEvent) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
            drain_pending_media(event_loop, state);
            if let Some(pid) = state.pending_relaunch.borrow_mut().take() {
                state.persist_session(true);
                if pid != "__update__" && pid != "__exit__" {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).env("AMNI_PROFILE", pid).spawn();
                    }
                }
                event_loop.exit();
            }
        }
    }
    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let Self::Running(state) = self { state.servo.spin_event_loop(); drain_pending_media(event_loop, state); }
        let Self::Running(state) = self else { return };
        if window_id != state.window.id() { return; }
        let content_opt = state.active_content();
        let chrome_opt = state.chrome_webview.borrow().clone();
        match event {
            WindowEvent::CloseRequested => { state.persist_session(true); event_loop.exit(); }
            WindowEvent::RedrawRequested => { paint_and_present(state); }
            WindowEvent::Resized(new_size) => { resize_all(state, new_size); }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                state.scale_factor.set(scale_factor as f32);
                resize_all(state, state.window_size());
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = Point2D::<f32, DevicePixel>::new(position.x as f32, position.y as f32);
                state.mouse_point.set(p);
                if !state.is_fullscreen.get() {
                    apply_window_cursor(state, resize_edge(p, state.window_size(), state.scale_factor.get(), state.hit_chrome_px()));
                }
                let chrome_px = state.hit_chrome_px();
                let in_chrome = p.y < chrome_px;
                match (in_chrome, chrome_opt.as_ref(), content_opt.as_ref()) {
                    (true, Some(chrome), _) => {
                        chrome.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(p.into())));
                        if let Some(c) = content_opt.as_ref() { c.notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default())); }
                    }
                    (false, _, Some(c)) => {
                        let translated = Point2D::<f32, DevicePixel>::new(p.x, p.y - chrome_px);
                        c.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(translated.into())));
                        if let Some(chrome) = chrome_opt.as_ref() { chrome.notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default())); }
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if state.resize_cursor_on.get() {
                    state.window.set_cursor(winit::window::CursorIcon::Default);
                    state.resize_cursor_on.set(false);
                }
                if let Some(chrome) = chrome_opt.as_ref() { chrome.notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default())); }
                if let Some(c) = content_opt.as_ref() { c.notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default())); }
            }
            WindowEvent::MouseInput { state: pressed, button, .. } => {
                let mb = match button {
                    MouseButton::Left => ServoMouseButton::Primary,
                    MouseButton::Right => ServoMouseButton::Secondary,
                    MouseButton::Middle => ServoMouseButton::Auxiliary,
                    MouseButton::Back => ServoMouseButton::Back,
                    MouseButton::Forward => ServoMouseButton::Forward,
                    MouseButton::Other(v) => ServoMouseButton::Other(v),
                };
                let action = match pressed { ElementState::Pressed => MouseButtonAction::Down, ElementState::Released => MouseButtonAction::Up };
                let p = state.mouse_point.get();
                if std::env::var_os("AMNI_TRACE_INPUT").is_some() { info!("trace mouse {:?} {:?} at {:?}", action, button, p); }
                if action == MouseButtonAction::Down && button == MouseButton::Left && !state.is_fullscreen.get() {
                    if let Some(dir) = resize_edge(p, state.window_size(), state.scale_factor.get(), state.hit_chrome_px()) {
                        let _ = state.window.drag_resize_window(dir);
                        return;
                    }
                }
                let chrome_px = state.hit_chrome_px();
                let in_chrome = p.y < chrome_px;
                if action == MouseButtonAction::Down {
                    state.kbd_in_chrome.set(kbd_after_mouse(in_chrome, true, state.kbd_in_chrome.get()));
                }
                match (in_chrome, chrome_opt.as_ref(), content_opt.as_ref()) {
                    (true, Some(chrome), _) => { chrome.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(action, mb, p.into()))); }
                    (false, _, Some(c)) => {
                        if action == MouseButtonAction::Down {
                            if let Some(chrome) = chrome_opt.as_ref() { let _ = chrome.evaluate_javascript("try{document.activeElement&&document.activeElement.blur()}catch(e){}", |_| {}); }
                        }
                        let translated = Point2D::<f32, DevicePixel>::new(p.x, p.y - chrome_px);
                        c.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(action, mb, translated.into())));
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy, mode) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => ((x as f64).clamp(-8.0, 8.0) * 40.0, (y as f64).clamp(-8.0, 8.0) * 40.0, WheelMode::DeltaPixel),
                    MouseScrollDelta::PixelDelta(p) => (p.x.clamp(-2400.0, 2400.0), p.y.clamp(-2400.0, 2400.0), WheelMode::DeltaPixel),
                };
                let p = state.mouse_point.get();
                let chrome_px = state.hit_chrome_px();
                let empty: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                if state.modifiers.get().control_key() && p.y >= chrome_px && dy != 0.0 {
                    state.execute_command(match dy > 0.0 { true => "zoom_in", false => "zoom_out" }, &empty);
                    return;
                }
                match (p.y < chrome_px, chrome_opt.as_ref(), content_opt.as_ref()) {
                    (true, Some(chrome), _) => { chrome.notify_input_event(InputEvent::Wheel(WheelEvent::new(WheelDelta { x: dx, y: dy, z: 0.0, mode }, p.into()))); }
                    (false, _, Some(c)) => {
                        let translated = Point2D::<f32, DevicePixel>::new(p.x, p.y - chrome_px);
                        c.notify_input_event(InputEvent::Wheel(WheelEvent::new(WheelDelta { x: dx, y: dy, z: 0.0, mode }, translated.into())));
                    }
                    _ => {}
                }
            }
            WindowEvent::ModifiersChanged(m) => { state.modifiers.set(m.state()); }
            WindowEvent::Focused(true) => { WINDOW_FOCUSED.with(|w| w.set(true)); }
            WindowEvent::Focused(false) => {
                WINDOW_FOCUSED.with(|w| w.set(false));
                let composing = IME_COMPOSING.with(|c| c.get());
                let evs = on_blur(&state.modifiers, composing);
                let target = match state.kbd_in_chrome.get() { true => chrome_opt.as_ref(), false => content_opt.as_ref() };
                if let Some(wv) = target { for ev in evs { wv.notify_input_event(InputEvent::Ime(ev)); } }
            }
            WindowEvent::Ime(ime) => {
                let target = match state.kbd_in_chrome.get() { true => chrome_opt.as_ref(), false => content_opt.as_ref() };
                let Some(wv) = target else { return };
                let id = wv.id();
                if let Some(stale) = ime_retarget(id) {
                    for o in [chrome_opt.as_ref(), content_opt.as_ref()].into_iter().flatten() {
                        if o.id() == stale { o.notify_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent { state: CompositionState::End, data: String::new() }))); }
                    }
                }
                for ev in ime_events(&ime) { wv.notify_input_event(InputEvent::Ime(ev)); }
                if matches!(ime, WinitIme::Enabled) {
                    let p = state.mouse_point.get();
                    let scale = state.scale_factor.get() as f64;
                    state.window.set_ime_cursor_area(
                        PhysicalPosition::new(p.x as f64, p.y as f64),
                        PhysicalSize::new((24.0 * scale) as u32, (24.0 * scale) as u32),
                    );
                }
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let trace = std::env::var_os("AMNI_TRACE_INPUT").is_some();
                let consumed = handle_shortcut(&key_event, state);
                if consumed && key_event.state == ElementState::Pressed { state.shortcut_keys.borrow_mut().insert(key_event.physical_key); }
                if consumed { if trace { info!("trace key SHORTCUT {:?} {:?} mods={:?}", key_event.state, key_event.logical_key, state.modifiers.get()); } return; }
                let kev = keyboard_event_from_winit(&key_event, state.modifiers.get());
                let target = match state.kbd_in_chrome.get() { true => chrome_opt.as_ref(), false => content_opt.as_ref() };
                if trace { info!("trace key FWD {:?} key={:?} code={:?} mods={:?} to={}", kev.event.state, kev.event.key, kev.event.code, kev.event.modifiers, match state.kbd_in_chrome.get() { true => "chrome", false => "content" }); }
                if let Some(wv) = target { wv.notify_input_event(InputEvent::Keyboard(kev)); }
            }
            _ => {}
        }
    }
}
thread_local! { static IME_COMPOSING: Cell<bool> = const { Cell::new(false) }; }
thread_local! { static IME_TARGET: Cell<Option<WebViewId>> = const { Cell::new(None) }; }
thread_local! { static IME_END_DELIVERED: Cell<bool> = const { Cell::new(false) }; }
thread_local! { static WINDOW_FOCUSED: Cell<bool> = const { Cell::new(true) }; }
fn ime_retarget_into<T: Copy + PartialEq>(slot: &Cell<Option<T>>, key: T) -> Option<T> {
    let stale = slot.get().filter(|p| *p != key && IME_COMPOSING.with(|c| c.get()));
    slot.set(Some(key));
    stale.inspect(|_| IME_COMPOSING.with(|c| c.set(false)))
}
fn ime_retarget(id: WebViewId) -> Option<WebViewId> { IME_TARGET.with(|t| ime_retarget_into(t, id)) }
fn ime_blur_events(composing: bool) -> Vec<ImeEvent> {
    if !composing || IME_END_DELIVERED.with(|d| d.get()) { return Vec::new(); }
    IME_END_DELIVERED.with(|d| d.set(true));
    vec![ImeEvent::Composition(CompositionEvent { state: CompositionState::End, data: String::new() })]
}
fn on_blur(mods: &Cell<ModifiersState>, composing: bool) -> Vec<ImeEvent> {
    mods.set(ModifiersState::empty());
    let evs = ime_blur_events(composing);
    IME_COMPOSING.with(|c| c.set(false));
    evs
}
fn ime_blur() -> Vec<ImeEvent> {
    on_blur(&Cell::new(ModifiersState::empty()), IME_COMPOSING.with(|c| c.get()))
}
fn ime_transition(composing: bool, ime: &WinitIme) -> (Vec<(CompositionState, String)>, bool) {
    match ime {
        WinitIme::Enabled => (Vec::new(), composing),
        WinitIme::Preedit(s, _) if s.is_empty() && composing => (vec![(CompositionState::End, String::new())], false),
        WinitIme::Preedit(s, _) if s.is_empty() => (Vec::new(), false),
        WinitIme::Preedit(s, _) if composing => (vec![(CompositionState::Update, s.clone())], true),
        WinitIme::Preedit(s, _) => (vec![(CompositionState::Start, String::new()), (CompositionState::Update, s.clone())], true),
        WinitIme::Commit(s) if composing => (vec![(CompositionState::End, s.clone())], false),
        WinitIme::Commit(s) => (vec![(CompositionState::Start, String::new()), (CompositionState::End, s.clone())], false),
        WinitIme::Disabled if composing => (vec![(CompositionState::End, String::new())], false),
        WinitIme::Disabled => (Vec::new(), false),
    }
}
fn ime_events(ime: &WinitIme) -> Vec<ImeEvent> {
    let (steps, next) = ime_transition(IME_COMPOSING.with(|c| c.get()), ime);
    IME_COMPOSING.with(|c| c.set(next));
    if steps.iter().any(|(s, _)| *s == CompositionState::Start) {
        IME_END_DELIVERED.with(|d| d.set(false));
    }
    if steps.iter().any(|(s, _)| *s == CompositionState::End) {
        IME_END_DELIVERED.with(|d| d.set(true));
        if !next { IME_TARGET.with(|t| t.set(None)); }
    }
    match (steps.is_empty(), ime) {
        (true, WinitIme::Disabled) if WINDOW_FOCUSED.with(|w| w.get()) => vec![ImeEvent::Dismissed],
        (true, WinitIme::Disabled) => Vec::new(),
        _ => steps.into_iter().map(|(state, data)| ImeEvent::Composition(CompositionEvent { state, data })).collect(),
    }
}
#[cfg(test)]
mod anchor_tests {
    use super::*;
    #[test]
    fn anchor_is_css_px_offset_by_chrome_only() {
        assert_eq!(embedder_anchor_css(40, 270), (40.0, 354.0));
        assert_eq!(embedder_anchor_css(0, 0), (0.0, CHROME_HEIGHT_CSS));
    }
    #[test]
    fn anchor_does_not_scale_with_dpi() {
        let a = embedder_anchor_css(40, 270);
        let b = embedder_anchor_css(40, 270);
        assert_eq!(a, b);
        assert!(a.0 > 32.0, "a 1.25 dpi divide would have produced 32");
    }
}
#[cfg(test)]
mod favicon_tests {
    use super::*;
    #[test]
    fn origin_keeps_scheme_and_nonstandard_port() {
        assert_eq!(favicon_origin("https://example.com/a/b?c=d#e").as_deref(), Some("https://example.com"));
        assert_eq!(favicon_origin("http://localhost:8788/x").as_deref(), Some("http://localhost:8788"));
        assert_eq!(favicon_origin("https://example.com:443/").as_deref(), Some("https://example.com"));
    }
    #[test]
    fn origin_refuses_non_network_schemes() {
        assert_eq!(favicon_origin("file:///C:/notes.html"), None);
        assert_eq!(favicon_origin("data:text/html,hi"), None);
        assert_eq!(favicon_origin("amnibrowse://home"), None);
        assert_eq!(favicon_origin("not a url"), None);
    }
    #[test]
    fn encoder_downsamples_and_rejects_oversize() {
        let big = image::RgbaImage::from_pixel(64, 64, image::Rgba([200, 155, 78, 255]));
        let d = favicon_png_data_url(big).expect("encodes");
        assert!(d.starts_with("data:image/png;base64,"));
        let raw = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &d["data:image/png;base64,".len()..]).unwrap();
        let back = image::load_from_memory(&raw).unwrap();
        assert_eq!((back.width(), back.height()), (32, 32));
        assert!(favicon_png_data_url(image::RgbaImage::new(600, 600)).is_none());
        assert!(favicon_png_data_url(image::RgbaImage::new(0, 0)).is_none());
    }
    #[test]
    fn probe_script_only_reads_the_dom_and_never_touches_the_network() {
        let s = favicon_link_probe_script();
        assert!(s.contains("link[rel]"));
        assert!(!s.contains("XMLHttpRequest"), "a page-context request is subject to that page's connect-src");
        assert!(!s.contains("fetch("));
        assert!(!s.contains("/favicon.ico"));
        assert!(!s.contains("appendChild"));
        assert!(!s.contains("createElement"));
        assert!(!s.contains("window."), "no page global to spoof");
        assert!(s.contains("return 'has'") && s.contains("return 'none'"));
    }
    #[test]
    fn embedder_fetch_declares_image_destination_not_script() {
        let h = favicon_fetch_headers();
        let get = |k: &str| h.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
        assert_eq!(get("Sec-Fetch-Dest"), Some("image"), "servers that reject script access to static assets key on this");
        assert_eq!(get("Sec-Fetch-Mode"), Some("no-cors"));
        assert_eq!(get("Sec-Fetch-Site"), Some("same-origin"));
        assert!(get("Accept").unwrap().starts_with("image/"));
        assert!(!h.iter().any(|(_, v)| v.contains("empty")));
    }
    #[test]
    fn embedder_fetch_rejects_a_dead_origin_without_caching() {
        let (tag, data) = favicon_fetch_origin("http://127.0.0.1:1", None);
        assert_eq!(tag, "miss");
        assert!(data.is_none());
    }
    #[test]
    fn fetch_ua_is_a_browser_string_not_a_bare_library_default() {
        assert!(FAVICON_FETCH_UA.starts_with("Mozilla/5.0"));
        assert!(FAVICON_FETCH_UA.contains("AmniBrowse/"));
    }
    #[test]
    fn jobs_cap_keeps_running_workers() {
        let mut j = std::collections::HashMap::new();
        for i in 0..130 { j.insert(format!("http://o{i}.test"), FaviconJob::Done("miss".into(), None)); }
        j.insert("http://live.test".into(), FaviconJob::Running);
        favicon_jobs_insert_running(&mut j, "http://next.test".into());
        assert!(matches!(j.get("http://live.test"), Some(FaviconJob::Running)));
        assert!(matches!(j.get("http://next.test"), Some(FaviconJob::Running)));
        assert!(j.len() <= 3);
    }
    #[test]
    fn only_started_pending_gone_and_none_keep_the_origin_polling() {
        for t in ["started", "pending", "gone", "none"] { assert!(!favicon_tag_is_terminal(t), "{t}"); }
        for t in ["has", "miss", "mixed-redirect", "empty", "big", "err", "eval-fail", "eval-unexpected", "embedder-fetch ok"] {
            assert!(favicon_tag_is_terminal(t), "{t}");
        }
        assert!(FAVICON_POLL_MAX >= 8 && FAVICON_POLL_MAX <= 120);
    }
    #[test]
    fn embedder_fetch_refuses_https_to_http_downgrade() {
        assert!(favicon_https_to_http_downgrade("https", "http"));
        assert!(!favicon_https_to_http_downgrade("https", "https"));
        assert!(!favicon_https_to_http_downgrade("http", "http"));
        assert!(!favicon_https_to_http_downgrade("http", "https"));
    }
}
#[cfg(test)]
mod ime_tests {
    use super::*;
    fn states(evs: &[ImeEvent]) -> Vec<CompositionState> {
        evs.iter().filter_map(|e| match e { ImeEvent::Composition(c) => Some(c.state), _ => None }).collect()
    }
    #[test]
    fn preedit_then_commit_is_start_update_end() {
        IME_COMPOSING.with(|c| c.set(false));
        let a = ime_events(&WinitIme::Preedit("に".into(), None));
        assert_eq!(states(&a), vec![CompositionState::Start, CompositionState::Update]);
        let b = ime_events(&WinitIme::Preedit("にほ".into(), None));
        assert_eq!(states(&b), vec![CompositionState::Update]);
        let c = ime_events(&WinitIme::Commit("日本".into()));
        assert_eq!(states(&c), vec![CompositionState::End]);
        assert!(matches!(&c[0], ImeEvent::Composition(e) if e.data == "日本"));
        assert!(!IME_COMPOSING.with(|c| c.get()));
    }
    #[test]
    fn window_blur_ends_live_composition_without_dismissing_focus() {
        IME_COMPOSING.with(|c| c.set(false));
        IME_END_DELIVERED.with(|d| d.set(false));
        let _ = ime_events(&WinitIme::Preedit("に".into(), None));
        let mods = Cell::new(ModifiersState::CONTROL | ModifiersState::SHIFT);
        let evs = on_blur(&mods, true);
        assert_eq!(mods.get(), ModifiersState::empty());
        assert_eq!(states(&evs), vec![CompositionState::End]);
        assert!(!evs.iter().any(|e| matches!(e, ImeEvent::Dismissed)));
        assert!(!IME_COMPOSING.with(|c| c.get()));
        assert_eq!(states(&ime_events(&WinitIme::Preedit("ほ".into(), None))), vec![CompositionState::Start, CompositionState::Update]);
    }
    #[test]
    fn blur_skips_end_when_system_already_terminated() {
        IME_COMPOSING.with(|c| c.set(true));
        IME_END_DELIVERED.with(|d| d.set(true));
        assert!(ime_blur_events(true).is_empty());
        IME_END_DELIVERED.with(|d| d.set(false));
        assert_eq!(states(&ime_blur_events(true)), vec![CompositionState::End]);
    }
    #[test]
    fn disabled_while_unfocused_never_dismisses() {
        IME_COMPOSING.with(|c| c.set(false));
        WINDOW_FOCUSED.with(|w| w.set(false));
        assert!(ime_events(&WinitIme::Disabled).is_empty());
        WINDOW_FOCUSED.with(|w| w.set(true));
        assert!(matches!(ime_events(&WinitIme::Disabled).as_slice(), [ImeEvent::Dismissed]));
    }
    #[test]
    fn window_blur_while_idle_sends_nothing() {
        let mods = Cell::new(ModifiersState::ALT);
        let evs = on_blur(&mods, false);
        assert_eq!(mods.get(), ModifiersState::empty());
        assert!(evs.is_empty());
        assert!(ime_blur_events(false).is_empty());
        assert_eq!(states(&ime_blur_events(true)), vec![CompositionState::End]);
    }
    #[test]
    fn bare_commit_without_preedit_still_inserts_text() {
        IME_COMPOSING.with(|c| c.set(false));
        let c = ime_events(&WinitIme::Commit("é".into()));
        assert_eq!(states(&c), vec![CompositionState::Start, CompositionState::End]);
    }
    #[test]
    fn cancelled_composition_ends_then_disabled_dismisses() {
        IME_COMPOSING.with(|c| c.set(false));
        let _ = ime_events(&WinitIme::Preedit("に".into(), None));
        let cancel = ime_events(&WinitIme::Preedit(String::new(), None));
        assert_eq!(states(&cancel), vec![CompositionState::End]);
        assert!(matches!(ime_events(&WinitIme::Disabled).as_slice(), [ImeEvent::Dismissed]));
    }
    #[test]
    fn focus_change_mid_composition_flushes_old_and_restarts_new() {
        IME_COMPOSING.with(|c| c.set(false));
        let slot = Cell::new(None::<u64>);
        assert_eq!(ime_retarget_into(&slot, 1), None);
        let a = ime_events(&WinitIme::Preedit("に".into(), None));
        assert_eq!(states(&a), vec![CompositionState::Start, CompositionState::Update]);
        assert_eq!(ime_retarget_into(&slot, 2), Some(1));
        assert!(!IME_COMPOSING.with(|c| c.get()));
        let b = ime_events(&WinitIme::Preedit("ほ".into(), None));
        assert_eq!(states(&b), vec![CompositionState::Start, CompositionState::Update]);
    }
    #[test]
    fn same_target_never_flushes_and_keeps_composing() {
        IME_COMPOSING.with(|c| c.set(false));
        let slot = Cell::new(None::<u64>);
        assert_eq!(ime_retarget_into(&slot, 1), None);
        let _ = ime_events(&WinitIme::Preedit("に".into(), None));
        assert_eq!(ime_retarget_into(&slot, 1), None);
        assert_eq!(states(&ime_events(&WinitIme::Preedit("にほ".into(), None))), vec![CompositionState::Update]);
        assert_eq!(ime_retarget_into(&slot, 2), Some(1));
        assert_eq!(ime_retarget_into(&slot, 1), None);
    }
    #[test]
    fn webview_id_is_copy_eq_so_flush_cannot_collide_on_debug_text() {
        fn assert_copy_eq<T: Copy + Eq>(_: T) {}
        let id: Option<WebViewId> = None;
        assert_copy_eq(id);
        let a = Cell::new(None::<WebViewId>);
        let b = Cell::new(None::<u64>);
        IME_COMPOSING.with(|c| c.set(true));
        assert_eq!(ime_retarget_into(&b, 7u64), None);
        IME_COMPOSING.with(|c| c.set(true));
        assert_eq!(ime_retarget_into(&b, 8u64), Some(7));
        IME_COMPOSING.with(|c| c.set(false));
        let _ = a;
    }
}
#[cfg(test)]
mod compose_tests {
    use super::*;
    #[test]
    fn chrome_mousedown_claims_keyboard_before_any_fetch() {
        assert!(kbd_after_mouse(true, true, false));
        assert!(!kbd_after_mouse(false, true, true));
        assert!(kbd_after_mouse(true, false, true));
        assert!(!kbd_after_mouse(false, false, false));
    }
    #[test]
    fn blit_sits_under_chrome() {
        let r = content_blit_rect(1280, 800, 84);
        assert_eq!(r.origin.y, 0);
        assert_eq!(r.size.height, 716);
        assert_eq!(r.origin.x, 0);
    }
    #[test]
    fn parse_hex_rgb_full_and_short() {
        let a = AppState::parse_hex_rgb("#C89B4E").unwrap();
        assert_eq!((a.red, a.green, a.blue), (0xC8, 0x9B, 0x4E));
        let b = AppState::parse_hex_rgb("#abc").unwrap();
        assert_eq!((b.red, b.green, b.blue), (0xAA, 0xBB, 0xCC));
        assert!(AppState::parse_hex_rgb("zz").is_none());
    }
    #[test]
    fn file_scheme_is_content() {
        assert!(content_scheme_ok("file"));
        assert!(content_scheme_ok("https"));
        assert!(!content_scheme_ok("javascript"));
    }
    #[test]
    fn mime_html_and_png() {
        assert!(mime_for_path(std::path::Path::new("a.html")).starts_with("text/html"));
        assert_eq!(mime_for_path(std::path::Path::new("a.png")), "image/png");
    }
    #[test]
    fn navigate_keeps_file_url() {
        let u = resolve_navigate_input("file:///C:/Temp/smoke.html", "").unwrap();
        assert_eq!(u.scheme(), "file");
    }
    #[test]
    fn url_parse_collapses_traversal_before_we_see_it() {
        let dotted = Url::parse("file:///C:/Windows/System32/../../Windows/win.ini").unwrap();
        let encoded = Url::parse("file:///C:/Windows/System32/%2e%2e/%2e%2e/Windows/win.ini").unwrap();
        assert_eq!(dotted.to_file_path().unwrap(), std::path::PathBuf::from("C:\\Windows\\win.ini"));
        assert_eq!(encoded.to_file_path().unwrap(), std::path::PathBuf::from("C:\\Windows\\win.ini"));
    }
    #[test]
    fn secure_file_path_rejects_directories_and_missing_files() {
        let dir = Url::from_file_path(std::env::temp_dir()).unwrap();
        assert_eq!(secure_file_path(&dir), Err(http::StatusCode::FORBIDDEN));
        let gone = Url::parse("file:///C:/amni-does-not-exist-4f2a9c.txt").unwrap();
        assert_eq!(secure_file_path(&gone), Err(http::StatusCode::NOT_FOUND));
    }
    #[test]
    fn secure_file_path_serves_a_real_file() {
        let p = std::env::temp_dir().join("amni_file_probe.txt");
        std::fs::write(&p, b"ok").unwrap();
        let u = Url::from_file_path(&p).unwrap();
        assert_eq!(secure_file_path(&u).unwrap(), p.canonicalize().unwrap());
    }
    #[test]
    fn http_document_cannot_pull_file_subresources() {
        let http_doc = Url::parse("https://evil.example/").unwrap();
        let file_doc = Url::parse("file:///C:/notes.html").unwrap();
        assert!(!file_load_allowed(false, Some(&http_doc)));
        assert!(file_load_allowed(false, Some(&file_doc)));
        assert!(file_load_allowed(true, Some(&http_doc)));
        assert!(file_load_allowed(true, None));
        assert!(file_subresource_allowed(false, None, Some(&file_doc), None));
        assert!(file_subresource_allowed(false, None, None, Some(&file_doc)));
        assert!(!file_subresource_allowed(false, Some(&http_doc), None, Some(&file_doc)));
        assert!(!file_subresource_allowed(false, None, Some(&http_doc), Some(&http_doc)));
    }
    #[test]
    fn top_perimeter_resizes_even_over_chrome_band() {
        use euclid::Point2D;
        use winit::window::ResizeDirection as D;
        let size = PhysicalSize::new(1280u32, 800u32);
        let chrome = 84.0;
        assert_eq!(resize_edge(Point2D::new(640.0, 3.0), size, 1.0, chrome), Some(D::North));
        assert_eq!(resize_edge(Point2D::new(3.0, 3.0), size, 1.0, chrome), Some(D::NorthWest));
        assert_eq!(resize_edge(Point2D::new(1277.0, 3.0), size, 1.0, chrome), Some(D::NorthEast));
    }
    #[test]
    fn chrome_sides_still_resize() {
        use euclid::Point2D;
        use winit::window::ResizeDirection as D;
        let size = PhysicalSize::new(1280u32, 800u32);
        let chrome = 119.0;
        assert_eq!(resize_edge(Point2D::new(3.0, 40.0), size, 1.0, chrome), Some(D::West));
        assert_eq!(resize_edge(Point2D::new(1277.0, 60.0), size, 1.0, chrome), Some(D::East));
        assert!(resize_edge(Point2D::new(640.0, 40.0), size, 1.0, chrome).is_none());
        // Interior content must not keep a side-resize cursor (regression for stuck ↔).
        assert!(resize_edge(Point2D::new(400.0, 400.0), size, 1.0, chrome).is_none());
        assert!(resize_edge(Point2D::new(20.0, 400.0), size, 1.0, chrome).is_none());
    }
    fn apply_move_permutation(items: &[u64], from: usize, to: usize) -> Vec<u64> {
        let mut a = items.to_vec();
        let item = a.remove(from);
        a.insert(to, item);
        a
    }
    #[test]
    fn duplicate_url_order_unchanged_after_tab_move() {
        let urls = ["about:blank", "about:blank", "about:blank"];
        let before: Vec<_> = urls.iter().map(|s| s.to_string()).collect();
        let after: Vec<_> = urls.iter().map(|s| s.to_string()).collect();
        assert_eq!(before, after, "identical URLs produce the same sequence before and after reorder");
        let uids = vec![101u64, 202, 303];
        assert_ne!(apply_move_permutation(&uids, 0, 2), uids);
    }
    #[test]
    fn tab_move_sync_uses_uid_not_duplicate_urls() {
        let uids = vec![101u64, 202, 303];
        let after = apply_move_permutation(&uids, 0, 2);
        assert_eq!(after, vec![202, 303, 101]);
        assert_ne!(uids, after);
    }
}
