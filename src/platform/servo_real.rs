use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use euclid::{Point2D, Scale};
use euclid::default::{Point2D as DefaultPoint2D, Rect as DefaultRect, Size2D as DefaultSize2D};
use servo::{
    DevicePixel, EventLoopWaker, InputEvent, LoadStatus, MouseButton as ServoMouseButton, MouseButtonAction,
    MouseButtonEvent, MouseLeftViewportEvent, MouseMoveEvent, NavigationRequest, OffscreenRenderingContext,
    Preferences, RenderingContext, Servo, ServoBuilder, WebResourceLoad, WebResourceResponse, WebView, WebViewBuilder,
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
use crate::storage::bookmarks::BookmarkManager;
use crate::storage::config::{BrowserConfig, DEFAULT_SEARCH_ENGINE, LITE_DDG_HOME};
/// Must match `#shell{height:NNpx}` in `assets/chrome/toolbar.html` (66 = 32 tab + 32 nav + 2 progress).
/// Current tree is deliberately 66/66 — do not "fix" by reverting to 74 (edca787 park was about
/// mismatched pairs, not that 66 is wrong when HTML matches).
const CHROME_HEIGHT_CSS: f32 = 66.0;
const TOOLBAR_HTML_EMBEDDED: &str = include_str!("../../assets/chrome/toolbar.html");
/// Rewrite full Next.js DuckDuckGo URLs to the lite HTML endpoint Servo can paint.
fn prefer_servo_friendly_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let Ok(u) = Url::parse(trimmed) else { return trimmed.to_string() };
    let host = u.host_str().unwrap_or("");
    if host != "duckduckgo.com" && host != "www.duckduckgo.com" { return trimmed.to_string(); }
    let path = u.path();
    if path.starts_with("/html") { return trimmed.to_string(); }
    let q = u.query_pairs().find(|(k, _)| k == "q").map(|(_, v)| v.into_owned());
    match q {
        Some(query) => format!("{}?q={}", LITE_DDG_HOME.trim_end_matches('/'), urlencoding::encode(&query)),
        None => LITE_DDG_HOME.to_string(),
    }
}
fn is_client_side_exception_title(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    t.contains("application error") && t.contains("client-side exception")
}

fn parse_shell_height_css(html: &str) -> Option<f32> {
    let marker = "#shell{height:";
    let i = html.find(marker)?;
    let rest = &html[i + marker.len()..];
    let num: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    if num.is_empty() { return None; }
    num.parse().ok()
}
fn assert_chrome_height_synced(html: &str, source: &str) {
    match parse_shell_height_css(html) {
        Some(h) if (h - CHROME_HEIGHT_CSS).abs() < 0.01 => {
            info!("chrome height sync ok: #shell={}px == CHROME_HEIGHT_CSS={} ({})", h, CHROME_HEIGHT_CSS, source);
        }
        Some(h) => {
            log::warn!(
                "CHROME HEIGHT MISMATCH: #shell={}px in {} vs CHROME_HEIGHT_CSS={} — blit/hit-test will disagree (blank viewport risk). Align both.",
                h, source, CHROME_HEIGHT_CSS
            );
        }
        None => log::warn!("chrome toolbar: could not parse #shell height from {} — expected height:{}px", source, CHROME_HEIGHT_CSS),
    }
}
fn load_toolbar_html() -> String {
    if let Ok(content) = std::fs::read_to_string("assets/chrome/toolbar.html") {
        info!("chrome toolbar: loaded from cwd assets/chrome/toolbar.html ({} bytes)", content.len());
        assert_chrome_height_synced(&content, "cwd assets/chrome/toolbar.html");
        return content;
    }
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or(exe);
        let asset_path = exe_dir.join("assets").join("chrome").join("toolbar.html");
        if let Ok(content) = std::fs::read_to_string(&asset_path) {
            info!("chrome toolbar: loaded from {} ({} bytes)", asset_path.display(), content.len());
            assert_chrome_height_synced(&content, &asset_path.display().to_string());
            return content;
        }
    }
    info!("chrome toolbar: using embedded fallback ({} bytes)", TOOLBAR_HTML_EMBEDDED.len());
    assert_chrome_height_synced(TOOLBAR_HTML_EMBEDDED, "embedded fallback");
    TOOLBAR_HTML_EMBEDDED.to_string()
}

fn chrome_data_url() -> Url {
    let html = load_toolbar_html();
    let encoded = urlencoding::encode(&html);
    Url::parse(&format!("data:text/html;charset=utf-8,{}", encoded)).expect("chrome data url")
}
const SETTINGS_TPL: &str = r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>Settings &#8212; Amni Browse</title><style>
:root{--bg:#0a0e1a;--elev:#141a2a;--stroke:#222a3d;--text:#d6dbe8;--dim:#8a92a6;--accent:#00d4ff;--accent-dim:#0089a8}
body{font:15px/1.55 -apple-system,'Segoe UI',Roboto,Arial,sans-serif;max-width:680px;margin:36px auto;padding:0 24px;color:var(--text);background:var(--bg)}
h1{font-size:22px;margin:0 0 2px}.tag{color:var(--dim);font-size:13px;margin:0 0 26px}
h2{color:var(--accent);font-size:12px;text-transform:uppercase;letter-spacing:1.5px;margin:30px 0 12px}
.opt{display:inline-flex;align-items:center;gap:7px;padding:7px 14px;margin:0 8px 8px 0;background:var(--elev);border:1px solid var(--stroke);border-radius:999px;cursor:pointer;transition:border-color .12s}
.opt:hover{border-color:var(--accent-dim)}
input[type=radio],input[type=checkbox]{accent-color:var(--accent)}
input[type=text],select{width:100%;max-width:420px;padding:8px 12px;background:var(--elev);border:1px solid var(--stroke);border-radius:8px;color:var(--text);font:inherit;outline:none;margin-top:6px}
input[type=text]:focus,select:focus{border-color:var(--accent)}
.row{display:flex;justify-content:space-between;align-items:center;gap:16px;padding:9px 0;border-bottom:1px solid var(--stroke)}.row:last-child{border-bottom:0}
.row a{color:var(--text);text-decoration:none;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.row a:hover{color:var(--accent)}
.x{background:none;border:1px solid var(--stroke);border-radius:6px;color:var(--dim);padding:3px 10px;cursor:pointer;font-size:12px}.x:hover{border-color:#ff5577;color:#ff5577}
.dim{color:var(--dim)}.note{color:var(--dim);font-size:12.5px;margin:6px 0 0}
.switch{display:flex;align-items:center;gap:10px;padding:8px 0}
kbd{background:var(--elev);border:1px solid var(--stroke);border-radius:4px;padding:1px 6px;font-family:'Cascadia Code',monospace;font-size:12px}
</style></head><body>
<h1>Settings</h1><p class='tag'>Amni Browse v__VER__ &#8212; changes save instantly</p>
<h2>Search engine</h2><div>__RADIOS__</div>
<h2>Homepage</h2><input type='text' value='__HOME__' placeholder='https://&#8230; (blank = built-in start page)' onchange='set("home_page",this.value)'><p class='note'>New tabs open this.</p>
<h2>Privacy</h2><label class='switch'><input type='checkbox'__SHIELD__ onchange='set("block_ads",this.checked)'><span>Shield &#8212; block ads &amp; trackers</span></label>
<h2>Appearance</h2><label>Default zoom for new tabs<select onchange='set("default_zoom",this.value)'>__ZOOMS__</select></label>
<h2>Advanced</h2><label>User-agent override<input type='text' value='__UA__' placeholder='(Servo default)' onchange='set("custom_user_agent",this.value)'></label><p class='note'>Some sites gate features on UA. Takes effect after restart.</p>
<h2>Bookmarks</h2><div>__BMS__</div>
<h2>Shortcuts</h2><p class='dim'><kbd>Ctrl+L</kbd> URL bar &#183; <kbd>Ctrl+D</kbd> bookmark &#183; <kbd>Ctrl+T</kbd>/<kbd>W</kbd> tabs &#183; <kbd>Ctrl+=</kbd>/<kbd>-</kbd>/<kbd>0</kbd> zoom &#183; <kbd>Ctrl+1&#8230;9</kbd> switch &#183; <kbd>Ctrl+Shift+T</kbd> reopen &#183; <kbd>F11</kbd> fullscreen</p>
<script>
const T='__TOK__';
function set(k,v){fetch('amnibrowse://cmd/setting_set?tok='+T+'&k='+encodeURIComponent(k)+'&v='+encodeURIComponent(v),{mode:'no-cors'}).catch(function(){})}
function rmbm(id){fetch('amnibrowse://cmd/bookmark_remove?tok='+T+'&id='+encodeURIComponent(id),{mode:'no-cors'}).catch(function(){});var e=document.getElementById('bm-'+id);e&&e.remove()}
</script></body></html>"##;
const NEWTAB_TPL: &str = r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>New Tab</title><style>
body{font:15px -apple-system,'Segoe UI',Roboto,Arial,sans-serif;background:#0a0e1a;color:#d6dbe8;display:flex;flex-direction:column;align-items:center;min-height:100vh;margin:0;padding-top:14vh}
h1{font-size:34px;letter-spacing:.5px;margin:0 0 6px;color:#00d4ff}
p{color:#8a92a6;margin:0 0 40px}
.grid{display:flex;flex-wrap:wrap;gap:14px;justify-content:center;max-width:760px}
.tile{display:flex;flex-direction:column;align-items:center;gap:8px;width:108px;padding:16px 6px;background:#141a2a;border:1px solid #222a3d;border-radius:14px;text-decoration:none;color:#d6dbe8;font-size:12px;transition:border-color .12s}
.tile:hover{border-color:#00d4ff}
.mono{width:40px;height:40px;border-radius:10px;display:flex;align-items:center;justify-content:center;font-size:18px;font-weight:600;color:#fff}
.tile span{max-width:100px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.dim{color:#8a92a6;font-size:13px}
</style></head><body><h1>Amni Browse</h1><p>Private by default &#8212; search from the bar above</p><div class='grid'>__TILES__</div></body></html>"##;
fn esc_html(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;") }
fn chrome_height_px(scale: f32) -> u32 { (CHROME_HEIGHT_CSS * scale).round().max(1.0) as u32 }
fn content_size(window_size: PhysicalSize<u32>, chrome_px: u32) -> PhysicalSize<u32> {
    PhysicalSize::new(window_size.width.max(1), window_size.height.saturating_sub(chrome_px).max(1))
}
pub fn run(state: BrowserState) {
    info!("Amni Browse Real Servo backend initializing...");
    info!("Media engine platform: {}", media_engine::platform_label());
    let event_loop = EventLoop::<WakerEvent>::with_user_event().build().expect("event loop");
    let ad_blocker = Arc::new(Mutex::new(state.ad_blocker.clone()));
    let config = state.config.clone();
    let bookmarks = state.bookmarks.clone();
    let mut initial_urls: Vec<(String, EngineKind)> = state.tabs.tabs.iter().map(|t| {
        let kind = match t.engine { TabEngine::Media => EngineKind::Media, _ => media_engine::route(&t.url) };
        (t.url.clone(), kind)
    }).collect();
    if let Ok(test_url) = std::env::var("AMNI_TEST_MEDIA_URL") { info!("AMNI_TEST_MEDIA_URL set \u{2192} injecting media tab: {}", test_url); initial_urls.push((test_url, EngineKind::Media)); }
    let mut app = App::new(&event_loop, ad_blocker, initial_urls, config, bookmarks);
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
    config: RefCell<BrowserConfig>,
    bookmarks: RefCell<BookmarkManager>,
    cmd_token: String,
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
        let z = self.default_zoom();
        if (z - 1.0).abs() > 0.01 { wv.set_page_zoom(z); }
        wv
    }
    fn default_zoom(&self) -> f32 { (self.config.borrow().default_zoom as f32).clamp(0.25, 5.0) }
    fn home_url(&self) -> String {
        let hp = self.config.borrow().home_page.trim().to_string();
        match hp.starts_with("http") { true => hp, false => format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&self.newtab_html())) }
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
        NEWTAB_TPL.replace("__TILES__", &tiles)
    }
    fn settings_page_html(&self) -> String {
        let c = self.config.borrow();
        let b = self.bookmarks.borrow();
        let engines = [("DuckDuckGo", DEFAULT_SEARCH_ENGINE), ("Brave", "https://search.brave.com/search?q="), ("Startpage", "https://www.startpage.com/sp/search?query="), ("Google", "https://www.google.com/search?q=")];
        let radios: String = engines.iter().map(|(n, p)| format!("<label class='opt'><input type='radio' name='se' value='{}'{} onchange='set(\"search_engine\",this.value)'><span>{}</span></label>", p, match c.search_engine == *p { true => " checked", false => "" }, n)).collect();
        let zooms: String = [(0.8, "80%"), (0.9, "90%"), (1.0, "100%"), (1.1, "110%"), (1.25, "125%"), (1.5, "150%")].iter().map(|(z, l)| format!("<option value='{}'{}>{}</option>", z, match (*z - c.default_zoom).abs() < 0.01 { true => " selected", false => "" }, l)).collect();
        let bms: String = match b.bookmarks.is_empty() {
            true => "<p class='dim'>No bookmarks yet \u{2014} hit \u{2606} in the URL bar or Ctrl+D.</p>".into(),
            false => b.bookmarks.iter().map(|bm| format!("<div class='row' id='bm-{}'><a href='{}' title='{}'>{}</a><button class='x' onclick='rmbm(\"{}\")'>remove</button></div>", esc_html(&bm.id), esc_html(&bm.url), esc_html(&bm.url), esc_html(&bm.title), esc_html(&bm.id))).collect(),
        };
        SETTINGS_TPL.replace("__VER__", env!("CARGO_PKG_VERSION")).replace("__RADIOS__", &radios).replace("__HOME__", &esc_html(match c.home_page.starts_with("http") { true => c.home_page.as_str(), false => "" })).replace("__SHIELD__", match c.block_ads { true => " checked", false => "" }).replace("__ZOOMS__", &zooms).replace("__UA__", &esc_html(c.custom_user_agent.as_deref().unwrap_or(""))).replace("__BMS__", &bms).replace("__TOK__", &self.cmd_token)
    }
    fn execute_command(&self, name: &str, args: &std::collections::HashMap<String, String>) {
        match name {
            "back" => { if let Some(c) = self.active_content() { if c.can_go_back() { let _ = c.go_back(1); info!("cmd back"); } } }
            "forward" => { if let Some(c) = self.active_content() { if c.can_go_forward() { let _ = c.go_forward(1); info!("cmd forward"); } } }
            "reload" => { if let Some(c) = self.active_content() { c.reload(); info!("cmd reload"); } }
            "navigate" => {
                let raw = args.get("url").cloned().unwrap_or_default();
                let engine = self.config.borrow().search_engine.clone();
                match resolve_navigate_input(&raw, &engine) {
                    Some(u) => {
                        let friendly = prefer_servo_friendly_url(u.as_str());
                        let dest = Url::parse(&friendly).unwrap_or(u);
                        let us = dest.as_str().to_string();
                        match media_engine::wants_media_window(&us) {
                            true => { info!("cmd navigate \u{2192} media engine: {}", us); self.pending_media_urls.borrow_mut().push(us); }
                            false => { if let Some(c) = self.active_content() { info!("cmd navigate \u{2192} {}", dest); c.load(dest); } }
                        }
                    }
                    None => info!("cmd navigate: empty/invalid input"),
                }
            }
            "open_lite_ddg" => {
                if let Some(c) = self.active_content() {
                    let u = Url::parse(LITE_DDG_HOME).unwrap();
                    info!("cmd open_lite_ddg \u{2192} {}", u);
                    c.load(u);
                }
            }
            "new_tab" => {
                let raw = args.get("url").cloned().unwrap_or_else(|| self.home_url());
                let friendly = prefer_servo_friendly_url(&raw);
                let start = Url::parse(&friendly).unwrap_or_else(|_| Url::parse(LITE_DDG_HOME).unwrap());
                let us = start.as_str().to_string();
                if media_engine::wants_media_window(&us) {
                    info!("cmd new_tab \u{2192} media engine: {}", us);
                    self.pending_media_urls.borrow_mut().push(us);
                    return;
                }
                let wv = self.spawn_content_webview(start);
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(self.default_zoom());
                self.active_content_index.set(tabs.len() - 1);
                info!("cmd new_tab \u{2192} idx {}", tabs.len() - 1);
                self.window.request_redraw();
            }
            "reopen_tab" => {
                let Some(url) = self.closed_tabs.borrow_mut().pop() else { info!("cmd reopen_tab: stack empty"); return };
                let us = url.as_str().to_string();
                if media_engine::wants_media_window(&us) {
                    info!("cmd reopen_tab \u{2192} media engine: {}", us);
                    self.pending_media_urls.borrow_mut().push(us);
                    return;
                }
                let wv = self.spawn_content_webview(url.clone());
                let mut tabs = self.content_webviews.borrow_mut();
                tabs.push(wv);
                self.tab_zoom.borrow_mut().push(self.default_zoom());
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
                if let Some(midx) = id.strip_prefix('m').and_then(|s| s.parse::<usize>().ok()) {
                    let media = self.media_windows.borrow();
                    if let Some((_wid, mw)) = media.iter().nth(midx) {
                        mw.window.focus_window();
                        info!("cmd switch_tab \u{2192} media m{}", midx);
                    }
                    return;
                }
                let Some(idx) = Self::parse_tab_index(id) else { return };
                let len = self.content_webviews.borrow().len();
                if idx < len { self.active_content_index.set(idx); info!("cmd switch_tab \u{2192} idx {}", idx); self.window.request_redraw(); }
            }
            "close_tab" => {
                let Some(id) = args.get("id") else { return };
                if let Some(midx) = id.strip_prefix('m').and_then(|s| s.parse::<usize>().ok()) {
                    let mut media = self.media_windows.borrow_mut();
                    let key = media.keys().nth(midx).copied();
                    if let Some(wid) = key {
                        media.remove(&wid);
                        info!("cmd close_tab \u{2192} media m{}", midx);
                    }
                    return;
                }
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
                {
                    let mut c = self.config.borrow_mut();
                    match k.as_str() {
                        "search_engine" => c.search_engine = v.clone(),
                        "home_page" => c.home_page = v.clone(),
                        "block_ads" => { c.block_ads = v == "true"; c.block_trackers = c.block_ads; }
                        "default_zoom" => c.default_zoom = v.parse().unwrap_or(1.0),
                        "custom_user_agent" => c.custom_user_agent = match v.trim().is_empty() { true => None, false => Some(v.trim().to_string()) },
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
        let media_tabs: Vec<serde_json::Value> = self.media_windows.borrow().iter().enumerate().map(|(i, (_wid, mw))| {
            let host = url::Url::parse(&mw.url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| "Media".into());
            serde_json::json!({
                "id": format!("m{}", i),
                "url": mw.url,
                "title": format!("\u{25B6} {}", host),
                "active": false,
                "loading": false,
                "engine": "media",
            })
        }).collect();
        let mut all_tabs = tabs;
        all_tabs.extend(media_tabs);
        let zoom = self.tab_zoom.borrow().get(active_idx).copied().unwrap_or(1.0);
        let compat_hint = is_client_side_exception_title(&title);
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
            "shield": self.config.borrow().block_ads,
            "bookmarked": !url.is_empty() && self.bookmarks.borrow().find_by_url(&url).is_some(),
            "compatHint": compat_hint,
            "compatMessage": if compat_hint {
                "This site needs modern JS Servo can't run yet. Open DuckDuckGo lite instead?"
            } else { "" },
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
        (Key::Character(c), true, false) if c.eq_ignore_ascii_case("d") => { state.execute_command("bookmark", &empty); true }
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
        return Url::parse(&prefer_servo_friendly_url(u.as_str())).ok().or(Some(u));
    }
    let has_dot = trimmed.contains('.');
    let has_space = trimmed.contains(' ');
    match has_dot && !has_space {
        true => {
            let candidate = format!("https://{}", trimmed);
            Url::parse(&prefer_servo_friendly_url(&candidate)).ok().or_else(|| Url::parse(&candidate).ok())
        }
        false => {
            let prefix = match search_prefix.starts_with("http") { true => search_prefix, false => DEFAULT_SEARCH_ENGINE };
            Url::parse(&format!("{}{}", prefix, urlencoding::encode(trimmed))).ok()
        }
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
    fn load_web_resource(&self, webview: WebView, load: WebResourceLoad) {
        let req_url = load.request().url.clone();
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
    fn request_navigation(&self, _webview: WebView, req: NavigationRequest) {
        let url = req.url.as_str().to_string();
        match media_engine::wants_media_window(&url) {
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
static LAST_MISMATCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static LAST_OFFSCREEN_DRIFT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn paint_and_present(state: &AppState) {
    // Servoshell binds the GL context before every composite. Without this, ANGLE/surfman
    // can leave the wrong context current after input/wake → clear/blit become no-ops and
    // present shows a cleared (black) or untouched (white) content band.
    if let Err(e) = state.rendering_context.make_current() {
        log::warn!("paint_and_present: make_current failed: {:?}", e);
    }

    let win_now = state.window_size();
    let ctx_now = state.rendering_context.size();
    let key = ((win_now.width as u64) << 32) | win_now.height as u64;
    if (ctx_now.width, ctx_now.height) != (win_now.width, win_now.height) && LAST_MISMATCH.swap(key, std::sync::atomic::Ordering::Relaxed) != key {
        info!("paint mismatch: ctx {}x{} vs win {}x{}", ctx_now.width, ctx_now.height, win_now.width, win_now.height);
    }

    let chrome_px = state.chrome_px();
    let expected_content = content_size(win_now, chrome_px);
    let off_now = state.offscreen_context.size();
    if (off_now.width, off_now.height) != (expected_content.width, expected_content.height) {
        let off_key = ((off_now.width as u64) << 32) | off_now.height as u64;
        if LAST_OFFSCREEN_DRIFT.swap(off_key, std::sync::atomic::Ordering::Relaxed) != off_key {
            info!(
                "offscreen size drift {}x{} vs expected content {}x{} (chrome_px={}) — resizing webviews",
                off_now.width, off_now.height, expected_content.width, expected_content.height, chrome_px
            );
        }
        // WebView::resize drives OffscreenRenderingContext::resize (do not pre-resize contexts).
        if let Some(chrome) = state.chrome_webview.borrow().as_ref() { chrome.resize(win_now); }
        for c in state.content_webviews.borrow().iter() { c.resize(expected_content); }
    }

    let chrome_opt = state.chrome_webview.borrow().clone();
    let content_opt = state.active_content();
    if let Some(chrome) = chrome_opt.as_ref() { chrome.paint(); }
    if let Some(content) = content_opt.as_ref() { content.paint(); }

    // v0.10.1 invariant: after content.paint() the offscreen FB is the DRAW target. Servo's
    // blit callback scissor-clears *before* rebinding, so we must rebind the window FB first
    // or the clear blacks the source → black content void (_shot_nav.png).
    state.rendering_context.prepare_for_rendering();

    let content_h = win_now.height.saturating_sub(chrome_px).max(1);
    // GL framebuffer origin is BOTTOM-LEFT (servoshell uses clip.from_bottom_px). Content
    // lives under the top chrome strip ⇒ target_rect.y = 0, height = content_h. Using
    // y=chrome_px here would wipe chrome and leave a blank band at the bottom.
    let target_rect = DefaultRect::new(
        DefaultPoint2D::new(0i32, 0i32),
        DefaultSize2D::new(win_now.width.max(1) as i32, content_h as i32),
    );
    match state.offscreen_context.render_to_parent_callback() {
        Some(callback) => {
            let gl = state.rendering_context.glow_gl_api();
            callback(&gl, target_rect);
        }
        None => {
            // framebuffer_id==0 → FBO never created (context not current at offscreen init).
            log::warn!(
                "paint_and_present: render_to_parent_callback=None (offscreen FBO missing) — content void; offscreen={}x{} win={}x{}",
                off_now.width, off_now.height, win_now.width, win_now.height
            );
        }
    }
    state.rendering_context.present();
}
fn resize_all(state: &AppState, new_size: PhysicalSize<u32>) {
    info!("resize_all \u{2192} {}x{}", new_size.width, new_size.height);
    let _ = state.rendering_context.make_current();
    let chrome_px = state.chrome_px();
    let content = content_size(new_size, chrome_px);
    if let Some(chrome) = state.chrome_webview.borrow().as_ref() { chrome.resize(new_size); }
    for c in state.content_webviews.borrow().iter() { c.resize(content); }
    state.window.request_redraw();
}
enum App { Initial(Waker, Arc<Mutex<AdBlocker>>, Vec<(String, EngineKind)>, BrowserConfig, BookmarkManager), Running(Rc<AppState>) }
impl App {
    fn new(event_loop: &EventLoop<WakerEvent>, ad_blocker: Arc<Mutex<AdBlocker>>, initial_urls: Vec<(String, EngineKind)>, config: BrowserConfig, bookmarks: BookmarkManager) -> Self {
        Self::Initial(Waker::new(event_loop), ad_blocker, initial_urls, config, bookmarks)
    }
}
impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Initial(waker, ad_blocker, initial_urls, config, bookmarks) = self {
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
            let mut prefs = Preferences::default();
            if let Some(ua) = config.custom_user_agent.as_ref().filter(|u| !u.trim().is_empty()) { info!("custom user agent: {}", ua); prefs.user_agent = ua.clone(); }
            let servo = ServoBuilder::default().event_loop_waker(Box::new(waker.clone())).preferences(prefs).build();
            let ad_blocker_clone = ad_blocker.clone();
            let cmd_token = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x5eed) ^ 0x9e37_79b9_7f4a_7c15u64);
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
                config: RefCell::new(config.clone()),
                bookmarks: RefCell::new(bookmarks.clone()),
                cmd_token,
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
                .map(|u| prefer_servo_friendly_url(&u))
                .unwrap_or_else(|| app_state.home_url());
            let content_url = Url::parse(&servo_url).unwrap_or_else(|_| Url::parse(LITE_DDG_HOME).unwrap());
            info!("servo content initial url: {}", content_url);
            let content_webview = WebViewBuilder::new(&app_state.servo, app_state.offscreen_context.clone())
                .url(content_url)
                .hidpi_scale_factor(Scale::new(scale))
                .delegate(app_state.clone())
                .build();
            // Explicit resize attaches the content WebView to the offscreen surface size.
            // Without this, some boots leave a zero/stale viewport and blit an empty FB.
            let off_sz = app_state.offscreen_context.size();
            content_webview.resize(off_sz);
            if let Some(chrome) = app_state.chrome_webview.borrow().as_ref() {
                chrome.resize(window_size);
            }
            let z0 = app_state.default_zoom();
            if (z0 - 1.0).abs() > 0.01 { content_webview.set_page_zoom(z0); }
            app_state.content_webviews.borrow_mut().push(content_webview);
            app_state.tab_zoom.borrow_mut().push(z0);
            for (u, k) in initial_urls.iter() {
                if *k != EngineKind::Media { continue; }
                if let Some((id, mw)) = media_engine::spawn_media_window(event_loop, u) {
                    app_state.media_windows.borrow_mut().insert(id, mw);
                }
            }
            info!(
                "Servo embedder ready: win={}x{} chrome_px={} offscreen={}x{} scale={} (blit GL y=0,h=content)",
                window_size.width, window_size.height, chrome_px, off_sz.width, off_sz.height, scale
            );
            app_state.window.request_redraw();
            *self = Self::Running(app_state);
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
