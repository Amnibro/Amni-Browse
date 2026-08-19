#[cfg(feature = "webview")]
use log::{error, info};
#[cfg(feature = "webview")]
use tao::{event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{Key, ModifiersState}, window::WindowBuilder};
#[cfg(all(feature = "webview", target_os = "linux"))]
use tao::platform::unix::WindowExtUnix;
#[cfg(feature = "webview")]
use wry::WebViewBuilder;
#[cfg(all(feature = "webview", target_os = "linux"))]
use wry::WebViewBuilderExtUnix;
#[cfg(feature = "webview")]
use crate::{app::BrowserState, storage::config::{APP_NAME, APP_VERSION, WEBVIEW_COLD_START}, net::ipc::{parse_ipc_message, IpcMessage, IpcResponse}, ui::webview as spa, engine::adblocker::AdBlocker};
#[cfg(all(feature = "webview", target_os = "linux"))]
use crate::storage::config::DEFAULT_SEARCH_ENGINE;
#[cfg(all(feature = "webview", target_os = "linux"))]
use gtk::prelude::*;
#[cfg(feature = "webview")]
use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};
#[cfg(feature = "webview")]
enum Act { Nav(String), Js(String), Title(String), Omni(String) }
#[cfg(feature = "webview")]
pub struct Browser;
#[cfg(feature = "webview")]
impl Browser {
    pub fn new() -> Self { Self }
    pub fn run(self) {
        let state = Rc::new(RefCell::new(BrowserState::new()));
        let acts: Rc<RefCell<Vec<Act>>> = Rc::new(RefCell::new(Vec::new()));
        let event_loop = EventLoop::new();
        let proxy = event_loop.create_proxy();
        let (async_tx, async_rx) = std::sync::mpsc::channel::<String>();
        {
            let mut s = state.borrow_mut();
            s.async_tx = Some(async_tx);
            let px_notify = proxy.clone();
            s.async_notify = Some(Arc::new(move || { px_notify.send_event(()).ok(); }));
        }
        let window = WindowBuilder::new()
            .with_title(format!("{} v{} — Privacy First", APP_NAME, APP_VERSION))
            .with_inner_size(tao::dpi::LogicalSize::new(1400.0, 900.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(640.0, 400.0))
            .build(&event_loop).expect("window");
        let newtab_html = spa::browser_html(&state.borrow().themes.active_theme());
        let s1 = Rc::clone(&state);
        let a1 = Rc::clone(&acts);
        let px1 = proxy.clone();
        let a_nav = Rc::clone(&acts);
        let px_nav = proxy.clone();
        let a_load = Rc::clone(&acts);
        let px_load = proxy.clone();
        let proto_html = newtab_html.into_bytes();
        // Do not call .build() on this chain. wry 0.46 build(&window) on Linux is X11-only
        // and does not pack WebKit into tao's GTK vbox → title-only white window (B3).
        let builder = WebViewBuilder::new()
            .with_custom_protocol("amnibrowse".to_string(), move |_, _request| {
                wry::http::Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .header("Pragma", "no-cache")
                    .header("Expires", "0")
                    .body(Cow::Owned(proto_html.clone()))
                    .unwrap()
            })
            .with_url(WEBVIEW_COLD_START)
            .with_devtools(cfg!(debug_assertions))
            .with_initialization_script(&chrome_init_js())
            .with_navigation_handler(move |url| {
                a_nav.borrow_mut().push(Act::Omni(url));
                px_nav.send_event(()).ok();
                true
            })
            .with_on_page_load_handler(move |ev, url| {
                match ev {
                    wry::PageLoadEvent::Started | wry::PageLoadEvent::Finished => {
                        a_load.borrow_mut().push(Act::Omni(url));
                        px_load.send_event(()).ok();
                    }
                }
            })
            .with_ipc_handler(move |msg| {
                let body = msg.body();
                match parse_ipc_message(body) {
                    Ok(m) => {
                        let rel = matches!(&m, IpcMessage::Refresh);
                        let mut s = s1.borrow_mut();
                        if let Some(resp) = s.handle_command(m) {
                            drop(s);
                            match &resp {
                                IpcResponse::NavigateTo { url } => {
                                    a1.borrow_mut().push(Act::Js(resp.to_js_call()));
                                    a1.borrow_mut().push(Act::Nav(url.clone()));
                                    a1.borrow_mut().push(Act::Title(url.clone()));
                                }
                                _ => a1.borrow_mut().push(Act::Js(resp.to_js_call())),
                            }
                        } else { drop(s); }
                        if rel { a1.borrow_mut().push(Act::Js("location.reload()".into())); }
                    }
                    Err(e) => error!("IPC: {}", e),
                }
                px1.send_event(()).ok();
            });
        #[cfg(target_os = "linux")]
        let (webview, omnibox) = {
            let vbox = window.default_vbox().expect("gtk vbox");
            let gtk_win = window.gtk_window();
            // Overlay fills the vbox. build_gtk FIRST so WebKit is the main
            // child (paint). add_overlay / show_all only after it exists —
            // showing the bar first covered WebKit (blank white content).
            let overlay = gtk::Overlay::new();
            overlay.set_hexpand(true);
            overlay.set_vexpand(true);
            vbox.pack_start(&overlay, true, true, 0);
            let webview = builder.build_gtk(&overlay).expect("webview");
            let (bar, omnibox) = make_native_omnibox(gtk_win, Rc::clone(&acts), proxy.clone());
            bar.set_vexpand(false);
            bar.set_valign(gtk::Align::Start);
            bar.set_halign(gtk::Align::Fill);
            bar.set_size_request(-1, 44);
            overlay.add_overlay(&bar);
            overlay.show_all();
            (webview, omnibox)
        };
        #[cfg(not(target_os = "linux"))]
        let webview = builder.build(&window).expect("webview");
        info!("Amni Browse v{} running! webview cold_start={}", APP_VERSION, WEBVIEW_COLD_START);
        proxy.send_event(()).ok();
        let mut modifiers = ModifiersState::empty();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(()) => {
                    while let Ok(js) = async_rx.try_recv() {
                        webview.evaluate_script(&js).ok();
                    }
                    let pending: Vec<_> = acts.borrow_mut().drain(..).collect();
                    for act in pending {
                        match act {
                            Act::Nav(url) => {
                                let nav_url = url.trim().to_string();
                                if nav_url.is_empty() {
                                    error!("Ignoring empty navigation URL");
                                    continue;
                                }
                                if nav_url.starts_with("http://") || nav_url.starts_with("https://") {
                                    if let Err(e) = webview.load_url(&nav_url) {
                                        error!("Failed to navigate to '{}': {}", nav_url, e);
                                    }
                                    #[cfg(target_os = "linux")]
                                    omnibox.set_text(&nav_url);
                                } else {
                                    let raw = if nav_url.starts_with("amnibrowse://") { nav_url.clone() } else { "amnibrowse://newtab/".to_string() };
                                    let rest = &raw["amnibrowse://".len()..];
                                    let (host, path) = match rest.find('/') { Some(i) => (&rest[..i], &rest[i..]), None => (rest, "/") };
                                    let target = format!("http://amnibrowse.{}{}", host, path);
                                    if let Err(e) = webview.load_url(&target) {
                                        error!("Failed to load internal page '{}' (from '{}'): {}", target, raw, e);
                                    }
                                }
                                if nav_url.starts_with("amnibrowse://") {
                                    window.set_title(&format!("{} v{} — Privacy First", APP_NAME, APP_VERSION));
                                }
                            }
                            Act::Js(js) => { webview.evaluate_script(&js).ok(); }
                            Act::Title(t) => {
                                let s: String = t.chars().take(80).collect();
                                window.set_title(&format!("{} — {}", s, APP_NAME));
                            }
                            // Address-bar sync only. Never load_url here — that loops with the nav handler.
                            Act::Omni(url) => {
                                let u = url.trim();
                                if u.is_empty() { continue; }
                                let s: String = u.chars().take(80).collect();
                                window.set_title(&format!("{} — {}", s, APP_NAME));
                                #[cfg(target_os = "linux")]
                                if !omnibox.has_focus() { omnibox.set_text(u); }
                            }
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    state.borrow_mut().shutdown();
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent { event: WindowEvent::ModifiersChanged(m), .. } => {
                    modifiers = m;
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } => {
                    if key_event.state == ElementState::Pressed && is_accel_l(&key_event, modifiers) {
                        #[cfg(target_os = "linux")]
                        { omnibox_focus_replace(&omnibox); }
                        #[cfg(not(target_os = "linux"))]
                        { webview.evaluate_script(FOCUS_URL_BAR_JS).ok(); }
                    }
                }
                _ => {}
            }
        });
    }
}
#[cfg(all(feature = "webview", not(target_os = "linux")))]
const FOCUS_URL_BAR_JS: &str = r#"(function(){try{if(typeof window.__amni_steal_focus==='function')window.__amni_steal_focus();}catch(_){}})();"#;
#[cfg(feature = "webview")]
fn is_accel_l(key_event: &tao::event::KeyEvent, modifiers: ModifiersState) -> bool {
    let accel = modifiers.control_key() || modifiers.super_key();
    accel && !modifiers.alt_key() && matches!(key_event.key_without_modifiers(), Key::Character(c) if c.eq_ignore_ascii_case("l"))
}
#[cfg(all(feature = "webview", target_os = "linux"))]
fn resolve_omnibox_input(raw: &str) -> String {
    let v = raw.trim();
    if v.starts_with("http://") || v.starts_with("https://") { v.to_string() }
    else if v.contains('.') && !v.contains(' ') { format!("https://{}", v) }
    else { format!("{}{}", DEFAULT_SEARCH_ENGINE, urlencoding::encode(v)) }
}
/// grab_focus() on GtkEntry defers caret-to-end and drops a same-tick select_region,
/// so the first typed URL prepends and leaves a stale suffix. Re-select on idle.
#[cfg(all(feature = "webview", target_os = "linux"))]
fn omnibox_focus_replace(entry: &gtk::Entry) {
    entry.grab_focus();
    entry.select_region(0, -1);
    let e = entry.clone();
    glib::idle_add_local_once(move || { e.select_region(0, -1); });
}
#[cfg(all(feature = "webview", target_os = "linux"))]
fn make_native_omnibox(
    gtk_win: &gtk::ApplicationWindow,
    acts: Rc<RefCell<Vec<Act>>>,
    proxy: tao::event_loop::EventLoopProxy<()>,
) -> (gtk::Box, gtk::Entry) {
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    bar.set_widget_name("amni-omni");
    bar.set_margin_start(6);
    bar.set_margin_end(6);
    let css = gtk::CssProvider::new();
    if css.load_from_data(b"#amni-omni { background-color: #12122a; }").is_ok() {
        bar.style_context().add_provider(&css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
    let mk_btn = |label: &str, tip: &str| {
        let b = gtk::Button::with_label(label);
        b.set_tooltip_text(Some(tip));
        b
    };
    let back = mk_btn("◀", "Back");
    let fwd = mk_btn("▶", "Forward");
    let reload = mk_btn("⟳", "Reload");
    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("Search or enter URL…"));
    entry.set_hexpand(true);
    bar.pack_start(&back, false, false, 0);
    bar.pack_start(&fwd, false, false, 0);
    bar.pack_start(&reload, false, false, 0);
    bar.pack_start(&entry, true, true, 0);
    let bind_js = |btn: &gtk::Button, js: &str| {
        let acts = Rc::clone(&acts);
        let proxy = proxy.clone();
        let js = js.to_string();
        btn.connect_clicked(move |_| {
            acts.borrow_mut().push(Act::Js(js.clone()));
            proxy.send_event(()).ok();
        });
    };
    bind_js(&back, "history.back()");
    bind_js(&fwd, "history.forward()");
    bind_js(&reload, "location.reload()");
    {
        let acts = Rc::clone(&acts);
        let proxy = proxy.clone();
        entry.connect_activate(move |e| {
            let dest = resolve_omnibox_input(&e.text());
            if dest.is_empty() { return; }
            acts.borrow_mut().push(Act::Nav(dest));
            proxy.send_event(()).ok();
        });
    }
    let focus_entry = entry.clone();
    gtk_win.connect_key_press_event(move |_, ev| {
        let raw = ev.as_ref();
        let state = gdk::ModifierType::from_bits_truncate(raw.state);
        let accel = state.contains(gdk::ModifierType::CONTROL_MASK)
            || state.contains(gdk::ModifierType::SUPER_MASK)
            || state.contains(gdk::ModifierType::MOD4_MASK);
        let alt = state.contains(gdk::ModifierType::MOD1_MASK);
        let kv = raw.keyval;
        let is_l = kv == *gdk::keys::constants::l || kv == *gdk::keys::constants::L;
        if accel && !alt && is_l {
            omnibox_focus_replace(&focus_entry);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    info!("Linux native GTK omnibox overlay valign-start (outside WebKit)");
    (bar, entry)
}
#[cfg(feature = "webview")]
fn chrome_init_js() -> String {
        let native = if cfg!(target_os = "linux") { "true" } else { "false" };
        let mut js = String::from("(function(){\nwindow.__amni_native_omnibox = ");
        js.push_str(native);
        js.push_str(";\n");
        js.push_str(r#"
try { if (window.self !== window.top) return; } catch(_) { return; }
if (location.protocol !== 'http:' && location.protocol !== 'https:') return;
if ((location.hostname || '').indexOf('amnibrowse.') === 0) return;
function injectNativePush(){
    try {
        const d = document;
        if (!d.documentElement) return false;
        if (d.getElementById('__amni_push_style')) return true;
        const s = d.createElement('style');
        s.id = '__amni_push_style';
        s.textContent = 'html{margin-top:48px!important}';
        (d.head || d.documentElement).appendChild(s);
        return true;
    } catch(_) { return false; }
}
function bindOmniboxHotkey(){
    if (window.__amni_native_omnibox) return;
    if (window.__amni_l_bound) return;
    window.__amni_l_bound = 1;
    window.addEventListener('keydown', function(e){
        if ((e.ctrlKey || e.metaKey) && !e.altKey && (e.key === 'l' || e.key === 'L')) {
            if (typeof stealOmnibox === 'function' && stealOmnibox()) { e.preventDefault(); e.stopPropagation(); }
        }
    }, true);
    window.addEventListener('pointerdown', function(e){
        if (e.clientY >= 48) return;
        const host = document.getElementById('__atb_host');
        if (!host || e.target === host || host.contains(e.target)) return;
        e.preventDefault();
        if (typeof stealOmnibox === 'function') stealOmnibox();
    }, true);
}
if (window.__amni_native_omnibox) {
    function startNativePush(){
        if (!injectNativePush()) {
            let tries = 0;
            const tid = setInterval(function(){ tries++; if (injectNativePush() || tries > 80) clearInterval(tid); }, 50);
        }
    }
    startNativePush();
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', startNativePush, { once:true });
    return;
}
function ipc(o){ try { window.ipc && window.ipc.postMessage(JSON.stringify(o)); } catch(_) {} }
function stealOmnibox(){
    try {
        document.querySelectorAll('dialog[open]').forEach(function(d){ try { d.close(); d.show(); } catch(_) {} });
        document.querySelectorAll('[inert]').forEach(function(el){ try { el.removeAttribute('inert'); } catch(_) {} });
        const host = document.getElementById('__atb_host');
        const root = host && host.shadowRoot;
        const u = root && root.getElementById('_au');
        if (!host || !u) return false;
        if (typeof host.showPopover === 'function') {
            try { host.popover = 'manual'; host.showPopover(); } catch(_) {}
        }
        u.focus();
        if (u.select) u.select();
        return true;
    } catch(_) { return false; }
}
window.__amni_steal_focus = stealOmnibox;
bindOmniboxHotkey();
function wireHandlers(host){
    const root = host && host.shadowRoot;
    if (!root) return;
    const q = (id) => root.getElementById(id);
    const u = q('_au');
    if (u) {
        u.value = location.href;
        u.onkeydown = function(e){
            if (e.key !== 'Enter') return;
            const v = (this.value || '').trim();
            if (!v) return;
            const msg = /^https?:\/\//.test(v) ? { type:'navigate', url:v } : (v.indexOf('.') > -1 && v.indexOf(' ') < 0 ? { type:'navigate', url:'https://' + v } : { type:'search', query:v });
            ipc(msg);
        };
    }
    const bind = (id, fn) => { const el = q(id); if (el) el.onclick = fn; };
    bind('_ab', function(){ ipc({ type:'back' }); });
    bind('_af', function(){ ipc({ type:'forward' }); });
    bind('_ar', function(){ ipc({ type:'refresh' }); });
    bind('_ah', function(){ ipc({ type:'navigate', url:'amnibrowse://newtab' }); });
    bind('_abk', function(){ ipc({ type:'bookmark_add', title:document.title || location.href, url:location.href }); });
}
function ensureToolbar(){
    try {
        const d = document;
        if (!d.documentElement || !d.head || !d.body) return false;
        let host = d.getElementById('__atb_host');
        if (!host || !host.shadowRoot) {
            host = d.createElement('div');
            host.id = '__atb_host';
            host.style.cssText = 'position:fixed;top:0;left:0;right:0;height:44px;z-index:2147483647;pointer-events:auto;';
            const root = host.attachShadow({ mode:'open' });
            const style = d.createElement('style');
            style.textContent = ':host{all:initial;position:fixed;top:0;left:0;right:0;height:44px;z-index:2147483647}*{box-sizing:border-box}#__atb{position:fixed;top:0;left:0;right:0;height:44px;background:linear-gradient(180deg,#0a0a18 0%,#12122a 100%);z-index:2147483647;display:flex;align-items:center;padding:0 8px;gap:4px;font-family:system-ui,-apple-system,sans-serif;box-shadow:0 2px 16px rgba(0,0,0,0.7);border-bottom:1px solid rgba(0,212,255,0.2)}button{background:none;border:none;color:#7af;cursor:pointer;padding:5px 9px;font-size:15px;border-radius:6px;transition:all .15s;line-height:1}button:hover{background:rgba(0,180,255,0.15);color:#0df}input{flex:1;background:#151530;border:1px solid #252550;color:#ddf;padding:8px 18px;border-radius:22px;font-size:13px;outline:none;transition:border .2s,box-shadow .2s;min-width:0}input:focus{border-color:#0af;box-shadow:0 0 0 2px rgba(0,170,255,0.25)}.ab{font-size:9px;background:#0c6;color:#000;padding:1px 5px;border-radius:8px;font-weight:700;margin-left:2px}.logo{color:#0af;font-weight:800;font-size:13px;letter-spacing:-0.5px;margin:0 4px}';
            root.appendChild(style);
            const bar = d.createElement('div');
            bar.id = '__atb';
            bar.innerHTML = '<span class="logo">A</span>'
                + '<button title="Back" id="_ab">◀</button>'
                + '<button title="Forward" id="_af">▶</button>'
                + '<button title="Reload" id="_ar">⟳</button>'
                + '<button title="Home" id="_ah">⌂</button>'
                + '<input id="_au" value="" placeholder="Search or enter URL…"/>'
                + '<button title="Bookmark" id="_abk">★</button>'
                + '<span class="ab" id="_as" title="Ads blocked">🛡</span>';
            root.appendChild(bar);
            d.body.prepend(host);
        }
        if (!d.getElementById('__amni_push_style')) {
            const s = d.createElement('style');
            s.id = '__amni_push_style';
            s.textContent = 'html{margin-top:48px!important}';
            d.head.appendChild(s);
        }
        wireHandlers(host);
        if (host && !host.__amni_stop) {
            host.__amni_stop = 1;
            host.addEventListener('keydown', function(e){ e.stopPropagation(); }, true);
            if (host.shadowRoot) host.shadowRoot.addEventListener('keydown', function(e){ e.stopPropagation(); });
        }
        ipc({ type:'get_stats' });
        return true;
    } catch(_) {
        return false;
    }
}
window.__amni_receive = function(msg){
    const host = document.getElementById('__atb_host');
    const root = host && host.shadowRoot;
    if (!root) return;
    const u = root.getElementById('_au');
    const sh = root.getElementById('_as');
    if (msg.type === 'stats' && sh) sh.textContent = '🛡 ' + msg.ads_blocked;
    if (msg.type === 'navigate_to' && u) u.value = msg.url;
};
function start(){
    if (!ensureToolbar()) {
        let tries = 0;
        const tid = setInterval(function(){ tries++; if (ensureToolbar() || tries > 80) clearInterval(tid); }, 50);
    }
    const observer = new MutationObserver(function(){ ensureToolbar(); });
    observer.observe(document.documentElement || document, { childList:true, subtree:true });
    window.addEventListener('pageshow', function(){ ensureToolbar(); });
}
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, { once:true });
else start();
})();"#);
        js
}
#[cfg(all(test, feature = "webview", target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn omnibox_http_as_is() {
        assert_eq!(resolve_omnibox_input("http://neverssl.com/"), "http://neverssl.com/");
        assert_eq!(resolve_omnibox_input(" https://example.com "), "https://example.com");
    }
    #[test]
    fn omnibox_dotted_token_gets_https() {
        assert_eq!(resolve_omnibox_input("example.com"), "https://example.com");
    }
    #[test]
    fn omnibox_search_uses_lite_ddg() {
        let u = resolve_omnibox_input("hello world");
        assert!(u.starts_with(DEFAULT_SEARCH_ENGINE), "{u}");
        assert!(u.contains("hello"), "{u}");
    }
    #[test]
    fn chrome_js_native_pushes_content_skips_page_steal_and_wheel() {
        let js = chrome_init_js();
        assert!(!js.contains("__amni_last_wheel"), "wheel click-suppress eats real clicks after scroll");
        assert!(js.contains("html{margin-top:48px!important}"), "overlay bar must not cover page content");
        assert!(js.contains("function bindOmniboxHotkey"), "page steal must be a named skip point");
        let bind = js.find("function bindOmniboxHotkey").expect("bindOmniboxHotkey");
        assert!(js[bind..].contains("if (window.__amni_native_omnibox) return"),
            "B2 steal is native GTK, not the old 48px page steal");
    }
}
