use std::{borrow::Cow, cell::{Cell, RefCell}, collections::HashMap, path::PathBuf, rc::Rc, time::Instant};
use log::{debug, info, warn};
use tao::{dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize}, event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy}, keyboard::{Key, ModifiersState}, window::{Fullscreen, Window, WindowBuilder}};
use wry::{http, PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder};
#[cfg(windows)]
use tao::platform::windows::WindowExtWindows;
#[cfg(not(windows))]
use tao::platform::unix::WindowExtUnix;
#[cfg(not(windows))]
use wry::WebViewBuilderExtUnix;
#[cfg(not(windows))]
use gtk::prelude::{BoxExt, ContainerExt, LayoutExt, OverlayExt, WidgetExt as GtkWidgetExt};
#[cfg(windows)]
use wry::WebViewExtWindows;
#[cfg(windows)]
use webview2_com::{Microsoft::Web::WebView2::Win32::*, BytesReceivedChangedEventHandler, ClearBrowsingDataCompletedHandler, ContainsFullScreenElementChangedEventHandler, DownloadStartingEventHandler, FaviconChangedEventHandler, HistoryChangedEventHandler, IsDocumentPlayingAudioChangedEventHandler, StateChangedEventHandler, WebResourceRequestedEventHandler};
#[cfg(windows)]
use windows::{core::{w, Interface, HSTRING, PWSTR}, Win32::Foundation::BOOL, Win32::System::{Com::CoTaskMemFree, WinRT::EventRegistrationToken}};
#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, SetWindowPos, GW_CHILD, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
#[cfg(windows)]
type Core = ICoreWebView2;
#[cfg(not(windows))]
type Core = ();
use crate::{app::BrowserState, engine::{adblocker::AdBlocker, ai_search, permissions::{PermissionState, PermissionType}}, storage::{config::{APP_NAME, APP_VERSION}, downloads::{DownloadItem, DownloadManager, DownloadStatus}, session::{SessionManager, SessionTab}}, ui::internal_pages::{esc_html, newtab_html, theme_root_vars, SETTINGS_TPL, TUTORIAL_TPL}, ui::tokens::SERVO_CHROME_HEIGHT_CSS};
/// Chrome's bookmarks bar height, added under the nav row when it is shown.
const BOOKMARKS_BAR_CSS: u32 = 28;
#[cfg(windows)]
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
// Linux runs WebKitGTK, so say so. Claiming Chrome on a WebKit engine is what
// bot checks look for: Cloudflare Turnstile saw the mismatch, reset the checkbox
// after every click and blocked some sites outright. This is the Safari-style
// string WebKitGTK browsers (GNOME Web) send, and they pass.
#[cfg(not(windows))]
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
#[cfg(windows)]
const ENGINE: &str = "Chromium (WebView2)";
#[cfg(not(windows))]
const ENGINE: &str = "WebKitGTK";
const FRAME_CSS: f64 = 5.0;
const DL_INTERRUPTED: i32 = 1;
const DL_COMPLETED: i32 = 2;
const AUTH_POPUP_HOSTS: &[&str] = &[
    "accounts.google.com", "accounts.youtube.com", "accounts.x.ai", "auth.x.ai",
    "login.microsoftonline.com", "login.live.com", "login.microsoft.com",
    "appleid.apple.com", "id.apple.com",
    "facebook.com/dialog", "facebook.com/login",
    "github.com/login", "github.com/sessions",
    "auth0.com", "okta.com",
    "oauth", "openid", "signin", "sign-in", "sso.",
];
/// Sites that branch on a Chrome user-agent then read `navigator.userAgentData`.
/// Only injected when the user agent claims Chrome (Windows, or a custom one).
/// WebKit does not implement Client Hints; without this shim the Chrome-shaped
/// string sends xAI / Google / Apple sign-in down an API that throws.
const UA_SCRIPT: &str = "(function(){try{if(navigator.userAgentData&&navigator.userAgentData.getHighEntropyValues)return;var ua=navigator.userAgent||'';var m=/Chrome\\/(\\d+)/.exec(ua);var major=m?m[1]:'153';var platform=/Windows/.test(ua)?'Windows':(/Mac/.test(ua)?'macOS':'Linux');var brands=[{brand:'Chromium',version:major},{brand:'Google Chrome',version:major},{brand:'Not)A;Brand',version:'24'}];var full=brands.map(function(b){return{brand:b.brand,version:b.version+'.0.0.0'}});var data={brands:brands,mobile:false,platform:platform,toJSON:function(){return{brands:brands,mobile:false,platform:platform}},getHighEntropyValues:function(){return Promise.resolve({brands:brands,mobile:false,platform:platform,platformVersion:'',architecture:'x86',bitness:'64',model:'',uaFullVersion:major+'.0.0.0',fullVersionList:full,wow64:false})}};Object.defineProperty(navigator,'userAgentData',{get:function(){return data},configurable:true});if(!window.chrome)window.chrome={}}catch(e){}})()";
const FETCH_SHIM: &str = "(function(){var f=window.fetch.bind(window);window.fetch=function(u,o){if(typeof u==='string'&&u.indexOf('amnibrowse://')===0){u=u.replace(/^amnibrowse:\\/\\/([^\\/?#]+)\\/?/,function(_,h){return 'http://amnibrowse.'+h+'/'})}return f(u,o)}})()";
const KEY_SCRIPT: &str = "(function(){document.addEventListener('keydown',function(e){var k=e.key.toLowerCase();var fn={f5:1,f11:1,f12:1,f3:1,escape:1};var alt={arrowleft:1,arrowright:1,home:1,d:1,a:1};var send=function(){e.preventDefault();e.stopPropagation();try{window.ipc.postMessage(JSON.stringify({type:'key',k:k,shift:e.shiftKey?1:0,alt:e.altKey?1:0}))}catch(_){}};if(!e.ctrlKey&&!e.altKey&&!e.metaKey&&fn[k]){if(k==='escape'&&document.activeElement&&document.activeElement.tagName!=='BODY')return;send();return}if(e.altKey&&!e.ctrlKey&&alt[k]){send();return}if(!e.ctrlKey||e.altKey||e.metaKey)return;var hot={t:1,w:1,l:1,d:1,tab:1,h:1,j:1,u:1,f:1,p:1,r:1,n:1,s:1,g:1,e:1,k:1,pageup:1,pagedown:1,'1':1,'2':1,'3':1,'4':1,'5':1,'6':1,'7':1,'8':1,'9':1,'=':1,'+':1,'-':1,'0':1,i:e.shiftKey?1:0,b:e.shiftKey?1:0,o:e.shiftKey?1:0,a:e.shiftKey?1:0,delete:e.shiftKey?1:0};if(!hot[k])return;send()},true)})()";
const ICON_SCRIPT: &str = "(function(){function s(){try{var l=document.querySelector('link[rel~=\"icon\"],link[rel=\"shortcut icon\"]');var h=l&&l.href?l.href:(location.origin+'/favicon.ico');if(/^https?:/.test(h))window.ipc.postMessage(JSON.stringify({type:'icon',href:h}))}catch(_){}}if(document.readyState==='complete')s();else window.addEventListener('load',s)})()";
const FIND_SCRIPT: &str ="(function(){var H=window.CSS&&CSS.highlights;var st={q:'',ranges:[],i:-1};function report(){try{window.ipc.postMessage(JSON.stringify({type:'find',n:st.ranges.length,i:st.i+1}))}catch(_){}}function clear(){if(H){CSS.highlights.delete('amni-find');CSS.highlights.delete('amni-find-cur')}st={q:'',ranges:[],i:-1}}function collect(q){var out=[],w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT,{acceptNode:function(n){var p=n.parentElement;if(!p)return NodeFilter.FILTER_REJECT;var t=p.tagName;if(t==='SCRIPT'||t==='STYLE'||t==='NOSCRIPT')return NodeFilter.FILTER_REJECT;return n.nodeValue.toLowerCase().indexOf(q)>=0?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_SKIP}}),n;while((n=w.nextNode())){var s=n.nodeValue.toLowerCase(),k=0;while((k=s.indexOf(q,k))>=0){var r=document.createRange();r.setStart(n,k);r.setEnd(n,k+q.length);out.push(r);k+=q.length;if(out.length>5000)return out}}return out}function paint(){if(!H)return;var h=new Highlight();st.ranges.forEach(function(r){h.add(r)});CSS.highlights.set('amni-find',h);if(st.i>=0)CSS.highlights.set('amni-find-cur',new Highlight(st.ranges[st.i]))}function ensureCss(){if(document.getElementById('amni-find-css'))return;var s=document.createElement('style');s.id='amni-find-css';s.textContent='::highlight(amni-find){background:#ffd54a;color:#111}::highlight(amni-find-cur){background:#ff8a00;color:#111}';(document.head||document.documentElement).appendChild(s)}window.__amniFind=function(q,dir){q=(q||'').toLowerCase();if(!q){clear();report();return 0}ensureCss();if(q!==st.q){st.q=q;st.ranges=collect(q);st.i=st.ranges.length?0:-1}else if(st.ranges.length){st.i=(st.i+(dir<0?-1:1)+st.ranges.length)%st.ranges.length}if(!st.ranges.length){paint();report();return 0}var r=st.ranges[st.i];try{var sel=window.getSelection();sel.removeAllRanges();if(!H)sel.addRange(r)}catch(e){}try{var el=r.startContainer.parentElement;el&&el.scrollIntoView({block:'center',inline:'nearest'})}catch(e){}paint();report();return st.ranges.length};window.__amniFindClear=function(){clear();report()}})()";
/// Chrome niceties that live in the page: the link-target status bubble at the bottom-left,
/// Ctrl/middle-click on links into a background tab, and a selection courier for "Ask AI"/"Search".
const LINK_SCRIPT: &str = "(function(){if(window.top!==window)return;var b=null;function bub(){if(b)return b;b=document.createElement('div');b.id='__amni_status';b.style.cssText='position:fixed;left:0;bottom:0;max-width:60vw;padding:3px 9px;font:12px system-ui,sans-serif;background:#1c1f24;color:#e8e8e8;border:1px solid #2c3038;border-bottom:none;border-left:none;border-top-right-radius:5px;z-index:2147483647;pointer-events:none;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;display:none';(document.body||document.documentElement).appendChild(b);return b}function link(e){var t=e.target;if(!t||!t.closest)return null;var a=t.closest('a[href]');return a&&/^(https?|file|ftp):/i.test(a.href)?a:null}document.addEventListener('mouseover',function(e){var a=link(e);if(!a){if(b)b.style.display='none';return}var x=bub();x.textContent=a.href;x.style.display='block'},true);document.addEventListener('mouseout',function(e){if(b&&link(e))b.style.display='none'},true);document.addEventListener('click',function(e){var a=link(e);if(!a||e.button!==0||!(e.ctrlKey||e.metaKey))return;e.preventDefault();e.stopPropagation();try{window.ipc.postMessage(JSON.stringify({type:'open',href:a.href,bg:e.shiftKey?0:1}))}catch(_){}},true);document.addEventListener('auxclick',function(e){var a=link(e);if(!a||e.button!==1)return;e.preventDefault();e.stopPropagation();try{window.ipc.postMessage(JSON.stringify({type:'open',href:a.href,bg:1}))}catch(_){}},true);window.__amniSel=function(p){var s='';try{s=String(window.getSelection())}catch(_){}try{window.ipc.postMessage(JSON.stringify({type:'sel',purpose:p,text:s}))}catch(_){}}})()";
#[allow(dead_code)]
enum Ev {
    Cmd(String, HashMap<String, String>), Title(u64, String), Load(u64, bool, String), Popup(String), Key(u64, String, bool, bool), History(u64, bool, bool), Favicon(u64, String), PageFullscreen(u64, bool), Audio(u64, bool),
    DlStart(String, String, String, Option<u64>), DlProgress(String, u64), DlState(String, i32, String),
    /// Ctrl/middle-click on a link: open `url` next to the tab, in the background when `bg`.
    Open(u64, String, bool),
    /// Selection text couriered back from the page for `purpose` ("ask" | "search").
    Sel(u64, String, String),
    Find(u64, u32, u32),
    /// A site asked for a permission we have no stored answer for.
    #[cfg(not(windows))]
    Perm(u64, PermissionType, String, webkit2gtk::PermissionRequest),
    #[cfg(not(windows))]
    LoadFailed(u64, String, String),
    #[cfg(not(windows))]
    TlsFail(u64, String, String, webkit2gtk::gio::TlsCertificate),
    Crash(u64),
    /// Page-context-menu pick: action id plus the link/image URL the hit test saw.
    Ctx(u64, String, String),
    /// http:// navigation intercepted by HTTPS-only mode.
    Upgrade(u64, String),
    Tick,
}
enum View { Wry(WebView), #[cfg(all(feature = "cef-engine", target_os = "linux"))] Cef(super::cef_tabs::CefTab) }
impl View {
    fn wry(&self) -> Option<&WebView> { match self { View::Wry(v) => Some(v), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(_) => None } }
    #[cfg(not(windows))]
    fn webview(&self) -> Option<webkit2gtk::WebView> { use wry::WebViewExtUnix; self.wry().map(|v| v.webview()) }
    fn evaluate_script(&self, js: &str) -> wry::Result<()> { match self { View::Wry(v) => v.evaluate_script(js), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.eval(js); Ok(()) } } }
    fn load_url(&self, u: &str) -> wry::Result<()> { match self { View::Wry(v) => v.load_url(u), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.load_url(u); Ok(()) } } }
    fn load_html(&self, h: &str) -> wry::Result<()> { match self { View::Wry(v) => v.load_html(h), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.load_url(&format!("data:text/html;charset=utf-8,{}", urlencoding::encode(h))); Ok(()) } } }
    fn zoom(&self, z: f64) -> wry::Result<()> { match self { View::Wry(v) => v.zoom(z), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.zoom(z); Ok(()) } } }
    fn set_visible(&self, on: bool) -> wry::Result<()> { match self { View::Wry(v) => v.set_visible(on), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.set_visible(on); Ok(()) } } }
    fn print(&self) -> wry::Result<()> { match self { View::Wry(v) => v.print(), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.print(); Ok(()) } } }
    fn focus(&self) -> wry::Result<()> { match self { View::Wry(v) => v.focus(), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => { c.focus(); Ok(()) } } }
    fn open_devtools(&self) { match self { View::Wry(v) => v.open_devtools(), #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => c.devtools() } }
    #[cfg(not(windows))]
    fn go_back(&self) { use webkit2gtk::WebViewExt; match self { View::Wry(_) => { if let Some(w) = self.webview() { w.go_back(); } } #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => c.go_back() } }
    #[cfg(not(windows))]
    fn go_forward(&self) { use webkit2gtk::WebViewExt; match self { View::Wry(_) => { if let Some(w) = self.webview() { w.go_forward(); } } #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => c.go_forward() } }
    #[cfg(not(windows))]
    fn reload(&self, hard: bool) { use webkit2gtk::WebViewExt; match self { View::Wry(_) => { if let Some(w) = self.webview() { if hard { w.reload_bypass_cache() } else { w.reload() } } } #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => c.reload(hard) } }
    #[cfg(not(windows))]
    fn stop(&self) { use webkit2gtk::WebViewExt; match self { View::Wry(_) => { if let Some(w) = self.webview() { w.stop_loading(); } } #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => c.stop() } }
    #[cfg(not(windows))]
    fn mute(&self, on: bool) { use webkit2gtk::WebViewExt; match self { View::Wry(_) => { if let Some(w) = self.webview() { w.set_is_muted(on); } } #[cfg(all(feature = "cef-engine", target_os = "linux"))] View::Cef(c) => c.mute(on) } }
    fn is_cef(&self) -> bool { self.wry().is_none() }
}
fn ipc_ev(uid: u64, body: &str) -> Option<Ev> {
    let v = serde_json::from_str::<serde_json::Value>(body).ok()?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    match v.get("type").and_then(|t| t.as_str()) {
        Some("key") => Some(Ev::Key(uid, s("k"), v.get("shift").and_then(|s| s.as_i64()).unwrap_or(0) == 1, v.get("alt").and_then(|s| s.as_i64()).unwrap_or(0) == 1)),
        Some("icon") => Some(s("href")).filter(|h| !h.is_empty()).map(|h| Ev::Favicon(uid, h)),
        Some("open") => Some(s("href")).filter(|h| h.starts_with("http") || h.starts_with("file:")).map(|h| Ev::Open(uid, h, v.get("bg").and_then(|b| b.as_i64()).unwrap_or(1) == 1)),
        Some("sel") => Some(Ev::Sel(uid, s("purpose"), s("text"))),
        Some("find") => Some(Ev::Find(uid, v.get("n").and_then(|n| n.as_u64()).unwrap_or(0) as u32, v.get("i").and_then(|n| n.as_u64()).unwrap_or(0) as u32)),
        _ => None,
    }
}
struct Tab {
    uid: u64, view: View, core: Option<Core>, url: String, title: String, private: bool, loading: bool, zoom: f64, can_back: bool, can_forward: bool, icon: Option<String>, audio: bool, pinned: bool, group: Option<String>,
    muted: bool,
    /// Memory saver unloaded the page; `url` is reloaded when the tab is shown again.
    discarded: bool,
    last_active: Instant,
    /// Script to run once the page finishes loading (Ask-AI composer fill).
    inject: Option<String>,
    /// HTTPS-only upgraded this http:// URL; a failed secure load falls back to it once.
    upgraded_from: Option<String>,
    #[cfg(not(windows))]
    tls: Option<(String, webkit2gtk::gio::TlsCertificate)>,
    find: (u32, u32),
}
#[cfg(not(windows))]
struct PendingPerm { id: u64, uid: u64, kind: PermissionType, origin: String, req: webkit2gtk::PermissionRequest }
struct App {
    window: Window,
    #[cfg(not(windows))]
    overlay: gtk::Overlay,
    #[cfg(not(windows))]
    canvas: gtk::Layout,
    #[cfg(not(windows))]
    chrome_canvas: gtk::Layout,
    #[cfg(not(windows))]
    web_context: Option<WebContext>,
    dl_handler_registered: bool,
    proto_registered: bool,
    decorated: bool,
    chrome: Option<WebView>,
    chrome_hwnd: usize,
    tabs: Vec<Tab>,
    active: usize,
    /// Recently closed tabs: (url, title, private), newest last.
    closed: Vec<(String, String, bool)>,
    state: Rc<RefCell<BrowserState>>,
    token: String,
    next_uid: u64,
    overlay_css: u32,
    /// Last placed chrome and content rectangles. Resize events fire while a page
    /// is scrolled or hovered; re-placing an unchanged webview repaints the omnibar.
    last_chrome: Cell<(i32, i32, u32, u32)>,
    last_content: Cell<(i32, i32, u32, u32)>,
    fullscreen: bool,
    page_fullscreen: bool,
    find_query: String,
    protocol: Rc<dyn Fn(&str, http::Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>>>,
    events: Rc<RefCell<Vec<Ev>>>,
    proxy: EventLoopProxy<()>,
    blocker: Rc<RefCell<AdBlocker>>,
    shield: Rc<Cell<bool>>,
    #[cfg(not(windows))]
    filter: Option<usize>,
    collapsed: Vec<String>,
    ephemeral: bool,
    https_only: Rc<Cell<bool>>,
    /// http:// URLs allowed through HTTPS-only once (the fallback after a failed upgrade).
    http_allow: Rc<RefCell<Vec<String>>>,
    ask_dl_location: Rc<Cell<bool>>,
    bookmarks_bar: bool,
    #[cfg(not(windows))]
    perms: Vec<PendingPerm>,
    #[cfg(not(windows))]
    live_downloads: Rc<RefCell<HashMap<String, webkit2gtk::Download>>>,
    next_perm: u64,
    dl_progress_wired: bool,
    /// Last shortcut handled: on GTK the same keypress reaches us twice (tao's window handler and
    /// the in-page key script), so a repeat of the same combo inside ~100ms is dropped.
    last_key: (String, bool, bool, Instant),
}
type Push = Rc<dyn Fn(Ev)>;
fn load_toolbar_html() -> String {
    let candidates = [std::env::var_os("AMNI_CHROME_HTML").map(PathBuf::from), Some(PathBuf::from("assets/chrome/toolbar.html")), std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("assets/chrome/toolbar.html")))];
    candidates.into_iter().flatten().find_map(|p| std::fs::read_to_string(p).ok()).unwrap_or_else(|| include_str!("../../assets/chrome/toolbar.html").to_string())
}
fn internal_url(host: &str) -> String { match cfg!(windows) { true => format!("http://amnibrowse.{}/", host), false => format!("amnibrowse://{}/", host) } }
fn fetch_shim() -> &'static str { match cfg!(windows) { true => FETCH_SHIM, false => "" } }
fn is_internal(url: &str) -> bool { url.starts_with("https://amnibrowse.") || url.starts_with("http://amnibrowse.") || url.starts_with("amnibrowse://") }
fn display_url(url: &str) -> String {
    match url.strip_prefix("https://amnibrowse.").or_else(|| url.strip_prefix("http://amnibrowse.")) { Some(rest) => format!("amnibrowse://{}", rest.trim_end_matches('/')), None => url.to_string() }
}
fn wants_native_popup(url: &str) -> bool {
    let l = url.trim().to_ascii_lowercase();
    // OAuth SDKs open about:blank first, then assign the provider URL. Ignoring
    // that window makes window.open return null and the site shows a generic error.
    if l.is_empty() || l == "about:blank" || l.starts_with("about:blank") || l == "about:srcdoc" {
        return true;
    }
    AUTH_POPUP_HOSTS.iter().any(|h| l.contains(h))
}
fn host_of(url: &str) -> String { url::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string())).unwrap_or_default() }
/// Hosts HTTPS-only leaves alone: localhost, .local, bare IPs.
fn is_local_host(url: &str) -> bool {
    let h = host_of(url);
    h.is_empty() || h == "localhost" || h.ends_with(".localhost") || h.ends_with(".local") || h.parse::<std::net::IpAddr>().is_ok()
}
/// GTK "Save as" for downloads when Settings → Downloads asks where to save each file.
#[cfg(not(windows))]
fn pick_save_path(suggested: &std::path::Path) -> Option<PathBuf> {
    use gtk::prelude::{DialogExt, FileChooserExt, GtkWindowExt, WidgetExt};
    let dlg = gtk::FileChooserDialog::with_buttons::<gtk::Window>(Some("Save file"), None, gtk::FileChooserAction::Save, &[("Cancel", gtk::ResponseType::Cancel), ("Save", gtk::ResponseType::Accept)]);
    dlg.set_do_overwrite_confirmation(true);
    if let Some(dir) = suggested.parent() { let _ = dlg.set_current_folder(dir); }
    if let Some(name) = suggested.file_name() { dlg.set_current_name(name.to_string_lossy().as_ref()); }
    dlg.set_modal(true);
    let r = dlg.run();
    let out = match r == gtk::ResponseType::Accept { true => dlg.filename(), false => None };
    dlg.close();
    while gtk::events_pending() { gtk::main_iteration(); }
    out
}
fn resolve_input(raw: &str, search_prefix: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() { return None; }
    if let Some(rest) = t.strip_prefix("amnibrowse://") { return Some(internal_url(rest.trim_matches('/'))); }
    if t.contains("://") || t.starts_with("about:") || t.starts_with("view-source:") { return Some(t.to_string()); }
    if t == "localhost" || t.starts_with("localhost:") { return Some(format!("http://{}", t)); }
    if std::path::Path::new(t).is_file() { return url::Url::from_file_path(std::fs::canonicalize(t).ok()?).ok().map(|u| u.to_string()); }
    match t.contains('.') && !t.contains(' ') {
        true => Some(format!("https://{}", t)),
        false => Some(format!("{}{}", match search_prefix.starts_with("http") { true => search_prefix, false => "https://html.duckduckgo.com/html/?q=" }, urlencoding::encode(t))),
    }
}
fn json_headers(ct: &'static str) -> http::response::Builder {
    http::Response::builder().header("Content-Type", ct).header("Cache-Control", "no-store").header("Access-Control-Allow-Origin", "*")
}
fn respond(ct: &'static str, body: String) -> http::Response<Cow<'static, [u8]>> { json_headers(ct).body(Cow::Owned(body.into_bytes())).unwrap() }
fn empty(status: u16) -> http::Response<Cow<'static, [u8]>> { json_headers("text/plain").status(status).body(Cow::Borrowed(&[][..])).unwrap() }
#[cfg(windows)]
fn take_pwstr(p: PWSTR) -> String {
    if p.is_null() { return String::new(); }
    let s = unsafe { p.to_string() }.unwrap_or_default();
    unsafe { CoTaskMemFree(Some(p.0 as *const _)) };
    s
}
/// wry on Linux only takes an X11 handle from `build_as_child` and then wraps the window a second
/// time, so under Wayland it panics (UnsupportedWindowHandle) and under X11 the omnibox never gets
/// keyboard focus. Webviews go into a gtk::Fixed inside tao's own vbox instead; bounds still apply.
#[cfg(windows)]
fn build_view(b: WebViewBuilder<'_>, host: &Window) -> wry::Result<WebView> { b.build_as_child(host) }
#[cfg(not(windows))]
fn build_view(b: WebViewBuilder<'_>, host: &gtk::Layout) -> wry::Result<WebView> {
    let v = b.build_gtk(host)?;
    focus_on_click(&v);
    Ok(v)
}
/// GTK keeps keyboard focus on whichever webview had it last, so a click in the omnibox reached
/// the DOM while the keystrokes still went to the page. Every view grabs GTK focus when clicked.
#[cfg(not(windows))]
fn focus_on_click(v: &WebView) {
    use gtk::prelude::WidgetExt;
    use wry::WebViewExtUnix;
    let wv = v.webview();
    wv.set_can_focus(true);
    wv.connect_button_press_event(|w, _| { #[cfg(all(feature = "cef-engine", target_os = "linux"))] super::cef_tabs::grab_x_focus(w); if !w.has_focus() { w.grab_focus(); } gtk::glib::Propagation::Proceed });
}
/// gtk::Layout, not gtk::Fixed: a Fixed re-allocates children at their original put() spot with
/// their original size request on every pass, so views stayed the size the window opened at and
/// the window could never shrink. A Layout is a canvas whose own size request ignores its
/// children; `place()` moves and resizes each view on every layout.
/// A gtk::Overlay ensures the chrome canvas is stacked above the tabs canvas so menus and popups
/// are never occluded by tab webviews.
#[cfg(not(windows))]
fn gtk_host(window: &Window) -> (gtk::Overlay, gtk::Layout, gtk::Layout) {
    use gtk::prelude::*;
    let overlay = gtk::Overlay::new();
    let content_canvas = gtk::Layout::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    let chrome_canvas = gtk::Layout::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    chrome_canvas.set_valign(gtk::Align::Start);
    chrome_canvas.set_halign(gtk::Align::Fill);
    chrome_canvas.set_app_paintable(true);
    let provider = gtk::CssProvider::new();
    let _ = provider.load_from_data(b"layout, widget { background-color: transparent; }");
    chrome_canvas.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    overlay.add(&content_canvas);
    overlay.add_overlay(&chrome_canvas);
    if let Some(vb) = window.default_vbox() { vb.pack_start(&overlay, true, true, 0); }
    overlay.show_all();
    (overlay, content_canvas, chrome_canvas)
}
fn sanitize_filename(raw: &str) -> String {
    let s: String = raw.chars().filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'..='\x1f')).collect();
    let s = s.trim().trim_matches('.');
    if s.is_empty() { "download".to_string() } else { s.to_string() }
}
fn unique_path(dir: &std::path::Path, base_name: &str) -> PathBuf {
    let sanitized = sanitize_filename(base_name);
    let mut candidate = dir.join(&sanitized);
    if !candidate.exists() {
        return candidate;
    }
    let (stem, ext) = match sanitized.rsplit_once('.') {
        Some((s, e)) => (s, format!(".{}", e)),
        None => (sanitized.as_str(), String::new()),
    };
    for i in 1..1000 {
        let name = format!("{} ({}){}", stem, i, ext);
        candidate = dir.join(&name);
        if !candidate.exists() {
            return candidate;
        }
    }
    candidate
}
fn guess_filename(uri: &str, raw_path: &std::path::Path) -> String {
    if let Some(f) = raw_path.file_name() {
        let s = f.to_string_lossy().trim().to_string();
        if !s.is_empty() && s != "download" {
            return sanitize_filename(&s);
        }
    }
    if uri.starts_with("data:text/csv") {
        return "passwords.csv".to_string();
    }
    if let Ok(parsed) = url::Url::parse(uri) {
        for (k, v) in parsed.query_pairs() {
            if (k == "filename" || k == "file") && !v.trim().is_empty() {
                return sanitize_filename(&v);
            }
        }
        if let Some(mut segs) = parsed.path_segments() {
            if let Some(last) = segs.next_back() {
                let unescaped = urlencoding::decode(last).unwrap_or(std::borrow::Cow::Borrowed(last));
                let trimmed = unescaped.trim();
                if !trimmed.is_empty() && trimmed != "download" && trimmed.contains('.') {
                    return sanitize_filename(trimmed);
                }
            }
        }
        if parsed.host_str().map(|h| h.contains("password")).unwrap_or(false)
            || parsed.path().contains("password")
            || parsed.path().contains("export") {
            return "Google_Passwords.csv".to_string();
        }
    }
    if uri.contains("password") || uri.contains("export") {
        return "Google_Passwords.csv".to_string();
    }
    "download".to_string()
}
fn render_settings_html(state: &BrowserState, shield: bool, token: &str) -> String {
    let c = &state.config;
    let engines = [("DuckDuckGo", "https://html.duckduckgo.com/html/?q="), ("Brave", "https://search.brave.com/search?q="), ("Startpage", "https://www.startpage.com/sp/search?query="), ("Kagi", "https://kagi.com/search?q="), ("Google", "https://www.google.com/search?q=")];
    let radios: String = engines.iter().map(|(n, p)| format!("<label class='opt'><input type='radio' name='se' value='{}'{} onchange='set(\"search_engine\",this.value)'><span>{}</span></label>", p, match c.search_engine == *p { true => " checked", false => "" }, n)).collect();
    let zooms: String = [(0.8, "80%"), (0.9, "90%"), (1.0, "100%"), (1.1, "110%"), (1.25, "125%"), (1.5, "150%")].iter().map(|(z, l)| format!("<option value='{}'{}>{}</option>", z, match (*z - c.default_zoom).abs() < 0.01 { true => " selected", false => "" }, l)).collect();
    let bms: String = match state.bookmarks.bookmarks.is_empty() {
        true => "<p class='dim'>No bookmarks yet \u{2014} hit \u{2606} in the URL bar or Ctrl+D.</p>".into(),
        false => state.bookmarks.bookmarks.iter().map(|bm| format!("<div class='row' id='bm-{}'><a href='{}' title='{}'>{}</a><button class='x' onclick='rmbm(\"{}\")'>remove</button></div>", esc_html(&bm.id), esc_html(&bm.url), esc_html(&bm.url), esc_html(&bm.title), esc_html(&bm.id))).collect(),
    };
    let (follows, active_id) = (state.themes.follows_system(), state.themes.active_theme_id.clone());
    let themes: String = format!("<label class='opt'><input type='radio' name='th' value='system'{} onchange='set(\"theme\",this.value)'><span>Match system (light or dark)</span></label>", match follows { true => " checked", false => "" }) + &state.themes.all_themes().iter().map(|t| format!("<label class='opt'><input type='radio' name='th' value='{}'{} onchange='set(\"theme\",this.value)'><span>{}</span></label>", esc_html(&t.id), match !follows && t.id == active_id { true => " checked", false => "" }, esc_html(&t.name))).collect::<String>();
    let home = match c.home_page.starts_with("http") { true => c.home_page.clone(), false => String::new() };
    let chk = |b: bool| match b { true => " checked", false => "" };
    let toggles = format!("<label class='opt'><input type='checkbox'{} onchange='set(\"clear_data_on_exit\",this.checked?1:0)'><span>Clear browsing data (cookies, cache, history) when Amni Browse closes</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"autofill_on_load\",this.checked?1:0)'><span>Let the engine save passwords and fill forms (Chromium profile store)</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"enable_do_not_track\",this.checked?1:0)'><span>Send Do Not Track + Global Privacy Control headers</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"enable_doh\",this.checked?1:0)'><span>DNS over HTTPS (restart to apply)</span></label>", chk(c.clear_data_on_exit), chk(c.autofill_on_load), chk(c.enable_do_not_track), chk(c.enable_doh));
    // Ask-AI pane: the provider the user already subscribes to, opened with the prompt prefilled.
    let ai_radios: String = ai_search::PROVIDERS.iter().map(|p| format!("<label class='opt' title='{}'><input type='radio' name='ai' value='{}'{} onchange='set(\"ai_provider\",this.value)'><span>{}</span></label>", esc_html(p.note), p.id, chk(c.ai_provider == p.id), esc_html(p.name))).collect::<String>()
        + &format!("<label class='opt' title='Any site: use %s where the prompt goes'><input type='radio' name='ai' value='custom'{} onchange='set(\"ai_provider\",this.value)'><span>Custom\u{2026}</span></label>", chk(c.ai_provider == "custom"));
    let ai_pane = format!("<section class='pane' id='ai'><h2>Ask AI</h2><p class='note'>The \u{2726} button next to the address bar, <kbd>Alt+A</kbd>, or typing <kbd>@ai</kbd> sends your question to the AI you already pay for \u{2014} in its own website, signed in as you. Amni never proxies or stores the prompt.</p><div>{}</div><input type='text' value='{}' placeholder='Custom URL template, e.g. https://ai.example.com/?q=%s' onchange='set(\"ai_custom_url\",this.value)'><label class='switch'><input type='checkbox'{} onchange='set(\"ai_new_tab\",this.checked?1:0)'><span>Open answers in a new tab (off = reuse the current tab)</span></label><div class='call'><p><strong>How each one behaves</strong></p>{}</div><p class='dim'>\u{201c}Ask about this page\u{201d} (menu and right-click) sends the page title and address, not its contents; the AI reads the page itself. Selection prompts send the selected text.</p></section>",
        ai_radios, esc_html(c.ai_custom_url.as_deref().unwrap_or("")), chk(c.ai_new_tab), ai_search::PROVIDERS.iter().map(|p| format!("<p class='dim'><strong>{}</strong> \u{2014} {}</p>", esc_html(p.name), esc_html(p.note))).collect::<String>());
    let dl_dir = c.downloads_dir.clone().unwrap_or_else(|| DownloadManager::downloads_dir().to_string_lossy().to_string());
    let dl_pane = format!("<section class='pane' id='downloads'><h2>Downloads</h2><label>Save files to<input type='text' value='{}' placeholder='Downloads folder' onchange='set(\"downloads_dir\",this.value)'></label><label class='switch'><input type='checkbox'{} onchange='set(\"ask_download_location\",this.checked?1:0)'><span>Ask where to save each file before downloading</span></label><p class='dim'><kbd>Ctrl+J</kbd> opens the downloads list; <kbd>Ctrl+S</kbd> saves the current page as a single .mhtml file.</p></section>", esc_html(&dl_dir), chk(c.ask_download_location));
    let look_extra = format!("<label class='switch'><input type='checkbox'{} onchange='set(\"show_bookmarks_bar\",this.checked?1:0)'><span>Show bookmarks bar under the address bar (<kbd>Ctrl+Shift+B</kbd>)</span></label>", chk(c.show_bookmarks_bar));
    let perm_rows: String = state.permissions.sites.iter().filter(|s| !s.permissions.is_empty()).map(|s| format!("<div class='row'><span>{} <span class='dim'>{}</span></span><button class='x' onclick='cmd(\"perm_reset\",{{host:\"{}\"}});this.closest(\".row\").remove()'>reset</button></div>", esc_html(&s.site), esc_html(&s.permissions.iter().map(|(k, v)| format!("{}: {:?}", k, v).to_lowercase()).collect::<Vec<_>>().join(", ")), esc_html(&s.site))).collect();
    let priv_extra = format!("<label class='switch'><input type='checkbox'{} onchange='set(\"https_only\",this.checked?1:0)'><span>Always use secure connections (upgrade http:// to https://, fall back if it fails)</span></label><label class='switch'><input type='checkbox'{} onchange='set(\"memory_saver\",this.checked?1:0)'><span>Memory saver \u{2014} put background tabs to sleep after</span><select style='width:auto;display:inline-block;margin:0 0 0 8px' onchange='set(\"memory_saver_minutes\",this.value)'>{}</select></label><h2 style='margin-top:22px'>Site permissions</h2><p class='note'>Camera, microphone, location and notifications are asked per site and remembered here. Click the lock in the address bar to change a site.</p><div>{}</div>",
        chk(c.https_only), chk(c.memory_saver), [(15u32, "15 min"), (30, "30 min"), (45, "45 min"), (90, "1.5 h"), (240, "4 h")].iter().map(|(m, l)| format!("<option value='{}'{}>{}</option>", m, match *m == c.memory_saver_minutes { true => " selected", false => "" }, l)).collect::<String>(), match perm_rows.is_empty() { true => "<p class='dim'>No site has asked for anything yet.</p>".to_string(), false => perm_rows });
    SETTINGS_TPL.replace("__THEME__", &theme_root_vars(&state.themes.active_theme())).replace("__THEMES__", &themes).replace("__VER__", APP_VERSION).replace("__RADIOS__", &radios).replace("__HOME__", &esc_html(&home)).replace("__ZOOMS__", &zooms)
        .replace("__SHIELD__", chk(shield)).replace("__RESTORE__", chk(c.restore_session)).replace("__UA__", &esc_html(c.custom_user_agent.as_deref().unwrap_or(""))).replace("__TOK__", token)
        .replace("__VAULT__", "Chromium profile store").replace("__PMRADIOS__", &toggles).replace("__PMLABEL__", "").replace("__PMCLI__", "").replace("__PMDB__", "").replace("__AUTOFILL__", "").replace("__CHKUPD__", chk(c.check_updates))
        .replace("__UPD__", "checked on the site feed").replace("__PROFS__", "<div class='row'><span>Local \u{00b7} active</span></div>").replace("__CRASH__", "").replace("__IMPORTNOTE__", "").replace("__BMS__", &bms).replace("__ENGINE__", ENGINE)
        .replace("__NAVEXTRA__", "<button data-p='ai'>Ask AI</button><button data-p='downloads'>Downloads</button>").replace("__LOOKEXTRA__", &look_extra).replace("__AIPANE__", &ai_pane).replace("__DLPANE__", &dl_pane).replace("__PRIVEXTRA__", &priv_extra)
        .replace("function rmbm(id)", "function cmd(n,a){fetch('amnibrowse://cmd/'+n+'?'+new URLSearchParams(Object.assign({tok:T},a||{})),{mode:'no-cors'}).catch(function(){})}\nfunction rmbm(id)")
}
fn render_tutorial_html(state: &BrowserState, token: &str) -> String {
    let blurb = match cfg!(windows) { true => "Pages render in the Chromium engine (WebView2) under Amni\u{2019}s own chrome: no Google account, no sync, no telemetry.", false => "Pages render in WebKitGTK (the engine behind Safari and GNOME Web) under Amni\u{2019}s own chrome: no telemetry, no sync, your profile stays local." };
    let ai = ai_search::provider_name(&state.config);
    TUTORIAL_TPL.replace("__THEME__", &theme_root_vars(&state.themes.active_theme())).replace("__VER__", APP_VERSION).replace("__TOK__", token).replace("__BROWSERS__", "<p class='dim'>Import from Settings once you are in.</p>")
        .replace("__ENGINE__", ENGINE).replace("__ENGINEBLURB__", blurb)
        .replace("__MEDIANOTE__", "Video, audio, WebRTC and DRM-protected streams play through the engine as they would in any WebKit browser. The shield blocks ad and tracker requests at the request level.")
        .replace("__STEP3__", &format!("The \u{2726} button next to the address bar (or <kbd>Alt+A</kbd>, or typing <kbd>@ai</kbd>) hands your question to {} in its own website, signed in as you. Pick the AI you subscribe to under Settings \u{2192} Ask AI. Type <kbd>@tabs</kbd>, <kbd>@history</kbd> or <kbd>@bookmarks</kbd> to search those from the bar.", esc_html(&ai)))
}
fn render_downloads_html(state: &BrowserState, token: &str) -> String {
    let theme_vars = theme_root_vars(&state.themes.active_theme());
    let dl_dir = state.config.downloads_dir.clone().unwrap_or_else(|| DownloadManager::downloads_dir().to_string_lossy().to_string());
    let items = &state.downloads.downloads;
    let count = items.len();
    let rows: String = if items.is_empty() {
        "<div class='call' style='text-align:center;padding:32px 16px;'><p style='font-size:16px;font-weight:600;margin-bottom:8px;'>No downloads yet</p><p class='dim'>Files and exports you download will appear here.</p><p style='margin-top:16px;'><button class='btn primary' onclick='cmd(\"open_downloads_folder\")'>Open Downloads Folder</button></p></div>".to_string()
    } else {
        items.iter().rev().map(|d| {
            let status_badge = match d.status {
                DownloadStatus::Completed => "<span style='color:var(--success,#4ADE80);font-size:11px;font-weight:600;'>Completed</span>",
                DownloadStatus::Downloading => "<span style='color:var(--accent);font-size:11px;font-weight:600;'>Downloading...</span>",
                DownloadStatus::Failed => "<span style='color:var(--danger,#FF6B6B);font-size:11px;font-weight:600;'>Failed</span>",
                _ => "<span class='dim' style='font-size:11px;'>Pending</span>",
            };
            let size_str = if d.downloaded_bytes > 0 {
                if d.downloaded_bytes >= 1_048_576 {
                    format!("{:.1} MB", d.downloaded_bytes as f64 / 1_048_576.0)
                } else if d.downloaded_bytes >= 1024 {
                    format!("{:.1} KB", d.downloaded_bytes as f64 / 1024.0)
                } else {
                    format!("{} B", d.downloaded_bytes)
                }
            } else {
                String::new()
            };
            let date_str = d.created_at.format("%b %d, %Y %H:%M").to_string();
            let path_str = esc_html(&d.save_path.to_string_lossy());
            let fname = esc_html(&d.filename);
            let id = esc_html(&d.id);
            format!(
                "<div class='row' id='dl-{}' style='display:flex;align-items:center;justify-content:space-between;padding:14px 0;border-bottom:1px solid var(--stroke);'>\
                    <div style='flex:1;min-width:0;padding-right:16px;'>\
                        <div style='font-size:14px;font-weight:600;color:var(--text);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;'>{}</div>\
                        <div style='font-size:12px;color:var(--dim);margin-top:4px;'>{} &middot; {} &middot; <span title='{}'>{}</span></div>\
                    </div>\
                    <div style='display:flex;gap:8px;flex-shrink:0;'>\
                        <button class='btn' onclick='cmd(\"open_download\",{{id:\"{}\"}})' title='Open with default application'>Open</button>\
                        <button class='btn' onclick='cmd(\"show_in_folder\",{{id:\"{}\"}})' title='Show file in Downloads directory'>Folder</button>\
                        <button class='x' onclick='cmd(\"download_remove\",{{id:\"{}\"}});let el=document.getElementById(\"dl-{}\");if(el)el.remove()' title='Remove from list'>&times;</button>\
                    </div>\
                </div>",
                id, fname, status_badge, size_str, path_str, date_str, id, id, id, id
            )
        }).collect()
    };

    format!(r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>Downloads &#8212; Amni Browse</title><style>
:root{{{}}}
*{{box-sizing:border-box}}
body{{font:14px/1.5 'Segoe UI Variable Text','Segoe UI',sans-serif;margin:0;color:var(--text);background:var(--bg);min-height:100%;overflow-y:auto}}
.wrap{{display:flex;min-height:100vh}}
nav{{width:200px;flex:0 0 200px;border-right:1px solid var(--stroke);padding:28px 14px;background:var(--bg-secondary,#0D0F12)}}
nav .mark{{width:7px;height:7px;background:var(--accent);display:inline-block;margin-right:8px}}
nav h1{{font-size:13px;letter-spacing:.16em;text-transform:uppercase;margin:0 0 22px;font-weight:700}}
nav button{{display:block;width:100%;text-align:left;background:transparent;border:1px solid transparent;color:var(--dim);padding:8px 10px;margin:0 0 4px;border-radius:3px;cursor:pointer;font:650 11px/1.2 inherit;letter-spacing:.12em;text-transform:uppercase}}
nav button.on,nav button:hover{{color:var(--text);border-color:var(--stroke);background:var(--elev)}}
main{{flex:1;padding:32px 36px 80px;max-width:760px}}
h2{{color:var(--accent);font-size:11px;text-transform:uppercase;letter-spacing:.16em;margin:0 0 14px}}
.row{{display:flex;justify-content:space-between;align-items:center;padding:9px 0;border-bottom:1px solid var(--stroke)}}
.x,.btn{{background:var(--elev);border:1px solid var(--stroke);border-radius:3px;color:var(--text);padding:8px 14px;cursor:pointer;font:650 11px inherit;letter-spacing:.1em;text-transform:uppercase;margin:0 4px}}
.x:hover,.btn:hover{{border-color:var(--accent)}}
.btn.primary{{background:var(--accent);color:#08090B;border-color:transparent}}
.dim,.note{{color:var(--dim);font-size:13px;margin:8px 0}}
.call{{border:1px solid var(--stroke);background:var(--elev);border-radius:3px;padding:14px 16px;margin:0 0 14px}}
</style>
<script>
window.__amniToken={:?};
function cmd(name,args){{
    const qObj=Object.assign({{}},args||{{}});
    if(window.__amniToken)qObj.tok=window.__amniToken;
    const q='?'+new URLSearchParams(qObj).toString();
    fetch('amnibrowse://cmd/'+name+q,{{mode:'no-cors'}}).catch(()=>{{}});
}}
</script>
</head><body>
<div class='wrap'>
<nav>
<div><span class='mark'></span><h1 style='display:inline'>Amni</h1></div>
<p class='dim' style='letter-spacing:.1em;text-transform:uppercase;font-size:10px'>{} download(s)</p>
<button class='on'>Downloads</button>
<button onclick='window.location.href="amnibrowse://history"'>History</button>
<button onclick='window.location.href="amnibrowse://settings"'>Settings</button>
<button onclick='window.location.href="amnibrowse://newtab"'>New Tab</button>
</nav>
<main>
<div style='display:flex;justify-content:space-between;align-items:center;margin-bottom:20px;'>
    <div>
        <h2>Downloads</h2>
        <p class='dim' style='margin:0;'>Folder: {}</p>
    </div>
    <div style='display:flex;gap:8px;'>
        <button class='btn primary' onclick='cmd("open_downloads_folder")'>Open Folder</button>
        <button class='btn' onclick='cmd("download_clear");window.location.reload()'>Clear List</button>
    </div>
</div>
<div id='dl-list'>
{}
</div>
</main>
</div>
</body></html>"##, theme_vars, token, count, esc_html(&dl_dir), rows)
}
fn render_history_html(state: &BrowserState, token: &str) -> String {
    let theme_vars = theme_root_vars(&state.themes.active_theme());
    let entries = state.history.recent(500);
    let count = state.history.entries.len();
    let rows: String = if entries.is_empty() {
        "<div class='call' style='text-align:center;padding:32px 16px;'><p style='font-size:16px;font-weight:600;margin-bottom:8px;'>No browsing history</p><p class='dim'>Pages you visit will appear here.</p></div>".to_string()
    } else {
        let mut out = String::new();
        let mut last_day = String::new();
        let today = chrono::Local::now().date_naive();
        for e in entries {
            let local = e.last_visited.with_timezone(&chrono::Local);
            let day = local.date_naive();
            let label = match (today - day).num_days() { 0 => "Today".to_string(), 1 => "Yesterday".to_string(), _ => local.format("%A, %B %-d, %Y").to_string() };
            if label != last_day { out.push_str(&format!("<h2 class='day' style='margin-top:22px'>{}</h2>", esc_html(&label))); last_day = label; }
            let u = esc_html(&e.url);
            let t = if e.title.trim().is_empty() { u.clone() } else { esc_html(&e.title) };
            out.push_str(&format!("<div class='row hrow' data-t='{}' data-u='{}' style='display:flex;align-items:center;gap:12px;padding:8px 0;border-bottom:1px solid var(--stroke);'><span class='dim' style='font-size:12px;flex:0 0 46px;'>{}</span><div style='flex:1;min-width:0;'><a href='{}' style='font-size:14px;font-weight:600;color:var(--text);text-decoration:none;display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;'>{}</a><div style='font-size:12px;color:var(--dim);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;'>{}</div></div><button class='x' title='Remove from history' onclick='rm(this)'>&times;</button></div>", t.to_lowercase(), u.to_lowercase(), local.format("%H:%M"), u, t, u));
        }
        out
    };
    format!(r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>History &#8212; Amni Browse</title><style>
:root{{{}}}
*{{box-sizing:border-box}}
body{{font:14px/1.5 'Segoe UI Variable Text','Segoe UI',sans-serif;margin:0;color:var(--text);background:var(--bg);min-height:100%;overflow-y:auto}}
.wrap{{display:flex;min-height:100vh}}
nav{{width:200px;flex:0 0 200px;border-right:1px solid var(--stroke);padding:28px 14px;background:var(--bg-secondary,#0D0F12)}}
nav .mark{{width:7px;height:7px;background:var(--accent);display:inline-block;margin-right:8px}}
nav h1{{font-size:13px;letter-spacing:.16em;text-transform:uppercase;margin:0 0 22px;font-weight:700}}
nav button{{display:block;width:100%;text-align:left;background:transparent;border:1px solid transparent;color:var(--dim);padding:8px 10px;margin:0 0 4px;border-radius:3px;cursor:pointer;font:650 11px/1.2 inherit;letter-spacing:.12em;text-transform:uppercase}}
nav button.on,nav button:hover{{color:var(--text);border-color:var(--stroke);background:var(--elev)}}
main{{flex:1;padding:32px 36px 80px;max-width:760px}}
h2{{color:var(--accent);font-size:11px;text-transform:uppercase;letter-spacing:.16em;margin:0 0 14px}}
.row{{display:flex;justify-content:space-between;align-items:center;padding:9px 0;border-bottom:1px solid var(--stroke)}}
.row.hide{{display:none}}
.x,.btn{{background:var(--elev);border:1px solid var(--stroke);border-radius:3px;color:var(--text);padding:8px 14px;cursor:pointer;font:650 11px inherit;letter-spacing:.1em;text-transform:uppercase;margin:0 4px}}
.x{{padding:4px 9px;font-size:13px;color:var(--dim)}}
.x:hover,.btn:hover{{border-color:var(--accent)}}
.dim,.note{{color:var(--dim);font-size:13px;margin:8px 0}}
.call{{border:1px solid var(--stroke);background:var(--elev);border-radius:3px;padding:14px 16px;margin:0 0 14px}}
#q{{width:100%;max-width:460px;padding:9px 12px;background:var(--elev);border:1px solid var(--stroke);border-radius:3px;color:var(--text);font:inherit;margin:0 0 6px;outline:none}}
#q:focus{{border-color:var(--accent)}}
</style>
<script>
window.__amniToken={:?};
function cmd(name,args){{
    const qObj=Object.assign({{}},args||{{}});
    if(window.__amniToken)qObj.tok=window.__amniToken;
    const q='?'+new URLSearchParams(qObj).toString();
    fetch('amnibrowse://cmd/'+name+q,{{mode:'no-cors'}}).catch(()=>{{}});
}}
function rm(b){{const r=b.closest('.hrow');cmd('history_remove',{{url:r.querySelector('a').getAttribute('href')}});r.remove()}}
function filt(){{const v=document.getElementById('q').value.trim().toLowerCase();document.querySelectorAll('.hrow').forEach(r=>r.classList.toggle('hide',!!v&&!(r.dataset.t.includes(v)||r.dataset.u.includes(v))));document.querySelectorAll('h2.day').forEach(h=>{{let n=h.nextElementSibling,any=false;while(n&&!n.matches('h2.day')){{if(!n.classList.contains('hide'))any=true;n=n.nextElementSibling}}h.style.display=any?'':'none'}})}}
</script>
</head><body>
<div class='wrap'>
<nav>
<div><span class='mark'></span><h1 style='display:inline'>Amni</h1></div>
<p class='dim' style='letter-spacing:.1em;text-transform:uppercase;font-size:10px'>{} page(s)</p>
<button class='on'>History</button>
<button onclick='window.location.href="amnibrowse://downloads"'>Downloads</button>
<button onclick='window.location.href="amnibrowse://settings"'>Settings</button>
</nav>
<main>
<div style='display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;'>
    <div>
        <h2>History</h2>
        <input id='q' type='search' placeholder='Search history' oninput='filt()' autofocus>
    </div>
    <div>
        <button class='btn' onclick='if(confirm("Clear all browsing history?")){{cmd("clear_data");window.location.reload()}}'>Clear History</button>
    </div>
</div>
<div>
{}
</div>
</main>
</div>
</body></html>"##, theme_vars, token, count, rows)
}
fn render_page_html(state: &BrowserState, shield: bool, token: &str, host: &str) -> Option<String> {
    match host {
        "newtab" | "home" => Some(newtab_html(&state.themes.active_theme(), &state.bookmarks.bookmarks, ENGINE)),
        "settings" => Some(render_settings_html(state, shield, token)),
        "downloads" => Some(render_downloads_html(state, token)),
        "history" => Some(render_history_html(state, token)),
        "tutorial" => Some(render_tutorial_html(state, token)),
        _ => None,
    }
}
fn hex_rgba(hex: &str) -> Option<tao::window::RGBA> {
    let h = hex.trim().trim_start_matches('#');
    match h.len() { 6 => Some((u8::from_str_radix(&h[0..2], 16).ok()?, u8::from_str_radix(&h[2..4], 16).ok()?, u8::from_str_radix(&h[4..6], 16).ok()?, 255)), _ => None }
}
fn privacy_env(cfg: &crate::storage::config::BrowserConfig) {
    let mut args = String::from("--disable-features=msEdgeSmartScreen,AutoUpgradeAllUpgradableMixedContent,OptimizationHints,InterestGroupStorage,BrowsingTopics,PrivacySandboxSettings4,msEdgeCollections,msShoppingTrigger,msEdgeSidebarV2 --disable-background-networking --disable-sync --disable-breakpad --disable-domain-reliability --no-default-browser-check --no-first-run --no-pings");
    if cfg.enable_doh {
        let tpl = match cfg.doh_provider.as_str() { p if p.starts_with("http") => p.to_string(), "quad9" => "https://dns.quad9.net/dns-query".into(), "google" => "https://dns.google/dns-query".into(), _ => "https://cloudflare-dns.com/dns-query".into() };
        args.push_str(&format!(" --enable-features=DnsOverHttps --dns-over-https-mode=secure --dns-over-https-templates={}", tpl));
    }
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);
    #[cfg(not(windows))]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() && std::env::var("AMNI_VM").map(|v| v == "1").unwrap_or(false) { std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1"); }
    }
    if let Some(dir) = dirs::config_dir() {
        let ud = dir.join("amni-browse").join("webview2-data");
        std::fs::create_dir_all(&ud).ok();
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", ud);
    }
}
/// WebKitGTK request shield: the ad/tracker lists compiled once into a WebKit content rule list
/// (blocks subresources like the Windows `WebResourceRequested` hook) and attached to every tab.
#[cfg(not(windows))]
unsafe extern "C" fn on_filter_saved(src: *mut webkit2gtk::glib::gobject_ffi::GObject, res: *mut webkit2gtk::gio::ffi::GAsyncResult, data: webkit2gtk::glib::ffi::gpointer) {
    let mut err: *mut webkit2gtk::glib::ffi::GError = std::ptr::null_mut();
    let f = webkit2gtk_sys::webkit_user_content_filter_store_save_finish(src as *mut webkit2gtk_sys::WebKitUserContentFilterStore, res, &mut err);
    if !err.is_null() { let e: webkit2gtk::glib::Error = webkit2gtk::glib::translate::from_glib_full(err); warn!("shield: content rule list failed: {}", e); }
    let done: Rc<Cell<Option<usize>>> = Rc::from_raw(data as *const Cell<Option<usize>>);
    done.set(Some(f as usize));
}
#[cfg(not(windows))]
fn compile_filter() -> Option<usize> {
    let dir = dirs::config_dir()?.join("amni-browse").join("filters");
    std::fs::create_dir_all(&dir).ok();
    let path = std::ffi::CString::new(dir.to_str()?).ok()?;
    let id = std::ffi::CString::new("amni-shield").ok()?;
    let rules = AdBlocker::content_rules();
    let done: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
    unsafe {
        let store = webkit2gtk_sys::webkit_user_content_filter_store_new(path.as_ptr());
        let bytes = webkit2gtk::glib::ffi::g_bytes_new(rules.as_ptr() as *const _, rules.len());
        webkit2gtk_sys::webkit_user_content_filter_store_save(store, id.as_ptr(), bytes, std::ptr::null_mut(), Some(on_filter_saved), Rc::into_raw(done.clone()) as *mut _);
        let ctx = webkit2gtk::glib::MainContext::default();
        while done.get().is_none() { ctx.iteration(true); }
        webkit2gtk::glib::ffi::g_bytes_unref(bytes);
    }
    let f = done.get()?;
    match f { 0 => None, _ => { info!("shield: content rule list compiled ({} bytes)", rules.len()); Some(f) } }
}
#[cfg(not(windows))]
fn attach_filter(view: &WebView, filter: Option<usize>, on: bool) {
    use webkit2gtk::glib::translate::ToGlibPtr;
    use webkit2gtk::WebViewExt;
    use wry::WebViewExtUnix;
    if let (Some(f), Some(m)) = (filter, view.webview().user_content_manager()) {
        let mp: *mut webkit2gtk_sys::WebKitUserContentManager = m.to_glib_none().0;
        unsafe { match on { true => webkit2gtk_sys::webkit_user_content_manager_add_filter(mp, f as *mut _), false => webkit2gtk_sys::webkit_user_content_manager_remove_filter(mp, f as *mut _) } }
    }
}
/// Everything wry does not expose: request-level shield + DNT/GPC headers, real history state,
/// favicons, HTML5 fullscreen, audio state, download progress, password/form autofill.
/// Linux: history, HTML5 fullscreen and audio state come straight from WebKitGTK signals on the
/// underlying WebKitWebView; favicons arrive through the ICON_SCRIPT ipc note; downloads report
/// completion through wry's download-completed handler (see spawn_tab).
#[cfg(not(windows))]
fn perm_kind(req: &webkit2gtk::PermissionRequest) -> Option<PermissionType> {
    use webkit2gtk::glib::object::Cast;
    use webkit2gtk::UserMediaPermissionRequestExt;
    if req.dynamic_cast_ref::<webkit2gtk::GeolocationPermissionRequest>().is_some() { return Some(PermissionType::Location); }
    if req.dynamic_cast_ref::<webkit2gtk::NotificationPermissionRequest>().is_some() { return Some(PermissionType::Notifications); }
    if let Some(m) = req.dynamic_cast_ref::<webkit2gtk::UserMediaPermissionRequest>() { return Some(match m.is_for_video_device() { true => PermissionType::Camera, false => PermissionType::Microphone }); }
    None
}
#[cfg(not(windows))]
fn origin_of(uri: &str) -> String { url::Url::parse(uri).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| match uri.starts_with("file:") { true => "This local file".into(), false => uri.to_string() }) }
/// Errors WebKit reports for loads we cancelled ourselves (stop, policy ignore, a download
/// taking over the navigation); those never get an error page.
/// WebKit ITP and the default third-party cookie block drop the cookies Google,
/// Apple, X and xAI need during sign-in. Firefox's partitioning still completes
/// those top-level and popup flows. Relax both on this profile.
#[cfg(not(windows))]
fn allow_auth_storage(wv: &webkit2gtk::WebView) {
    use webkit2gtk::{CookieAcceptPolicy, CookieManagerExt, WebContextExt, WebViewExt, WebsiteDataManagerExt};
    let Some(ctx) = wv.context() else { return };
    if let Some(cm) = ctx.cookie_manager() {
        cm.set_accept_policy(CookieAcceptPolicy::Always);
    }
    if let Some(dm) = ctx.website_data_manager() {
        dm.set_itp_enabled(false);
    }
}
/// `window.open` for an auth provider is allowed by the policy handler, which
/// makes WebKit emit `create`. Nothing was connected, so the call returned null
/// and xAI (and the other providers) showed "Something went wrong". A related
/// view keeps `window.opener` and the session cookies.
#[cfg(not(windows))]
thread_local! {
    static AUTH_POPUPS: RefCell<Vec<gtk::Window>> = RefCell::new(Vec::new());
}
#[cfg(not(windows))]
fn attach_auth_popup(parent: &webkit2gtk::WebView) {
    use gtk::prelude::{Cast, ContainerExt, GtkWindowExt, WidgetExt};
    use webkit2gtk::{URIRequestExt, WebViewExt};
    parent.connect_create(|parent, action| {
        let uri = action.request().and_then(|r| r.uri()).map(|u| u.to_string()).unwrap_or_default();
        let child = webkit2gtk::WebView::with_related_view(parent);
        let win = gtk::Window::new(gtk::WindowType::Toplevel);
        win.set_title("Sign in");
        win.set_default_size(520, 720);
        if let Some(top) = parent.toplevel() {
            if let Ok(pw) = top.downcast::<gtk::Window>() {
                win.set_transient_for(Some(&pw));
            }
        }
        win.set_destroy_with_parent(true);
        child.set_hexpand(true);
        child.set_vexpand(true);
        win.add(&child);
        let win_close = win.clone();
        child.connect_close(move |_| { win_close.close(); });
        let tracked = win.clone();
        win.connect_destroy(move |_| {
            AUTH_POPUPS.with(|popups| popups.borrow_mut().retain(|w| w != &tracked));
        });
        AUTH_POPUPS.with(|popups| popups.borrow_mut().push(win.clone()));
        win.show_all();
        info!("auth popup: {}", uri);
        Some(child.upcast())
    });
}
#[cfg(not(windows))]
fn benign_load_error(e: &webkit2gtk::glib::Error) -> bool {
    if let Some(n) = e.kind::<webkit2gtk::NetworkError>() { return matches!(n, webkit2gtk::NetworkError::Cancelled); }
    e.kind::<webkit2gtk::PolicyError>().is_some() || e.kind::<webkit2gtk::PluginError>().is_some()
}
#[cfg(not(windows))]
fn wire_engine(view: &WebView, uid: u64, push: Push, _blocker: Rc<RefCell<AdBlocker>>, _shield: Rc<Cell<bool>>, dnt: bool, _autofill: bool, state: Rc<RefCell<BrowserState>>) -> Option<Core> {
    use wry::WebViewExtUnix;
    use webkit2gtk::{ContextMenuExt, HitTestResultExt, NotificationExt, PermissionRequestExt, SettingsExt, WebViewExt};
    use webkit2gtk::gio::prelude::ActionExt;
    if dnt {
        let _ = view.evaluate_script("try{Object.defineProperty(navigator,'doNotTrack',{value:'1',configurable:false});Object.defineProperty(navigator,'globalPrivacyControl',{value:true,configurable:false})}catch(e){}");
    }
    let wv = view.webview();
    if let Some(settings) = WebViewExt::settings(&wv) {
        settings.set_enable_developer_extras(true);
        settings.set_enable_webgl(true);
        settings.set_enable_webaudio(true);
        settings.set_enable_media_stream(true);
        settings.set_enable_mediasource(true);
        settings.set_enable_webrtc(true);
        settings.set_enable_smooth_scrolling(true);
        settings.set_enable_encrypted_media(true);
        settings.set_enable_back_forward_navigation_gestures(true);
        settings.set_enable_page_cache(true);
        settings.set_enable_site_specific_quirks(true);
        settings.set_javascript_can_access_clipboard(true);
        // Sign-in buttons often await a token, then window.open. WebKit treats
        // that as not a user gesture unless this is on, and the call returns null.
        settings.set_javascript_can_open_windows_automatically(true);
    }
    allow_auth_storage(&wv);
    attach_auth_popup(&wv);
    let (p1, p2, p3, p4) = (push.clone(), push.clone(), push.clone(), push.clone());
    wv.connect_load_changed(move |w, _| p1(Ev::History(uid, w.can_go_back(), w.can_go_forward())));
    wv.connect_enter_fullscreen(move |_| { p2(Ev::PageFullscreen(uid, true)); false });
    wv.connect_leave_fullscreen(move |_| { p3(Ev::PageFullscreen(uid, false)); false });
    wv.connect_is_playing_audio_notify(move |w| p4(Ev::Audio(uid, w.is_playing_audio())));
    // Permission prompts: stored per-site answers are applied silently, everything else asks in the chrome.
    let (pp, ps) = (push.clone(), state.clone());
    wv.connect_permission_request(move |w, req| {
        use webkit2gtk::glib::object::Cast;
        let origin = w.uri().map(|u| u.to_string()).unwrap_or_default();
        if req.dynamic_cast_ref::<webkit2gtk::PointerLockPermissionRequest>().is_some() || req.dynamic_cast_ref::<webkit2gtk::MediaKeySystemPermissionRequest>().is_some() || req.dynamic_cast_ref::<webkit2gtk::WebsiteDataAccessPermissionRequest>().is_some() { req.allow(); return true; }
        let Some(kind) = perm_kind(req) else { req.deny(); return true; };
        let stored = ps.try_borrow().map(|st| st.permissions.get_permission(&origin, &kind)).unwrap_or(PermissionState::Ask);
        match stored {
            PermissionState::Allow => req.allow(),
            PermissionState::Deny => req.deny(),
            PermissionState::Ask => pp(Ev::Perm(uid, kind, origin_of(&origin), req.clone())),
        }
        true
    });
    // Web Notifications go to the desktop through notify-send (KDE/GNOME both pick it up).
    wv.connect_show_notification(move |w, n| {
        let host = w.uri().map(|u| origin_of(&u)).unwrap_or_default();
        let _ = std::process::Command::new("notify-send").arg("-a").arg(APP_NAME).arg(n.title().map(|t| t.to_string()).unwrap_or(host)).arg(n.body().map(|b| b.to_string()).unwrap_or_default()).spawn();
        true
    });
    let pf = push.clone();
    wv.connect_load_failed(move |_, _, uri, err| {
        if benign_load_error(err) { return false; }
        pf(Ev::LoadFailed(uid, uri.to_string(), err.to_string()));
        true
    });
    let pt = push.clone();
    wv.connect_load_failed_with_tls_errors(move |_, uri, cert, flags| { pt(Ev::TlsFail(uid, uri.to_string(), format!("{:?}", flags), cert.clone())); true });
    let pc = push.clone();
    wv.connect_web_process_terminated(move |_, reason| { warn!("tab {} web process terminated: {:?}", uid, reason); pc(Ev::Crash(uid)); });
    // Page context menu: Chrome's extras (open in new/private tab, search selection, Ask AI) on top of WebKit's stock items.
    let (pm, sm) = (push.clone(), state.clone());
    wv.connect_context_menu(move |_, menu, _, hit| {
        let ai = sm.try_borrow().map(|st| ai_search::provider_name(&st.config)).unwrap_or_else(|_| "AI".into());
        let link = hit.link_uri().map(|u| u.to_string()).filter(|u| u.starts_with("http"));
        let image = hit.image_uri().map(|u| u.to_string());
        let mut items: Vec<(String, String, String)> = Vec::new();
        if let Some(l) = link.clone() { items.push(("open_bg".into(), "Open link in new tab".into(), l.clone())); items.push(("open_private".into(), "Open link in private tab".into(), l)); }
        if let Some(i) = image.clone() { items.push(("open_bg".into(), "Open image in new tab".into(), i)); }
        if hit.context_is_selection() { items.push(("search_sel".into(), "Search the web for selection".into(), String::new())); items.push(("ask_sel".into(), format!("Ask {} about selection", ai), String::new())); }
        if link.is_none() && image.is_none() && !hit.context_is_selection() && !hit.context_is_editable() { items.push(("ask_page".into(), format!("Ask {} about this page", ai), String::new())); items.push(("screenshot".into(), "Take screenshot".into(), String::new())); }
        if items.is_empty() { return false; }
        let mut pos = 0;
        for (n, (id, label, data)) in items.into_iter().enumerate() {
            // The menu item holds the action; the closure holds the pick.
            let act = webkit2gtk::gio::SimpleAction::new(&format!("amni{}", n), None);
            let (p, id2, d2) = (pm.clone(), id.clone(), data.clone());
            act.connect_activate(move |_, _| p(Ev::Ctx(uid, id2.clone(), d2.clone())));
            menu.insert(&webkit2gtk::ContextMenuItem::from_gaction(&act, &label, None), pos);
            pos += 1;
        }
        menu.insert(&webkit2gtk::ContextMenuItem::new_separator(), pos);
        false
    });
    None
}
#[cfg(not(windows))]
fn dl_id(u: &str) -> String { format!("{:x}", u.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))) }
#[cfg(windows)]
fn wire_engine(view: &WebView, uid: u64, push: Push, blocker: Rc<RefCell<AdBlocker>>, shield: Rc<Cell<bool>>, dnt: bool, autofill: bool, _state: Rc<RefCell<BrowserState>>) -> Option<ICoreWebView2> {
    unsafe {
        let core = view.controller().CoreWebView2().ok()?;
        let env = core.cast::<ICoreWebView2_2>().and_then(|c| c.Environment()).ok()?;
        if let Ok(settings) = core.Settings() {
            let _ = settings.SetIsStatusBarEnabled(BOOL(0));
            let _ = settings.SetAreDefaultContextMenusEnabled(BOOL(1));
            if let Ok(s4) = settings.cast::<ICoreWebView2Settings4>() { let _ = s4.SetIsPasswordAutosaveEnabled(BOOL(autofill as i32)); let _ = s4.SetIsGeneralAutofillEnabled(BOOL(autofill as i32)); }
        }
        let mut token = EventRegistrationToken::default();
        let _ = core.AddWebResourceRequestedFilter(&HSTRING::from("*"), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL);
        let _ = core.add_WebResourceRequested(&WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else { return Ok(()) };
            let req = args.Request()?;
            let mut p = PWSTR::null();
            req.Uri(&mut p)?;
            let uri = take_pwstr(p);
            if dnt { if let Ok(h) = req.Headers() { let _ = h.SetHeader(w!("DNT"), w!("1")); let _ = h.SetHeader(w!("Sec-GPC"), w!("1")); } }
            if shield.get() && !is_internal(&uri) && blocker.borrow_mut().should_block(&uri) {
                let resp = env.CreateWebResourceResponse(None, 403, w!("Blocked by Amni Shield"), w!("Content-Type: text/plain"))?;
                args.SetResponse(&resp)?;
            }
            Ok(())
        })), &mut token);
        let (p1, p2, p3, p4, p5) = (push.clone(), push.clone(), push.clone(), push.clone(), push.clone());
        let _ = core.add_HistoryChanged(&HistoryChangedEventHandler::create(Box::new(move |wv, _| {
            if let Some(wv) = wv { let (mut b, mut f) = (BOOL(0), BOOL(0)); let _ = wv.CanGoBack(&mut b); let _ = wv.CanGoForward(&mut f); p1(Ev::History(uid, b.as_bool(), f.as_bool())); }
            Ok(())
        })), &mut token);
        let _ = core.add_ContainsFullScreenElementChanged(&ContainsFullScreenElementChangedEventHandler::create(Box::new(move |wv, _| {
            if let Some(wv) = wv { let mut on = BOOL(0); let _ = wv.ContainsFullScreenElement(&mut on); p2(Ev::PageFullscreen(uid, on.as_bool())); }
            Ok(())
        })), &mut token);
        if let Ok(c15) = core.cast::<ICoreWebView2_15>() {
            let _ = c15.add_FaviconChanged(&FaviconChangedEventHandler::create(Box::new(move |wv, _| {
                if let Some(wv) = wv { if let Ok(c) = wv.cast::<ICoreWebView2_15>() { let mut p = PWSTR::null(); let _ = c.FaviconUri(&mut p); p3(Ev::Favicon(uid, take_pwstr(p))); } }
                Ok(())
            })), &mut token);
        }
        if let Ok(c8) = core.cast::<ICoreWebView2_8>() {
            let _ = c8.add_IsDocumentPlayingAudioChanged(&IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |wv, _| {
                if let Some(wv) = wv { if let Ok(c) = wv.cast::<ICoreWebView2_8>() { let mut on = BOOL(0); let _ = c.IsDocumentPlayingAudio(&mut on); p4(Ev::Audio(uid, on.as_bool())); } }
                Ok(())
            })), &mut token);
        }
        if let Ok(c4) = core.cast::<ICoreWebView2_4>() {
            let _ = c4.add_DownloadStarting(&DownloadStartingEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let op = args.DownloadOperation()?;
                let (mut u, mut path) = (PWSTR::null(), PWSTR::null());
                let _ = op.Uri(&mut u);
                let _ = op.ResultFilePath(&mut path);
                let mut total = 0i64;
                let _ = op.TotalBytesToReceive(&mut total);
                let id = format!("dl{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0));
                p5(Ev::DlStart(id.clone(), take_pwstr(u), take_pwstr(path), (total > 0).then_some(total as u64)));
                let (pb, ps, idb, ids) = (p5.clone(), p5.clone(), id.clone(), id);
                let mut t1 = EventRegistrationToken::default();
                let _ = op.add_BytesReceivedChanged(&BytesReceivedChangedEventHandler::create(Box::new(move |o, _| { if let Some(o) = o { let mut n = 0i64; let _ = o.BytesReceived(&mut n); pb(Ev::DlProgress(idb.clone(), n.max(0) as u64)); } Ok(()) })), &mut t1);
                let mut t2 = EventRegistrationToken::default();
                let _ = op.add_StateChanged(&StateChangedEventHandler::create(Box::new(move |o, _| { if let Some(o) = o { let mut st = COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS; let _ = o.State(&mut st); let mut path = PWSTR::null(); let _ = o.ResultFilePath(&mut path); ps(Ev::DlState(ids.clone(), st.0, take_pwstr(path))); } Ok(()) })), &mut t2);
                Ok(())
            })), &mut token);
        }
        Some(core)
    }
}
impl App {
    fn scale(&self) -> f64 { self.window.scale_factor() }
    #[cfg(windows)]
    fn host(&self) -> &Window { &self.window }
    #[cfg(not(windows))]
    fn host(&self) -> &gtk::Layout { &self.canvas }
    #[cfg(windows)]
    fn chrome_host(&self) -> &Window { &self.window }
    #[cfg(not(windows))]
    fn chrome_host(&self) -> &gtk::Layout { &self.chrome_canvas }
    #[cfg(windows)]
    fn place(&self, v: &WebView, r: Rect) { let _ = v.set_bounds(r); }
    #[cfg(not(windows))]
    fn place_in(&self, canvas: &gtk::Layout, v: &WebView, r: Rect) {
        use wry::WebViewExtUnix;
        let s = self.scale();
        let (x, y): (i32, i32) = r.position.to_logical::<i32>(s).into();
        let (w, h): (i32, i32) = r.size.to_logical::<i32>(s).into();
        let wv = v.webview();
        canvas.move_(&wv, x, y);
        wv.set_size_request(w.max(1), h.max(1));
    }
    #[cfg(not(windows))]
    fn place(&self, v: &WebView, r: Rect) {
        self.place_in(&self.canvas, v, r);
    }
    #[cfg(windows)]
    fn place_chrome(&self, c: &WebView, r: Rect) { self.place(c, r); }
    #[cfg(not(windows))]
    fn place_chrome(&self, c: &WebView, r: Rect) {
        use gtk::prelude::*;
        use wry::WebViewExtUnix;
        let s = self.scale();
        let (x, y): (i32, i32) = r.position.to_logical::<i32>(s).into();
        let (w, h): (i32, i32) = r.size.to_logical::<i32>(s).into();
        let wv = c.webview();
        self.chrome_canvas.move_(&wv, x, y);
        wv.set_size_request(w.max(1), h.max(1));
        self.chrome_canvas.set_size_request(w.max(1), h.max(1));
        self.chrome_canvas.queue_resize();
        self.overlay.check_resize();
    }
    fn place_view(&self, v: &View, r: Rect) {
        match v {
            View::Wry(w) => self.place(w, r),
            #[cfg(all(feature = "cef-engine", target_os = "linux"))]
            View::Cef(c) => { let s = self.scale(); let p = r.position.to_physical::<i32>(s); let z = r.size.to_physical::<u32>(s); c.place((p.x, p.y, z.width as i32, z.height as i32)); }
        }
    }
    fn frame_px(&self) -> u32 { match self.decorated || self.fullscreen || self.page_fullscreen || self.window.is_maximized() { true => 0, false => (FRAME_CSS * self.scale()).round() as u32 } }
    fn chrome_css(&self) -> u32 { SERVO_CHROME_HEIGHT_CSS + match self.bookmarks_bar { true => BOOKMARKS_BAR_CSS, false => 0 } }
    fn chrome_px(&self) -> u32 { match self.fullscreen || self.page_fullscreen { true => 0, false => (self.chrome_css() as f64 * self.scale()).round() as u32 } }
    fn chrome_rect(&self) -> Rect {
        let sz = self.window.inner_size();
        let f = self.frame_px();
        let h = ((self.overlay_css as f64 * self.scale()).round() as u32).max(self.chrome_px()).min(sz.height.saturating_sub(f).max(1));
        Rect { position: PhysicalPosition::new(f as i32, f as i32).into(), size: PhysicalSize::new(sz.width.saturating_sub(2 * f).max(1), h.max(1)).into() }
    }
    fn content_rect(&self) -> Rect {
        let sz = self.window.inner_size();
        let f = self.frame_px();
        let y = (self.chrome_px() + f).min(sz.height.saturating_sub(1));
        Rect { position: PhysicalPosition::new(f as i32, y as i32).into(), size: PhysicalSize::new(sz.width.saturating_sub(2 * f).max(1), sz.height.saturating_sub(y + f).max(1)).into() }
    }
    fn rect_key(r: &Rect, scale: f64) -> (i32, i32, u32, u32) {
        let p = r.position.to_physical::<i32>(scale);
        let s = r.size.to_physical::<u32>(scale);
        (p.x, p.y, s.width, s.height)
    }
    fn css_rgb(hex: &str) -> (f64, f64, f64) {
        let h = hex.trim().trim_start_matches('#');
        if h.len() >= 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (u8::from_str_radix(&h[0..2], 16), u8::from_str_radix(&h[2..4], 16), u8::from_str_radix(&h[4..6], 16)) {
                return (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
            }
        }
        (8.0 / 255.0, 9.0 / 255.0, 11.0 / 255.0)
    }
    /// The chrome surface is transparent so menus can float over the page. While it
    /// is only the toolbar, an opaque backdrop stops page repaints from flashing
    /// through the omnibar. Menus expand the surface and need the clear color back.
    fn paint_chrome_backdrop(&self) {
        #[cfg(not(windows))]
        {
            use webkit2gtk::WebViewExt;
            use wry::WebViewExtUnix;
            let Some(c) = self.chrome.as_ref() else { return };
            let expanded = self.overlay_css > self.chrome_css().saturating_add(2);
            let (r, g, b, a) = if expanded {
                (0.0, 0.0, 0.0, 0.0)
            } else {
                let hex = self.state().themes.active_theme().bg_primary.clone();
                let (r, g, b) = Self::css_rgb(&hex);
                (r, g, b, 1.0)
            };
            c.webview().set_background_color(&gtk::gdk::RGBA::new(r, g, b, a));
        }
    }
    fn layout(&self) {
        let hide_chrome = self.fullscreen || self.page_fullscreen;
        let scale = self.scale();
        if let Some(c) = self.chrome.as_ref() {
            let cr = self.chrome_rect();
            let ck = Self::rect_key(&cr, scale);
            if self.last_chrome.get() != ck {
                self.place_chrome(c, cr);
                self.last_chrome.set(ck);
                self.paint_chrome_backdrop();
            }
            let _ = c.set_visible(!hide_chrome);
            #[cfg(not(windows))]
            {
                use gtk::prelude::WidgetExt;
                self.chrome_canvas.set_visible(!hide_chrome);
            }
        }
        let r = self.content_rect();
        let rk = Self::rect_key(&r, scale);
        let moved = self.last_content.get() != rk;
        if moved { self.last_content.set(rk); }
        for (i, t) in self.tabs.iter().enumerate() {
            if moved { self.place_view(&t.view, r); }
            let _ = t.view.set_visible(i == self.active);
        }
        if moved { self.raise_chrome(); }
    }
    #[cfg(windows)]
    fn raise_chrome(&self) {
        if self.chrome_hwnd != 0 { unsafe { SetWindowPos(self.chrome_hwnd as HWND, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE); } }
    }
    #[cfg(not(windows))]
    fn raise_chrome(&self) {}
    #[cfg(windows)]
    fn go_back(&self) { self.with_core(|c| unsafe { let _ = c.GoBack(); }); }
    #[cfg(windows)]
    fn go_forward(&self) { self.with_core(|c| unsafe { let _ = c.GoForward(); }); }
    #[cfg(windows)]
    fn reload_page(&self) { self.with_core(|c| unsafe { let _ = c.Reload(); }); }
    #[cfg(windows)]
    fn stop_page(&self) { self.with_core(|c| unsafe { let _ = c.Stop(); }); }
    #[cfg(windows)]
    fn open_devtools(&self) { self.with_core(|c| unsafe { let _ = c.OpenDevToolsWindow(); }); }
    #[cfg(not(windows))]
    fn go_back(&self) {
        if let Some(t) = self.active_tab() { t.view.go_back(); }
    }
    #[cfg(not(windows))]
    fn go_forward(&self) {
        if let Some(t) = self.active_tab() { t.view.go_forward(); }
    }
    #[cfg(not(windows))]
    fn reload_page(&self) {
        if let Some(t) = self.active_tab() { t.view.reload(false); }
    }
    #[cfg(not(windows))]
    fn stop_page(&self) {
        if let Some(t) = self.active_tab() { t.view.stop(); }
    }
    #[cfg(not(windows))]
    fn open_devtools(&self) { if let Some(t) = self.active_tab() { t.view.open_devtools(); } }
    fn state(&self) -> std::cell::Ref<BrowserState> { self.state.borrow() }
    fn state_mut(&self) -> std::cell::RefMut<BrowserState> { self.state.borrow_mut() }
    fn active_tab(&self) -> Option<&Tab> { self.tabs.get(self.active) }
    fn tab_index(&self, uid: u64) -> Option<usize> { self.tabs.iter().position(|t| t.uid == uid) }
    fn home_url(&self) -> String {
        let st = self.state();
        if !st.config.seen_onboarding || std::env::var("AMNI_TUTORIAL").is_ok() { return internal_url("tutorial"); }
        let hp = st.config.home_page.trim().to_string();
        match hp.starts_with("http") { true => hp, false => internal_url("newtab") }
    }
    fn pusher(&self) -> Push {
        let ev = self.events.clone();
        let px = self.proxy.clone();
        Rc::new(move |e: Ev| { ev.borrow_mut().push(e); let _ = px.send_event(()); })
    }
    fn reshield(&mut self) {
        #[cfg(not(windows))]
        for t in &self.tabs { if let Some(v) = t.view.wry() { attach_filter(v, self.filter, self.shield.get()); } }
    }
    #[cfg(all(feature = "cef-engine", target_os = "linux"))]
    fn spawn_cef_tab(&mut self, url: &str, private: bool, at: Option<usize>) -> Option<usize> {
        let uid = self.next_uid;
        let (s, r) = (self.scale(), self.content_rect());
        let (p, z) = (r.position.to_physical::<i32>(s), r.size.to_physical::<u32>(s));
        let c = super::cef_tabs::CefTab::new(uid, &self.canvas, (p.x, p.y, z.width as i32, z.height as i32), url)?;
        self.next_uid += 1;
        let zoom = self.site_zoom_for(url).unwrap_or(self.state().config.default_zoom);
        c.zoom(zoom);
        let tab = Tab { uid, view: View::Cef(c), core: None, url: url.to_string(), title: String::new(), private, loading: true, zoom, can_back: false, can_forward: false, icon: None, audio: false, pinned: false, group: None, muted: false, discarded: false, last_active: Instant::now(), inject: None, upgraded_from: None, tls: None, find: (0, 0) };
        let idx = at.unwrap_or(self.tabs.len()).min(self.tabs.len());
        self.tabs.insert(idx, tab);
        Some(idx)
    }
    #[cfg(all(feature = "cef-engine", target_os = "linux"))]
    fn cef_event(&mut self, e: super::cef_tabs::CefEv) {
        use super::cef_tabs::CefEv as C;
        match e {
            C::DlWanted(id, url, name) => {
                let dir = self.downloads_dir();
                std::fs::create_dir_all(&dir).ok();
                let fallback = unique_path(&dir, &sanitize_filename(if name.trim().is_empty() { "download" } else { &name }));
                match if self.ask_dl_location.get() { pick_save_path(&fallback) } else { Some(fallback) } {
                    Some(p) => { super::cef_tabs::download_to(id, &p); self.handle(Ev::DlStart(format!("cef{}", id), url, p.to_string_lossy().to_string(), None)); }
                    None => super::cef_tabs::cancel_download(id),
                }
            }
            C::DlProgress(id, got, total) => {
                let mut st = self.state_mut();
                if let Some(d) = st.downloads.downloads.iter_mut().find(|d| d.id == format!("cef{}", id)) { d.downloaded_bytes = got.max(0) as u64; if total > 0 { d.total_bytes = Some(total as u64); } }
            }
            C::DlDone(id, ok, path) => self.handle(Ev::DlState(format!("cef{}", id), if ok { DL_COMPLETED } else { DL_INTERRUPTED }, path)),
            C::Title(u, t) => self.handle(Ev::Title(u, t)),
            C::Address(u, url) => { let l = self.tab_index(u).map(|i| self.tabs[i].loading).unwrap_or(true); self.handle(Ev::Load(u, l, url)) }
            C::Loading(u, l, b, f) => { self.handle(Ev::History(u, b, f)); let url = self.tab_index(u).map(|i| self.tabs[i].url.clone()).unwrap_or_default(); self.handle(Ev::Load(u, l, url)) }
            C::Popup(url) => self.handle(Ev::Popup(url)),
            C::Ipc(u, body) => { if let Some(ev) = ipc_ev(u, &body) { self.handle(ev) } }
            C::Fullscreen(u, on) => self.handle(Ev::PageFullscreen(u, on)),
            C::Created(u) => { debug!("cef created {} active={:?}", u, self.active_tab().map(|t| t.uid)); self.last_content.set((0, 0, 0, 0)); self.layout(); }
        }
    }
    fn spawn_tab(&mut self, url: &str, private: bool, at: Option<usize>) -> usize {
        #[cfg(all(feature = "cef-engine", target_os = "linux"))]
        if !private && super::cef_tabs::wants(url) { if let Some(i) = self.spawn_cef_tab(url, private, at) { debug!("tab {} cef {}", self.tabs[i].uid, url); return i; } }
        let uid = self.next_uid;
        self.next_uid += 1;
        let push = self.pusher();
        let (p1, p2, p3, p4) = (push.clone(), push.clone(), push.clone(), push.clone());
        #[cfg(not(windows))]
        let (p5, p6) = (push.clone(), push.clone());
        let proto = self.protocol.clone();
        let blocker = self.blocker.clone();
        let shield = self.shield.clone();
        let (ua, dl_dir, default_zoom, autofill, dnt) = {
            let st = self.state();
            (
                st.config.custom_user_agent.clone().filter(|u| !u.trim().is_empty()).unwrap_or_else(|| UA.to_string()),
                st.config.downloads_dir.clone().map(PathBuf::from).unwrap_or_else(DownloadManager::downloads_dir),
                st.config.default_zoom,
                !private && st.config.autofill_on_load,
                st.config.enable_do_not_track,
            )
        };
        let content_rect = self.content_rect();
        #[cfg(not(windows))]
        let host = self.canvas.clone();
        #[cfg(not(windows))]
        let already_registered = !private && self.proto_registered;
        #[cfg(windows)]
        let already_registered = false;
        if !private {
            self.proto_registered = true;
        }
        let attach_downloads = private || !self.dl_handler_registered;
        if attach_downloads && !private {
            self.dl_handler_registered = true;
        }

        #[cfg(not(windows))]
        let mut builder = match (!private, self.web_context.as_mut()) {
            (true, Some(ctx)) => WebViewBuilder::with_web_context(ctx),
            _ => WebViewBuilder::new().with_incognito(private),
        };
        #[cfg(windows)]
        let mut builder = WebViewBuilder::new().with_incognito(private);

        if !already_registered {
            builder = builder.with_custom_protocol("amnibrowse".to_string(), move |id, req| proto(id, req));
        }

        let (https_only, http_allow, p7) = (self.https_only.clone(), self.http_allow.clone(), push.clone());
        builder = builder
            .with_url(url)
            .with_bounds(content_rect)
            .with_user_agent(&ua)
            .with_devtools(true)
            .with_hotkeys_zoom(true)
            .with_back_forward_navigation_gestures(true)
            .with_initialization_script(&format!("{};{};{};{};{};{}", fetch_shim(), if ua.contains("Chrome/") { UA_SCRIPT } else { "" }, KEY_SCRIPT, FIND_SCRIPT, ICON_SCRIPT, LINK_SCRIPT))
            .with_navigation_handler(move |u| {
                let blocked = shield.get() && !is_internal(&u) && blocker.borrow_mut().should_block(&u);
                if blocked { info!("adblock: blocked navigation {}", u); return false; }
                if https_only.get() && u.starts_with("http://") && !is_local_host(&u) {
                    let mut allow = http_allow.borrow_mut();
                    match allow.iter().position(|a| a == &u) {
                        Some(i) => { allow.remove(i); }
                        None => { p7(Ev::Upgrade(uid, u)); return false; }
                    }
                }
                true
            })
            .with_new_window_req_handler(move |u| { match wants_native_popup(&u) { true => true, false => { p1(Ev::Popup(u)); false } } })
            .with_document_title_changed_handler(move |t| p2(Ev::Title(uid, t)))
            .with_on_page_load_handler(move |e, u| p3(Ev::Load(uid, matches!(e, PageLoadEvent::Started), u)))
            .with_ipc_handler(move |req| { if let Some(e) = ipc_ev(uid, req.body()) { p4(e); } });

        if attach_downloads {
            let dl_dir_c = dl_dir.clone();
            let ask_where = self.ask_dl_location.clone();
            builder = builder.with_download_started_handler(move |u, path| {
                let name = guess_filename(&u, path);
                std::fs::create_dir_all(&dl_dir_c).ok();
                *path = unique_path(&dl_dir_c, &name);
                #[cfg(not(windows))]
                if ask_where.get() {
                    match pick_save_path(path) { Some(p) => *path = p, None => { info!("download: cancelled by user {}", u); return false; } }
                }
                #[cfg(windows)]
                let _ = &ask_where;
                info!("download: {} -> {:?}", u, path);
                #[cfg(not(windows))]
                p5(Ev::DlStart(dl_id(&u), u.clone(), path.to_string_lossy().to_string(), None));
                true
            });
            #[cfg(not(windows))]
            {
                builder = builder.with_download_completed_handler(move |u, path, ok| {
                    p6(Ev::DlState(dl_id(&u), if ok { DL_COMPLETED } else { DL_INTERRUPTED }, path.map(|p| p.to_string_lossy().to_string()).unwrap_or_default()));
                });
            }
        }
        #[cfg(not(windows))]
        let view = build_view(builder, &host);
        #[cfg(windows)]
        let view = build_view(builder, self.host());
        let view = match view { Ok(v) => v, Err(e) => { warn!("webview2 tab failed: {}", e); return self.active; } };
        #[cfg(not(windows))]
        attach_filter(&view, self.filter, self.shield.get());
        #[cfg(not(windows))]
        if !private && !self.dl_progress_wired { self.wire_download_progress(&view); self.dl_progress_wired = true; }
        self.place(&view, content_rect);
        let core = wire_engine(&view, uid, push, self.blocker.clone(), self.shield.clone(), dnt, autofill, self.state.clone());
        let zoom = self.site_zoom_for(url).unwrap_or(default_zoom);
        let _ = view.zoom(zoom.max(0.25));
        let tab = Tab { uid, view: View::Wry(view), core, url: url.to_string(), title: String::new(), private, loading: true, zoom, can_back: false, can_forward: false, icon: None, audio: false, pinned: false, group: None, muted: false, discarded: false, last_active: Instant::now(), inject: None, upgraded_from: None, #[cfg(not(windows))] tls: None, find: (0, 0) };
        let idx = at.unwrap_or(self.tabs.len()).min(self.tabs.len());
        self.tabs.insert(idx, tab);
        self.raise_chrome();
        idx
    }
    fn site_zoom_for(&self, url: &str) -> Option<f64> {
        let h = host_of(url);
        match h.is_empty() { true => None, false => self.state().config.site_zoom.get(&h).copied() }
    }
    /// Chrome remembers zoom per site, not per tab.
    fn remember_zoom(&self, url: &str, zoom: f64) {
        let h = host_of(url);
        if h.is_empty() || is_internal(url) { return; }
        let mut st = self.state_mut();
        let default = st.config.default_zoom;
        match (zoom - default).abs() < 0.01 { true => { st.config.site_zoom.remove(&h); } false => { st.config.site_zoom.insert(h, zoom); } }
        st.config.save();
    }
    /// Ask the configured AI: open its site with the prompt (new tab or in place per Settings).
    fn ask_ai(&mut self, query: &str) {
        let (url, inject, new_tab) = { let st = self.state(); let (u, j) = ai_search::ask(&st.config, query); (u, j, st.config.ai_new_tab) };
        let idx = match new_tab || self.active_tab().map(|t| is_internal(&t.url)).unwrap_or(true) {
            true => { let i = self.spawn_tab(&url, self.active_tab().map(|t| t.private).unwrap_or(false), Some(self.active + 1)); self.active = i; i }
            false => { self.navigate_active(&url); self.active }
        };
        if let Some(t) = self.tabs.get_mut(idx) { t.inject = inject; }
        self.layout();
        self.sync_title();
        self.focus_content();
    }
    fn ask_ai_about_page(&mut self) {
        let Some(t) = self.active_tab() else { return };
        if is_internal(&t.url) { self.ask_ai(""); return; }
        let q = ai_search::page_prompt(&t.title, &display_url(&t.url));
        self.ask_ai(&q);
    }
    fn search_url(&self, q: &str) -> String {
        let se = self.state().config.search_engine.clone();
        format!("{}{}", match se.starts_with("http") { true => se, false => "https://html.duckduckgo.com/html/?q=".into() }, urlencoding::encode(q))
    }
    #[cfg(not(windows))]
    fn mute_tab(&mut self, idx: usize) {
        if let Some(t) = self.tabs.get_mut(idx) { t.muted = !t.muted; t.view.mute(t.muted); }
    }
    #[cfg(windows)]
    fn mute_tab(&mut self, idx: usize) {
        let m = match self.tabs.get_mut(idx) { Some(t) => { t.muted = !t.muted; t.muted } None => return };
        if let Some(c) = self.tabs.get(idx).and_then(|t| t.core.as_ref()) { unsafe { if let Ok(c8) = c.cast::<ICoreWebView2_8>() { let _ = c8.SetIsMuted(BOOL(m as i32)); } } }
    }
    /// Memory saver: unload a background tab; it reloads when shown again.
    fn discard_tab(&mut self, idx: usize) {
        if idx == self.active { return; }
        if let Some(t) = self.tabs.get_mut(idx) {
            if t.discarded || t.pinned || t.audio || t.private || is_internal(&t.url) || t.loading { return; }
            t.discarded = true;
            let title = esc_html(match t.title.trim().is_empty() { true => &t.url, false => &t.title });
            let _ = t.view.load_html(&format!("<!doctype html><title>{}</title><body style='background:#0D0F12'></body>", title));
        }
    }
    fn hard_reload(&self) {
        #[cfg(not(windows))]
        {
            if let Some(t) = self.active_tab() { t.view.reload(true); }
        }
        #[cfg(windows)]
        self.reload_page();
    }
    fn downloads_dir(&self) -> PathBuf { self.state().config.downloads_dir.clone().map(PathBuf::from).unwrap_or_else(DownloadManager::downloads_dir) }
    fn file_stem_for_page(&self) -> String {
        let t = self.active_tab().map(|t| match t.title.trim().is_empty() { true => host_of(&t.url), false => t.title.clone() }).unwrap_or_default();
        let s = sanitize_filename(&t);
        match s.is_empty() || s == "download" { true => "page".into(), false => s.chars().take(80).collect() }
    }
    /// Ctrl+S: the page as a single .mhtml file in Downloads.
    #[cfg(not(windows))]
    fn save_page(&mut self) {
        use webkit2gtk::WebViewExt;
        let Some(t) = self.active_tab() else { return };
        if is_internal(&t.url) { return; }
        let dir = self.downloads_dir();
        std::fs::create_dir_all(&dir).ok();
        let path = unique_path(&dir, &format!("{}.mhtml", self.file_stem_for_page()));
        let (u, p, push) = (t.url.clone(), path.to_string_lossy().to_string(), self.pusher());
        let id = format!("save{}", self.next_uid);
        push(Ev::DlStart(id.clone(), u, p.clone(), None));
        let Some(wv) = t.view.webview() else { return };
        wv.save_to_file(&webkit2gtk::gio::File::for_path(&path), webkit2gtk::SaveMode::Mhtml, None::<&webkit2gtk::gio::Cancellable>, move |r| {
            let ok = r.is_ok();
            if let Err(e) = r { warn!("save page: {}", e); }
            push(Ev::DlState(id, if ok { DL_COMPLETED } else { DL_INTERRUPTED }, p));
        });
    }
    #[cfg(windows)]
    fn save_page(&mut self) {}
    /// Screenshot of the visible page to Downloads as PNG.
    #[cfg(not(windows))]
    fn screenshot(&mut self) {
        use webkit2gtk::WebViewExt;
        let Some(t) = self.active_tab() else { return };
        let dir = self.downloads_dir();
        std::fs::create_dir_all(&dir).ok();
        let path = unique_path(&dir, &format!("{} {}.png", self.file_stem_for_page(), chrono::Local::now().format("%Y-%m-%d %H%M%S")));
        let (u, p, push) = (t.url.clone(), path.to_string_lossy().to_string(), self.pusher());
        let id = format!("shot{}", self.next_uid);
        push(Ev::DlStart(id.clone(), u, p.clone(), None));
        let Some(wv) = t.view.webview() else { return };
        wv.snapshot(webkit2gtk::SnapshotRegion::Visible, webkit2gtk::SnapshotOptions::NONE, None::<&webkit2gtk::gio::Cancellable>, move |r| {
            let ok = r.ok().and_then(|s| gtk::cairo::ImageSurface::try_from(s).ok()).and_then(|img| std::fs::File::create(&path).ok().map(|mut f| img.write_to_png(&mut f).is_ok())).unwrap_or(false);
            push(Ev::DlState(id, if ok { DL_COMPLETED } else { DL_INTERRUPTED }, p));
        });
    }
    #[cfg(windows)]
    fn screenshot(&mut self) {}
    fn interstitial(&self, kind: &str, title: &str, body: &str, url: &str, actions: &str) -> String {
        let vars = theme_root_vars(&self.state().themes.active_theme());
        format!("<!DOCTYPE html><html><head><meta charset='utf-8'><title>{t}</title><style>:root{{{v}}}body{{font:15px/1.55 'Segoe UI Variable Text','Segoe UI',system-ui,sans-serif;background:var(--bg);color:var(--text);margin:0;display:flex;align-items:center;justify-content:center;min-height:100vh}}main{{max-width:560px;padding:32px}}.k{{font-size:11px;letter-spacing:.16em;text-transform:uppercase;color:var(--accent);margin:0 0 10px}}h1{{font-size:22px;margin:0 0 12px}}p{{color:var(--dim);margin:0 0 10px}}code{{color:var(--text);word-break:break-all}}.acts{{margin-top:22px;display:flex;gap:8px;flex-wrap:wrap}}button{{font:650 11px/1 inherit;letter-spacing:.1em;text-transform:uppercase;padding:11px 16px;border-radius:3px;border:1px solid var(--stroke);background:var(--elev);color:var(--text);cursor:pointer}}button.primary{{background:var(--accent);color:#08090B;border-color:transparent}}button:hover{{border-color:var(--accent)}}</style></head><body><main><p class='k'>{k}</p><h1>{t}</h1>{b}<p><code>{u}</code></p><div class='acts'>{a}</div></main><script>window.__amniToken={tok:?};function cmd(n,a){{const q=new URLSearchParams(Object.assign({{tok:window.__amniToken}},a||{{}}));fetch('amnibrowse://cmd/'+n+'?'+q,{{mode:'no-cors'}}).catch(()=>{{}})}}</script></body></html>", v = vars, k = esc_html(kind), t = esc_html(title), b = body, u = esc_html(url), a = actions, tok = self.token)
    }
    #[cfg(not(windows))]
    fn show_alternate(&self, idx: usize, html: &str, url: &str) {
        use webkit2gtk::WebViewExt;
        if let Some(w) = self.tabs.get(idx).and_then(|t| t.view.webview()) { w.load_alternate_html(html, url, Some(url)); }
    }
    #[cfg(not(windows))]
    fn prompt_permission(&mut self, uid: u64, kind: PermissionType, origin: String, req: webkit2gtk::PermissionRequest) {
        let id = self.next_perm;
        self.next_perm += 1;
        self.perms.push(PendingPerm { id, uid, kind: kind.clone(), origin: origin.clone(), req });
        if self.tab_index(uid) == Some(self.active) { self.show_permission_prompt(); }
    }
    #[cfg(not(windows))]
    fn show_permission_prompt(&self) {
        let Some(active) = self.active_tab().map(|t| t.uid) else { return };
        let Some(p) = self.perms.iter().find(|p| p.uid == active) else { return };
        let what = match p.kind { PermissionType::Camera => "use your camera", PermissionType::Microphone => "use your microphone", PermissionType::Location => "know your location", PermissionType::Notifications => "show notifications", _ => "a permission" };
        let payload = serde_json::json!({"kind":"dialog","type":"perm","id":p.id,"message":format!("{} wants to {}", p.origin, what),"ok":"Allow","cancel":"Block"});
        if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); }
        self.chrome_js(&format!("window.__amni&&window.__amni.showEmbedder&&window.__amni.showEmbedder({})", payload));
    }
    #[cfg(not(windows))]
    fn answer_permission(&mut self, id: u64, allow: bool) {
        use webkit2gtk::PermissionRequestExt;
        let Some(pos) = self.perms.iter().position(|p| p.id == id) else { return };
        let p = self.perms.remove(pos);
        match allow { true => p.req.allow(), false => p.req.deny() }
        {
            let mut st = self.state_mut();
            st.permissions.set_permission(&format!("https://{}/", p.origin), p.kind, match allow { true => PermissionState::Allow, false => PermissionState::Deny });
        }
        self.show_permission_prompt();
    }
    /// Download progress + cancel come from WebKit's Download objects on the shared context
    /// (wry only reports start and completion).
    #[cfg(not(windows))]
    fn wire_download_progress(&self, view: &WebView) {
        use webkit2gtk::{DownloadExt, URIRequestExt, WebContextExt, WebViewExt};
        use wry::WebViewExtUnix;
        let Some(ctx) = view.webview().context() else { return };
        let (push, live) = (self.pusher(), self.live_downloads.clone());
        ctx.connect_download_started(move |_, d| {
            let Some(u) = d.request().and_then(|r| r.uri()).map(|u| u.to_string()) else { return };
            let id = dl_id(&u);
            live.borrow_mut().insert(id.clone(), d.clone());
            let (p, id2) = (push.clone(), id.clone());
            d.connect_received_data(move |d, _| p(Ev::DlProgress(id2.clone(), d.received_data_length())));
            let (l2, id3) = (live.clone(), id.clone());
            d.connect_finished(move |_| { l2.borrow_mut().remove(&id3); });
            let (l3, id4) = (live.clone(), id);
            d.connect_failed(move |_, _| { l3.borrow_mut().remove(&id4); });
        });
    }
    #[cfg(not(windows))]
    fn tls_proceed(&mut self) {
        use webkit2gtk::{WebContextExt, WebViewExt};
        let Some(t) = self.tabs.get(self.active) else { return };
        let Some((host, cert)) = t.tls.clone() else { return };
        if let Some(ctx) = t.view.webview().and_then(|w| w.context()) { ctx.allow_tls_certificate_for_host(&cert, &host); }
        let u = t.url.clone();
        self.navigate_active(&u);
    }
    fn open_tab(&mut self, url: Option<String>, private: bool) {
        self.overlay_css = 0;
        let target = url.unwrap_or_else(|| match private { true => internal_url("newtab"), false => self.home_url() });
        let idx = self.spawn_tab(&target, private, None);
        self.active = idx;
        self.layout();
        self.sync_title();
        if target.contains("newtab") {
            self.focus_omnibox(true);
        } else {
            self.focus_content();
            if is_internal(&target) {
                let disp = display_url(&target);
                self.chrome_js(&format!("try{{var u=document.getElementById('url');if(u){{u.value={:?};}}}}catch(e){{}}", disp));
            }
        }
    }
    fn handle_single_instance(&mut self, msg: crate::net::single_instance::SingleInstanceMessage) {
        if let Some(raw) = msg.url {
            let raw = raw.trim();
            if !raw.is_empty() {
                let resolved = resolve_input(raw, &self.state().config.search_engine);
                self.open_tab(resolved, msg.private);
            }
        }
        self.window.set_minimized(false);
        self.window.set_focus();
        self.chrome_js("try{poll()}catch(e){}");
    }
    fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() { return; }
        self.overlay_css = 0;
        let t = self.tabs.remove(idx);
        let _ = t.view.evaluate_script("try{window.stop()}catch(e){}try{document.querySelectorAll('video,audio').forEach(function(m){m.pause()})}catch(e){}");
        let _ = t.view.set_visible(false);
        if !t.private && !is_internal(&t.url) { self.closed.push((t.url.clone(), t.title.clone(), t.private)); if self.closed.len() > 25 { self.closed.remove(0); } }
        drop(t);
        if self.tabs.is_empty() { let h = self.home_url(); self.spawn_tab(&h, false, None); self.active = 0; } else if self.active >= self.tabs.len() { self.active = self.tabs.len() - 1; } else if idx < self.active { self.active -= 1; }
        self.layout();
        self.sync_title();
        self.persist();
        self.focus_content();
    }
    fn switch_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.overlay_css = 0;
            if let Some(prev) = self.tabs.get_mut(self.active) { prev.last_active = Instant::now(); }
            self.active = idx;
            if let Some(t) = self.tabs.get_mut(idx) {
                t.last_active = Instant::now();
                if t.discarded { t.discarded = false; t.loading = true; let _ = t.view.load_url(&t.url); }
            }
            self.layout();
            self.sync_title();
            self.persist();
            self.focus_content();
        }
    }
    fn focus_content(&self) {
        if let Some(t) = self.active_tab() {
            #[cfg(not(windows))]
            {
                use gtk::prelude::WidgetExt;
                #[cfg(all(feature = "cef-engine", target_os = "linux"))]
                if let Some(w) = t.view.webview() { super::cef_tabs::grab_x_focus(&w); }
                if let Some(w) = t.view.webview() { w.grab_focus(); }
            }
            let _ = t.view.focus();
        }
    }
    fn navigate_active(&mut self, url: &str) {
        self.overlay_css = 0;
        #[cfg(all(feature = "cef-engine", target_os = "linux"))]
        if let Some(t) = self.tabs.get(self.active).filter(|t| !t.private && t.view.is_cef() != super::cef_tabs::wants(url)) {
            debug!("engine switch tab {} ({}) -> {}", t.uid, if t.view.is_cef() { "cef" } else { "webkit" }, url);
            let (pinned, group, at) = (t.pinned, t.group.clone(), self.active);
            let old = self.tabs.remove(at);
            let _ = old.view.set_visible(false);
            drop(old);
            let i = self.spawn_tab(url, false, Some(at));
            if let Some(n) = self.tabs.get_mut(i) { n.pinned = pinned; n.group = group; }
            self.active = i;
            self.last_content.set((0, 0, 0, 0));
            self.layout();
            self.sync_title();
            self.focus_content();
            return;
        }
        if let Some(t) = self.tabs.get_mut(self.active) {
            t.url = url.to_string();
            t.loading = true;
            t.icon = None;
            t.discarded = false;
            let _ = t.view.load_url(url);
        }
        self.layout();
        self.focus_content();
    }
    fn focus_omnibox(&self, clear: bool) {
        #[cfg(all(feature = "cef-engine", target_os = "linux"))]
        { use wry::WebViewExtUnix; for t in &self.tabs { if let View::Cef(c) = &t.view { c.blur(); } } if let Some(c) = self.chrome.as_ref() { super::cef_tabs::grab_x_focus(&c.webview()); } }
        if let Some(c) = self.chrome.as_ref() {
            #[cfg(not(windows))]
            {
                use gtk::prelude::WidgetExt;
                use wry::WebViewExtUnix;
                c.webview().grab_focus();
            }
            let _ = c.focus();
            let _ = c.evaluate_script(&format!("try{{var u=document.getElementById('url');{}u.focus();u.select()}}catch(e){{}}", match clear { true => "u.value='';", false => "" }));
        }
    }
    /// The hamburger. Both platforms drive the same list through the chrome's embedder-menu
    /// renderer (showEmbedder); picks come back as ctx_pick and route to normal commands.
    fn show_app_menu(&self) {
        let zoom = self.active_tab().map(|t| t.zoom).unwrap_or(1.0);
        let internal = self.active_tab().map(|t| is_internal(&t.url)).unwrap_or(true);
        let bookmarked = self.active_tab().map(|t| self.state().bookmarks.find_by_url(&display_url(&t.url)).is_some()).unwrap_or(false);
        let ai = ai_search::provider_name(&self.state().config);
        let items = serde_json::json!([
            {"id":"new_tab","label":"New tab","enabled":true},
            {"id":"private_tab","label":"New private tab","enabled":true},
            {"id":"new_window","label":"New window","enabled":true},
            {"id":"tile_view","label":"Tab tile view","enabled":true},
            {"sep":true},
            {"id":"am_history","label":"History","enabled":true},
            {"id":"am_downloads","label":"Downloads","enabled":true},
            {"id":"bookmark","label":match bookmarked { true => "Remove bookmark", false => "Bookmark this page" },"enabled":!internal},
            {"id":"toggle_bookmarks_bar","label":match self.bookmarks_bar { true => "Hide bookmarks bar", false => "Show bookmarks bar" },"enabled":true},
            {"sep":true},
            {"id":"ask_ai_page","label":format!("Ask {} about this page", ai),"enabled":!internal},
            {"id":"am_find","label":"Find on page","enabled":true},
            {"id":"save_page","label":"Save page as\u{2026}","enabled":!internal},
            {"id":"screenshot","label":"Take screenshot","enabled":!internal},
            {"id":"translate","label":"Translate page (Google)","enabled":!internal},
            {"id":"copy_url","label":"Copy link","enabled":!internal},
            {"id":"zoom_in","label":"Zoom in","enabled":true},
            {"id":"zoom_out","label":"Zoom out","enabled":true},
            {"id":"zoom_reset","label":format!("Reset zoom ({}%)", (zoom * 100.0).round() as i64),"enabled":(zoom - 1.0).abs() > 0.01},
            {"id":"fullscreen","label":match self.fullscreen { true => "Leave full screen", false => "Full screen" },"enabled":true},
            {"sep":true},
            {"id":"print","label":"Print","enabled":!internal},
            {"id":"view_source","label":"View page source","enabled":!internal},
            {"id":"devtools","label":"Developer tools","enabled":true},
            {"sep":true},
            {"id":"clear_data","label":"Clear browsing data","enabled":true},
            {"id":"show_tutorial","label":"Guide","enabled":true},
            {"id":"settings","label":"Settings","enabled":true},
        ]);
        let w = self.window.inner_size().to_logical::<f64>(self.scale()).width;
        let payload = serde_json::json!({"kind":"menu","x":(w - 244.0).max(4.0).round() as i64,"y":(self.chrome_css() as i64) - 8,"items":items});
        if let Some(c) = self.chrome.as_ref() {
            #[cfg(not(windows))]
            {
                use gtk::prelude::WidgetExt;
                use wry::WebViewExtUnix;
                c.webview().grab_focus();
            }
            let _ = c.focus();
        }
        self.chrome_js(&format!("window.__amni&&window.__amni.showEmbedder&&window.__amni.showEmbedder({})", payload));
    }
    fn chrome_js(&self, js: &str) { if let Some(c) = self.chrome.as_ref() { let _ = c.evaluate_script(js); } }
    fn active_js(&self, js: &str) { if let Some(t) = self.active_tab() { let _ = t.view.evaluate_script(js); } }
    #[cfg(windows)]
    fn with_core(&self, f: impl FnOnce(&Core)) { if let Some(c) = self.active_tab().and_then(|t| t.core.as_ref()) { f(c); } }
    fn sync_title(&self) {
        let title = self.active_tab().map(|t| match t.title.trim().is_empty() { true => display_url(&t.url), false => t.title.clone() }).unwrap_or_default();
        self.window.set_title(&match title.is_empty() || is_internal(&title) { true => APP_NAME.to_string(), false => format!("{} \u{2014} {}", title.chars().take(80).collect::<String>(), APP_NAME) });
    }
    fn persist(&mut self) {
        if !self.state().config.restore_session || self.ephemeral { return; }
        let scale = self.scale();
        let tabs: Vec<SessionTab> = self.tabs.iter().filter(|t| !t.private && !t.url.is_empty()).map(|t| SessionTab { url: display_url(&t.url), title: t.title.clone(), is_active: self.tabs.get(self.active).map(|a| a.uid == t.uid).unwrap_or(false), history: vec![display_url(&t.url)], history_index: 0, engine: "chromium".into(), pinned: t.pinned, group: t.group.clone() }).collect();
        if tabs.is_empty() { return; }
        let sz = self.window.inner_size();
        let mut sm = SessionManager::new(true);
        sm.state.window_width = sz.width as f64 / scale;
        sm.state.window_height = sz.height as f64 / scale;
        sm.state.maximized = self.window.is_maximized();
        if let Ok(p) = self.window.outer_position() { sm.state.window_x = Some(p.x as f64 / scale); sm.state.window_y = Some(p.y as f64 / scale); }
        sm.capture(tabs);
        sm.save();
    }
    fn state_json(&self) -> String {
        let active = self.active_tab();
        let url = active.map(|t| display_url(&t.url)).unwrap_or_default();
        let shown = match url.starts_with("amnibrowse://newtab") || url.starts_with("amnibrowse://tutorial") { true => String::new(), false => url.clone() };
        let tabs: Vec<serde_json::Value> = self.tabs.iter().enumerate().map(|(i, t)| serde_json::json!({
            "id": format!("t{}", i), "title": match t.title.trim().is_empty() { true => match is_internal(&t.url) { true => "New Tab".to_string(), false => url::Url::parse(&t.url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| t.url.clone()) }, false => t.title.clone() },
            "url": display_url(&t.url), "active": i == self.active, "loading": t.loading, "engine": "chromium", "icon": t.icon, "is_private": t.private, "audio": t.audio, "pinned": t.pinned, "group": t.group, "collapsed": t.group.as_ref().map(|g| self.collapsed.contains(g)).unwrap_or(false),
            "muted": t.muted, "discarded": t.discarded,
        })).collect();
        let (theme, active_dl, dl_count, bookmarked, ai, engine_name, bookmarks) = {
            let st = self.state();
            let th: serde_json::Value = serde_json::from_str(&st.themes.active_theme_json()).unwrap_or(serde_json::Value::Null);
            let adl = st.downloads.downloads.iter().filter(|d| matches!(d.status, DownloadStatus::Downloading | DownloadStatus::Pending)).count();
            let bm = !url.is_empty() && st.bookmarks.find_by_url(&url).is_some();
            let se = st.config.search_engine.clone();
            let engine_name = match se.as_str() { s if s.contains("duckduckgo") => "DuckDuckGo", s if s.contains("brave") => "Brave", s if s.contains("startpage") => "Startpage", s if s.contains("google") => "Google", s if s.contains("bing") => "Bing", s if s.contains("kagi") => "Kagi", _ => "the web" };
            let bms: Vec<serde_json::Value> = match self.bookmarks_bar { true => st.bookmarks.bookmarks.iter().take(40).map(|b| serde_json::json!({"title": b.title, "url": b.url})).collect(), false => Vec::new() };
            (th, adl, st.downloads.downloads.len(), bm, ai_search::provider_name(&st.config), engine_name, bms)
        };
        let closed: Vec<serde_json::Value> = self.closed.iter().enumerate().rev().take(10).map(|(i, (u, t, _))| serde_json::json!({"idx": i, "url": u, "title": t})).collect();
        serde_json::json!({
            "url": shown, "title": active.map(|t| t.title.clone()).unwrap_or_default(), "loading": active.map(|t| t.loading).unwrap_or(false),
            "canBack": active.map(|t| t.can_back).unwrap_or(false), "canForward": active.map(|t| t.can_forward).unwrap_or(false), "tabs": tabs, "theme": theme,
            "zoom": active.map(|t| t.zoom).unwrap_or(1.0), "fullscreen": self.fullscreen, "maximized": self.window.is_maximized(), "canReopen": !self.closed.is_empty(),
            "shield": self.shield.get(), "blocked": self.blocker.borrow().blocked_count(), "bookmarked": bookmarked, "vault": false, "downloads": active_dl, "dlcount": dl_count, "profile": "Local",
            "find": self.find_query, "findn": active.map(|t| t.find.0).unwrap_or(0), "findi": active.map(|t| t.find.1).unwrap_or(0), "winh": (self.window.inner_size().to_logical::<f64>(self.scale()).height).round() as i64, "pm": "Passwords", "logins": [], "update": serde_json::Value::Null, "engine": ENGINE, "decorated": self.decorated,
            "ai": ai, "searchName": engine_name, "closed": closed, "bmbar": self.bookmarks_bar, "bookmarks": bookmarks, "chromeh": self.chrome_css(),
        }).to_string()
    }
    fn settings_html(&self) -> String {
        render_settings_html(&self.state(), self.shield.get(), &self.token)
    }
    fn tutorial_html(&self) -> String {
        render_tutorial_html(&self.state(), &self.token)
    }
    fn page_html(&self, host: &str) -> Option<String> {
        render_page_html(&self.state(), self.shield.get(), &self.token, host)
    }
    fn setting_set(&mut self, k: &str, v: &str) {
        let on = v == "1" || v == "true" || v == "on";
        {
            let mut st = self.state_mut();
            match k {
                "search_engine" => st.config.search_engine = v.to_string(),
                "home_page" => st.config.home_page = v.to_string(),
                "theme" => { st.themes.set_theme(v); st.themes.save(); }
                "default_zoom" => { st.config.default_zoom = v.parse().unwrap_or(1.0); }
                "block_ads" | "shield" => { st.config.block_ads = on; }
                "restore_session" => st.config.restore_session = on,
                "custom_user_agent" => st.config.custom_user_agent = Some(v.to_string()).filter(|s| !s.trim().is_empty()),
                "check_updates" => st.config.check_updates = on,
                "clear_data_on_exit" => st.config.clear_data_on_exit = on,
                "autofill_on_load" => st.config.autofill_on_load = on,
                "enable_do_not_track" => st.config.enable_do_not_track = on,
                "enable_doh" => st.config.enable_doh = on,
                "ai_provider" => st.config.ai_provider = v.to_string(),
                "ai_custom_url" => st.config.ai_custom_url = Some(v.trim().to_string()).filter(|s| !s.is_empty()),
                "ai_new_tab" => st.config.ai_new_tab = on,
                "show_bookmarks_bar" => st.config.show_bookmarks_bar = on,
                "https_only" => st.config.https_only = on,
                "ask_download_location" => st.config.ask_download_location = on,
                "memory_saver" => st.config.memory_saver = on,
                "memory_saver_minutes" => st.config.memory_saver_minutes = v.parse().unwrap_or(45),
                "downloads_dir" => st.config.downloads_dir = Some(v.trim().to_string()).filter(|s| !s.is_empty()),
                _ => info!("setting_set: ignored {}={}", k, v),
            }
            st.config.save();
        }
        if k == "https_only" { self.https_only.set(on); }
        if k == "ask_download_location" { self.ask_dl_location.set(on); }
        if k == "show_bookmarks_bar" && self.bookmarks_bar != on { self.bookmarks_bar = on; self.layout(); }
        if k == "theme" { self.apply_frame_color(); }
        if k == "block_ads" || k == "shield" { self.shield.set(on); self.reshield(); }
        if self.active_tab().map(|t| t.url.contains("amnibrowse.settings")).unwrap_or(false) && k == "theme" { self.active_js("location.reload()"); }
    }
    fn apply_frame_color(&self) { self.window.set_background_color(hex_rgba(&self.state().themes.active_theme().bg_primary)); }
    #[cfg(not(windows))]
    fn clear_browsing_data(&self, _kinds: u32) {
        use webkit2gtk::{WebContextExt, WebsiteDataManagerExtManual, WebsiteDataTypes, WebViewExt};
        if let Some(ctx) = self.tabs.iter().find_map(|t| t.view.webview()).and_then(|w| w.context()) {
            if let Some(dm) = ctx.website_data_manager() {
                dm.clear(WebsiteDataTypes::all(), webkit2gtk::glib::TimeSpan::from_seconds(0), None::<&webkit2gtk::gio::Cancellable>, |_| {});
            }
        }
        #[cfg(all(feature = "cef-engine", target_os = "linux"))]
        super::cef_tabs::clear_data(&crate::storage::config::BrowserConfig::config_dir(), &self.tabs.iter().filter_map(|t| match &t.view { View::Cef(c) => Some(c), _ => None }).collect::<Vec<_>>());
    }
    #[cfg(windows)]
    fn clear_browsing_data(&self, kinds: COREWEBVIEW2_BROWSING_DATA_KINDS) {
        if let Some(c) = self.tabs.iter().find_map(|t| t.core.clone()) {
            unsafe { if let Ok(p) = c.cast::<ICoreWebView2_13>().and_then(|c| c.Profile()).and_then(|p| p.cast::<ICoreWebView2Profile2>()) { let _ = p.ClearBrowsingData(kinds, &ClearBrowsingDataCompletedHandler::create(Box::new(|_| Ok(())))); } }
        }
    }
    fn handle_key(&mut self, k: &str, shift: bool, alt: bool) {
        if self.last_key.0 == k && self.last_key.1 == shift && self.last_key.2 == alt && self.last_key.3.elapsed() < std::time::Duration::from_millis(100) { return; }
        self.last_key = (k.to_string(), shift, alt, Instant::now());
        match (k, shift, alt) {
            ("arrowleft", _, true) => self.command("back", &HashMap::new()),
            ("arrowright", _, true) => self.command("forward", &HashMap::new()),
            ("home", _, true) => self.command("home", &HashMap::new()),
            ("f5", _, _) | ("r", false, false) => self.command("reload", &HashMap::new()),
            ("r", true, false) => self.hard_reload(),
            ("s", false, false) => self.save_page(),
            ("g", false, false) | ("f3", false, _) => self.command("find_next", &HashMap::new()),
            ("g", true, false) | ("f3", true, _) => self.command("find_prev", &HashMap::new()),
            ("e", false, false) | ("k", false, false) => self.focus_omnibox(true),
            ("d", _, true) => self.focus_omnibox(false),
            ("a", _, true) => self.command("ai_mode", &HashMap::new()),
            ("pageup", false, false) | ("pagedown", false, false) => { let n = self.tabs.len(); if n > 1 { let a = self.active; self.switch_tab(match k { "pageup" => (a + n - 1) % n, _ => (a + 1) % n }); } }
            ("pageup", true, false) | ("pagedown", true, false) => { let n = self.tabs.len(); if n > 1 { let a = self.active; let to = match k { "pageup" => (a + n - 1) % n, _ => (a + 1) % n }; let mut args = HashMap::new(); args.insert("from".into(), format!("t{}", a)); args.insert("to".into(), to.to_string()); self.command("move_tab", &args); } }
            ("delete", true, false) => self.open_tab(Some(format!("{}#privacy", internal_url("settings"))), false),
            ("b", true, false) => self.command("toggle_bookmarks_bar", &HashMap::new()),
            ("o", true, false) => self.open_tab(Some(format!("{}#import", internal_url("settings"))), false),
            ("w", true, false) => self.command("win_close", &HashMap::new()),
            ("j", true, false) => self.open_devtools(),
            ("f11", _, _) => self.command("fullscreen", &HashMap::new()),
            ("f12", _, _) | ("i", true, false) => self.open_devtools(),
            ("escape", _, _) => self.command("stop", &HashMap::new()),
            ("t", false, false) => self.open_tab(None, false),
            ("t", true, false) => self.command("reopen_tab", &HashMap::new()),
            ("n", true, false) => self.open_tab(None, true),
            ("n", false, false) => self.command("new_window", &HashMap::new()),
            ("w", false, false) => { let a = self.active; self.close_tab(a); }
            ("l", false, false) => self.focus_omnibox(false),
            ("d", false, false) => self.command("bookmark", &HashMap::new()),
            ("f", false, false) => { if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); } self.chrome_js("window.__amni&&window.__amni.showFind&&window.__amni.showFind()"); }
            ("p", false, false) => self.command("print", &HashMap::new()),
            ("=", _, false) | ("+", _, false) => self.command("zoom_in", &HashMap::new()),
            ("-", _, false) => self.command("zoom_out", &HashMap::new()),
            ("0", false, false) => self.command("zoom_reset", &HashMap::new()),
            ("tab", s, false) => { let n = self.tabs.len(); if n > 1 { let a = self.active; self.switch_tab(match s { true => (a + n - 1) % n, false => (a + 1) % n }); } }
            ("k", true, false) => self.command("duplicate_tab", &HashMap::new()),
            ("a", true, false) => {
                if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); }
                self.chrome_js("window.__amni&&window.__amni.showTileView&&window.__amni.showTileView()");
            }
            ("h", false, false) => self.chrome_js("window.__amni&&window.__amni.showPanel&&window.__amni.showPanel('hist')"),
            ("j", false, false) => self.chrome_js("window.__amni&&window.__amni.showPanel&&window.__amni.showPanel('dl')"),
            ("u", false, false) => self.command("view_source", &HashMap::new()),
            (d, false, false) if d.len() == 1 && d.as_bytes()[0].is_ascii_digit() => { let n: usize = d.parse().unwrap_or(1); let len = self.tabs.len(); if len > 0 { self.switch_tab(match n { 9 => len - 1, n => (n.max(1) - 1).min(len - 1) }); } }
            _ => {}
        }
    }
    fn command(&mut self, name: &str, a: &HashMap<String, String>) {
        let idx_of = |s: &str| s.trim_start_matches('t').parse::<usize>().ok();
        match name {
            "navigate" => {
                self.overlay_css = 0;
                let se = self.state().config.search_engine.clone();
                if let Some(u) = a.get("url").and_then(|u| resolve_input(u, &se)) {
                    match a.get("newtab").map(|v| v == "1").unwrap_or(false) { true => self.open_tab(Some(u), self.active_tab().map(|t| t.private).unwrap_or(false)), false => self.navigate_active(&u) }
                }
            }
            "back" => { self.overlay_css = 0; self.go_back(); self.layout(); self.focus_content(); }
            "forward" => { self.overlay_css = 0; self.go_forward(); self.layout(); self.focus_content(); }
            "reload" => { self.overlay_css = 0; self.reload_page(); self.layout(); self.focus_content(); }
            "stop" => { self.overlay_css = 0; self.stop_page(); self.layout(); self.focus_content(); }
            "home" => { self.overlay_css = 0; let h = self.home_url(); self.navigate_active(&h); }
            "new_tab" => self.open_tab(a.get("url").cloned(), false),
            "private_tab" => self.open_tab(a.get("url").cloned(), true),
            "tile_view" => {
                if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); }
                self.chrome_js("window.__amni&&window.__amni.showTileView&&window.__amni.showTileView()");
            }
            "amni_newtab" => { if let Some(u) = a.get("url").cloned() { self.spawn_tab(&u, false, Some(self.active + 1)); self.layout(); } }
            "close_tab" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)) { self.close_tab(i); } }
            "switch_tab" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)) { self.switch_tab(i); } }
            "move_tab" => {
                if let (Some(from), Some(to)) = (a.get("from").and_then(|s| idx_of(s)), a.get("to").and_then(|s| s.parse::<usize>().ok())) {
                    if from < self.tabs.len() { let t = self.tabs.remove(from); let to = to.min(self.tabs.len()); self.tabs.insert(to, t); self.active = match self.active { x if x == from => to, x if from < x && to >= x => x - 1, x if from > x && to <= x => x + 1, x => x }; self.layout(); }
                }
            }
            "duplicate_tab" => { if let Some(t) = self.active_tab() { let (u, p) = (t.url.clone(), t.private); let i = self.spawn_tab(&u, p, Some(self.active + 1)); self.active = i; self.layout(); } }
            "reopen_tab" => {
                let pick = a.get("idx").and_then(|i| i.parse::<usize>().ok()).filter(|i| *i < self.closed.len()).map(|i| self.closed.remove(i)).or_else(|| self.closed.pop());
                if let Some((u, _, p)) = pick { self.open_tab(Some(u), p); }
            }
            "zoom_in" | "zoom_out" | "zoom_reset" => {
                let default = self.state().config.default_zoom;
                if let Some(t) = self.tabs.get_mut(self.active) {
                    t.zoom = match name { "zoom_in" => (t.zoom + 0.1).min(3.0), "zoom_out" => (t.zoom - 0.1).max(0.3), _ => default };
                    let _ = t.view.zoom(t.zoom);
                    let (u, z) = (t.url.clone(), t.zoom);
                    self.remember_zoom(&u, z);
                }
            }
            "find" | "find_next" | "find_prev" => {
                let q = a.get("q").cloned().unwrap_or_else(|| self.find_query.clone());
                let dir = match name { "find_prev" => -1, _ => a.get("dir").and_then(|d| d.parse::<i32>().ok()).unwrap_or(1) };
                self.find_query = q.clone();
                self.active_js(&format!("window.__amniFind&&window.__amniFind({},{})", serde_json::to_string(&q).unwrap_or_default(), dir));
            }
            "find_close" => { self.find_query.clear(); self.active_js("window.__amniFindClear&&window.__amniFindClear()"); self.focus_content(); }
            "pin_tab" => {
                if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) {
                    let active_uid = self.tabs.get(self.active).map(|t| t.uid);
                    let mut t = self.tabs.remove(i);
                    t.pinned = !t.pinned;
                    let pinned_count = self.tabs.iter().filter(|x| x.pinned).count();
                    self.tabs.insert(pinned_count, t);
                    self.active = active_uid.and_then(|u| self.tabs.iter().position(|x| x.uid == u)).unwrap_or(0);
                    self.layout(); self.persist();
                }
            }
            "tab_set_group" => {
                if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) {
                    let active_uid = self.tabs.get(self.active).map(|t| t.uid);
                    let g = a.get("group").map(|g| g.trim().to_string()).filter(|g| !g.is_empty());
                    let mut t = self.tabs.remove(i);
                    t.group = g.clone();
                    let dest = g.as_ref().and_then(|g| self.tabs.iter().rposition(|x| x.group.as_deref() == Some(g.as_str())).map(|p| p + 1)).unwrap_or(i.min(self.tabs.len()));
                    self.tabs.insert(dest, t);
                    self.active = active_uid.and_then(|u| self.tabs.iter().position(|x| x.uid == u)).unwrap_or(0);
                    self.layout(); self.persist();
                }
            }
            "group_toggle" => {
                if let Some(g) = a.get("group").cloned() {
                    match self.collapsed.iter().position(|x| x == &g) { Some(p) => { self.collapsed.remove(p); } None => { self.collapsed.push(g.clone()); if self.active_tab().and_then(|t| t.group.clone()).as_deref() == Some(g.as_str()) { if let Some(n) = self.tabs.iter().position(|t| t.group.as_deref() != Some(g.as_str())) { self.switch_tab(n); } } } }
                }
            }
            "new_window" => { let _ = std::process::Command::new(std::env::current_exe().unwrap_or_default()).arg("--new-window").spawn(); }
            "bookmark" => {
                if let Some(t) = self.active_tab() {
                    let (u, ti) = (display_url(&t.url), match t.title.trim().is_empty() { true => display_url(&t.url), false => t.title.clone() });
                    if is_internal(&u) { return; }
                    let mut st = self.state_mut();
                    match st.bookmarks.find_by_url(&u).map(|b| b.id.clone()) {
                        Some(id) => { st.bookmarks.remove(&id); }
                        None => { st.bookmarks.add(&ti, &u, None); }
                    }
                    st.bookmarks.save();
                }
            }
            "bookmark_remove" => {
                if let Some(id) = a.get("id") {
                    let mut st = self.state_mut();
                    st.bookmarks.remove(id);
                    st.bookmarks.save();
                }
            }
            "shield" | "block_ads" => {
                let on = !self.shield.get();
                self.shield.set(on);
                {
                    let mut st = self.state_mut();
                    st.config.block_ads = on;
                    st.config.save();
                }
                self.reshield();
            }
            "setting_set" => { if let (Some(k), Some(v)) = (a.get("k").cloned(), a.get("v").cloned()) { self.setting_set(&k, &v); } }
            "settings" => {
                let s_url = internal_url("settings");
                if let Some(pos) = self.tabs.iter().position(|t| t.url.contains("settings")) {
                    self.switch_tab(pos);
                } else {
                    self.open_tab(Some(s_url), false);
                }
            }
            "downloads" | "am_downloads" => {
                let d_url = internal_url("downloads");
                if let Some(pos) = self.tabs.iter().position(|t| t.url.contains("downloads")) {
                    self.switch_tab(pos);
                } else {
                    self.open_tab(Some(d_url), false);
                }
            }
            "history" | "am_history" => {
                let h_url = internal_url("history");
                if let Some(pos) = self.tabs.iter().position(|t| t.url.contains("history")) {
                    self.switch_tab(pos);
                } else {
                    self.open_tab(Some(h_url), false);
                }
            }
            "open_downloads_folder" => {
                let dir = self.state().config.downloads_dir.clone().map(PathBuf::from).unwrap_or_else(DownloadManager::downloads_dir);
                let _ = std::process::Command::new(match cfg!(windows) { true => "explorer", false => "xdg-open" }).arg(&dir).spawn();
            }
            "show_in_folder" => {
                let path = a.get("id").and_then(|id| {
                    self.state()
                        .downloads
                        .downloads
                        .iter()
                        .find(|d| &d.id == id)
                        .map(|d| d.save_path.clone())
                });
                if let Some(p) = path {
                    let dir = p.parent().unwrap_or(&p);
                    let _ = std::process::Command::new(match cfg!(windows) { true => "explorer", false => "xdg-open" }).arg(dir).spawn();
                }
            }
            "show_tutorial" => self.open_tab(Some(internal_url("tutorial")), false),
            "tutorial_done" => {
                {
                    let mut st = self.state_mut();
                    st.config.seen_onboarding = true;
                    st.config.save();
                }
                let h = self.home_url();
                self.navigate_active(&h);
            }
            "overlay" => { self.overlay_css = a.get("h").and_then(|h| h.parse::<u32>().ok()).unwrap_or(0); self.layout(); }
            "win_min" => self.window.set_minimized(true),
            "win_max" => { let m = !self.window.is_maximized(); self.window.set_maximized(m); }
            "win_close" => { self.shutdown(); std::process::exit(0); }
            "win_drag" => { let _ = self.window.drag_window(); }
            "fullscreen" => { self.fullscreen = !self.fullscreen; self.window.set_fullscreen(match self.fullscreen { true => Some(Fullscreen::Borderless(None)), false => None }); self.layout(); }
            "print" => { if let Some(t) = self.active_tab() { let _ = t.view.print(); } }
            "devtools" => self.open_devtools(),
            "view_source" => { if let Some(t) = self.active_tab() { let u = t.url.clone(); if !is_internal(&u) && !u.starts_with("view-source:") { let i = self.spawn_tab(&format!("view-source:{}", u), false, Some(self.active + 1)); self.active = i; self.layout(); } } }
            "download" => { if let Some(t) = self.active_tab() { let u = t.url.clone(); self.state_mut().downloads.start_download(&u); } }
            "open_download" => {
                let path = a.get("id").and_then(|id| {
                    self.state()
                        .downloads
                        .downloads
                        .iter()
                        .find(|d| &d.id == id)
                        .map(|d| d.save_path.clone())
                });
                if let Some(p) = path {
                    let _ = std::process::Command::new(match cfg!(windows) { true => "explorer", false => "xdg-open" }).arg(&p).spawn();
                }
            }
            "download_cancel" => {
                if let Some(id) = a.get("id") {
                    #[cfg(not(windows))]
                    { use webkit2gtk::DownloadExt; if let Some(d) = self.live_downloads.borrow().get(id) { d.cancel(); } }
                    #[cfg(all(feature = "cef-engine", target_os = "linux"))]
                    if let Some(n) = id.strip_prefix("cef").and_then(|n| n.parse::<u32>().ok()) { super::cef_tabs::cancel_download(n); }
                    let mut st = self.state_mut();
                    if let Some(d) = st.downloads.downloads.iter_mut().find(|d| &d.id == id) { d.status = DownloadStatus::Failed; }
                    st.downloads.save();
                }
            }
            "download_remove" => {
                if let Some(id) = a.get("id") {
                    let mut st = self.state_mut();
                    st.downloads.remove_download(id);
                    st.downloads.save();
                }
            }
            "download_clear" => {
                let mut st = self.state_mut();
                st.downloads.clear_completed();
                st.downloads.save();
            }
            "clear_data" => {
                #[cfg(windows)] self.clear_browsing_data(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE);
                #[cfg(not(windows))] self.clear_browsing_data(0);
                let mut st = self.state_mut();
                st.history.clear_all();
                st.history.save();
            }
            "menu" => self.show_app_menu(),
            "ask_ai" => { let q = a.get("q").cloned().unwrap_or_default(); self.ask_ai(&q); }
            "ask_ai_page" => self.ask_ai_about_page(),
            "ai_mode" => { self.focus_omnibox(true); self.chrome_js("window.__amni&&window.__amni.aiMode&&window.__amni.aiMode()"); }
            "ai_home" => self.ask_ai(""),
            "mute_tab" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)) { self.mute_tab(i); } }
            "discard_tab" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)) { self.discard_tab(i); self.layout(); } }
            "close_others" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) { self.switch_tab(i); let mut j = self.tabs.len(); while j > 0 { j -= 1; if j != self.active && !self.tabs[j].pinned { self.close_tab(j); } } } }
            "close_right" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) { if self.active > i { self.switch_tab(i); } let mut j = self.tabs.len(); while j > i + 1 { j -= 1; if !self.tabs[j].pinned { self.close_tab(j); } } } }
            "reload_tab" => {
                if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) {
                    #[cfg(not(windows))]
                    { let t = &mut self.tabs[i]; if t.discarded { t.discarded = false; let _ = t.view.load_url(&t.url); } else { t.view.reload(false); } }
                    #[cfg(windows)]
                    { let u = self.tabs[i].url.clone(); let _ = self.tabs[i].view.load_url(&u); }
                }
            }
            "tab_new_window" => {
                if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) {
                    let u = self.tabs[i].url.clone();
                    if !is_internal(&u) && std::process::Command::new(std::env::current_exe().unwrap_or_default()).arg("--new-window").arg(&u).spawn().is_ok() { self.close_tab(i); }
                }
            }
            "new_tab_right" => { let h = self.home_url(); let i = self.spawn_tab(&h, false, Some(self.active + 1)); self.active = i; self.layout(); self.sync_title(); self.focus_omnibox(true); }
            "hard_reload" => self.hard_reload(),
            "save_page" => self.save_page(),
            "screenshot" => self.screenshot(),
            "translate" => {
                if let Some(t) = self.active_tab() {
                    let u = display_url(&t.url);
                    if !is_internal(&u) {
                        let lang = std::env::var("LANG").ok().and_then(|l| l.split(['_', '.']).next().map(|s| s.to_string())).filter(|l| !l.is_empty() && l != "C").unwrap_or_else(|| "en".into());
                        let tu = format!("https://translate.google.com/translate?sl=auto&tl={}&u={}", lang, urlencoding::encode(&u));
                        let i = self.spawn_tab(&tu, t.private, Some(self.active + 1)); self.active = i; self.layout(); self.sync_title();
                    }
                }
            }
            "copy_url" => { if let Some(t) = self.active_tab() { let u = display_url(&t.url); self.chrome_js(&format!("try{{navigator.clipboard.writeText({:?})}}catch(e){{}}", u)); } }
            "toggle_bookmarks_bar" => {
                self.bookmarks_bar = !self.bookmarks_bar;
                { let mut st = self.state_mut(); st.config.show_bookmarks_bar = self.bookmarks_bar; st.config.save(); }
                self.layout();
            }
            "history_remove" => { if let Some(u) = a.get("url") { let mut st = self.state_mut(); st.history.delete_by_url(u); } }
            "perm_answer" => {
                #[cfg(not(windows))]
                if let Some(id) = a.get("id").and_then(|i| i.parse::<u64>().ok()) { self.answer_permission(id, a.get("allow").map(|v| v == "1").unwrap_or(false)); }
                self.overlay_css = 0; self.layout(); self.focus_content();
            }
            "perm_set" => {
                if let (Some(host), Some(kind), Some(state)) = (a.get("host"), a.get("kind"), a.get("state")) {
                    let kind = match kind.as_str() { "Camera" => PermissionType::Camera, "Microphone" => PermissionType::Microphone, "Location" => PermissionType::Location, "Notifications" => PermissionType::Notifications, "Clipboard" => PermissionType::Clipboard, "Autoplay" => PermissionType::Autoplay, "Popups" => PermissionType::Popups, _ => PermissionType::Fullscreen };
                    let st = match state.as_str() { "allow" => PermissionState::Allow, "deny" => PermissionState::Deny, _ => PermissionState::Ask };
                    self.state_mut().permissions.set_permission(&format!("https://{}/", host), kind, st);
                }
            }
            "perm_reset" => { match a.get("host") { Some(h) => self.state_mut().permissions.reset_site(&format!("https://{}/", h)), None => self.state_mut().permissions.reset_all() } }
            "tls_proceed" => {
                #[cfg(not(windows))]
                self.tls_proceed();
            }
            "http_fallback" => {
                if let Some(t) = self.tabs.get_mut(self.active) {
                    if let Some(h) = t.upgraded_from.take() { self.http_allow.borrow_mut().push(h.clone()); self.navigate_active(&h); }
                }
            }
            "search_sel" | "ask_sel" => self.active_js(&format!("window.__amniSel&&window.__amniSel({:?})", name.trim_end_matches("_sel"))),
            "open_bg" => { if let Some(u) = a.get("url").cloned() { let p = self.active_tab().map(|t| t.private).unwrap_or(false); self.spawn_tab(&u, p, Some(self.active + 1)); self.layout(); } }
            "open_private" => { if let Some(u) = a.get("url").cloned() { let i = self.spawn_tab(&u, true, Some(self.active + 1)); self.active = i; self.layout(); self.sync_title(); self.focus_content(); } }
            "ctx_pick" => {
                self.overlay_css = 0;
                let Some(id) = a.get("id").cloned() else { self.layout(); self.focus_content(); return };
                match id.as_str() {
                    "am_history" => {
                        let h_url = internal_url("history");
                        if let Some(pos) = self.tabs.iter().position(|t| t.url.contains("history")) {
                            self.switch_tab(pos);
                        } else {
                            self.open_tab(Some(h_url), false);
                        }
                    }
                    "am_downloads" => {
                        let d_url = internal_url("downloads");
                        if let Some(pos) = self.tabs.iter().position(|t| t.url.contains("downloads")) {
                            self.switch_tab(pos);
                        } else {
                            self.open_tab(Some(d_url), false);
                        }
                    }
                    "am_find" => { if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); } self.chrome_js("window.__amni&&window.__amni.showFind&&window.__amni.showFind()"); }
                    "tile_view" => {
                        if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); }
                        self.chrome_js("window.__amni&&window.__amni.showTileView&&window.__amni.showTileView()");
                        return;
                    }
                    "am_newtab" => { let u = a.get("url").cloned().unwrap_or_default(); if !u.is_empty() { self.open_tab(Some(u), false); } }
                    other => { let mut args = a.clone(); args.remove("id"); let verb = other.to_string(); self.command(&verb, &args); }
                }
                self.layout();
                self.focus_content();
            }
            "ctx_dismiss" => {
                self.overlay_css = 0;
                self.layout();
                self.focus_content();
            }
            "chrome_err" => warn!("chrome js error: {} (line {})", a.get("m").map(|s| s.as_str()).unwrap_or(""), a.get("l").map(|s| s.as_str()).unwrap_or("?")),
            "kbd" | "overlay_rect" | "favicon_cache" | "dialog_ok" | "dialog_cancel" | "select_pick" | "color_pick" | "update_check" | "update_now" | "fill_login" | "vault_pw" | "import_browser" | "profile_new" | "profile_switch" => {}
            other => info!("cmd: unhandled {}", other),
        }
    }
    fn shutdown(&mut self) {
        self.persist();
        let (c_exit, c_cache, c_cookies, c_hist) = {
            let st = self.state();
            (st.config.clear_data_on_exit, st.config.clear_cache_on_exit, st.config.clear_cookies_on_exit, st.config.clear_history_on_exit)
        };
        #[cfg(windows)]
        {
            let mut kinds: Option<COREWEBVIEW2_BROWSING_DATA_KINDS> = None;
            let mut add = |k: COREWEBVIEW2_BROWSING_DATA_KINDS| { kinds = Some(match kinds { Some(x) => x | k, None => k }); };
            if c_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE); }
            if c_cache { add(COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE); }
            if c_cookies { add(COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES); }
            if let Some(k) = kinds { self.clear_browsing_data(k); std::thread::sleep(std::time::Duration::from_millis(400)); }
        }
        #[cfg(not(windows))]
        {
            if c_exit || c_cache || c_cookies {
                self.clear_browsing_data(0);
            }
        }
        {
            let mut st = self.state_mut();
            if c_hist || c_exit { st.history.clear_all(); st.history.save(); }
            st.shutdown();
        }
        crate::net::single_instance::cleanup();
    }
    fn handle(&mut self, ev: Ev) {
        match ev {
            Ev::Cmd(name, args) => { debug!("cmd {} {:?}", name, args.keys().collect::<Vec<_>>()); self.command(&name, &args) }
            Ev::Title(uid, t) => { if let Some(i) = self.tab_index(uid) { self.tabs[i].title = t; if i == self.active { self.sync_title(); } } }
            Ev::Load(uid, started, u) => {
                if let Some(i) = self.tab_index(uid) {
                    if self.tabs[i].discarded { return; }
                    let site_zoom = self.site_zoom_for(&u);
                    let default_zoom = self.state().config.default_zoom;
                    let t = &mut self.tabs[i];
                    t.loading = started;
                    if !u.is_empty() && t.url != u {
                        t.icon = None;
                        if host_of(&t.url) != host_of(&u) { let z = site_zoom.unwrap_or(default_zoom); if (z - t.zoom).abs() > 0.001 { t.zoom = z; let _ = t.view.zoom(z); } }
                        t.url = u.clone();
                    }
                    if started { t.find = (0, 0); }
                    if !started && u.starts_with("https://") { t.upgraded_from = None; }
                    if !started { if let Some(js) = t.inject.take() { let _ = t.view.evaluate_script(&js); } }
                    let (private, title) = (t.private, t.title.clone());
                    if !started && !private && !is_internal(&u) && u.starts_with("http") {
                        let mut st = self.state_mut();
                        st.history.record_visit(&u, &title);
                        st.history.save();
                    }
                    if !started && !cfg!(windows) && u.starts_with("http") { if let Some(t) = self.tabs.get_mut(i) { if t.icon.is_none() { t.icon = url::Url::parse(&u).ok().and_then(|p| p.host_str().map(|h| format!("{}://{}/favicon.ico", p.scheme(), h))); } } }
                    if !started { if let Some(js) = std::env::var_os("AMNI_PROBE_JS").and_then(|f| std::fs::read_to_string(f).ok()) { if let Some(t) = self.tabs.get(i) { let _ = t.view.evaluate_script(&js); } } }
                    if !started && !is_internal(&u) {
                        let scripts = self.state().extensions.get_content_scripts(&u);
                        if let Some(t) = self.tabs.get(i) { for (_id, js, css) in scripts { for sheet in css { let _ = t.view.evaluate_script(&crate::engine::daily_driver::inject_css_script(&sheet)); } for code in js { let _ = t.view.evaluate_script(&code); } } }
                    }
                    if !started { self.persist(); }
                    if i == self.active { self.sync_title(); }
                }
            }
            Ev::History(uid, b, f) => { if let Some(i) = self.tab_index(uid) { self.tabs[i].can_back = b; self.tabs[i].can_forward = f; } }
            Ev::Favicon(uid, uri) => { if let Some(i) = self.tab_index(uid) { self.tabs[i].icon = Some(uri).filter(|u| u.starts_with("http") || u.starts_with("data:")); } }
            Ev::PageFullscreen(uid, on) => { if self.tab_index(uid) == Some(self.active) { self.page_fullscreen = on; self.window.set_fullscreen(match on || self.fullscreen { true => Some(Fullscreen::Borderless(None)), false => None }); self.layout(); } }
            Ev::Audio(uid, on) => { if let Some(i) = self.tab_index(uid) { self.tabs[i].audio = on; } }
            Ev::DlStart(id, url, path, total) => {
                let p = PathBuf::from(&path);
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "download".into());
                let mut item = DownloadItem::new(&url, &name, p);
                item.id = id;
                item.total_bytes = total;
                item.status = DownloadStatus::Downloading;
                {
                    let mut st = self.state_mut();
                    st.downloads.downloads.insert(0, item);
                    st.downloads.save();
                }
                self.chrome_js("window.__amni&&window.__amni.showPanel&&window.__amni.showPanel('dl')");
            }
            Ev::DlProgress(id, n) => {
                #[cfg(not(windows))]
                let total = { use webkit2gtk::{DownloadExt, URIResponseExt}; self.live_downloads.borrow().get(&id).and_then(|d| d.response()).map(|r| r.content_length()).filter(|t| *t > 0) };
                #[cfg(windows)]
                let total: Option<u64> = None;
                let mut st = self.state_mut();
                if let Some(d) = st.downloads.downloads.iter_mut().find(|d| d.id == id) { d.downloaded_bytes = n; if d.total_bytes.is_none() { d.total_bytes = total; } }
            }
            Ev::DlState(id, st, path) => {
                let mut state = self.state_mut();
                if let Some(d) = state.downloads.downloads.iter_mut().find(|d| d.id == id) {
                    if !path.is_empty() { d.save_path = PathBuf::from(&path); d.filename = d.save_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(d.filename.clone()); }
                    d.status = match st { x if x == DL_COMPLETED => { d.completed_at = Some(chrono::Utc::now()); if let Some(t) = d.total_bytes { d.downloaded_bytes = t; } DownloadStatus::Completed } x if x == DL_INTERRUPTED => DownloadStatus::Failed, _ => DownloadStatus::Downloading };
                }
                state.downloads.save();
            }
            Ev::Popup(u) => { let i = self.spawn_tab(&u, self.active_tab().map(|t| t.private).unwrap_or(false), Some(self.active + 1)); self.active = i; self.layout(); }
            Ev::Key(u, k, shift, alt) => { debug!("key {} from {}", k, u); self.handle_key(&k, shift, alt) }
            Ev::Open(uid, u, bg) => {
                let (private, at) = self.tab_index(uid).map(|i| (self.tabs[i].private, i + 1)).unwrap_or((false, self.tabs.len()));
                let i = self.spawn_tab(&u, private, Some(at));
                if !bg { self.active = i; self.sync_title(); }
                self.layout();
                if !bg { self.focus_content(); }
            }
            Ev::Sel(uid, purpose, text) => {
                let text = text.trim().to_string();
                if text.is_empty() { return; }
                let Some(i) = self.tab_index(uid) else { return };
                let (u, p) = (display_url(&self.tabs[i].url), self.tabs[i].private);
                match purpose.as_str() {
                    "ask" => { let q = ai_search::selection_prompt(&text, &u); self.ask_ai(&q); }
                    _ => { let su = self.search_url(&text.chars().take(400).collect::<String>()); let n = self.spawn_tab(&su, p, Some(i + 1)); self.active = n; self.layout(); self.sync_title(); self.focus_content(); }
                }
            }
            Ev::Find(uid, n, i) => { if let Some(t) = self.tab_index(uid) { self.tabs[t].find = (n, i); } }
            #[cfg(not(windows))]
            Ev::Perm(uid, kind, origin, req) => self.prompt_permission(uid, kind, origin, req),
            #[cfg(not(windows))]
            Ev::LoadFailed(uid, u, msg) => {
                let Some(i) = self.tab_index(uid) else { return };
                if let Some(h) = self.tabs[i].upgraded_from.take() {
                    if u.starts_with("https://") { info!("https-only: {} failed, falling back to http", u); self.http_allow.borrow_mut().push(h.clone()); self.tabs[i].url = h.clone(); let _ = self.tabs[i].view.load_url(&h); return; }
                }
                self.tabs[i].loading = false;
                let host = host_of(&u);
                let html = self.interstitial("Can\u{2019}t reach this site", &match host.is_empty() { true => "This page isn\u{2019}t available".to_string(), false => format!("{} isn\u{2019}t responding", host) }, &format!("<p>{}</p><p>Check the address, your connection, or try again in a moment.</p>", esc_html(&msg)), &u, &format!("<button class='primary' onclick='cmd(\"reload\")'>Try again</button><button onclick='cmd(\"back\")'>Go back</button>{}", match self.state().config.https_only && u.starts_with("https://") { true => "<button onclick='cmd(\"http_fallback\")'>Try http://</button>", false => "" }));
                self.show_alternate(i, &html, &u);
                if i == self.active { self.sync_title(); }
            }
            #[cfg(not(windows))]
            Ev::TlsFail(uid, u, flags, cert) => {
                let Some(i) = self.tab_index(uid) else { return };
                let host = host_of(&u);
                self.tabs[i].tls = Some((url::Url::parse(&u).ok().and_then(|p| p.host_str().map(|h| h.to_string())).unwrap_or(host.clone()), cert));
                self.tabs[i].loading = false;
                let why = match flags.as_str() { f if f.contains("EXPIRED") => "The site\u{2019}s certificate has expired.", f if f.contains("UNKNOWN_CA") => "The site\u{2019}s certificate isn\u{2019}t issued by a trusted authority.", f if f.contains("BAD_IDENTITY") => "The certificate doesn\u{2019}t match this site\u{2019}s name.", f if f.contains("REVOKED") => "The certificate has been revoked.", _ => "The site\u{2019}s certificate can\u{2019}t be verified." };
                let html = self.interstitial("Your connection is not private", &format!("Attackers might be trying to steal your information from {}", host), &format!("<p>{}</p><p class='dim'>({})</p>", why, esc_html(&flags)), &u, "<button class='primary' onclick='cmd(\"back\")'>Back to safety</button><button onclick='if(confirm(\"Proceed to an unverified site? Anyone on the network could read what you send.\"))cmd(\"tls_proceed\")'>Proceed (unsafe)</button>");
                self.show_alternate(i, &html, &u);
            }
            Ev::Crash(uid) => {
                #[cfg(not(windows))]
                if let Some(i) = self.tab_index(uid) {
                    let u = self.tabs[i].url.clone();
                    self.tabs[i].loading = false;
                    let html = self.interstitial("Aw, snap", "This page crashed", "<p>Something went wrong while displaying this page.</p>", &u, "<button class='primary' onclick='cmd(\"reload\")'>Reload</button>");
                    self.show_alternate(i, &html, &u);
                }
                #[cfg(windows)]
                let _ = uid;
            }
            Ev::Ctx(uid, action, data) => {
                let Some(i) = self.tab_index(uid) else { return };
                let private = self.tabs[i].private;
                match action.as_str() {
                    "open_bg" => { self.spawn_tab(&data, private, Some(i + 1)); self.layout(); }
                    "open_private" => { let n = self.spawn_tab(&data, true, Some(i + 1)); self.active = n; self.layout(); self.sync_title(); self.focus_content(); }
                    "search_sel" | "ask_sel" => { let _ = self.tabs[i].view.evaluate_script(&format!("window.__amniSel&&window.__amniSel({:?})", action.trim_end_matches("_sel"))); }
                    "ask_page" => { if i != self.active { self.switch_tab(i); } self.ask_ai_about_page(); }
                    "screenshot" => { if i != self.active { self.switch_tab(i); } self.screenshot(); }
                    _ => {}
                }
            }
            Ev::Upgrade(uid, u) => {
                let Some(i) = self.tab_index(uid) else { return };
                let https = format!("https://{}", &u["http://".len()..]);
                info!("https-only: {} -> {}", u, https);
                let t = &mut self.tabs[i];
                t.upgraded_from = Some(u);
                t.url = https.clone();
                t.loading = true;
                let _ = t.view.load_url(&https);
            }
            Ev::Tick => {
                let (on, mins) = { let st = self.state(); (st.config.memory_saver, st.config.memory_saver_minutes.max(5)) };
                if !on { return; }
                let idle = std::time::Duration::from_secs(mins as u64 * 60);
                let stale: Vec<usize> = self.tabs.iter().enumerate().filter(|(i, t)| *i != self.active && !t.discarded && !t.pinned && !t.audio && !t.private && !t.loading && !is_internal(&t.url) && t.last_active.elapsed() > idle).map(|(i, _)| i).collect();
                for i in stale { info!("memory saver: discarding tab {}", i); self.discard_tab(i); }
            }
        }
    }
}
pub fn run(state: BrowserState, single_instance: Option<crate::net::single_instance::SingleInstanceListener>) {
    privacy_env(&state.config);
    let token = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x5eed) ^ 0x9e37_79b9_7f4a_7c15u64);
    let ephemeral = std::env::args().any(|a| a == "--new-window");
    let saved = SessionManager::load().filter(|_| state.config.restore_session && !ephemeral);
    let decorated = std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(false);
    // The Wayland app id comes from the program name, which GTK takes from
    // argv[0]. Pin it so a renamed or wrapped binary still maps to
    // amni-browse.desktop and gets the browser's icon in the taskbar.
    #[cfg(target_os = "linux")]
    gtk::glib::set_prgname(Some("amni-browse"));
    #[cfg(all(feature = "cef-engine", target_os = "linux"))]
    let _ = super::cef_tabs::init(&crate::storage::config::BrowserConfig::config_dir());
    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let (ipc_tx, ipc_rx) = std::sync::mpsc::channel::<crate::net::single_instance::SingleInstanceMessage>();
    let mut _instance_guard = single_instance.map(|l| l.listen(proxy.clone(), ipc_tx));
    let (w, h) = saved.as_ref().map(|s| (s.window_width.max(720.0), s.window_height.max(480.0))).unwrap_or((1400.0, 900.0));
    let mut builder = WindowBuilder::new().with_title(APP_NAME).with_decorations(decorated).with_inner_size(LogicalSize::new(w, h)).with_min_inner_size(LogicalSize::new(720.0, 480.0)).with_maximized(saved.as_ref().map(|s| s.maximized).unwrap_or(false));
    if let Some(c) = hex_rgba(&state.themes.active_theme().bg_primary) { builder = builder.with_background_color(c); }
    if let Some((x, y)) = saved.as_ref().and_then(|s| Some((s.window_x?, s.window_y?))) {
        let mon = event_loop.primary_monitor().map(|m| { let s = m.scale_factor(); let sz = m.size(); let p = m.position(); (p.x as f64 / s, p.y as f64 / s, sz.width as f64 / s, sz.height as f64 / s - 48.0) }).unwrap_or((0.0, 0.0, f64::MAX, f64::MAX));
        builder = builder.with_position(LogicalPosition::new(x.max(mon.0).min((mon.0 + mon.2 - w).max(mon.0)), y.max(mon.1).min((mon.1 + mon.3 - h).max(mon.1))));
    }
    let window = builder.build(&event_loop).expect("window");
    let state = Rc::new(RefCell::new(state));
    let blocker = Rc::new(RefCell::new(AdBlocker::new(state.borrow().config.block_ads, state.borrow().config.block_trackers)));
    let shield = Rc::new(Cell::new(state.borrow().config.block_ads));
    let events: Rc<RefCell<Vec<Ev>>> = Rc::new(RefCell::new(Vec::new()));
    let last_state: Rc<RefCell<String>> = Rc::new(RefCell::new("{}".into()));
    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));
    let (pa, pe, pl, ptok, ppx, pstate, pshield) = (app.clone(), events.clone(), last_state.clone(), token.clone(), proxy.clone(), state.clone(), shield.clone());
    let protocol: Rc<dyn Fn(&str, http::Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>>> = Rc::new(move |_id, req| {
        let uri = req.uri().to_string();
        let parsed = match url::Url::parse(&uri) { Ok(u) => u, Err(_) => return empty(400) };
        let host = parsed.host_str().unwrap_or("").trim_start_matches("amnibrowse.").to_string();
        // Every caller that may drive the browser (the chrome, settings, history, downloads, interstitials)
        // is rendered with the per-run token; web pages never see it, so a page cannot issue commands.
        let tok_ok = parsed.query_pairs().any(|(k, v)| k == "tok" && v == ptok.as_str());
        let from_chrome = tok_ok;
        let args: HashMap<String, String> = parsed.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        match host.as_str() {
            "chrome" => respond("text/html; charset=utf-8", format!("<script>{}window.__amniToken={:?};</script>{}{}", fetch_shim(), ptok, load_toolbar_html().replace("__CHROMEREV__", APP_VERSION), match decorated { true => "<style>.win-btn{display:none!important}</style>", false => "" })),
            "cmd" if from_chrome || tok_ok => { pe.borrow_mut().push(Ev::Cmd(parsed.path().trim_start_matches('/').to_string(), args)); let _ = ppx.send_event(()); empty(204) }
            "state" if from_chrome || tok_ok => {
                let body = match pa.try_borrow() { Ok(g) => g.as_ref().map(|a| a.state_json()).unwrap_or_else(|| "{}".into()), Err(_) => { debug!("state poll while app busy"); pl.borrow().clone() } };
                if *pl.borrow() != body { debug!("state changed: {}", body.chars().take(900).collect::<String>()); }
                *pl.borrow_mut() = body.clone();
                respond("application/json; charset=utf-8", body)
            }
            "suggest" if from_chrome || tok_ok => {
                let q = args.get("q").cloned().unwrap_or_default();
                let scope = args.get("scope").cloned().unwrap_or_default();
                let ql = q.to_lowercase();
                let mut rows: Vec<serde_json::Value> = Vec::new();
                // Open tabs ("Switch to this tab"), like Chrome's tab-switch suggestions and @tabs.
                if scope.is_empty() || scope == "tabs" {
                    if let Ok(g) = pa.try_borrow() {
                        if let Some(a) = g.as_ref() {
                            for (i, t) in a.tabs.iter().enumerate() {
                                if i == a.active || is_internal(&t.url) { continue; }
                                let du = display_url(&t.url);
                                if ql.is_empty() && scope.is_empty() { continue; }
                                if ql.is_empty() || t.title.to_lowercase().contains(&ql) || du.to_lowercase().contains(&ql) { rows.push(serde_json::json!({"kind": "tab", "id": format!("t{}", i), "title": t.title, "url": du})); }
                                if rows.len() >= 3 && scope.is_empty() { break; }
                            }
                        }
                    }
                }
                if scope.is_empty() || scope == "history" || scope == "bookmarks" {
                    if let Ok(st) = pstate.try_borrow() {
                        let limit = match scope.is_empty() { true => 8usize.saturating_sub(rows.len()), false => 12 };
                        let body = match scope.as_str() {
                            "bookmarks" => { let mut v: Vec<serde_json::Value> = st.bookmarks.bookmarks.iter().filter(|b| ql.is_empty() || b.title.to_lowercase().contains(&ql) || b.url.to_lowercase().contains(&ql)).take(limit).map(|b| serde_json::json!({"kind": "bookmark", "title": b.title, "url": b.url})).collect(); v.drain(..).collect::<Vec<_>>() }
                            "history" => serde_json::from_str::<Vec<serde_json::Value>>(&st.history.omnibox_json(&q, &[], limit)).unwrap_or_default().into_iter().map(|mut r| { r["kind"] = "history".into(); r }).collect(),
                            _ => {
                                let bms: Vec<(String, String)> = st.bookmarks.bookmarks.iter().map(|b| (b.url.clone(), b.title.clone())).collect();
                                let extra: Vec<(&str, &str)> = bms.iter().map(|(u, t)| (u.as_str(), t.as_str())).collect();
                                serde_json::from_str::<Vec<serde_json::Value>>(&st.history.omnibox_json(&q, &extra, limit)).unwrap_or_default().into_iter().map(|mut r| { let is_bm = r.get("url").and_then(|u| u.as_str()).map(|u| st.bookmarks.find_by_url(u).is_some()).unwrap_or(false); r["kind"] = (if is_bm { "bookmark" } else { "history" }).into(); r }).collect()
                            }
                        };
                        rows.extend(body);
                    }
                }
                respond("application/json; charset=utf-8", serde_json::Value::Array(rows).to_string())
            }
            "siteinfo" if from_chrome || tok_ok => {
                let u = args.get("url").cloned().unwrap_or_default();
                let host = host_of(&u);
                let body = match pstate.try_borrow() {
                    Ok(st) => {
                        let kinds = [PermissionType::Camera, PermissionType::Microphone, PermissionType::Location, PermissionType::Notifications];
                        let site = st.permissions.sites.iter().find(|s| s.site == host || s.site == format!("www.{}", host));
                        let perms: Vec<serde_json::Value> = kinds.iter().map(|k| serde_json::json!({"kind": k.to_string(), "state": match site.and_then(|s| s.permissions.get(k)) { Some(PermissionState::Allow) => "allow", Some(PermissionState::Deny) => "deny", _ => "ask" }})).collect();
                        let zoom = st.config.site_zoom.get(&host).copied();
                        serde_json::json!({"host": host, "secure": u.starts_with("https://"), "perms": perms, "zoom": zoom, "shield": pshield.get(), "dnt": st.config.enable_do_not_track}).to_string()
                    }
                    Err(_) => "{}".into(),
                };
                respond("application/json; charset=utf-8", body)
            }
            "history" if from_chrome || tok_ok => respond("application/json; charset=utf-8", pstate.try_borrow().ok().map(|st| st.history.recent_json(40)).unwrap_or_else(|| "[]".into())),
            "downloads" if from_chrome || tok_ok => respond("application/json; charset=utf-8", pstate.try_borrow().ok().map(|st| st.downloads.to_json()).unwrap_or_else(|| "[]".into())),
            "import" => respond("application/json; charset=utf-8", "{}".into()),
            page => match pstate.try_borrow().ok().and_then(|st| render_page_html(&st, pshield.get(), &ptok, page)) {
                Some(html) => respond("text/html; charset=utf-8", format!("<script>{}</script>{}", fetch_shim(), html)),
                None => empty(404),
            },
        }
    });
    #[cfg(windows)]
    let parent = (window.hwnd() as isize) as HWND;
    #[cfg(not(windows))]
    let (overlay, canvas, chrome_canvas) = gtk_host(&window);
    #[cfg(not(windows))]
    let web_context = {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".local").join("share"))
            .join("amni-browse");
        std::fs::create_dir_all(&data_dir).ok();
        Some(WebContext::new(Some(data_dir)))
    };
    let mut a = App {
        window,
        #[cfg(not(windows))]
        overlay,
        #[cfg(not(windows))]
        canvas,
        #[cfg(not(windows))]
        chrome_canvas,
        #[cfg(not(windows))]
        web_context,
        dl_handler_registered: false,
        proto_registered: false,
        decorated,
        chrome: None,
        chrome_hwnd: 0,
        tabs: Vec::new(),
        active: 0,
        closed: Vec::new(),
        state: state.clone(),
        token,
        next_uid: 1,
        overlay_css: 0,
        last_chrome: Cell::new((0, 0, 0, 0)),
        last_content: Cell::new((0, 0, 0, 0)),
        fullscreen: false,
        page_fullscreen: false,
        find_query: String::new(),
        protocol: protocol.clone(),
        events: events.clone(),
        proxy: proxy.clone(),
        blocker,
        shield,
        #[cfg(not(windows))]
        filter: compile_filter(),
        collapsed: Vec::new(),
        ephemeral,
        https_only: Rc::new(Cell::new(state.borrow().config.https_only)),
        http_allow: Rc::new(RefCell::new(Vec::new())),
        ask_dl_location: Rc::new(Cell::new(state.borrow().config.ask_download_location)),
        bookmarks_bar: state.borrow().config.show_bookmarks_bar,
        #[cfg(not(windows))]
        perms: Vec::new(),
        #[cfg(not(windows))]
        live_downloads: Rc::new(RefCell::new(HashMap::new())),
        next_perm: 1,
        dl_progress_wired: false,
        last_key: (String::new(), false, false, Instant::now()),
    };
    // Memory saver / housekeeping tick.
    #[cfg(not(windows))]
    {
        let push = a.pusher();
        gtk::glib::timeout_add_local(std::time::Duration::from_secs(60), move || { push(Ev::Tick); gtk::glib::ControlFlow::Continue });
    }
    #[cfg(all(feature = "cef-engine", target_os = "linux"))]
    if super::cef_tabs::enabled() {
        use gtk::prelude::*;
        let px = proxy.clone();
        super::cef_tabs::set_wake(move || { let _ = px.send_event(()); });
        let (blocker, shield, https_only, http_allow, push) = (a.blocker.clone(), a.shield.clone(), a.https_only.clone(), a.http_allow.clone(), a.pusher());
        super::cef_tabs::set_nav_filter(move |uid, u| {
            if shield.get() && !is_internal(u) && blocker.try_borrow_mut().map(|mut b| b.should_block(u)).unwrap_or(false) { info!("adblock: blocked navigation {}", u); return false; }
            if https_only.get() && u.starts_with("http://") && !is_local_host(u) {
                let mut allow = http_allow.borrow_mut();
                match allow.iter().position(|x| x == u) { Some(i) => { allow.remove(i); } None => { push(Ev::Upgrade(uid, u.to_string())); return false; } }
            }
            true
        });
        super::cef_tabs::set_page_script(&format!("{};{};{};{}", KEY_SCRIPT, FIND_SCRIPT, ICON_SCRIPT, LINK_SCRIPT));
        a.chrome_canvas.realize();
        if let Some(w) = a.chrome_canvas.bin_window() { w.ensure_native(); }
        info!("  Tabs: Chromium (CEF) for web pages, WebKitGTK for Amni pages and private tabs");
    }
    let chrome_proto = protocol.clone();
    let kpush = a.pusher();
    let chrome = WebViewBuilder::new()
        .with_url(&internal_url("chrome"))
        .with_bounds(a.chrome_rect())
        .with_devtools(true)
        .with_transparent(true)
        .with_initialization_script(&format!("{};{}", fetch_shim(), KEY_SCRIPT))
        .with_custom_protocol("amnibrowse".to_string(), move |id, req| chrome_proto(id, req))
        .with_ipc_handler(move |req| { if let Ok(v) = serde_json::from_str::<serde_json::Value>(req.body()) { if v.get("type").and_then(|t| t.as_str()) == Some("key") { kpush(Ev::Key(0, v.get("k").and_then(|k| k.as_str()).unwrap_or("").to_string(), v.get("shift").and_then(|s| s.as_i64()).unwrap_or(0) == 1, v.get("alt").and_then(|s| s.as_i64()).unwrap_or(0) == 1)); } } });
    let chrome = build_view(chrome, a.chrome_host()).expect("chrome webview");
    #[cfg(not(windows))]
    {
        use webkit2gtk::WebViewExt;
        use wry::WebViewExtUnix;
        if let Some(settings) = webkit2gtk::WebViewExt::settings(&chrome.webview()) {
            use webkit2gtk::{HardwareAccelerationPolicy, SettingsExt};
            // The GL compositor clears this transparent surface whenever the page
            // underneath is damaged, which flashes the omnibar. Software compositing
            // on the toolbar-sized view does not.
            settings.set_hardware_acceleration_policy(HardwareAccelerationPolicy::Never);
        }
        chrome.webview().set_background_color(&gtk::gdk::RGBA::new(8.0 / 255.0, 9.0 / 255.0, 11.0 / 255.0, 1.0));
    }
    #[cfg(windows)]
    if let Ok(cs) = unsafe { chrome.controller().CoreWebView2().and_then(|c| c.Settings()) } { unsafe { let _ = cs.SetIsStatusBarEnabled(BOOL(0)); let _ = cs.SetAreDefaultContextMenusEnabled(BOOL(0)); } }
    a.place_chrome(&chrome, a.chrome_rect());
    a.chrome = Some(chrome);
    #[cfg(windows)]
    { a.chrome_hwnd = unsafe { GetWindow(parent, GW_CHILD) } as usize; }
    let restore: Vec<SessionTab> = saved.map(|s| s.tabs).unwrap_or_default();
    let mut active = 0;
    for t in restore.iter() {
        let u = match t.url.strip_prefix("amnibrowse://") { Some(rest) => internal_url(rest.trim_matches('/')), None => t.url.clone() };
        if !(u.starts_with("http") || u.starts_with("file:")) { continue; }
        let i = a.spawn_tab(&u, false, None);
        if let Some(tab) = a.tabs.get_mut(i) { tab.pinned = t.pinned; tab.group = t.group.clone(); }
        if t.is_active { active = a.tabs.len().saturating_sub(1); }
    }
    if let Some(cli) = std::env::args().skip(1).find(|x| !x.starts_with('-')).and_then(|x| resolve_input(&x, &a.state().config.search_engine)) { a.spawn_tab(&cli, false, None); active = a.tabs.len() - 1; }
    if a.tabs.is_empty() { let h = a.home_url(); a.spawn_tab(&h, false, None); }
    a.active = active.min(a.tabs.len().saturating_sub(1));
    a.layout();
    a.sync_title();
    a.focus_content();
    info!("  Engine: {} \u{2014} {} tab(s), chrome {}px, frameless={}", ENGINE, a.tabs.len(), a.chrome_px(), !decorated);
    *app.borrow_mut() = Some(a);
    let app_loop = app.clone();
    let mut mods = ModifiersState::empty();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::ModifiersChanged(m), .. } => { mods = m; }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key, .. }, .. } if key.state == ElementState::Pressed => {
                let k = match &key.logical_key { Key::Character(c) => c.to_lowercase(), Key::Tab => "tab".into(), Key::F5 => "f5".into(), Key::F11 => "f11".into(), Key::F12 => "f12".into(), Key::Escape => "escape".into(), Key::ArrowLeft => "arrowleft".into(), Key::ArrowRight => "arrowright".into(), Key::Home => "home".into(), _ => String::new() };
                let plain = matches!(k.as_str(), "f5" | "f11" | "f12" | "escape");
                if !k.is_empty() && (mods.control_key() || mods.alt_key() || plain) { if let Ok(mut g) = app_loop.try_borrow_mut() { if let Some(a) = g.as_mut() { a.handle_key(&k, mods.shift_key(), mods.alt_key()); } } }
            }
            Event::UserEvent(()) => {
                let pending: Vec<Ev> = std::mem::take(&mut *events.borrow_mut());
                if let Ok(mut g) = app_loop.try_borrow_mut() {
                    if let Some(a) = g.as_mut() {
                        while let Ok(msg) = ipc_rx.try_recv() {
                            a.handle_single_instance(msg);
                        }
                        for ev in pending {
                            a.handle(ev);
                        }
                        #[cfg(all(feature = "cef-engine", target_os = "linux"))]
                        for ce in super::cef_tabs::drain() { a.cef_event(ce); }
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } | Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { .. }, .. } => {
                if let Ok(g) = app_loop.try_borrow() { if let Some(a) = g.as_ref() { a.layout(); } }
            }
            Event::WindowEvent { event: WindowEvent::Moved(_), .. } => {
                if let Ok(mut g) = app_loop.try_borrow_mut() { if let Some(a) = g.as_mut() { a.persist(); } }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                if let Ok(mut g) = app_loop.try_borrow_mut() { if let Some(a) = g.as_mut() { a.shutdown(); } }
                #[cfg(all(feature = "cef-engine", target_os = "linux"))]
                { if let Ok(mut g) = app_loop.try_borrow_mut() { g.take(); } super::cef_tabs::shutdown_all(); }
                _instance_guard.take();
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod popup_tests {
    use super::wants_native_popup;

    #[test]
    fn auth_blank_and_provider_windows_stay_native() {
        assert!(wants_native_popup("about:blank"));
        assert!(wants_native_popup(""));
        assert!(wants_native_popup("https://accounts.google.com/o/oauth2/v2/auth?client_id=x"));
        assert!(wants_native_popup("https://accounts.x.ai/sign-in?redirect=oauth2-provider"));
        assert!(wants_native_popup("https://appleid.apple.com/auth/authorize"));
        assert!(!wants_native_popup("https://example.com/article"));
    }
}
