use std::{borrow::Cow, cell::{Cell, RefCell}, collections::HashMap, path::PathBuf, rc::Rc};
use log::{info, warn};
use tao::{dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize}, event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy}, keyboard::{Key, ModifiersState}, window::{Fullscreen, Window, WindowBuilder}};
use wry::{http, PageLoadEvent, Rect, WebView, WebViewBuilder};
#[cfg(windows)]
use tao::platform::windows::WindowExtWindows;
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
use crate::{app::BrowserState, engine::adblocker::AdBlocker, storage::{config::{APP_NAME, APP_VERSION}, downloads::{DownloadItem, DownloadManager, DownloadStatus}, session::{SessionManager, SessionTab}}, ui::internal_pages::{esc_html, newtab_html, theme_root_vars, SETTINGS_TPL, TUTORIAL_TPL}, ui::tokens::SERVO_CHROME_HEIGHT_CSS};
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
#[cfg(windows)]
const ENGINE: &str = "Chromium (WebView2)";
#[cfg(not(windows))]
const ENGINE: &str = "WebKitGTK";
const FRAME_CSS: f64 = 5.0;
const DL_INTERRUPTED: i32 = 1;
const DL_COMPLETED: i32 = 2;
const AUTH_POPUP_HOSTS: &[&str] = &["accounts.google.com", "login.microsoftonline.com", "login.live.com", "appleid.apple.com", "facebook.com/dialog", "facebook.com/login", "github.com/login", "auth0.com", "okta.com", "oauth", "openid", "signin", "sso."];
const FETCH_SHIM: &str = "(function(){var f=window.fetch.bind(window);window.fetch=function(u,o){if(typeof u==='string'&&u.indexOf('amnibrowse://')===0){u=u.replace(/^amnibrowse:\\/\\/([^\\/?#]+)\\/?/,function(_,h){return 'http://amnibrowse.'+h+'/'})}return f(u,o)}})()";
const KEY_SCRIPT: &str = "(function(){document.addEventListener('keydown',function(e){var k=e.key.toLowerCase();var fn={f5:1,f11:1,f12:1,escape:1};var alt={arrowleft:1,arrowright:1,home:1};var send=function(){e.preventDefault();e.stopPropagation();try{window.ipc.postMessage(JSON.stringify({type:'key',k:k,shift:e.shiftKey?1:0,alt:e.altKey?1:0}))}catch(_){}};if(!e.ctrlKey&&!e.altKey&&!e.metaKey&&fn[k]){if(k==='escape'&&document.activeElement&&document.activeElement.tagName!=='BODY')return;send();return}if(e.altKey&&!e.ctrlKey&&alt[k]){send();return}if(!e.ctrlKey||e.altKey||e.metaKey)return;var hot={t:1,w:1,l:1,d:1,tab:1,h:1,j:1,u:1,f:1,p:1,r:1,n:1,'1':1,'2':1,'3':1,'4':1,'5':1,'6':1,'7':1,'8':1,'9':1,'=':1,'+':1,'-':1,'0':1,k:e.shiftKey?1:0,i:e.shiftKey?1:0};if(!hot[k])return;send()},true)})()";
const FIND_SCRIPT: &str = "(function(){var H=window.CSS&&CSS.highlights;var st={q:'',ranges:[],i:-1};function clear(){if(H){CSS.highlights.delete('amni-find');CSS.highlights.delete('amni-find-cur')}st={q:'',ranges:[],i:-1}}function collect(q){var out=[],w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT,{acceptNode:function(n){var p=n.parentElement;if(!p)return NodeFilter.FILTER_REJECT;var t=p.tagName;if(t==='SCRIPT'||t==='STYLE'||t==='NOSCRIPT')return NodeFilter.FILTER_REJECT;return n.nodeValue.toLowerCase().indexOf(q)>=0?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_SKIP}}),n;while((n=w.nextNode())){var s=n.nodeValue.toLowerCase(),k=0;while((k=s.indexOf(q,k))>=0){var r=document.createRange();r.setStart(n,k);r.setEnd(n,k+q.length);out.push(r);k+=q.length;if(out.length>5000)return out}}return out}function paint(){if(!H)return;var h=new Highlight();st.ranges.forEach(function(r){h.add(r)});CSS.highlights.set('amni-find',h);if(st.i>=0)CSS.highlights.set('amni-find-cur',new Highlight(st.ranges[st.i]))}function ensureCss(){if(document.getElementById('amni-find-css'))return;var s=document.createElement('style');s.id='amni-find-css';s.textContent='::highlight(amni-find){background:#ffd54a;color:#111}::highlight(amni-find-cur){background:#ff8a00;color:#111}';(document.head||document.documentElement).appendChild(s)}window.__amniFind=function(q,dir){q=(q||'').toLowerCase();if(!q){clear();return 0}ensureCss();if(q!==st.q){st.q=q;st.ranges=collect(q);st.i=st.ranges.length?0:-1}else if(st.ranges.length){st.i=(st.i+(dir<0?-1:1)+st.ranges.length)%st.ranges.length}if(!st.ranges.length){paint();return 0}var r=st.ranges[st.i];try{var sel=window.getSelection();sel.removeAllRanges();if(!H)sel.addRange(r)}catch(e){}try{var el=r.startContainer.parentElement;el&&el.scrollIntoView({block:'center',inline:'nearest'})}catch(e){}paint();return st.ranges.length};window.__amniFindClear=clear})()";
#[allow(dead_code)]
enum Ev { Cmd(String, HashMap<String, String>), Title(u64, String), Load(u64, bool, String), Popup(String), Key(u64, String, bool, bool), History(u64, bool, bool), Favicon(u64, String), PageFullscreen(u64, bool), Audio(u64, bool), DlStart(String, String, String, Option<u64>), DlProgress(String, u64), DlState(String, i32, String) }
struct Tab { uid: u64, view: WebView, core: Option<Core>, url: String, title: String, private: bool, loading: bool, zoom: f64, can_back: bool, can_forward: bool, icon: Option<String>, audio: bool, pinned: bool, group: Option<String> }
struct App {
    window: Window,
    decorated: bool,
    chrome: Option<WebView>,
    chrome_hwnd: usize,
    tabs: Vec<Tab>,
    active: usize,
    closed: Vec<(String, bool)>,
    state: BrowserState,
    token: String,
    next_uid: u64,
    overlay_css: u32,
    fullscreen: bool,
    page_fullscreen: bool,
    find_query: String,
    protocol: Rc<dyn Fn(&str, http::Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>>>,
    events: Rc<RefCell<Vec<Ev>>>,
    proxy: EventLoopProxy<()>,
    blocker: Rc<RefCell<AdBlocker>>,
    shield: Rc<Cell<bool>>,
    collapsed: Vec<String>,
    ephemeral: bool,
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
fn wants_native_popup(url: &str) -> bool { let l = url.to_ascii_lowercase(); AUTH_POPUP_HOSTS.iter().any(|h| l.contains(h)) }
fn resolve_input(raw: &str, search_prefix: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() { return None; }
    if let Some(rest) = t.strip_prefix("amnibrowse://") { return Some(internal_url(rest.trim_matches('/'))); }
    if t.contains("://") || t.starts_with("about:") || t.starts_with("view-source:") { return Some(t.to_string()); }
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
/// Everything wry does not expose: request-level shield + DNT/GPC headers, real history state,
/// favicons, HTML5 fullscreen, audio state, download progress, password/form autofill.
#[cfg(not(windows))]
fn wire_engine(_view: &WebView, _uid: u64, _push: Push, _blocker: Rc<RefCell<AdBlocker>>, _shield: Rc<Cell<bool>>, _dnt: bool, _autofill: bool) -> Option<Core> { None }
#[cfg(windows)]
fn wire_engine(view: &WebView, uid: u64, push: Push, blocker: Rc<RefCell<AdBlocker>>, shield: Rc<Cell<bool>>, dnt: bool, autofill: bool) -> Option<ICoreWebView2> {
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
    fn frame_px(&self) -> u32 { match self.decorated || self.fullscreen || self.page_fullscreen || self.window.is_maximized() { true => 0, false => (FRAME_CSS * self.scale()).round() as u32 } }
    fn chrome_px(&self) -> u32 { match self.fullscreen || self.page_fullscreen { true => 0, false => (SERVO_CHROME_HEIGHT_CSS as f64 * self.scale()).round() as u32 } }
    fn chrome_rect(&self) -> Rect {
        let sz = self.window.inner_size();
        let f = self.frame_px();
        let h = ((self.overlay_css as f64 * self.scale()).round() as u32).max(self.chrome_px()).min(sz.height.saturating_sub(f).max(1));
        Rect { position: PhysicalPosition::new(f as i32, f as i32).into(), size: PhysicalSize::new(sz.width.saturating_sub(2 * f).max(1), h.max(1)).into() }
    }
    fn content_rect(&self) -> Rect {
        let sz = self.window.inner_size();
        let f = self.frame_px();
        let overlay = match cfg!(windows) { true => 0, false => (self.overlay_css as f64 * self.scale()).round() as u32 };
        let y = (self.chrome_px().max(overlay) + f).min(sz.height.saturating_sub(1));
        Rect { position: PhysicalPosition::new(f as i32, y as i32).into(), size: PhysicalSize::new(sz.width.saturating_sub(2 * f).max(1), sz.height.saturating_sub(y + f).max(1)).into() }
    }
    fn layout(&self) {
        let hide_chrome = self.fullscreen || self.page_fullscreen;
        if let Some(c) = self.chrome.as_ref() { let _ = c.set_bounds(self.chrome_rect()); let _ = c.set_visible(!hide_chrome); }
        let r = self.content_rect();
        for (i, t) in self.tabs.iter().enumerate() { let _ = t.view.set_bounds(r); let _ = t.view.set_visible(i == self.active); }
        self.raise_chrome();
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
    fn go_back(&self) { self.active_js("history.back()"); }
    #[cfg(not(windows))]
    fn go_forward(&self) { self.active_js("history.forward()"); }
    #[cfg(not(windows))]
    fn reload_page(&self) { self.active_js("location.reload()"); }
    #[cfg(not(windows))]
    fn stop_page(&self) { self.active_js("window.stop()"); }
    #[cfg(not(windows))]
    fn open_devtools(&self) { if let Some(t) = self.active_tab() { t.view.open_devtools(); } }
    fn active_tab(&self) -> Option<&Tab> { self.tabs.get(self.active) }
    fn tab_index(&self, uid: u64) -> Option<usize> { self.tabs.iter().position(|t| t.uid == uid) }
    fn home_url(&self) -> String {
        if !self.state.config.seen_onboarding || std::env::var("AMNI_TUTORIAL").is_ok() { return internal_url("tutorial"); }
        let hp = self.state.config.home_page.trim().to_string();
        match hp.starts_with("http") { true => hp, false => internal_url("newtab") }
    }
    fn pusher(&self) -> Push {
        let ev = self.events.clone();
        let px = self.proxy.clone();
        Rc::new(move |e: Ev| { ev.borrow_mut().push(e); let _ = px.send_event(()); })
    }
    fn spawn_tab(&mut self, url: &str, private: bool, at: Option<usize>) -> usize {
        let uid = self.next_uid;
        self.next_uid += 1;
        let push = self.pusher();
        let (p1, p2, p3, p4) = (push.clone(), push.clone(), push.clone(), push.clone());
        let proto = self.protocol.clone();
        let blocker = self.blocker.clone();
        let shield = self.shield.clone();
        let ua = self.state.config.custom_user_agent.clone().filter(|u| !u.trim().is_empty()).unwrap_or_else(|| UA.to_string());
        let dl_dir = self.state.config.downloads_dir.clone().map(PathBuf::from).unwrap_or_else(DownloadManager::downloads_dir);
        let view = WebViewBuilder::new()
            .with_url(url)
            .with_bounds(self.content_rect())
            .with_user_agent(&ua)
            .with_incognito(private)
            .with_devtools(true)
            .with_hotkeys_zoom(true)
            .with_back_forward_navigation_gestures(true)
            .with_initialization_script(&format!("{};{};{}", fetch_shim(), KEY_SCRIPT, FIND_SCRIPT))
            .with_custom_protocol("amnibrowse".to_string(), move |id, req| proto(id, req))
            .with_navigation_handler(move |u| {
                let blocked = shield.get() && !is_internal(&u) && blocker.borrow_mut().should_block(&u);
                if blocked { info!("adblock: blocked navigation {}", u); }
                !blocked
            })
            .with_new_window_req_handler(move |u| { match wants_native_popup(&u) { true => true, false => { p1(Ev::Popup(u)); false } } })
            .with_document_title_changed_handler(move |t| p2(Ev::Title(uid, t)))
            .with_on_page_load_handler(move |e, u| p3(Ev::Load(uid, matches!(e, PageLoadEvent::Started), u)))
            .with_ipc_handler(move |req| {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(req.body()) {
                    if v.get("type").and_then(|t| t.as_str()) == Some("key") {
                        p4(Ev::Key(uid, v.get("k").and_then(|k| k.as_str()).unwrap_or("").to_string(), v.get("shift").and_then(|s| s.as_i64()).unwrap_or(0) == 1, v.get("alt").and_then(|s| s.as_i64()).unwrap_or(0) == 1));
                    }
                }
            })
            .with_download_started_handler(move |u, path| { let name = path.file_name().map(|n| n.to_os_string()).unwrap_or_else(|| "download".into()); std::fs::create_dir_all(&dl_dir).ok(); *path = dl_dir.join(name); info!("download: {} -> {:?}", u, path); true })
            .build_as_child(&self.window);
        let view = match view { Ok(v) => v, Err(e) => { warn!("webview2 tab failed: {}", e); return self.active; } };
        let core = wire_engine(&view, uid, push, self.blocker.clone(), self.shield.clone(), self.state.config.enable_do_not_track, !private && self.state.config.autofill_on_load);
        let _ = view.zoom(self.state.config.default_zoom.max(0.25));
        let tab = Tab { uid, view, core, url: url.to_string(), title: String::new(), private, loading: true, zoom: self.state.config.default_zoom, can_back: false, can_forward: false, icon: None, audio: false, pinned: false, group: None };
        let idx = at.unwrap_or(self.tabs.len()).min(self.tabs.len());
        self.tabs.insert(idx, tab);
        self.raise_chrome();
        idx
    }
    fn open_tab(&mut self, url: Option<String>, private: bool) {
        let target = url.unwrap_or_else(|| match private { true => internal_url("newtab"), false => self.home_url() });
        let idx = self.spawn_tab(&target, private, None);
        self.active = idx;
        self.layout();
        match is_internal(&target) { true => self.focus_omnibox(true), false => self.focus_content() }
    }
    fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() { return; }
        let t = self.tabs.remove(idx);
        let _ = t.view.evaluate_script("try{window.stop()}catch(e){}try{document.querySelectorAll('video,audio').forEach(function(m){m.pause()})}catch(e){}");
        let _ = t.view.set_visible(false);
        if !t.private && !is_internal(&t.url) { self.closed.push((t.url.clone(), t.private)); self.closed.truncate(20); }
        drop(t);
        if self.tabs.is_empty() { let h = self.home_url(); self.spawn_tab(&h, false, None); self.active = 0; } else if self.active >= self.tabs.len() { self.active = self.tabs.len() - 1; } else if idx < self.active { self.active -= 1; }
        self.layout();
        self.sync_title();
        self.persist();
    }
    fn switch_tab(&mut self, idx: usize) { if idx < self.tabs.len() { self.active = idx; self.layout(); self.sync_title(); self.persist(); self.focus_content(); } }
    fn focus_content(&self) { if let Some(t) = self.active_tab() { let _ = t.view.focus(); } }
    fn navigate_active(&mut self, url: &str) {
        if let Some(t) = self.tabs.get_mut(self.active) { t.url = url.to_string(); t.loading = true; t.icon = None; let _ = t.view.load_url(url); }
    }
    fn focus_omnibox(&self, clear: bool) {
        if let Some(c) = self.chrome.as_ref() { let _ = c.focus(); let _ = c.evaluate_script(&format!("try{{var u=document.getElementById('url');{}u.focus();u.select()}}catch(e){{}}", match clear { true => "u.value='';", false => "" })); }
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
        if !self.state.config.restore_session || self.ephemeral { return; }
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
        })).collect();
        let theme: serde_json::Value = serde_json::from_str(&self.state.themes.active_theme_json()).unwrap_or(serde_json::Value::Null);
        let active_dl = self.state.downloads.downloads.iter().filter(|d| matches!(d.status, DownloadStatus::Downloading | DownloadStatus::Pending)).count();
        serde_json::json!({
            "url": shown, "title": active.map(|t| t.title.clone()).unwrap_or_default(), "loading": active.map(|t| t.loading).unwrap_or(false),
            "canBack": active.map(|t| t.can_back).unwrap_or(false), "canForward": active.map(|t| t.can_forward).unwrap_or(false), "tabs": tabs, "theme": theme,
            "zoom": active.map(|t| t.zoom).unwrap_or(1.0), "fullscreen": self.fullscreen, "maximized": self.window.is_maximized(), "canReopen": !self.closed.is_empty(),
            "shield": self.shield.get(), "blocked": self.blocker.borrow().blocked_count(), "bookmarked": !url.is_empty() && self.state.bookmarks.find_by_url(&url).is_some(), "vault": false, "downloads": active_dl, "profile": "Local",
            "find": self.find_query, "pm": "Passwords", "logins": [], "update": serde_json::Value::Null, "engine": ENGINE, "decorated": self.decorated,
        }).to_string()
    }
    fn settings_html(&self) -> String {
        let c = &self.state.config;
        let engines = [("DuckDuckGo", "https://html.duckduckgo.com/html/?q="), ("Brave", "https://search.brave.com/search?q="), ("Startpage", "https://www.startpage.com/sp/search?query="), ("Google", "https://www.google.com/search?q=")];
        let radios: String = engines.iter().map(|(n, p)| format!("<label class='opt'><input type='radio' name='se' value='{}'{} onchange='set(\"search_engine\",this.value)'><span>{}</span></label>", p, match c.search_engine == *p { true => " checked", false => "" }, n)).collect();
        let zooms: String = [(0.8, "80%"), (0.9, "90%"), (1.0, "100%"), (1.1, "110%"), (1.25, "125%"), (1.5, "150%")].iter().map(|(z, l)| format!("<option value='{}'{}>{}</option>", z, match (*z - c.default_zoom).abs() < 0.01 { true => " selected", false => "" }, l)).collect();
        let bms: String = match self.state.bookmarks.bookmarks.is_empty() {
            true => "<p class='dim'>No bookmarks yet \u{2014} hit \u{2606} in the URL bar or Ctrl+D.</p>".into(),
            false => self.state.bookmarks.bookmarks.iter().map(|bm| format!("<div class='row' id='bm-{}'><a href='{}' title='{}'>{}</a><button class='x' onclick='rmbm(\"{}\")'>remove</button></div>", esc_html(&bm.id), esc_html(&bm.url), esc_html(&bm.url), esc_html(&bm.title), esc_html(&bm.id))).collect(),
        };
        let active_id = self.state.themes.active_theme().id;
        let themes: String = self.state.themes.all_themes().iter().map(|t| format!("<label class='opt'><input type='radio' name='th' value='{}'{} onchange='set(\"theme\",this.value)'><span>{}</span></label>", esc_html(&t.id), match t.id == active_id { true => " checked", false => "" }, esc_html(&t.name))).collect();
        let home = match c.home_page.starts_with("http") { true => c.home_page.clone(), false => String::new() };
        let toggles = format!("<label class='opt'><input type='checkbox'{} onchange='set(\"clear_data_on_exit\",this.checked?1:0)'><span>Clear browsing data (cookies, cache, history) when Amni Browse closes</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"autofill_on_load\",this.checked?1:0)'><span>Let the engine save passwords and fill forms (Chromium profile store)</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"enable_do_not_track\",this.checked?1:0)'><span>Send Do Not Track + Global Privacy Control headers</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"enable_doh\",this.checked?1:0)'><span>DNS over HTTPS (restart to apply)</span></label>", match c.clear_data_on_exit { true => " checked", false => "" }, match c.autofill_on_load { true => " checked", false => "" }, match c.enable_do_not_track { true => " checked", false => "" }, match c.enable_doh { true => " checked", false => "" });
        SETTINGS_TPL.replace("__THEME__", &theme_root_vars(&self.state.themes.active_theme())).replace("__THEMES__", &themes).replace("__VER__", APP_VERSION).replace("__RADIOS__", &radios).replace("__HOME__", &esc_html(&home)).replace("__ZOOMS__", &zooms)
            .replace("__SHIELD__", match self.shield.get() { true => " checked", false => "" }).replace("__RESTORE__", match c.restore_session { true => " checked", false => "" }).replace("__UA__", &esc_html(c.custom_user_agent.as_deref().unwrap_or(""))).replace("__TOK__", &self.token)
            .replace("__VAULT__", "Chromium profile store").replace("__PMRADIOS__", &toggles).replace("__PMLABEL__", "").replace("__PMCLI__", "").replace("__PMDB__", "").replace("__AUTOFILL__", "").replace("__CHKUPD__", match c.check_updates { true => " checked", false => "" })
            .replace("__UPD__", "checked on the site feed").replace("__PROFS__", "<div class='row'><span>Local \u{00b7} active</span></div>").replace("__CRASH__", "").replace("__IMPORTNOTE__", "").replace("__BMS__", &bms).replace("__ENGINE__", ENGINE)
    }
    fn tutorial_html(&self) -> String {
        TUTORIAL_TPL.replace("__THEME__", &theme_root_vars(&self.state.themes.active_theme())).replace("__VER__", APP_VERSION).replace("__TOK__", &self.token).replace("__BROWSERS__", "<p class='dim'>Import from Settings once you are in.</p>")
    }
    fn page_html(&self, host: &str) -> Option<String> {
        match host {
            "newtab" | "home" => Some(newtab_html(&self.state.themes.active_theme(), &self.state.bookmarks.bookmarks, ENGINE)),
            "settings" => Some(self.settings_html()),
            "tutorial" => Some(self.tutorial_html()),
            _ => None,
        }
    }
    fn setting_set(&mut self, k: &str, v: &str) {
        let on = v == "1" || v == "true" || v == "on";
        match k {
            "search_engine" => self.state.config.search_engine = v.to_string(),
            "home_page" => self.state.config.home_page = v.to_string(),
            "theme" => { self.state.themes.set_theme(v); self.state.themes.save(); self.apply_frame_color(); }
            "default_zoom" => { self.state.config.default_zoom = v.parse().unwrap_or(1.0); }
            "block_ads" | "shield" => { self.state.config.block_ads = on; self.shield.set(on); }
            "restore_session" => self.state.config.restore_session = on,
            "custom_user_agent" => self.state.config.custom_user_agent = Some(v.to_string()).filter(|s| !s.trim().is_empty()),
            "check_updates" => self.state.config.check_updates = on,
            "clear_data_on_exit" => self.state.config.clear_data_on_exit = on,
            "autofill_on_load" => self.state.config.autofill_on_load = on,
            "enable_do_not_track" => self.state.config.enable_do_not_track = on,
            "enable_doh" => self.state.config.enable_doh = on,
            _ => info!("setting_set: ignored {}={}", k, v),
        }
        self.state.config.save();
        if self.active_tab().map(|t| t.url.contains("amnibrowse.settings")).unwrap_or(false) && k == "theme" { self.active_js("location.reload()"); }
    }
    fn apply_frame_color(&self) { self.window.set_background_color(hex_rgba(&self.state.themes.active_theme().bg_primary)); }
    #[cfg(not(windows))]
    fn clear_browsing_data(&self, _kinds: u32) {}
    #[cfg(windows)]
    fn clear_browsing_data(&self, kinds: COREWEBVIEW2_BROWSING_DATA_KINDS) {
        if let Some(c) = self.tabs.iter().find_map(|t| t.core.clone()) {
            unsafe { if let Ok(p) = c.cast::<ICoreWebView2_13>().and_then(|c| c.Profile()).and_then(|p| p.cast::<ICoreWebView2Profile2>()) { let _ = p.ClearBrowsingData(kinds, &ClearBrowsingDataCompletedHandler::create(Box::new(|_| Ok(())))); } }
        }
    }
    fn handle_key(&mut self, k: &str, shift: bool, alt: bool) {
        match (k, shift, alt) {
            ("arrowleft", _, true) => self.command("back", &HashMap::new()),
            ("arrowright", _, true) => self.command("forward", &HashMap::new()),
            ("home", _, true) => self.command("home", &HashMap::new()),
            ("f5", _, _) | ("r", false, false) => self.command("reload", &HashMap::new()),
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
            "navigate" => { if let Some(u) = a.get("url").and_then(|u| resolve_input(u, &self.state.config.search_engine)) { self.navigate_active(&u); } }
            "back" => self.go_back(),
            "forward" => self.go_forward(),
            "reload" => self.reload_page(),
            "stop" => self.stop_page(),
            "home" => { let h = self.home_url(); self.navigate_active(&h); }
            "new_tab" => self.open_tab(a.get("url").cloned(), false),
            "private_tab" => self.open_tab(a.get("url").cloned(), true),
            "amni_newtab" => { if let Some(u) = a.get("url").cloned() { self.spawn_tab(&u, false, Some(self.active + 1)); self.layout(); } }
            "close_tab" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)) { self.close_tab(i); } }
            "switch_tab" => { if let Some(i) = a.get("id").and_then(|s| idx_of(s)) { self.switch_tab(i); } }
            "move_tab" => {
                if let (Some(from), Some(to)) = (a.get("from").and_then(|s| idx_of(s)), a.get("to").and_then(|s| s.parse::<usize>().ok())) {
                    if from < self.tabs.len() { let t = self.tabs.remove(from); let to = to.min(self.tabs.len()); self.tabs.insert(to, t); self.active = match self.active { x if x == from => to, x if from < x && to >= x => x - 1, x if from > x && to <= x => x + 1, x => x }; self.layout(); }
                }
            }
            "duplicate_tab" => { if let Some(t) = self.active_tab() { let (u, p) = (t.url.clone(), t.private); let i = self.spawn_tab(&u, p, Some(self.active + 1)); self.active = i; self.layout(); } }
            "reopen_tab" => { if let Some((u, p)) = self.closed.pop() { self.open_tab(Some(u), p); } }
            "zoom_in" | "zoom_out" | "zoom_reset" => {
                if let Some(t) = self.tabs.get_mut(self.active) { t.zoom = match name { "zoom_in" => (t.zoom + 0.1).min(3.0), "zoom_out" => (t.zoom - 0.1).max(0.3), _ => 1.0 }; let _ = t.view.zoom(t.zoom); }
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
                if let Some(t) = self.active_tab() { let (u, ti) = (display_url(&t.url), match t.title.trim().is_empty() { true => display_url(&t.url), false => t.title.clone() }); if is_internal(&u) { return; }
                    match self.state.bookmarks.find_by_url(&u).map(|b| b.id.clone()) { Some(id) => { self.state.bookmarks.remove(&id); } None => { self.state.bookmarks.add(&ti, &u, None); } }
                    self.state.bookmarks.save();
                }
            }
            "bookmark_remove" => { if let Some(id) = a.get("id") { self.state.bookmarks.remove(id); self.state.bookmarks.save(); } }
            "shield" | "block_ads" => { let on = !self.shield.get(); self.shield.set(on); self.state.config.block_ads = on; self.state.config.save(); }
            "setting_set" => { if let (Some(k), Some(v)) = (a.get("k").cloned(), a.get("v").cloned()) { self.setting_set(&k, &v); } }
            "settings" => self.open_tab(Some(internal_url("settings")), false),
            "show_tutorial" => self.open_tab(Some(internal_url("tutorial")), false),
            "tutorial_done" => { self.state.config.seen_onboarding = true; self.state.config.save(); let h = self.home_url(); self.navigate_active(&h); }
            "overlay" => { self.overlay_css = a.get("h").and_then(|h| h.parse::<u32>().ok()).unwrap_or(0); self.layout(); }
            "win_min" => self.window.set_minimized(true),
            "win_max" => { let m = !self.window.is_maximized(); self.window.set_maximized(m); }
            "win_close" => { self.shutdown(); std::process::exit(0); }
            "win_drag" => { let _ = self.window.drag_window(); }
            "fullscreen" => { self.fullscreen = !self.fullscreen; self.window.set_fullscreen(match self.fullscreen { true => Some(Fullscreen::Borderless(None)), false => None }); self.layout(); }
            "print" => { if let Some(t) = self.active_tab() { let _ = t.view.print(); } }
            "devtools" => self.open_devtools(),
            "view_source" => { if let Some(t) = self.active_tab() { let u = t.url.clone(); if !is_internal(&u) && !u.starts_with("view-source:") { let i = self.spawn_tab(&format!("view-source:{}", u), false, Some(self.active + 1)); self.active = i; self.layout(); } } }
            "download" => { if let Some(t) = self.active_tab() { let u = t.url.clone(); self.state.downloads.start_download(&u); } }
            "open_download" => {
                if let Some(p) = a.get("id").and_then(|id| self.state.downloads.downloads.iter().find(|d| &d.id == id)).map(|d| d.save_path.clone()) { let _ = std::process::Command::new(match cfg!(windows) { true => "explorer", false => "xdg-open" }).arg(&p).spawn(); }
            }
            "download_remove" => { if let Some(id) = a.get("id") { self.state.downloads.remove_download(id); self.state.downloads.save(); } }
            "download_clear" => { self.state.downloads.clear_completed(); self.state.downloads.save(); }
            "clear_data" => { #[cfg(windows)] self.clear_browsing_data(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE); #[cfg(not(windows))] self.clear_browsing_data(0); self.state.history.clear_all(); self.state.history.save(); }
            "kbd" | "overlay_rect" | "favicon_cache" | "ctx_dismiss" | "dialog_ok" | "dialog_cancel" | "select_pick" | "color_pick" | "ctx_pick" | "update_check" | "update_now" | "fill_login" | "vault_pw" | "import_browser" | "profile_new" | "profile_switch" => {}
            other => info!("cmd: unhandled {}", other),
        }
    }
    fn shutdown(&mut self) {
        self.persist();
        let c = &self.state.config;
        #[cfg(windows)]
        {
            let mut kinds: Option<COREWEBVIEW2_BROWSING_DATA_KINDS> = None;
            let mut add = |k: COREWEBVIEW2_BROWSING_DATA_KINDS| { kinds = Some(match kinds { Some(x) => x | k, None => k }); };
            if c.clear_data_on_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE); }
            if c.clear_cache_on_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE); }
            if c.clear_cookies_on_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES); }
            if let Some(k) = kinds { self.clear_browsing_data(k); std::thread::sleep(std::time::Duration::from_millis(400)); }
        }
        if c.clear_history_on_exit || c.clear_data_on_exit { self.state.history.clear_all(); self.state.history.save(); }
        self.state.shutdown();
    }
    fn handle(&mut self, ev: Ev) {
        match ev {
            Ev::Cmd(name, args) => self.command(&name, &args),
            Ev::Title(uid, t) => { if let Some(i) = self.tab_index(uid) { self.tabs[i].title = t; if i == self.active { self.sync_title(); } } }
            Ev::Load(uid, started, u) => {
                if let Some(i) = self.tab_index(uid) {
                    let t = &mut self.tabs[i];
                    t.loading = started;
                    if !u.is_empty() { if t.url != u { t.icon = None; } t.url = u.clone(); }
                    let (private, title) = (t.private, t.title.clone());
                    if !started && !private && !is_internal(&u) && u.starts_with("http") { self.state.history.record_visit(&u, &title); self.state.history.save(); }
                    if !started && !cfg!(windows) && u.starts_with("http") { if let Some(t) = self.tabs.get_mut(i) { if t.icon.is_none() { t.icon = url::Url::parse(&u).ok().and_then(|p| p.host_str().map(|h| format!("{}://{}/favicon.ico", p.scheme(), h))); } } }
                    if !started { if let Some(js) = std::env::var_os("AMNI_PROBE_JS").and_then(|f| std::fs::read_to_string(f).ok()) { if let Some(t) = self.tabs.get(i) { let _ = t.view.evaluate_script(&js); } } }
                    if !started && !is_internal(&u) {
                        let scripts = self.state.extensions.get_content_scripts(&u);
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
                self.state.downloads.downloads.insert(0, item);
                self.state.downloads.save();
                self.chrome_js("window.__amni&&window.__amni.showPanel&&window.__amni.showPanel('dl')");
            }
            Ev::DlProgress(id, n) => { if let Some(d) = self.state.downloads.downloads.iter_mut().find(|d| d.id == id) { d.downloaded_bytes = n; } }
            Ev::DlState(id, st, path) => {
                if let Some(d) = self.state.downloads.downloads.iter_mut().find(|d| d.id == id) {
                    if !path.is_empty() { d.save_path = PathBuf::from(&path); d.filename = d.save_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(d.filename.clone()); }
                    d.status = match st { x if x == DL_COMPLETED => { d.completed_at = Some(chrono::Utc::now()); if let Some(t) = d.total_bytes { d.downloaded_bytes = t; } DownloadStatus::Completed } x if x == DL_INTERRUPTED => DownloadStatus::Failed, _ => DownloadStatus::Downloading };
                }
                self.state.downloads.save();
            }
            Ev::Popup(u) => { let i = self.spawn_tab(&u, self.active_tab().map(|t| t.private).unwrap_or(false), Some(self.active + 1)); self.active = i; self.layout(); }
            Ev::Key(_, k, shift, alt) => self.handle_key(&k, shift, alt),
        }
    }
}
pub fn run(state: BrowserState) {
    privacy_env(&state.config);
    let token = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x5eed) ^ 0x9e37_79b9_7f4a_7c15u64);
    let ephemeral = std::env::args().any(|a| a == "--new-window");
    let saved = SessionManager::load().filter(|_| state.config.restore_session && !ephemeral);
    let decorated = std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(!cfg!(windows));
    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let (w, h) = saved.as_ref().map(|s| (s.window_width.max(720.0), s.window_height.max(480.0))).unwrap_or((1400.0, 900.0));
    let mut builder = WindowBuilder::new().with_title(APP_NAME).with_decorations(decorated).with_inner_size(LogicalSize::new(w, h)).with_min_inner_size(LogicalSize::new(720.0, 480.0)).with_maximized(saved.as_ref().map(|s| s.maximized).unwrap_or(false));
    if let Some(c) = hex_rgba(&state.themes.active_theme().bg_primary) { builder = builder.with_background_color(c); }
    if let Some((x, y)) = saved.as_ref().and_then(|s| Some((s.window_x?, s.window_y?))) {
        let mon = event_loop.primary_monitor().map(|m| { let s = m.scale_factor(); let sz = m.size(); let p = m.position(); (p.x as f64 / s, p.y as f64 / s, sz.width as f64 / s, sz.height as f64 / s - 48.0) }).unwrap_or((0.0, 0.0, f64::MAX, f64::MAX));
        builder = builder.with_position(LogicalPosition::new(x.max(mon.0).min((mon.0 + mon.2 - w).max(mon.0)), y.max(mon.1).min((mon.1 + mon.3 - h).max(mon.1))));
    }
    let window = builder.build(&event_loop).expect("window");
    let events: Rc<RefCell<Vec<Ev>>> = Rc::new(RefCell::new(Vec::new()));
    let last_state: Rc<RefCell<String>> = Rc::new(RefCell::new("{}".into()));
    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));
    let (pa, pe, pl, ptok, ppx) = (app.clone(), events.clone(), last_state.clone(), token.clone(), proxy.clone());
    let protocol: Rc<dyn Fn(&str, http::Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>>> = Rc::new(move |_id, req| {
        let uri = req.uri().to_string();
        let parsed = match url::Url::parse(&uri) { Ok(u) => u, Err(_) => return empty(400) };
        let host = parsed.host_str().unwrap_or("").trim_start_matches("amnibrowse.").to_string();
        let from_chrome = req.headers().get("referer").and_then(|v| v.to_str().ok()).map(|r| r.contains("amnibrowse.chrome") || r.contains("amnibrowse://chrome")).unwrap_or(false);
        let tok_ok = parsed.query_pairs().any(|(k, v)| k == "tok" && v == ptok.as_str());
        let args: HashMap<String, String> = parsed.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        match host.as_str() {
            "chrome" => respond("text/html; charset=utf-8", format!("<script>{}</script>{}{}", fetch_shim(), load_toolbar_html().replace("__CHROMEREV__", APP_VERSION), match decorated { true => "<style>.win-btn{display:none!important}</style>", false => "" })),
            "cmd" if from_chrome || tok_ok => { pe.borrow_mut().push(Ev::Cmd(parsed.path().trim_start_matches('/').to_string(), args)); let _ = ppx.send_event(()); empty(204) }
            "state" if from_chrome => {
                let body = match pa.try_borrow() { Ok(g) => g.as_ref().map(|a| a.state_json()).unwrap_or_else(|| "{}".into()), Err(_) => pl.borrow().clone() };
                *pl.borrow_mut() = body.clone();
                respond("application/json; charset=utf-8", body)
            }
            "suggest" if from_chrome => {
                let q = args.get("q").cloned().unwrap_or_default();
                let body = match pa.try_borrow() { Ok(g) => g.as_ref().map(|a| { let bms: Vec<(String, String)> = a.state.bookmarks.bookmarks.iter().map(|b| (b.url.clone(), b.title.clone())).collect(); let extra: Vec<(&str, &str)> = bms.iter().map(|(u, t)| (u.as_str(), t.as_str())).collect(); a.state.history.omnibox_json(&q, &extra, 8) }).unwrap_or_else(|| "[]".into()), Err(_) => "[]".into() };
                respond("application/json; charset=utf-8", body)
            }
            "history" if from_chrome => respond("application/json; charset=utf-8", pa.try_borrow().ok().and_then(|g| g.as_ref().map(|a| a.state.history.recent_json(40))).unwrap_or_else(|| "[]".into())),
            "downloads" if from_chrome => respond("application/json; charset=utf-8", pa.try_borrow().ok().and_then(|g| g.as_ref().map(|a| a.state.downloads.to_json())).unwrap_or_else(|| "[]".into())),
            "import" => respond("application/json; charset=utf-8", "{}".into()),
            page => match pa.try_borrow().ok().and_then(|g| g.as_ref().and_then(|a| a.page_html(page))) {
                Some(html) => respond("text/html; charset=utf-8", format!("<script>{}</script>{}", fetch_shim(), html)),
                None => empty(404),
            },
        }
    });
    let blocker = Rc::new(RefCell::new(AdBlocker::new(state.config.block_ads, state.config.block_trackers)));
    let shield = Rc::new(Cell::new(state.config.block_ads));
    #[cfg(windows)]
    let parent = (window.hwnd() as isize) as HWND;
    let mut a = App { window, decorated, chrome: None, chrome_hwnd: 0, tabs: Vec::new(), active: 0, closed: Vec::new(), state, token, next_uid: 1, overlay_css: 0, fullscreen: false, page_fullscreen: false, find_query: String::new(), protocol: protocol.clone(), events: events.clone(), proxy: proxy.clone(), blocker, shield, collapsed: Vec::new(), ephemeral };
    let chrome_proto = protocol.clone();
    let kpush = a.pusher();
    let chrome = WebViewBuilder::new().with_url(&internal_url("chrome")).with_bounds(a.chrome_rect()).with_devtools(true).with_initialization_script(&format!("{};{}", fetch_shim(), KEY_SCRIPT)).with_custom_protocol("amnibrowse".to_string(), move |id, req| chrome_proto(id, req))
        .with_ipc_handler(move |req| { if let Ok(v) = serde_json::from_str::<serde_json::Value>(req.body()) { if v.get("type").and_then(|t| t.as_str()) == Some("key") { kpush(Ev::Key(0, v.get("k").and_then(|k| k.as_str()).unwrap_or("").to_string(), v.get("shift").and_then(|s| s.as_i64()).unwrap_or(0) == 1, v.get("alt").and_then(|s| s.as_i64()).unwrap_or(0) == 1)); } } })
        .build_as_child(&a.window).expect("chrome webview");
    #[cfg(windows)]
    if let Ok(cs) = unsafe { chrome.controller().CoreWebView2().and_then(|c| c.Settings()) } { unsafe { let _ = cs.SetIsStatusBarEnabled(BOOL(0)); let _ = cs.SetAreDefaultContextMenusEnabled(BOOL(0)); } }
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
    if let Some(cli) = std::env::args().skip(1).find(|x| !x.starts_with('-')).and_then(|x| resolve_input(&x, &a.state.config.search_engine)) { a.spawn_tab(&cli, false, None); active = a.tabs.len() - 1; }
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
                if let Ok(mut g) = app_loop.try_borrow_mut() { if let Some(a) = g.as_mut() { for ev in pending { a.handle(ev); } } }
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } | Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { .. }, .. } => {
                if let Ok(g) = app_loop.try_borrow() { if let Some(a) = g.as_ref() { a.layout(); } }
            }
            Event::WindowEvent { event: WindowEvent::Moved(_), .. } => {
                if let Ok(mut g) = app_loop.try_borrow_mut() { if let Some(a) = g.as_mut() { a.persist(); } }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                if let Ok(mut g) = app_loop.try_borrow_mut() { if let Some(a) = g.as_mut() { a.shutdown(); } }
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
