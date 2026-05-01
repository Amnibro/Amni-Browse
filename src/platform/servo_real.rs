use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use euclid::{Point2D, Scale};
use euclid::default::{Point2D as DefaultPoint2D, Rect as DefaultRect, Size2D as DefaultSize2D};
use servo::{
    DevicePixel, EventLoopWaker, InputEvent, LoadStatus, MouseButton as ServoMouseButton, MouseButtonAction,
    MouseButtonEvent, MouseLeftViewportEvent, MouseMoveEvent, NavigationRequest, OffscreenRenderingContext,
    RenderingContext, Servo, ServoBuilder, WebResourceLoad, WebResourceResponse, WebView, WebViewBuilder,
    WebViewDelegate, WheelDelta, WheelEvent, WheelMode, WindowRenderingContext,
};
use url::Url;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::event::KeyEvent;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Fullscreen, Window, WindowId};
use log::info;
use crate::app::BrowserState;
use crate::engine::adblocker::AdBlocker;
use crate::engine::tabs::TabEngine;
use crate::platform::media_engine::{self, EngineKind, MediaWindow};
use crate::platform::servo_keys::keyboard_event_from_winit;
const CHROME_HEIGHT_CSS: f32 = 74.0;
const TOOLBAR_HTML: &str = include_str!("../../assets/chrome/toolbar.html");
fn chrome_data_url() -> Url {
    let encoded = urlencoding::encode(TOOLBAR_HTML);
    Url::parse(&format!("data:text/html;charset=utf-8,{}", encoded)).expect("chrome data url")
}
fn chrome_height_px(scale: f32) -> u32 { (CHROME_HEIGHT_CSS * scale).round().max(1.0) as u32 }
fn content_size(window_size: PhysicalSize<u32>, chrome_px: u32) -> PhysicalSize<u32> {
    PhysicalSize::new(window_size.width.max(1), window_size.height.saturating_sub(chrome_px).max(1))
}
pub fn run(state: BrowserState) {
    info!("Amni Browse Real Servo backend initializing...");
    info!("Media engine platform: {}", media_engine::platform_label());
    let event_loop = EventLoop::<WakerEvent>::with_user_event().build().expect("event loop");
    let ad_blocker = Arc::new(Mutex::new(state.ad_blocker.clone()));
    let mut initial_urls: Vec<(String, EngineKind)> = state.tabs.tabs.iter().map(|t| {
        let kind = match t.engine { TabEngine::Media => EngineKind::Media, _ => media_engine::route(&t.url) };
        (t.url.clone(), kind)
    }).collect();
    if let Ok(test_url) = std::env::var("AMNI_TEST_MEDIA_URL") { info!("AMNI_TEST_MEDIA_URL set \u{2192} injecting media tab: {}", test_url); initial_urls.push((test_url, EngineKind::Media)); }
    let mut app = App::new(&event_loop, ad_blocker, initial_urls);
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
    rendering_context: Rc<WindowRenderingContext>,
    offscreen_context: Rc<OffscreenRenderingContext>,
    chrome_webview: RefCell<Option<WebView>>,
    content_webviews: RefCell<Vec<WebView>>,
    active_content_index: Cell<usize>,
    mouse_point: Cell<Point2D<f32, DevicePixel>>,
    modifiers: Cell<ModifiersState>,
    scale_factor: Cell<f32>,
    ad_blocker: Arc<Mutex<AdBlocker>>,
    media_windows: RefCell<HashMap<WindowId, MediaWindow>>,
    pending_media_urls: RefCell<Vec<String>>,
    closed_tabs: RefCell<Vec<Url>>,
    tab_zoom: RefCell<Vec<f32>>,
    is_fullscreen: Cell<bool>,
    self_weak: Weak<AppState>,
}
impl AppState {
    fn chrome_px(&self) -> u32 { chrome_height_px(self.scale_factor.get()) }
    fn window_size(&self) -> PhysicalSize<u32> { self.window.inner_size() }
    fn active_content(&self) -> Option<WebView> {
        let tabs = self.content_webviews.borrow();
        let idx = self.active_content_index.get().min(tabs.len().saturating_sub(1));
        tabs.get(idx).cloned()
    }
    fn parse_tab_index(id: &str) -> Option<usize> { id.strip_prefix('t').and_then(|s| s.parse().ok()) }
    fn self_rc(&self) -> Rc<AppState> { self.self_weak.upgrade().expect("AppState alive") }
    fn spawn_content_webview(&self, url: Url) -> WebView {
        let scale = self.scale_factor.get();
        let wv = WebViewBuilder::new(&self.servo, self.offscreen_context.clone())
            .url(url)
            .hidpi_scale_factor(Scale::new(scale))
            .delegate(self.self_rc())
            .build();
        wv.resize(self.offscreen_context.size());
        wv
    }
    fn execute_command(&self, name: &str, args: &std::collections::HashMap<String, String>) {
        match name {
            "back" => { if let Some(c) = self.active_content() { if c.can_go_back() { let _ = c.go_back(1); info!("cmd back"); } } }
            "forward" => { if let Some(c) = self.active_content() { if c.can_go_forward() { let _ = c.go_forward(1); info!("cmd forward"); } } }
            "reload" => { if let Some(c) = self.active_content() { c.reload(); info!("cmd reload"); } }
            "navigate" => {
                let raw = args.get("url").cloned().unwrap_or_default();
                match resolve_navigate_input(&raw) {
                    Some(u) => { if let Some(c) = self.active_content() { info!("cmd navigate \u{2192} {}", u); c.load(u); } }
                    None => info!("cmd navigate: empty/invalid input"),
                }
            }
            "new_tab" => {
                let raw = args.get("url").cloned().unwrap_or_else(|| "https://duckduckgo.com".into());
                let start = Url::parse(&raw).unwrap_or_else(|_| Url::parse("https://duckduckgo.com").unwrap());
                let wv = self.spawn_content_webview(start);
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(1.0);
                self.active_content_index.set(tabs.len() - 1);
                info!("cmd new_tab \u{2192} idx {}", tabs.len() - 1);
                self.window.request_redraw();
            }
            "reopen_tab" => {
                let Some(url) = self.closed_tabs.borrow_mut().pop() else { info!("cmd reopen_tab: stack empty"); return };
                let wv = self.spawn_content_webview(url.clone());
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(1.0);
                self.active_content_index.set(tabs.len() - 1);
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
            "stop" => { if let Some(c) = self.active_content() { c.reload(); info!("cmd stop (reload as proxy)"); } }
            "switch_tab" => {
                let Some(id) = args.get("id") else { return };
                let Some(idx) = Self::parse_tab_index(id) else { return };
                let len = self.content_webviews.borrow().len();
                if idx < len { self.active_content_index.set(idx); info!("cmd switch_tab \u{2192} idx {}", idx); self.window.request_redraw(); }
            }
            "close_tab" => {
                let Some(id) = args.get("id") else { return };
                let Some(idx) = Self::parse_tab_index(id) else { return };
                let mut tabs = self.content_webviews.borrow_mut();
                if idx >= tabs.len() || tabs.len() <= 1 { info!("cmd close_tab: refusing (idx {} of {})", idx, tabs.len()); return; }
                if let Some(u) = tabs[idx].url() { self.closed_tabs.borrow_mut().push(u); }
                tabs.remove(idx);
                let mut zooms = self.tab_zoom.borrow_mut();
                if idx < zooms.len() { zooms.remove(idx); }
                drop(zooms);
                let active = self.active_content_index.get();
                let new_active = match active {
                    a if a == idx => idx.min(tabs.len() - 1),
                    a if a > idx => a - 1,
                    a => a,
                };
                self.active_content_index.set(new_active);
                info!("cmd close_tab \u{2192} removed {}, active now {}", idx, new_active);
                self.window.request_redraw();
            }
            "bookmark" => info!("cmd bookmark (stub)"),
            "menu" => info!("cmd menu (stub)"),
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
impl AppState {
    fn build_state_json(&self) -> String {
        let content_opt = self.active_content();
        let (url, title, loading, can_back, can_forward) = match content_opt.as_ref() {
            Some(c) => (
                c.url().map(|u| u.as_str().to_string()).unwrap_or_default(),
                c.page_title().unwrap_or_default(),
                !matches!(c.load_status(), LoadStatus::Complete),
                c.can_go_back(),
                c.can_go_forward(),
            ),
            None => (String::new(), String::new(), false, false, false),
        };
        let active_idx = self.active_content_index.get();
        let tabs: Vec<serde_json::Value> = self.content_webviews.borrow().iter().enumerate().map(|(i, c)| {
            serde_json::json!({
                "id": format!("t{}", i),
                "url": c.url().map(|u| u.as_str().to_string()).unwrap_or_default(),
                "title": c.page_title().unwrap_or_else(|| "New Tab".into()),
                "active": i == active_idx,
                "loading": !matches!(c.load_status(), LoadStatus::Complete),
                "engine": "servo",
            })
        }).collect();
        let media_tabs: Vec<serde_json::Value> = self.media_windows.borrow().iter().enumerate().map(|(i, (_wid, _mw))| {
            serde_json::json!({
                "id": format!("m{}", i),
                "url": "",
                "title": "Media",
                "active": false,
                "loading": false,
                "engine": "media",
            })
        }).collect();
        let mut all_tabs = tabs;
        all_tabs.extend(media_tabs);
        let zoom = self.tab_zoom.borrow().get(active_idx).copied().unwrap_or(1.0);
        serde_json::json!({
            "url": url,
            "title": title,
            "loading": loading,
            "canBack": can_back,
            "canForward": can_forward,
            "tabs": all_tabs,
            "zoom": zoom,
            "fullscreen": self.is_fullscreen.get(),
            "canReopen": !self.closed_tabs.borrow().is_empty(),
        }).to_string()
    }
}
fn handle_shortcut(key_event: &KeyEvent, state: &AppState) -> bool {
    if key_event.state != ElementState::Pressed { return false; }
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
        (Key::Named(NamedKey::Escape), false, false) => { if state.is_fullscreen.get() { state.execute_command("fullscreen", &empty); true } else { false } }
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
            if let Some(chrome) = state.chrome_webview.borrow().as_ref() {
                let _ = chrome.evaluate_javascript("document.getElementById('url').focus();document.getElementById('url').select();", |_| {});
            }
            true
        }
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
fn resolve_navigate_input(raw: &str) -> Option<Url> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return None; }
    if let Ok(u) = Url::parse(trimmed) { return Some(u); }
    let has_dot = trimmed.contains('.');
    let has_space = trimmed.contains(' ');
    match has_dot && !has_space {
        true => Url::parse(&format!("https://{}", trimmed)).ok(),
        false => Url::parse(&format!("https://duckduckgo.com/?q={}", urlencoding::encode(trimmed))).ok(),
    }
}
impl WebViewDelegate for AppState {
    fn notify_new_frame_ready(&self, _: WebView) { self.window.request_redraw(); }
    fn notify_page_title_changed(&self, webview: WebView, title: Option<String>) {
        let is_active = self.active_content().map(|a| a.id() == webview.id()).unwrap_or(false);
        if !is_active { return; }
        let t = title.unwrap_or_default();
        let display = match t.trim().is_empty() { true => "Amni Browse".to_string(), false => format!("{} \u{2014} Amni Browse", t) };
        self.window.set_title(&display);
    }
    fn load_web_resource(&self, _webview: WebView, load: WebResourceLoad) {
        let req_url = load.request().url.clone();
        if req_url.scheme() == "amnibrowse" {
            let host = req_url.host_str().unwrap_or("");
            let path = req_url.path();
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
                _ => {
                    info!("amnibrowse://: unknown host {:?} path {:?}", host, path);
                    load.intercept(WebResourceResponse::new(req_url).status_code(http::StatusCode::NOT_FOUND).headers(cors_headers())).finish();
                    return;
                }
            }
        }
        let url_str = req_url.as_str().to_string();
        let blocked = self.ad_blocker.lock().map(|mut b| b.should_block(&url_str)).unwrap_or(false);
        if blocked {
            info!("adblock: blocked {}", url_str);
            load.intercept(WebResourceResponse::new(req_url)).finish();
        }
    }
    fn request_navigation(&self, _webview: WebView, req: NavigationRequest) {
        let url = req.url.as_str().to_string();
        let lower = url.to_lowercase();
        let is_embed = lower.contains("/embed/") || lower.contains("/embed?") || lower.contains("player.") || lower.contains("/player/");
        let is_media = media_engine::route(&url) == EngineKind::Media && !is_embed;
        match is_media {
            true => { info!("nav \u{2192} media engine: {}", url); self.pending_media_urls.borrow_mut().push(url); req.deny(); }
            false => req.allow(),
        }
    }
}
fn drain_pending_media(event_loop: &winit::event_loop::ActiveEventLoop, state: &AppState) {
    let urls: Vec<String> = state.pending_media_urls.borrow_mut().drain(..).collect();
    for u in urls {
        if let Some((id, mw)) = media_engine::spawn_media_window(event_loop, &u) {
            state.media_windows.borrow_mut().insert(id, mw);
        }
    }
}
fn paint_and_present(state: &AppState) {
    let chrome_opt = state.chrome_webview.borrow().clone();
    let content_opt = state.active_content();
    if let Some(chrome) = chrome_opt.as_ref() { chrome.paint(); }
    if let Some(content) = content_opt.as_ref() { content.paint(); }
    if let Some(callback) = state.offscreen_context.render_to_parent_callback() {
        let win = state.window_size();
        let chrome_px = state.chrome_px();
        let content_h = win.height.saturating_sub(chrome_px).max(1);
        let target_rect = DefaultRect::new(
            DefaultPoint2D::new(0i32, 0i32),
            DefaultSize2D::new(win.width as i32, content_h as i32),
        );
        state.rendering_context.prepare_for_rendering();
        let gl = state.rendering_context.glow_gl_api();
        callback(&gl, target_rect);
    }
    state.rendering_context.present();
}
fn resize_all(state: &AppState, new_size: PhysicalSize<u32>) {
    state.rendering_context.resize(new_size);
    let chrome_px = state.chrome_px();
    let content = content_size(new_size, chrome_px);
    state.offscreen_context.resize(content);
    if let Some(chrome) = state.chrome_webview.borrow().as_ref() { chrome.resize(new_size); }
    for c in state.content_webviews.borrow().iter() { c.resize(content); }
}
enum App { Initial(Waker, Arc<Mutex<AdBlocker>>, Vec<(String, EngineKind)>), Running(Rc<AppState>) }
impl App {
    fn new(event_loop: &EventLoop<WakerEvent>, ad_blocker: Arc<Mutex<AdBlocker>>, initial_urls: Vec<(String, EngineKind)>) -> Self {
        Self::Initial(Waker::new(event_loop), ad_blocker, initial_urls)
    }
}
impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Initial(waker, ad_blocker, initial_urls) = self {
            let display_handle = event_loop.display_handle().expect("display handle");
            let window = event_loop.create_window(Window::default_attributes().with_title("Amni Browse \u{2014} Servo")).expect("window");
            let window_handle = window.window_handle().expect("window handle");
            let window_size = window.inner_size();
            let scale = window.scale_factor() as f32;
            let rendering_context = Rc::new(WindowRenderingContext::new(display_handle, window_handle, window_size).expect("rendering context"));
            let _ = rendering_context.make_current();
            let chrome_px = chrome_height_px(scale);
            let content_init = content_size(window_size, chrome_px);
            let offscreen_context = Rc::new(rendering_context.offscreen_context(content_init));
            let servo = ServoBuilder::default().event_loop_waker(Box::new(waker.clone())).build();
            let ad_blocker_clone = ad_blocker.clone();
            let app_state = Rc::new_cyclic(|weak: &Weak<AppState>| AppState {
                window, servo, rendering_context, offscreen_context,
                chrome_webview: RefCell::new(None),
                content_webviews: Default::default(),
                active_content_index: Cell::new(0),
                mouse_point: Cell::new(Point2D::zero()),
                modifiers: Cell::new(ModifiersState::empty()),
                scale_factor: Cell::new(scale),
                ad_blocker: ad_blocker_clone,
                media_windows: RefCell::new(HashMap::new()),
                pending_media_urls: RefCell::new(Vec::new()),
                closed_tabs: RefCell::new(Vec::new()),
                tab_zoom: RefCell::new(Vec::new()),
                is_fullscreen: Cell::new(false),
                self_weak: weak.clone(),
            });
            let chrome_url = chrome_data_url();
            info!("servo chrome data url len: {}", chrome_url.as_str().len());
            let chrome_webview = WebViewBuilder::new(&app_state.servo, app_state.rendering_context.clone())
                .url(chrome_url)
                .hidpi_scale_factor(Scale::new(scale))
                .delegate(app_state.clone())
                .build();
            *app_state.chrome_webview.borrow_mut() = Some(chrome_webview);
            let servo_url = initial_urls.iter().find(|(_, k)| *k == EngineKind::Servo).map(|(u, _)| u.clone())
                .filter(|u| !u.starts_with("amnibrowse://") && u.starts_with("http"))
                .unwrap_or_else(|| "https://duckduckgo.com".into());
            let content_url = Url::parse(&servo_url).unwrap_or_else(|_| Url::parse("https://duckduckgo.com").unwrap());
            info!("servo content initial url: {}", content_url);
            let content_webview = WebViewBuilder::new(&app_state.servo, app_state.offscreen_context.clone())
                .url(content_url)
                .hidpi_scale_factor(Scale::new(scale))
                .delegate(app_state.clone())
                .build();
            app_state.content_webviews.borrow_mut().push(content_webview);
            app_state.tab_zoom.borrow_mut().push(1.0);
            for (u, k) in initial_urls.iter() {
                if *k != EngineKind::Media { continue; }
                if let Some((id, mw)) = media_engine::spawn_media_window(event_loop, u) {
                    app_state.media_windows.borrow_mut().insert(id, mw);
                }
            }
            *self = Self::Running(app_state);
            info!("Servo embedder ready (chrome + content compositing)");
        }
    }
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _event: WakerEvent) {
        if let Self::Running(state) = self { state.servo.spin_event_loop(); drain_pending_media(event_loop, state); }
    }
    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let Self::Running(state) = self { state.servo.spin_event_loop(); drain_pending_media(event_loop, state); }
        let Self::Running(state) = self else { return };
        if window_id != state.window.id() {
            match event {
                WindowEvent::CloseRequested => {
                    let removed = state.media_windows.borrow_mut().remove(&window_id).is_some();
                    if removed { info!("media window {:?} closed", window_id); }
                    if state.media_windows.borrow().is_empty() && state.content_webviews.borrow().is_empty() { event_loop.exit(); }
                }
                WindowEvent::Resized(_) => {}
                _ => {}
            }
            return;
        }
        let content_opt = state.active_content();
        let chrome_opt = state.chrome_webview.borrow().clone();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => { paint_and_present(state); }
            WindowEvent::Resized(new_size) => { resize_all(state, new_size); }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                state.scale_factor.set(scale_factor as f32);
                resize_all(state, state.window_size());
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = Point2D::<f32, DevicePixel>::new(position.x as f32, position.y as f32);
                state.mouse_point.set(p);
                let chrome_px = state.chrome_px() as f32;
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
                if let Some(chrome) = chrome_opt.as_ref() { chrome.notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default())); }
                if let Some(c) = content_opt.as_ref() { c.notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default())); }
            }
            WindowEvent::MouseInput { state: pressed, button, .. } => {
                let mb = match button {
                    MouseButton::Left => ServoMouseButton::Left,
                    MouseButton::Right => ServoMouseButton::Right,
                    MouseButton::Middle => ServoMouseButton::Middle,
                    MouseButton::Back => ServoMouseButton::Back,
                    MouseButton::Forward => ServoMouseButton::Forward,
                    MouseButton::Other(v) => ServoMouseButton::Other(v),
                };
                let action = match pressed { ElementState::Pressed => MouseButtonAction::Down, ElementState::Released => MouseButtonAction::Up };
                let p = state.mouse_point.get();
                let chrome_px = state.chrome_px() as f32;
                match (p.y < chrome_px, chrome_opt.as_ref(), content_opt.as_ref()) {
                    (true, Some(chrome), _) => { chrome.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(action, mb, p.into()))); }
                    (false, _, Some(c)) => {
                        let translated = Point2D::<f32, DevicePixel>::new(p.x, p.y - chrome_px);
                        c.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(action, mb, translated.into())));
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy, mode) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => ((x * 76.0) as f64, (y * 76.0) as f64, WheelMode::DeltaPixel),
                    MouseScrollDelta::PixelDelta(p) => (p.x, p.y, WheelMode::DeltaPixel),
                };
                let p = state.mouse_point.get();
                let chrome_px = state.chrome_px() as f32;
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
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if handle_shortcut(&key_event, state) { return; }
                let kev = keyboard_event_from_winit(&key_event, state.modifiers.get());
                let p = state.mouse_point.get();
                let chrome_px = state.chrome_px() as f32;
                let target = match p.y < chrome_px { true => chrome_opt.as_ref(), false => content_opt.as_ref() };
                if let Some(wv) = target { wv.notify_input_event(InputEvent::Keyboard(kev)); }
            }
            _ => {}
        }
    }
}
