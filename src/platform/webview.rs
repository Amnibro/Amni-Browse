#[cfg(feature = "webview")]
use log::{error, info};
#[cfg(feature = "webview")]
use tao::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
#[cfg(feature = "webview")]
use wry::WebViewBuilder;
#[cfg(feature = "webview")]
use crate::{app::BrowserState, storage::config::{APP_NAME, APP_VERSION}, net::ipc::{parse_ipc_message, IpcMessage, IpcResponse}, ui::webview as spa, engine::adblocker::AdBlocker};
#[cfg(feature = "webview")]
use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};
#[cfg(feature = "webview")]
enum Act { Nav(String), Js(String), Title(String) }
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
        let proto_html = newtab_html.into_bytes();
        let webview = WebViewBuilder::new()
            .with_custom_protocol("amnibrowse".to_string(), move |_, _request| {
                wry::http::Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .header("Pragma", "no-cache")
                    .header("Expires", "0")
                    .body(Cow::Owned(proto_html.clone()))
                    .unwrap()
            })
            .with_url("amnibrowse://newtab/")
            .with_devtools(cfg!(debug_assertions))
            .with_initialization_script(&chrome_init_js())
            .with_navigation_handler(|_url| {
                // Allow all navigations; block/ads are handled via the engine if needed.
                true
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
            })
            .build(&window).expect("webview");
        info!("Amni Browse v{} running!", APP_VERSION);
        proxy.send_event(()).ok();
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
                                } else {
                                    let target = if nav_url.starts_with("amnibrowse://") {
                                        nav_url.clone()
                                    } else {
                                        "amnibrowse://newtab/".to_string()
                                    };
                                    if let Err(e) = webview.load_url(&target) {
                                        error!("Failed to load internal page '{}': {}", target, e);
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
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    state.borrow_mut().shutdown();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }
}
#[cfg(feature = "webview")]
fn chrome_init_js() -> String {
        r#"(function(){
try { if (window.self !== window.top) return; } catch(_) { return; }
if (location.protocol !== 'http:' && location.protocol !== 'https:') return;
function ipc(o){ try { window.ipc && window.ipc.postMessage(JSON.stringify(o)); } catch(_) {} }
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
})();"#.to_string()
}
