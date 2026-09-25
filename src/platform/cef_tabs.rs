use cef::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use x11_dl::xlib;
pub enum CefEv { DlWanted(u32, String, String), DlProgress(u32, i64, i64), DlDone(u32, bool, String), Title(u64, String), Address(u64, String), Loading(u64, bool, bool, bool), Popup(String), Ipc(u64, String), Fullscreen(u64, bool), Created(u64) }
thread_local! {
    static QUEUE: RefCell<Vec<CefEv>> = const { RefCell::new(Vec::new()) };
    static WAKE: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
    static NAV: RefCell<Option<Box<dyn Fn(u64, &str) -> bool>>> = const { RefCell::new(None) };
    static SCRIPT: RefCell<String> = const { RefCell::new(String::new()) };
    static DL_WAIT: RefCell<std::collections::HashMap<u32, BeforeDownloadCallback>> = RefCell::new(std::collections::HashMap::new());
    static DL_LIVE: RefCell<std::collections::HashMap<u32, DownloadItemCallback>> = RefCell::new(std::collections::HashMap::new());
}
pub fn download_to(id: u32, path: &Path) { if let Some(cb) = DL_WAIT.with(|w| w.borrow_mut().remove(&id)) { cb.cont(Some(&CefString::from(path.to_string_lossy().as_ref())), 0); } }
pub fn cancel_download(id: u32) { DL_WAIT.with(|w| w.borrow_mut().remove(&id)); if let Some(cb) = DL_LIVE.with(|l| l.borrow_mut().remove(&id)) { cb.cancel(); } }
static ON: OnceLock<bool> = OnceLock::new();
const IPC_TAG: &str = "\u{1}amni-ipc:";
fn push(e: CefEv) { QUEUE.with(|q| q.borrow_mut().push(e)); WAKE.with(|w| if let Some(f) = w.borrow().as_ref() { f() }); }
pub fn drain() -> Vec<CefEv> { QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut())) }
pub fn set_wake(f: impl Fn() + 'static) { WAKE.with(|w| *w.borrow_mut() = Some(Box::new(f))); }
pub fn set_nav_filter(f: impl Fn(u64, &str) -> bool + 'static) { NAV.with(|n| *n.borrow_mut() = Some(Box::new(f))); }
pub fn set_page_script(js: &str) { SCRIPT.with(|s| *s.borrow_mut() = format!("if(!window.__amniCef){{window.__amniCef=1;window.ipc=window.ipc||{{postMessage:function(m){{console.log({:?}+m)}}}};{}}}", IPC_TAG, js)); }
pub fn requested() -> bool { std::env::var("AMNI_ENGINE").map(|v| v != "webkit").unwrap_or(true) && !std::env::args().any(|a| a == "--engine=webkit") }
pub fn enabled() -> bool { *ON.get().unwrap_or(&false) }
pub fn wants(url: &str) -> bool { enabled() && (url.starts_with("http://") || url.starts_with("https://") || url.starts_with("about:")) && !url.starts_with("http://amnibrowse.") }
fn s(c: Option<&CefString>) -> String { c.map(|x| x.to_string()).unwrap_or_default() }
pub fn subprocess() -> Option<i32> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = cef::args::Args::new();
    let sub = args.as_cmd_line().map(|c| c.has_switch(Some(&CefString::from("type"))) == 1).unwrap_or(false);
    sub.then(|| execute_process(Some(args.as_main_args()), Some(&mut AmniApp::new()), std::ptr::null_mut()))
}
fn runtime_dir() -> PathBuf { std::env::var_os("AMNI_CEF_DIR").map(PathBuf::from).unwrap_or_else(|| std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)).unwrap_or_default()) }
fn seed_widevine(root: &Path) {
    let src = Path::new("/opt/google/chrome/WidevineCdm");
    let dst = root.join("WidevineCdm");
    let hint = dst.join("latest-component-updated-widevine-cdm");
    if !src.join("manifest.json").exists() || hint.exists() { return; }
    let _ = std::fs::create_dir_all(&dst);
    let _ = std::fs::write(&hint, format!("{{\"Path\":{:?}}}", src.to_string_lossy()));
}
pub fn init(data_dir: &Path) -> bool {
    if !requested() { let _ = ON.set(false); return false; }
    std::env::set_var("GDK_BACKEND", "x11");
    let exe = std::env::current_exe().unwrap_or_default();
    let dir = runtime_dir();
    let root = data_dir.join("cef");
    let _ = std::fs::create_dir_all(&root);
    seed_widevine(&root);
    wipe_pending(&root);
    let args = cef::args::Args::new();
    let settings = Settings { no_sandbox: 1, browser_subprocess_path: CefString::from(exe.to_string_lossy().as_ref()), resources_dir_path: CefString::from(dir.to_string_lossy().as_ref()), locales_dir_path: CefString::from(dir.join("locales").to_string_lossy().as_ref()), root_cache_path: CefString::from(root.to_string_lossy().as_ref()), cache_path: CefString::from(root.join("Default").to_string_lossy().as_ref()), persist_session_cookies: 1, log_severity: LogSeverity::WARNING, ..Default::default() };
    let ok = initialize(Some(args.as_main_args()), Some(&settings), Some(&mut AmniApp::new()), std::ptr::null_mut()) == 1;
    let _ = ON.set(ok);
    if ok {
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(4), || { do_message_loop_work(); gtk::glib::ControlFlow::Continue });
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || import_webkit_cookies(&root));
    }
    ok
}
fn basetime(unix: i64) -> Basetime { Basetime { val: (unix + 11_644_473_600) * 1_000_000 } }
fn import_webkit_cookies(root: &Path) {
    let mark = root.join(".webkit-cookies-imported");
    if mark.exists() { return; }
    let src = dirs::data_local_dir().unwrap_or_default().join("amni-browse").join("cookies");
    let Ok(txt) = std::fs::read_to_string(&src) else { let _ = std::fs::write(&mark, "0"); return };
    let Some(cm) = cookie_manager_get_global_manager(None) else { return };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let n: i32 = txt.lines().filter_map(|line| {
        let (http_only, l) = line.strip_prefix("#HttpOnly_").map(|r| (true, r)).unwrap_or((false, line));
        let f: Vec<&str> = l.split('\t').collect();
        (!l.starts_with('#') && f.len() >= 7).then_some(())?;
        let (secure, exp) = (f[3] == "TRUE", f[4].parse::<i64>().unwrap_or(0));
        (exp == 0 || exp > now).then_some(())?;
        let url = format!("{}://{}{}", if secure { "https" } else { "http" }, f[0].trim_start_matches('.'), f[2]);
        let c = Cookie { name: CefString::from(f[5]), value: CefString::from(f[6..].join("\t").as_str()), domain: CefString::from(f[0]), path: CefString::from(f[2]), secure: secure as _, httponly: http_only as _, creation: basetime(now), last_access: basetime(now), has_expires: (exp != 0) as _, expires: basetime(exp), same_site: CookieSameSite::UNSPECIFIED, priority: CookiePriority::MEDIUM, ..Default::default() };
        Some(cm.set_cookie(Some(&CefString::from(url.as_str())), Some(&c), None))
    }).sum();
    cm.flush_store(None);
    let _ = std::fs::write(&mark, n.to_string());
    log::info!("imported {} WebKit cookies into Chromium", n);
}
const SITE_DATA: [&str; 10] = ["Local Storage", "Session Storage", "IndexedDB", "Service Worker", "Cache", "Code Cache", "GPUCache", "blob_storage", "File System", "WebStorage"];
fn wipe_pending(root: &Path) {
    let mark = root.join(".wipe-site-data");
    if !mark.exists() { return; }
    SITE_DATA.iter().for_each(|d| { let _ = std::fs::remove_dir_all(root.join("Default").join(d)); });
    let _ = std::fs::remove_file(mark);
}
pub fn clear_data(root: &Path, tabs: &[&CefTab]) {
    if !enabled() { return; }
    if let Some(m) = cookie_manager_get_global_manager(None) { m.delete_cookies(None, None, None); m.flush_store(None); }
    tabs.iter().filter_map(|t| t.host()).for_each(|h| { h.execute_dev_tools_method(0, Some(&CefString::from("Network.clearBrowserCache")), None); });
    let _ = std::fs::write(root.join("cef").join(".wipe-site-data"), b"1");
}
pub fn shutdown_all() { if enabled() { for _ in 0..30 { do_message_loop_work(); std::thread::sleep(std::time::Duration::from_millis(10)); } shutdown(); } }
wrap_app! {
    struct AmniApp;
    impl App {
        fn on_before_command_line_processing(&self, process_type: Option<&CefString>, command_line: Option<&mut CommandLine>) {
            let (Some(c), true) = (command_line, s(process_type).is_empty()) else { return };
            for sw in ["no-first-run", "no-default-browser-check", "disable-background-networking"] { c.append_switch(Some(&CefString::from(sw))); }
            for (k, v) in [("ozone-platform", "x11"), ("password-store", "basic")] { c.append_switch_with_value(Some(&CefString::from(k)), Some(&CefString::from(v))); }
        }
    }
}
wrap_client! {
    struct TabClient { uid: u64 }
    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> { Some(TabDisplay::new(self.uid)) }
        fn load_handler(&self) -> Option<LoadHandler> { Some(TabLoad::new(self.uid)) }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> { Some(TabLife::new(self.uid)) }
        fn request_handler(&self) -> Option<RequestHandler> { Some(TabRequest::new(self.uid)) }
        fn download_handler(&self) -> Option<DownloadHandler> { Some(TabDownload::new(self.uid)) }
    }
}
wrap_display_handler! {
    struct TabDisplay { uid: u64 }
    impl DisplayHandler {
        fn on_title_change(&self, _b: Option<&mut Browser>, t: Option<&CefString>) { push(CefEv::Title(self.uid, s(t))); }
        fn on_address_change(&self, _b: Option<&mut Browser>, f: Option<&mut Frame>, u: Option<&CefString>) { if f.map(|f| f.is_main() == 1).unwrap_or(false) { push(CefEv::Address(self.uid, s(u))); } }
        fn on_fullscreen_mode_change(&self, _b: Option<&mut Browser>, on: ::std::os::raw::c_int) { push(CefEv::Fullscreen(self.uid, on == 1)); }
        fn on_console_message(&self, _b: Option<&mut Browser>, _l: LogSeverity, m: Option<&CefString>, _s: Option<&CefString>, _n: ::std::os::raw::c_int) -> ::std::os::raw::c_int {
            let m = s(m);
            match m.strip_prefix(IPC_TAG) { Some(body) => { push(CefEv::Ipc(self.uid, body.to_string())); 1 } None => 0 }
        }
    }
}
wrap_load_handler! {
    struct TabLoad { uid: u64 }
    impl LoadHandler {
        fn on_loading_state_change(&self, _b: Option<&mut Browser>, loading: ::std::os::raw::c_int, back: ::std::os::raw::c_int, fwd: ::std::os::raw::c_int) { push(CefEv::Loading(self.uid, loading == 1, back == 1, fwd == 1)); }
        fn on_load_start(&self, _b: Option<&mut Browser>, f: Option<&mut Frame>, _t: TransitionType) {
            let Some(f) = f else { return };
            if f.is_main() != 1 { return; }
            SCRIPT.with(|js| f.execute_java_script(Some(&CefString::from(js.borrow().as_str())), None, 0));
        }
    }
}
wrap_life_span_handler! {
    struct TabLife { uid: u64 }
    impl LifeSpanHandler {
        fn on_after_created(&self, _b: Option<&mut Browser>) { push(CefEv::Created(self.uid)); }
        fn on_before_popup(&self, _b: Option<&mut Browser>, _f: Option<&mut Frame>, _id: ::std::os::raw::c_int, url: Option<&CefString>, _n: Option<&CefString>, _d: WindowOpenDisposition, _g: ::std::os::raw::c_int, _pf: Option<&PopupFeatures>, _wi: Option<&mut WindowInfo>, _c: Option<&mut Option<Client>>, _st: Option<&mut BrowserSettings>, _e: Option<&mut Option<DictionaryValue>>, _nj: Option<&mut ::std::os::raw::c_int>) -> ::std::os::raw::c_int {
            let u = s(url);
            if !u.is_empty() { push(CefEv::Popup(u)); }
            1
        }
    }
}
wrap_request_handler! {
    struct TabRequest { uid: u64 }
    impl RequestHandler {
        fn on_before_browse(&self, _b: Option<&mut Browser>, _f: Option<&mut Frame>, r: Option<&mut Request>, _g: ::std::os::raw::c_int, _redir: ::std::os::raw::c_int) -> ::std::os::raw::c_int {
            let u = r.map(|r| CefString::from(&r.url()).to_string()).unwrap_or_default();
            NAV.with(|n| n.borrow().as_ref().map(|f| !f(self.uid, &u)).unwrap_or(false)) as _
        }
    }
}
wrap_download_handler! {
    struct TabDownload { uid: u64 }
    impl DownloadHandler {
        fn can_download(&self, _b: Option<&mut Browser>, _u: Option<&CefString>, _m: Option<&CefString>) -> ::std::os::raw::c_int { 1 }
        fn on_before_download(&self, _b: Option<&mut Browser>, item: Option<&mut DownloadItem>, name: Option<&CefString>, cb: Option<&mut BeforeDownloadCallback>) -> ::std::os::raw::c_int {
            let (Some(i), Some(cb)) = (item, cb) else { return 0 };
            DL_WAIT.with(|w| w.borrow_mut().insert(i.id(), cb.clone()));
            push(CefEv::DlWanted(i.id(), CefString::from(&i.url()).to_string(), s(name)));
            1
        }
        fn on_download_updated(&self, _b: Option<&mut Browser>, item: Option<&mut DownloadItem>, cb: Option<&mut DownloadItemCallback>) {
            let Some(i) = item else { return };
            let id = i.id();
            if let Some(cb) = cb { DL_LIVE.with(|l| l.borrow_mut().insert(id, cb.clone())); }
            let (done, failed) = (i.is_complete() == 1, i.is_canceled() == 1 || i.is_interrupted() == 1);
            if done || failed { DL_LIVE.with(|l| l.borrow_mut().remove(&id)); push(CefEv::DlDone(id, done, CefString::from(&i.full_path()).to_string())); } else { push(CefEv::DlProgress(id, i.received_bytes(), i.total_bytes())); }
        }
    }
}
struct Holder { xl: xlib::Xlib, d: *mut xlib::Display, win: xlib::Window }
impl Holder {
    fn new(parent: xlib::Window, r: (i32, i32, i32, i32)) -> Option<Self> {
        let xl = xlib::Xlib::open().ok()?;
        unsafe {
            let d = (xl.XOpenDisplay)(std::ptr::null());
            if d.is_null() { return None; }
            let scr = (xl.XDefaultScreen)(d);
            let mut a: xlib::XSetWindowAttributes = std::mem::zeroed();
            a.colormap = (xl.XDefaultColormap)(d, scr);
            let win = (xl.XCreateWindow)(d, parent, r.0, r.1, r.2.max(1) as u32, r.3.max(1) as u32, 0, (xl.XDefaultDepth)(d, scr), xlib::InputOutput as u32, (xl.XDefaultVisual)(d, scr), xlib::CWColormap | xlib::CWBorderPixel | xlib::CWBackPixel, &mut a);
            (xl.XMapWindow)(d, win);
            (xl.XSync)(d, 0);
            Some(Self { xl, d, win })
        }
    }
}
impl Drop for Holder { fn drop(&mut self) { unsafe { (self.xl.XDestroyWindow)(self.d, self.win); (self.xl.XCloseDisplay)(self.d); } } }
pub struct CefTab { browser: Browser, holder: Holder, rect: std::cell::Cell<(i32, i32, i32, i32)> }
impl CefTab {
    pub fn new(uid: u64, parent: &gtk::Layout, r: (i32, i32, i32, i32), url: &str) -> Option<Self> {
        use gtk::prelude::*;
        parent.realize();
        let gw = parent.bin_window()?;
        gw.ensure_native();
        gw.display().sync();
        let xid = gw.downcast::<gdkx11::X11Window>().ok()?.xid();
        let holder = Holder::new(xid, r)?;
        let info = WindowInfo { runtime_style: RuntimeStyle::ALLOY, ..Default::default() }.set_as_child(holder.win as _, &Rect { x: 0, y: 0, width: r.2.max(1), height: r.3.max(1) });
        let mut client = TabClient::new(uid);
        let browser = browser_host_create_browser_sync(Some(&info), Some(&mut client), Some(&CefString::from(url)), Some(&BrowserSettings::default()), None, None)?;
        Some(Self { browser, holder, rect: std::cell::Cell::new(r) })
    }
    fn host(&self) -> Option<BrowserHost> { self.browser.host() }
    pub fn place(&self, r: (i32, i32, i32, i32)) {
        self.rect.set(r);
        let h = &self.holder;
        unsafe {
            (h.xl.XMoveResizeWindow)(h.d, h.win, r.0, r.1, r.2.max(1) as u32, r.3.max(1) as u32);
            if let Some(b) = self.host() { (h.xl.XMoveResizeWindow)(h.d, b.window_handle() as _, 0, 0, r.2.max(1) as u32, r.3.max(1) as u32); }
            (h.xl.XFlush)(h.d);
        }
        if let Some(b) = self.host() { b.was_resized(); }
    }
    pub fn set_visible(&self, on: bool) {
        let h = &self.holder;
        unsafe { if on { (h.xl.XMapRaised)(h.d, h.win); } else { (h.xl.XUnmapWindow)(h.d, h.win); } (h.xl.XFlush)(h.d); }
        if let Some(b) = self.host() { b.was_hidden((!on) as _); }
    }
    pub fn load_url(&self, url: &str) { if let Some(f) = self.browser.main_frame() { f.load_url(Some(&CefString::from(url))); } }
    pub fn eval(&self, js: &str) { if let Some(f) = self.browser.main_frame() { f.execute_java_script(Some(&CefString::from(js)), None, 0); } }
    pub fn go_back(&self) { self.browser.go_back(); }
    pub fn go_forward(&self) { self.browser.go_forward(); }
    pub fn reload(&self, hard: bool) { if hard { self.browser.reload_ignore_cache() } else { self.browser.reload() } }
    pub fn stop(&self) { self.browser.stop_load(); }
    pub fn zoom(&self, factor: f64) { if let Some(h) = self.host() { h.set_zoom_level(factor.max(0.25).ln() / 1.2f64.ln()); } }
    pub fn mute(&self, on: bool) { if let Some(h) = self.host() { h.set_audio_muted(on as _); } }
    pub fn print(&self) { if let Some(h) = self.host() { h.print(); } }
    pub fn focus(&self) { if let Some(h) = self.host() { h.set_focus(1); } }
    pub fn blur(&self) { if let Some(h) = self.host() { h.set_focus(0); } }
    pub fn devtools(&self) { if let Some(h) = self.host() { h.show_dev_tools(Some(&WindowInfo::default()), Option::<&mut Client>::None, Some(&BrowserSettings::default()), None); } }
}
impl Drop for CefTab { fn drop(&mut self) { if let Some(h) = self.host() { h.close_browser(1); } } }
pub fn grab_x_focus(w: &impl gtk::prelude::IsA<gtk::Widget>) {
    use gtk::prelude::*;
    if !enabled() { return; }
    let Some(xid) = w.window().and_then(|g| g.downcast::<gdkx11::X11Window>().ok()).map(|x| x.xid()) else { return };
    set_x_focus(xid);
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(60), move || set_x_focus(xid));
}
fn set_x_focus(xid: xlib::Window) {
    let Ok(xl) = xlib::Xlib::open() else { return };
    unsafe { let d = (xl.XOpenDisplay)(std::ptr::null()); if d.is_null() { return; } (xl.XSetInputFocus)(d, xid, xlib::RevertToParent, xlib::CurrentTime); (xl.XFlush)(d); (xl.XCloseDisplay)(d); }
}
