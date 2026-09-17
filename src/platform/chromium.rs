use std::{borrow::Cow, cell::{Cell, RefCell}, collections::HashMap, path::PathBuf, rc::Rc};
use log::{info, warn};
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
use crate::{app::BrowserState, engine::adblocker::AdBlocker, storage::{config::{APP_NAME, APP_VERSION}, downloads::{DownloadItem, DownloadManager, DownloadStatus}, session::{SessionManager, SessionTab}}, ui::internal_pages::{esc_html, newtab_html, theme_root_vars, SETTINGS_TPL, TUTORIAL_TPL}, ui::tokens::SERVO_CHROME_HEIGHT_CSS};
#[cfg(windows)]
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
#[cfg(not(windows))]
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
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
const ICON_SCRIPT: &str = "(function(){function s(){try{var l=document.querySelector('link[rel~=\"icon\"],link[rel=\"shortcut icon\"]');var h=l&&l.href?l.href:(location.origin+'/favicon.ico');if(/^https?:/.test(h))window.ipc.postMessage(JSON.stringify({type:'icon',href:h}))}catch(_){}}if(document.readyState==='complete')s();else window.addEventListener('load',s)})()";
const FIND_SCRIPT: &str ="(function(){var H=window.CSS&&CSS.highlights;var st={q:'',ranges:[],i:-1};function clear(){if(H){CSS.highlights.delete('amni-find');CSS.highlights.delete('amni-find-cur')}st={q:'',ranges:[],i:-1}}function collect(q){var out=[],w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT,{acceptNode:function(n){var p=n.parentElement;if(!p)return NodeFilter.FILTER_REJECT;var t=p.tagName;if(t==='SCRIPT'||t==='STYLE'||t==='NOSCRIPT')return NodeFilter.FILTER_REJECT;return n.nodeValue.toLowerCase().indexOf(q)>=0?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_SKIP}}),n;while((n=w.nextNode())){var s=n.nodeValue.toLowerCase(),k=0;while((k=s.indexOf(q,k))>=0){var r=document.createRange();r.setStart(n,k);r.setEnd(n,k+q.length);out.push(r);k+=q.length;if(out.length>5000)return out}}return out}function paint(){if(!H)return;var h=new Highlight();st.ranges.forEach(function(r){h.add(r)});CSS.highlights.set('amni-find',h);if(st.i>=0)CSS.highlights.set('amni-find-cur',new Highlight(st.ranges[st.i]))}function ensureCss(){if(document.getElementById('amni-find-css'))return;var s=document.createElement('style');s.id='amni-find-css';s.textContent='::highlight(amni-find){background:#ffd54a;color:#111}::highlight(amni-find-cur){background:#ff8a00;color:#111}';(document.head||document.documentElement).appendChild(s)}window.__amniFind=function(q,dir){q=(q||'').toLowerCase();if(!q){clear();return 0}ensureCss();if(q!==st.q){st.q=q;st.ranges=collect(q);st.i=st.ranges.length?0:-1}else if(st.ranges.length){st.i=(st.i+(dir<0?-1:1)+st.ranges.length)%st.ranges.length}if(!st.ranges.length){paint();return 0}var r=st.ranges[st.i];try{var sel=window.getSelection();sel.removeAllRanges();if(!H)sel.addRange(r)}catch(e){}try{var el=r.startContainer.parentElement;el&&el.scrollIntoView({block:'center',inline:'nearest'})}catch(e){}paint();return st.ranges.length};window.__amniFindClear=clear})()";
#[allow(dead_code)]
enum Ev { Cmd(String, HashMap<String, String>), Title(u64, String), Load(u64, bool, String), Popup(String), Key(u64, String, bool, bool), History(u64, bool, bool), Favicon(u64, String), PageFullscreen(u64, bool), Audio(u64, bool), DlStart(String, String, String, Option<u64>), DlProgress(String, u64), DlState(String, i32, String) }
struct Tab { uid: u64, view: WebView, core: Option<Core>, url: String, title: String, private: bool, loading: bool, zoom: f64, can_back: bool, can_forward: bool, icon: Option<String>, audio: bool, pinned: bool, group: Option<String> }
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
    closed: Vec<(String, bool)>,
    state: Rc<RefCell<BrowserState>>,
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
    #[cfg(not(windows))]
    filter: Option<usize>,
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
    wv.connect_button_press_event(|w, _| { if !w.has_focus() { w.grab_focus(); } gtk::glib::Propagation::Proceed });
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
    let engines = [("DuckDuckGo", "https://html.duckduckgo.com/html/?q="), ("Brave", "https://search.brave.com/search?q="), ("Startpage", "https://www.startpage.com/sp/search?query="), ("Google", "https://www.google.com/search?q=")];
    let radios: String = engines.iter().map(|(n, p)| format!("<label class='opt'><input type='radio' name='se' value='{}'{} onchange='set(\"search_engine\",this.value)'><span>{}</span></label>", p, match c.search_engine == *p { true => " checked", false => "" }, n)).collect();
    let zooms: String = [(0.8, "80%"), (0.9, "90%"), (1.0, "100%"), (1.1, "110%"), (1.25, "125%"), (1.5, "150%")].iter().map(|(z, l)| format!("<option value='{}'{}>{}</option>", z, match (*z - c.default_zoom).abs() < 0.01 { true => " selected", false => "" }, l)).collect();
    let bms: String = match state.bookmarks.bookmarks.is_empty() {
        true => "<p class='dim'>No bookmarks yet \u{2014} hit \u{2606} in the URL bar or Ctrl+D.</p>".into(),
        false => state.bookmarks.bookmarks.iter().map(|bm| format!("<div class='row' id='bm-{}'><a href='{}' title='{}'>{}</a><button class='x' onclick='rmbm(\"{}\")'>remove</button></div>", esc_html(&bm.id), esc_html(&bm.url), esc_html(&bm.url), esc_html(&bm.title), esc_html(&bm.id))).collect(),
    };
    let active_id = state.themes.active_theme().id;
    let themes: String = state.themes.all_themes().iter().map(|t| format!("<label class='opt'><input type='radio' name='th' value='{}'{} onchange='set(\"theme\",this.value)'><span>{}</span></label>", esc_html(&t.id), match t.id == active_id { true => " checked", false => "" }, esc_html(&t.name))).collect();
    let home = match c.home_page.starts_with("http") { true => c.home_page.clone(), false => String::new() };
    let toggles = format!("<label class='opt'><input type='checkbox'{} onchange='set(\"clear_data_on_exit\",this.checked?1:0)'><span>Clear browsing data (cookies, cache, history) when Amni Browse closes</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"autofill_on_load\",this.checked?1:0)'><span>Let the engine save passwords and fill forms (Chromium profile store)</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"enable_do_not_track\",this.checked?1:0)'><span>Send Do Not Track + Global Privacy Control headers</span></label><label class='opt'><input type='checkbox'{} onchange='set(\"enable_doh\",this.checked?1:0)'><span>DNS over HTTPS (restart to apply)</span></label>", match c.clear_data_on_exit { true => " checked", false => "" }, match c.autofill_on_load { true => " checked", false => "" }, match c.enable_do_not_track { true => " checked", false => "" }, match c.enable_doh { true => " checked", false => "" });
    SETTINGS_TPL.replace("__THEME__", &theme_root_vars(&state.themes.active_theme())).replace("__THEMES__", &themes).replace("__VER__", APP_VERSION).replace("__RADIOS__", &radios).replace("__HOME__", &esc_html(&home)).replace("__ZOOMS__", &zooms)
        .replace("__SHIELD__", match shield { true => " checked", false => "" }).replace("__RESTORE__", match c.restore_session { true => " checked", false => "" }).replace("__UA__", &esc_html(c.custom_user_agent.as_deref().unwrap_or(""))).replace("__TOK__", token)
        .replace("__VAULT__", "Chromium profile store").replace("__PMRADIOS__", &toggles).replace("__PMLABEL__", "").replace("__PMCLI__", "").replace("__PMDB__", "").replace("__AUTOFILL__", "").replace("__CHKUPD__", match c.check_updates { true => " checked", false => "" })
        .replace("__UPD__", "checked on the site feed").replace("__PROFS__", "<div class='row'><span>Local \u{00b7} active</span></div>").replace("__CRASH__", "").replace("__IMPORTNOTE__", "").replace("__BMS__", &bms).replace("__ENGINE__", ENGINE)
}
fn render_tutorial_html(state: &BrowserState, token: &str) -> String {
    TUTORIAL_TPL.replace("__THEME__", &theme_root_vars(&state.themes.active_theme())).replace("__VER__", APP_VERSION).replace("__TOK__", token).replace("__BROWSERS__", "<p class='dim'>Import from Settings once you are in.</p>")
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
    let entries = &state.history.entries;
    let count = entries.len();
    let rows: String = if entries.is_empty() {
        "<div class='call' style='text-align:center;padding:32px 16px;'><p style='font-size:16px;font-weight:600;margin-bottom:8px;'>No browsing history</p><p class='dim'>Pages you visit will appear here.</p></div>".to_string()
    } else {
        entries.iter().rev().take(100).map(|e| {
            let u = esc_html(&e.url);
            let t = if e.title.trim().is_empty() { u.clone() } else { esc_html(&e.title) };
            let date_str = e.last_visited.format("%b %d, %H:%M").to_string();
            format!(
                "<div class='row' style='display:flex;align-items:center;justify-content:space-between;padding:10px 0;border-bottom:1px solid var(--stroke);'>\
                    <div style='flex:1;min-width:0;padding-right:16px;'>\
                        <a href='{}' style='font-size:14px;font-weight:600;color:var(--text);text-decoration:none;display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;'>{}</a>\
                        <div style='font-size:12px;color:var(--dim);margin-top:2px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;'>{}</div>\
                    </div>\
                    <span class='dim' style='font-size:12px;flex-shrink:0;'>{}</span>\
                </div>",
                u, t, u, date_str
            )
        }).collect()
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
.x,.btn{{background:var(--elev);border:1px solid var(--stroke);border-radius:3px;color:var(--text);padding:8px 14px;cursor:pointer;font:650 11px inherit;letter-spacing:.1em;text-transform:uppercase;margin:0 4px}}
.x:hover,.btn:hover{{border-color:var(--accent)}}
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
<p class='dim' style='letter-spacing:.1em;text-transform:uppercase;font-size:10px'>{} page(s)</p>
<button class='on'>History</button>
<button onclick='window.location.href="amnibrowse://downloads"'>Downloads</button>
<button onclick='window.location.href="amnibrowse://settings"'>Settings</button>
</nav>
<main>
<div style='display:flex;justify-content:space-between;align-items:center;margin-bottom:20px;'>
    <div>
        <h2>History</h2>
        <p class='dim' style='margin:0;'>Recently visited pages</p>
    </div>
    <div>
        <button class='btn' onclick='cmd("clear_data");window.location.reload()'>Clear History</button>
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
fn wire_engine(view: &WebView, uid: u64, push: Push, _blocker: Rc<RefCell<AdBlocker>>, _shield: Rc<Cell<bool>>, dnt: bool, _autofill: bool) -> Option<Core> {
    use wry::WebViewExtUnix;
    use webkit2gtk::{SettingsExt, WebViewExt};
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
    }
    let (p1, p2, p3, p4) = (push.clone(), push.clone(), push.clone(), push.clone());
    wv.connect_load_changed(move |w, _| p1(Ev::History(uid, w.can_go_back(), w.can_go_forward())));
    wv.connect_enter_fullscreen(move |_| { p2(Ev::PageFullscreen(uid, true)); false });
    wv.connect_leave_fullscreen(move |_| { p3(Ev::PageFullscreen(uid, false)); false });
    wv.connect_is_playing_audio_notify(move |w| p4(Ev::Audio(uid, w.is_playing_audio())));
    None
}
#[cfg(not(windows))]
fn dl_id(u: &str) -> String { format!("{:x}", u.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))) }
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
        let y = (self.chrome_px() + f).min(sz.height.saturating_sub(1));
        Rect { position: PhysicalPosition::new(f as i32, y as i32).into(), size: PhysicalSize::new(sz.width.saturating_sub(2 * f).max(1), sz.height.saturating_sub(y + f).max(1)).into() }
    }
    fn layout(&self) {
        let hide_chrome = self.fullscreen || self.page_fullscreen;
        if let Some(c) = self.chrome.as_ref() {
            self.place_chrome(c, self.chrome_rect());
            let _ = c.set_visible(!hide_chrome);
            #[cfg(not(windows))]
            {
                use gtk::prelude::WidgetExt;
                self.chrome_canvas.set_visible(!hide_chrome);
            }
        }
        let r = self.content_rect();
        for (i, t) in self.tabs.iter().enumerate() { self.place(&t.view, r); let _ = t.view.set_visible(i == self.active); }
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
    fn go_back(&self) {
        use webkit2gtk::WebViewExt;
        use wry::WebViewExtUnix;
        if let Some(t) = self.active_tab() { t.view.webview().go_back(); }
    }
    #[cfg(not(windows))]
    fn go_forward(&self) {
        use webkit2gtk::WebViewExt;
        use wry::WebViewExtUnix;
        if let Some(t) = self.active_tab() { t.view.webview().go_forward(); }
    }
    #[cfg(not(windows))]
    fn reload_page(&self) {
        use webkit2gtk::WebViewExt;
        use wry::WebViewExtUnix;
        if let Some(t) = self.active_tab() { t.view.webview().reload(); }
    }
    #[cfg(not(windows))]
    fn stop_page(&self) {
        use webkit2gtk::WebViewExt;
        use wry::WebViewExtUnix;
        if let Some(t) = self.active_tab() { t.view.webview().stop_loading(); }
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
        for t in &self.tabs { attach_filter(&t.view, self.filter, self.shield.get()); }
    }
    fn spawn_tab(&mut self, url: &str, private: bool, at: Option<usize>) -> usize {
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

        builder = builder
            .with_url(url)
            .with_bounds(content_rect)
            .with_user_agent(&ua)
            .with_devtools(true)
            .with_hotkeys_zoom(true)
            .with_back_forward_navigation_gestures(true)
            .with_initialization_script(&format!("{};{};{};{}", fetch_shim(), KEY_SCRIPT, FIND_SCRIPT, ICON_SCRIPT))
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
                    match v.get("type").and_then(|t| t.as_str()) {
                        Some("key") => p4(Ev::Key(uid, v.get("k").and_then(|k| k.as_str()).unwrap_or("").to_string(), v.get("shift").and_then(|s| s.as_i64()).unwrap_or(0) == 1, v.get("alt").and_then(|s| s.as_i64()).unwrap_or(0) == 1)),
                        Some("icon") => { if let Some(h) = v.get("href").and_then(|h| h.as_str()) { p4(Ev::Favicon(uid, h.to_string())); } }
                        _ => {}
                    }
                }
            });

        if attach_downloads {
            let dl_dir_c = dl_dir.clone();
            builder = builder.with_download_started_handler(move |u, path| {
                let name = guess_filename(&u, path);
                std::fs::create_dir_all(&dl_dir_c).ok();
                *path = unique_path(&dl_dir_c, &name);
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
        self.place(&view, content_rect);
        let core = wire_engine(&view, uid, push, self.blocker.clone(), self.shield.clone(), dnt, autofill);
        let _ = view.zoom(default_zoom.max(0.25));
        let tab = Tab { uid, view, core, url: url.to_string(), title: String::new(), private, loading: true, zoom: default_zoom, can_back: false, can_forward: false, icon: None, audio: false, pinned: false, group: None };
        let idx = at.unwrap_or(self.tabs.len()).min(self.tabs.len());
        self.tabs.insert(idx, tab);
        self.raise_chrome();
        idx
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
    fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() { return; }
        self.overlay_css = 0;
        let t = self.tabs.remove(idx);
        let _ = t.view.evaluate_script("try{window.stop()}catch(e){}try{document.querySelectorAll('video,audio').forEach(function(m){m.pause()})}catch(e){}");
        let _ = t.view.set_visible(false);
        if !t.private && !is_internal(&t.url) { self.closed.push((t.url.clone(), t.private)); self.closed.truncate(20); }
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
            self.active = idx;
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
                use wry::WebViewExtUnix;
                t.view.webview().grab_focus();
            }
            let _ = t.view.focus();
        }
    }
    fn navigate_active(&mut self, url: &str) {
        self.overlay_css = 0;
        if let Some(t) = self.tabs.get_mut(self.active) {
            t.url = url.to_string();
            t.loading = true;
            t.icon = None;
            let _ = t.view.load_url(url);
        }
        self.layout();
        self.focus_content();
    }
    fn focus_omnibox(&self, clear: bool) {
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
        let items = serde_json::json!([
            {"id":"new_tab","label":"New tab","enabled":true},
            {"id":"private_tab","label":"New private tab","enabled":true},
            {"id":"new_window","label":"New window","enabled":true},
            {"sep":true},
            {"id":"am_history","label":"History","enabled":true},
            {"id":"am_downloads","label":"Downloads","enabled":true},
            {"id":"bookmark","label":match bookmarked { true => "Remove bookmark", false => "Bookmark this page" },"enabled":!internal},
            {"sep":true},
            {"id":"am_find","label":"Find on page","enabled":true},
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
        let payload = serde_json::json!({"kind":"menu","x":(w - 244.0).max(4.0).round() as i64,"y":(SERVO_CHROME_HEIGHT_CSS as i64) - 8,"items":items});
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
        })).collect();
        let (theme, active_dl, bookmarked) = {
            let st = self.state();
            let th: serde_json::Value = serde_json::from_str(&st.themes.active_theme_json()).unwrap_or(serde_json::Value::Null);
            let adl = st.downloads.downloads.iter().filter(|d| matches!(d.status, DownloadStatus::Downloading | DownloadStatus::Pending)).count();
            let bm = !url.is_empty() && st.bookmarks.find_by_url(&url).is_some();
            (th, adl, bm)
        };
        serde_json::json!({
            "url": shown, "title": active.map(|t| t.title.clone()).unwrap_or_default(), "loading": active.map(|t| t.loading).unwrap_or(false),
            "canBack": active.map(|t| t.can_back).unwrap_or(false), "canForward": active.map(|t| t.can_forward).unwrap_or(false), "tabs": tabs, "theme": theme,
            "zoom": active.map(|t| t.zoom).unwrap_or(1.0), "fullscreen": self.fullscreen, "maximized": self.window.is_maximized(), "canReopen": !self.closed.is_empty(),
            "shield": self.shield.get(), "blocked": self.blocker.borrow().blocked_count(), "bookmarked": bookmarked, "vault": false, "downloads": active_dl, "profile": "Local",
            "find": self.find_query, "winh": (self.window.inner_size().to_logical::<f64>(self.scale()).height).round() as i64, "pm": "Passwords", "logins": [], "update": serde_json::Value::Null, "engine": ENGINE, "decorated": self.decorated,
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
                _ => info!("setting_set: ignored {}={}", k, v),
            }
            st.config.save();
        }
        if k == "theme" { self.apply_frame_color(); }
        if k == "block_ads" || k == "shield" { self.shield.set(on); self.reshield(); }
        if self.active_tab().map(|t| t.url.contains("amnibrowse.settings")).unwrap_or(false) && k == "theme" { self.active_js("location.reload()"); }
    }
    fn apply_frame_color(&self) { self.window.set_background_color(hex_rgba(&self.state().themes.active_theme().bg_primary)); }
    #[cfg(not(windows))]
    fn clear_browsing_data(&self, _kinds: u32) {
        use webkit2gtk::{WebContextExt, WebsiteDataManagerExtManual, WebsiteDataTypes, WebViewExt};
        use wry::WebViewExtUnix;
        if let Some(t) = self.tabs.first() {
            if let Some(ctx) = t.view.webview().context() {
                if let Some(dm) = ctx.website_data_manager() {
                    dm.clear(WebsiteDataTypes::all(), webkit2gtk::glib::TimeSpan::from_seconds(0), None::<&webkit2gtk::gio::Cancellable>, |_| {});
                }
            }
        }
    }
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
            "navigate" => {
                self.overlay_css = 0;
                let se = self.state().config.search_engine.clone();
                if let Some(u) = a.get("url").and_then(|u| resolve_input(u, &se)) { self.navigate_active(&u); }
            }
            "back" => { self.overlay_css = 0; self.go_back(); self.layout(); self.focus_content(); }
            "forward" => { self.overlay_css = 0; self.go_forward(); self.layout(); self.focus_content(); }
            "reload" => { self.overlay_css = 0; self.reload_page(); self.layout(); self.focus_content(); }
            "stop" => { self.overlay_css = 0; self.stop_page(); self.layout(); self.focus_content(); }
            "home" => { self.overlay_css = 0; let h = self.home_url(); self.navigate_active(&h); }
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
                let mut st = self.state_mut();
                if let Some(d) = st.downloads.downloads.iter_mut().find(|d| d.id == id) { d.downloaded_bytes = n; }
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
            Ev::Key(_, k, shift, alt) => self.handle_key(&k, shift, alt),
        }
    }
}
pub fn run(state: BrowserState) {
    privacy_env(&state.config);
    let token = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x5eed) ^ 0x9e37_79b9_7f4a_7c15u64);
    let ephemeral = std::env::args().any(|a| a == "--new-window");
    let saved = SessionManager::load().filter(|_| state.config.restore_session && !ephemeral);
    let decorated = std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(false);
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
        let tok_ok = parsed.query_pairs().any(|(k, v)| k == "tok" && v == ptok.as_str());
        let from_chrome = cfg!(not(windows))
            || tok_ok
            || req.headers().get("referer").and_then(|v| v.to_str().ok()).map(|r| r.contains("amnibrowse.chrome") || r.contains("amnibrowse://chrome") || r.contains("amnibrowse")).unwrap_or(false)
            || req.headers().get("origin").and_then(|v| v.to_str().ok()).map(|o| o.contains("amnibrowse.chrome") || o.contains("amnibrowse://chrome") || o.contains("amnibrowse")).unwrap_or(false);
        let args: HashMap<String, String> = parsed.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        match host.as_str() {
            "chrome" => respond("text/html; charset=utf-8", format!("<script>{}window.__amniToken={:?};</script>{}{}", fetch_shim(), ptok, load_toolbar_html().replace("__CHROMEREV__", APP_VERSION), match decorated { true => "<style>.win-btn{display:none!important}</style>", false => "" })),
            "cmd" if from_chrome || tok_ok => { pe.borrow_mut().push(Ev::Cmd(parsed.path().trim_start_matches('/').to_string(), args)); let _ = ppx.send_event(()); empty(204) }
            "state" if from_chrome || tok_ok => {
                let body = match pa.try_borrow() { Ok(g) => g.as_ref().map(|a| a.state_json()).unwrap_or_else(|| "{}".into()), Err(_) => pl.borrow().clone() };
                *pl.borrow_mut() = body.clone();
                respond("application/json; charset=utf-8", body)
            }
            "suggest" if from_chrome || tok_ok => {
                let q = args.get("q").cloned().unwrap_or_default();
                let body = match pstate.try_borrow() {
                    Ok(st) => {
                        let bms: Vec<(String, String)> = st.bookmarks.bookmarks.iter().map(|b| (b.url.clone(), b.title.clone())).collect();
                        let extra: Vec<(&str, &str)> = bms.iter().map(|(u, t)| (u.as_str(), t.as_str())).collect();
                        st.history.omnibox_json(&q, &extra, 8)
                    }
                    Err(_) => "[]".into(),
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
    };
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
        chrome.webview().set_background_color(&gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
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
