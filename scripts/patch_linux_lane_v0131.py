def rw(p,f,nl='\n'):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline=nl).write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
def c(s):
    s=sub1(s,'''use tao::{dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize}, event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy}, keyboard::{Key, ModifiersState}, platform::windows::WindowExtWindows, window::{Fullscreen, Window, WindowBuilder}};
use wry::{http, PageLoadEvent, Rect, WebView, WebViewBuilder, WebViewExtWindows};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, BytesReceivedChangedEventHandler, ClearBrowsingDataCompletedHandler, ContainsFullScreenElementChangedEventHandler, DownloadStartingEventHandler, FaviconChangedEventHandler, HistoryChangedEventHandler, IsDocumentPlayingAudioChangedEventHandler, StateChangedEventHandler, WebResourceRequestedEventHandler};
use windows::{core::{w, Interface, HSTRING, PWSTR}, Win32::Foundation::BOOL, Win32::System::{Com::CoTaskMemFree, WinRT::EventRegistrationToken}};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, SetWindowPos, GW_CHILD, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
''','''use tao::{dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize}, event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy}, keyboard::{Key, ModifiersState}, window::{Fullscreen, Window, WindowBuilder}};
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
''')
    s=sub1(s,'const ENGINE: &str = "Chromium (WebView2)";','#[cfg(windows)]\nconst ENGINE: &str = "Chromium (WebView2)";\n#[cfg(not(windows))]\nconst ENGINE: &str = "WebKitGTK";')
    s=sub1(s,'enum Ev { Cmd(String, HashMap<String, String>),','#[allow(dead_code)]\nenum Ev { Cmd(String, HashMap<String, String>),')
    s=sub1(s,'struct Tab { uid: u64, view: WebView, core: Option<ICoreWebView2>,','struct Tab { uid: u64, view: WebView, core: Option<Core>,')
    s=sub1(s,'    chrome_hwnd: HWND,\n','    chrome_hwnd: usize,\n')
    s=sub1(s,'''fn take_pwstr(p: PWSTR) -> String {''','''#[cfg(windows)]
fn take_pwstr(p: PWSTR) -> String {''')
    s=sub1(s,'''    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);''','''    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);
    #[cfg(not(windows))]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() && std::env::var("AMNI_VM").map(|v| v == "1").unwrap_or(false) { std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1"); }
    }''')
    s=sub1(s,'''/// Everything wry does not expose: request-level shield + DNT/GPC headers, real history state,
/// favicons, HTML5 fullscreen, audio state, download progress, password/form autofill.
fn wire_engine(''','''/// Everything wry does not expose: request-level shield + DNT/GPC headers, real history state,
/// favicons, HTML5 fullscreen, audio state, download progress, password/form autofill.
#[cfg(not(windows))]
fn wire_engine(_view: &WebView, _uid: u64, _push: Push, _blocker: Rc<RefCell<AdBlocker>>, _shield: Rc<Cell<bool>>, _dnt: bool, _autofill: bool) -> Option<Core> { None }
#[cfg(windows)]
fn wire_engine(''')
    s=sub1(s,'''    fn raise_chrome(&self) {
        if !self.chrome_hwnd.is_null() { unsafe { SetWindowPos(self.chrome_hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE); } }
    }''','''    #[cfg(windows)]
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
    fn open_devtools(&self) { if let Some(t) = self.active_tab() { t.view.open_devtools(); } }''')
    s=sub1(s,'''    fn content_rect(&self) -> Rect {
        let sz = self.window.inner_size();
        let f = self.frame_px();
        let y = (self.chrome_px() + f).min(sz.height.saturating_sub(1));''','''    fn content_rect(&self) -> Rect {
        let sz = self.window.inner_size();
        let f = self.frame_px();
        let overlay = match cfg!(windows) { true => 0, false => (self.overlay_css as f64 * self.scale()).round() as u32 };
        let y = (self.chrome_px().max(overlay) + f).min(sz.height.saturating_sub(1));''')
    s=sub1(s,'''        if let Some(c) = t.core.as_ref() { unsafe { let _ = c.Stop(); } }
        let _ = t.view.evaluate_script("try{document.querySelectorAll('video,audio').forEach(function(m){m.pause()})}catch(e){}");''','''        let _ = t.view.evaluate_script("try{window.stop()}catch(e){}try{document.querySelectorAll('video,audio').forEach(function(m){m.pause()})}catch(e){}");''')
    s=sub1(s,'''    fn with_core(&self, f: impl FnOnce(&ICoreWebView2)) { if let Some(c) = self.active_tab().and_then(|t| t.core.as_ref()) { f(c); } }''','''    #[cfg(windows)]
    fn with_core(&self, f: impl FnOnce(&Core)) { if let Some(c) = self.active_tab().and_then(|t| t.core.as_ref()) { f(c); } }''')
    s=sub1(s,'''    fn clear_browsing_data(&self, kinds: COREWEBVIEW2_BROWSING_DATA_KINDS) {''','''    #[cfg(not(windows))]
    fn clear_browsing_data(&self, _kinds: u32) {}
    #[cfg(windows)]
    fn clear_browsing_data(&self, kinds: COREWEBVIEW2_BROWSING_DATA_KINDS) {''')
    s=sub1(s,'''            ("f12", _, _) | ("i", true, false) => self.with_core(|c| unsafe { let _ = c.OpenDevToolsWindow(); }),''','''            ("f12", _, _) | ("i", true, false) => self.open_devtools(),''')
    s=sub1(s,'''            "back" => self.with_core(|c| unsafe { let _ = c.GoBack(); }),
            "forward" => self.with_core(|c| unsafe { let _ = c.GoForward(); }),
            "reload" => self.with_core(|c| unsafe { let _ = c.Reload(); }),
            "stop" => self.with_core(|c| unsafe { let _ = c.Stop(); }),''','''            "back" => self.go_back(),
            "forward" => self.go_forward(),
            "reload" => self.reload_page(),
            "stop" => self.stop_page(),''')
    s=sub1(s,'''            "devtools" => self.with_core(|c| unsafe { let _ = c.OpenDevToolsWindow(); }),''','''            "devtools" => self.open_devtools(),''')
    s=sub1(s,'''{ let _ = std::process::Command::new("explorer").arg(&p).spawn(); }''','''{ let _ = std::process::Command::new(match cfg!(windows) { true => "explorer", false => "xdg-open" }).arg(&p).spawn(); }''')
    s=sub1(s,'''            "clear_data" => { self.clear_browsing_data(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE); self.state.history.clear_all(); self.state.history.save(); }''','''            "clear_data" => { #[cfg(windows)] self.clear_browsing_data(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE); #[cfg(not(windows))] self.clear_browsing_data(0); self.state.history.clear_all(); self.state.history.save(); }''')
    s=sub1(s,'''    fn shutdown(&mut self) {
        self.persist();
        let c = &self.state.config;
        let mut kinds: Option<COREWEBVIEW2_BROWSING_DATA_KINDS> = None;
        let mut add = |k: COREWEBVIEW2_BROWSING_DATA_KINDS| { kinds = Some(match kinds { Some(x) => x | k, None => k }); };
        if c.clear_data_on_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_PROFILE); }
        if c.clear_cache_on_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE); }
        if c.clear_cookies_on_exit { add(COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES); }
        if let Some(k) = kinds { self.clear_browsing_data(k); std::thread::sleep(std::time::Duration::from_millis(400)); }
        if c.clear_history_on_exit || c.clear_data_on_exit { self.state.history.clear_all(); self.state.history.save(); }
        self.state.shutdown();
    }''','''    fn shutdown(&mut self) {
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
    }''')
    s=sub1(s,'''                    if !started { if let Some(js) = std::env::var_os("AMNI_PROBE_JS")''','''                    if !started && !cfg!(windows) && u.starts_with("http") { if let Some(t) = self.tabs.get_mut(i) { if t.icon.is_none() { t.icon = url::Url::parse(&u).ok().and_then(|p| p.host_str().map(|h| format!("{}://{}/favicon.ico", p.scheme(), h))); } } }
                    if !started { if let Some(js) = std::env::var_os("AMNI_PROBE_JS")''')
    s=sub1(s,'''    let decorated = std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(false);''','''    let decorated = std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(!cfg!(windows));''')
    s=sub1(s,'''    let parent = (window.hwnd() as isize) as HWND;
    let mut a = App { window, decorated, chrome: None, chrome_hwnd: std::ptr::null_mut(),''','''    #[cfg(windows)]
    let parent = (window.hwnd() as isize) as HWND;
    let mut a = App { window, decorated, chrome: None, chrome_hwnd: 0,''')
    s=sub1(s,'''    if let Ok(cs) = unsafe { chrome.controller().CoreWebView2().and_then(|c| c.Settings()) } { unsafe { let _ = cs.SetIsStatusBarEnabled(BOOL(0)); let _ = cs.SetAreDefaultContextMenusEnabled(BOOL(0)); } }
    a.chrome = Some(chrome);
    a.chrome_hwnd = unsafe { GetWindow(parent, GW_CHILD) };''','''    #[cfg(windows)]
    if let Ok(cs) = unsafe { chrome.controller().CoreWebView2().and_then(|c| c.Settings()) } { unsafe { let _ = cs.SetIsStatusBarEnabled(BOOL(0)); let _ = cs.SetAreDefaultContextMenusEnabled(BOOL(0)); } }
    a.chrome = Some(chrome);
    #[cfg(windows)]
    { a.chrome_hwnd = unsafe { GetWindow(parent, GW_CHILD) } as usize; }''')
    return s
rw('src/platform/chromium.rs',c)
rw('src/platform/mod.rs',lambda s: sub1(s,'#[cfg(all(feature = "webview", target_os = "windows"))]\npub mod chromium;','#[cfg(feature = "webview")]\npub mod chromium;'))
rw('src/main.rs',lambda s: sub1(s,'''    #[cfg(all(feature = "webview", not(feature = "servo-real"), target_os = "windows"))]
    { info!("  Backend: Chromium (WebView2 via wry/tao)"); platform::chromium::run(state); }
    #[cfg(all(feature = "webview", not(feature = "servo-real"), not(target_os = "windows")))]
    { let _ = state; info!("  Backend: WebView (wry/tao, WebKitGTK)"); platform::webview::Browser::new().run(); }''','''    #[cfg(all(feature = "webview", not(feature = "servo-real")))]
    { info!("  Backend: {}", match cfg!(windows) { true => "Chromium (WebView2 via wry/tao)", false => "WebKitGTK (wry/tao)" }); platform::chromium::run(state); }'''))
print('linux lane patched')
