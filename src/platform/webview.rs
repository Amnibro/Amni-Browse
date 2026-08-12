#[cfg(feature = "webview")]
use log::{error, info};
#[cfg(feature = "webview")]
use tao::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
#[cfg(feature = "webview")]
use wry::{PageLoadEvent, WebViewBuilder};
#[cfg(feature = "webview")]
use crate::{app::BrowserState, storage::config::{APP_NAME, APP_VERSION}, net::ipc::{parse_ipc_message, IpcMessage, IpcResponse}, ui::theme::Theme, ui::webview as spa, ui::tokens};
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
        let loaded_url = Rc::new(RefCell::new(String::from("amnibrowse://newtab")));
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
            .with_decorations(true)
            .with_inner_size(tao::dpi::LogicalSize::new(1400.0, 900.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(640.0, 400.0))
            .build(&event_loop).expect("window");
        let boot_theme = state.borrow().themes.active_theme();
        let boot_tabs = state.borrow().tabs.to_json();
        let last_tabs = Rc::new(RefCell::new(boot_tabs.clone()));
        let newtab_html = spa::browser_html(&boot_theme);
        let s1 = Rc::clone(&state);
        let a1 = Rc::clone(&acts);
        let lu1 = Rc::clone(&loaded_url);
        let lt1 = Rc::clone(&last_tabs);
        let px1 = proxy.clone();
        let proto_html = newtab_html.into_bytes();
        let a_load = Rc::clone(&acts);
        let lt_load = Rc::clone(&last_tabs);
        let px_load = proxy.clone();
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
            .with_initialization_script(&chrome_init_js(&boot_theme, &boot_tabs))
            .with_navigation_handler(|_url| true)
            .with_on_page_load_handler(move |ev, url| {
                if !matches!(ev, PageLoadEvent::Started | PageLoadEvent::Finished) { return; }
                if url.contains("amnibrowse.") { return; }
                let t = lt_load.borrow().clone();
                if t.is_empty() || t == "[]" { return; }
                a_load.borrow_mut().push(Act::Js(format!(
                    "(function(){{try{{window.__AMNI_TAB_SEED={0};if(window.__AMNI_SYNC_TABS)window.__AMNI_SYNC_TABS({0});}}catch(_){{}}}})()",
                    t
                )));
                px_load.send_event(()).ok();
            })
            .with_ipc_handler(move |msg| {
                let body = msg.body();
                match parse_ipc_message(body) {
                    Ok(m) => {
                        let rel = matches!(&m, IpcMessage::Refresh);
                        let tab_nav = matches!(&m,
                            IpcMessage::NewTab { .. } | IpcMessage::CloseTab { .. }
                            | IpcMessage::SwitchTab { .. } | IpcMessage::NewPrivateTab { .. });
                        let mut s = s1.borrow_mut();
                        if let Some(resp) = s.handle_command(m) {
                            let tabs_json = s.tabs.to_json();
                            *lt1.borrow_mut() = tabs_json.clone();
                            let active_url = s.tabs.active_tab().map(|t| t.url.clone());
                            drop(s);
                            match &resp {
                                IpcResponse::NavigateTo { url } => {
                                    a1.borrow_mut().push(Act::Js(resp.to_js_call()));
                                    a1.borrow_mut().push(Act::Js(format!(
                                        "window.__AMNI_TAB_SEED={0};window.__AMNI_SYNC_TABS&&window.__AMNI_SYNC_TABS({0})",
                                        tabs_json
                                    )));
                                    a1.borrow_mut().push(Act::Nav(url.clone()));
                                    a1.borrow_mut().push(Act::Title(url.clone()));
                                }
                                IpcResponse::TabsUpdated { .. } => {
                                    a1.borrow_mut().push(Act::Js(resp.to_js_call()));
                                    a1.borrow_mut().push(Act::Js(format!(
                                        "window.__AMNI_TAB_SEED={0};window.__AMNI_SYNC_TABS&&window.__AMNI_SYNC_TABS({0})",
                                        tabs_json
                                    )));
                                    if tab_nav {
                                        if let Some(u) = active_url {
                                            let cur = lu1.borrow().clone();
                                            if !urls_equiv(&u, &cur) {
                                                a1.borrow_mut().push(Act::Nav(u.clone()));
                                                a1.borrow_mut().push(Act::Title(u));
                                            }
                                        }
                                    }
                                }
                                IpcResponse::ActiveTheme { data } => {
                                    a1.borrow_mut().push(Act::Js(resp.to_js_call()));
                                    a1.borrow_mut().push(Act::Js(format!(
                                        "window.__AMNI_SYNC_THEME&&window.__AMNI_SYNC_THEME({})",
                                        data
                                    )));
                                }
                                _ => a1.borrow_mut().push(Act::Js(resp.to_js_call())),
                            }
                        } else {
                            drop(s);
                        }
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
                                *loaded_url.borrow_mut() = nav_url.clone();
                                if nav_url.starts_with("http://") || nav_url.starts_with("https://") {
                                    if let Err(e) = webview.load_url(&nav_url) {
                                        error!("Failed to navigate to '{}': {}", nav_url, e);
                                    }
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
fn urls_equiv(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        let t = s.trim().trim_end_matches('/');
        if let Some(rest) = t.strip_prefix("http://amnibrowse.") {
            format!("amnibrowse://{}", rest)
        } else if let Some(rest) = t.strip_prefix("https://amnibrowse.") {
            format!("amnibrowse://{}", rest)
        } else {
            t.to_string()
        }
    };
    norm(a).eq_ignore_ascii_case(&norm(b))
}
#[cfg(feature = "webview")]
fn chrome_init_js(theme: &Theme, tabs_seed: &str) -> String {
    let t0 = serde_json::json!({
        "bg_primary": theme.bg_primary,
        "bg_secondary": theme.bg_secondary,
        "bg_tertiary": theme.bg_tertiary,
        "bg_hover": theme.bg_hover,
        "border": theme.border,
        "text_primary": theme.text_primary,
        "text_secondary": theme.text_secondary,
        "accent": theme.accent,
        "accent_hover": theme.accent_hover,
        "accent_glow": theme.accent_glow,
        "success": theme.success,
        "danger": theme.danger,
        "tab_active": theme.tab_active,
        "tab_inactive": theme.tab_inactive,
        "font_family": theme.font_family,
        "border_radius": theme.border_radius,
    });
    let seed = if tabs_seed.trim().is_empty() { "[]" } else { tabs_seed };
    let tab_h = tokens::TAB_STRIP_H;
    let nav_h = tokens::NAV_H;
    let book_h = tokens::BOOKMARKS_H;
    let chrome_h = tokens::TOTAL_CHROME_H;
    let push_h = tokens::CONTENT_PUSH_H;
    format!(r#"(function(){{
try {{ if (window.self !== window.top) return; }} catch(_) {{ return; }}
if (location.protocol !== 'http:' && location.protocol !== 'https:') return;
if ((location.hostname || '').indexOf('amnibrowse.') === 0) return;
var T = {theme};
var SEED = {seed};
if (!window.__AMNI_TAB_SEED || !window.__AMNI_TAB_SEED.length) window.__AMNI_TAB_SEED = SEED;
function ipc(o){{ try {{ window.ipc && window.ipc.postMessage(JSON.stringify(o)); }} catch(_) {{}} }}
function applyTheme(th){{
  if (!th || typeof th !== 'object') return;
  T = Object.assign({{}}, T, th);
  var host = document.getElementById('__atb_host');
  if (!host || !host.shadowRoot) return;
  var root = host.shadowRoot;
  var st = root.getElementById('_ath_css');
  if (st) st.textContent = chromeCss();
  host.style.setProperty('--bg-primary', T.bg_primary || '#0a0e14');
  host.style.setProperty('--bg-secondary', T.bg_secondary || '#0f1419');
  host.style.setProperty('--bg-tertiary', T.bg_tertiary || '#1a1f2e');
  host.style.setProperty('--bg-hover', T.bg_hover || T.bg_tertiary || '#1a1f2e');
  host.style.setProperty('--border', T.border || '#1a2332');
  host.style.setProperty('--text-primary', T.text_primary || '#e0e6f0');
  host.style.setProperty('--text-secondary', T.text_secondary || '#6b7d99');
  host.style.setProperty('--accent', T.accent || '#00d4ff');
  host.style.setProperty('--accent-hover', T.accent_hover || '#33ddff');
  host.style.setProperty('--accent-glow', T.accent_glow || 'rgba(0,212,255,0.15)');
  host.style.setProperty('--success', T.success || '#2ed573');
  host.style.setProperty('--danger', T.danger || '#ff4757');
  host.style.setProperty('--tab-active', T.tab_active || T.bg_primary || '#0a0e14');
  host.style.setProperty('--tab-inactive', T.tab_inactive || T.bg_secondary || '#0f1419');
  host.style.setProperty('--font-family', T.font_family || 'system-ui,sans-serif');
  host.style.setProperty('--radius', T.border_radius || '8px');
}}
function chromeCss(){{
  return ':host{{all:initial;position:fixed;top:0;left:0;right:0;height:{chrome_h}px;z-index:2147483647;font-family:var(--font-family,system-ui,sans-serif);color:var(--text-primary,#e0e6f0)}}'
    + '*{{box-sizing:border-box}}'
    + '#__atb{{position:fixed;top:0;left:0;right:0;height:{chrome_h}px;display:flex;flex-direction:column;background:var(--bg-secondary,#0f1419);z-index:2147483647;box-shadow:0 2px 16px rgba(0,0,0,0.55);border-bottom:1px solid var(--border,#1a2332)}}'
    + '#_tabs{{height:{tab_h}px;display:flex;align-items:center;gap:2px;padding:0 8px;overflow-x:auto;overflow-y:hidden;scrollbar-width:thin;background:var(--bg-secondary,#0f1419);border-bottom:1px solid var(--border,#1a2332)}}'
    + '#_tabs::-webkit-scrollbar{{height:3px}}'
    + '.tab{{flex:0 1 160px;min-width:80px;max-width:200px;height:auto;display:flex;align-items:center;gap:6px;padding:6px 12px;margin:0;border:1px solid transparent;border-bottom:none;border-radius:var(--radius,8px) var(--radius,8px) 0 0;background:var(--tab-inactive,var(--bg-secondary));color:var(--text-secondary);font-size:12px;cursor:pointer;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}'
    + '.tab:hover{{background:var(--bg-hover,var(--bg-tertiary,#1a1f2e));color:var(--text-primary)}}'
    + '.tab.active{{background:var(--tab-active,var(--bg-primary));color:var(--text-primary);border-color:var(--border);border-bottom:2px solid var(--accent)}}'
    + '.tab .ttl{{flex:1;overflow:hidden;text-overflow:ellipsis}}'
    + '.tab .x{{flex:0 0 16px;width:16px;height:16px;display:inline-flex;align-items:center;justify-content:center;border-radius:50%;opacity:0.75;font-size:14px;padding:0;border:none;background:transparent;color:var(--text-secondary);cursor:pointer;line-height:1}}'
    + '.tab .x:hover{{opacity:1;background:var(--danger,#ff4757);color:#fff}}'
    + '#_newtab{{flex:0 0 28px;width:28px;height:28px;border:none;background:transparent;color:var(--text-secondary);font-size:18px;cursor:pointer;border-radius:var(--radius,8px);margin-left:4px}}'
    + '#_newtab:hover{{background:var(--bg-hover,var(--bg-tertiary,#1a1f2e));color:var(--accent)}}'
    + '#_nav{{position:relative;height:{nav_h}px;display:flex;align-items:center;padding:5px 10px;gap:4px;background:var(--bg-secondary,#0f1419)}}'
    + '#_bookmarks{{height:{book_h}px;display:flex;align-items:center;gap:4px;padding:3px 12px;background:var(--bg-secondary,#0f1419);border-bottom:1px solid var(--border,#1a2332);overflow-x:auto;font-size:12px}}'
    + '.bmk{{padding:3px 10px;border-radius:6px;background:transparent;border:none;color:var(--text-secondary);font-size:11px;cursor:pointer;white-space:nowrap}}'
    + '.bmk:hover{{background:var(--bg-hover,var(--bg-tertiary,#1a1f2e));color:var(--text-primary)}}'
    + 'button.nav{{background:none;border:none;color:var(--text-secondary);cursor:pointer;width:32px;height:32px;padding:0;font-size:15px;border-radius:var(--radius,8px);line-height:1;display:inline-flex;align-items:center;justify-content:center}}'
    + 'button.nav:hover{{background:var(--bg-hover,var(--bg-tertiary,#1a1f2e));color:var(--text-primary)}}'
    + 'input{{flex:1;height:32px;background:var(--bg-primary,#0a0e14);border:1px solid var(--border);color:var(--text-primary);padding:0 16px;border-radius:20px;font-size:13px;outline:none;min-width:0}}'
    + 'input:focus{{border-color:var(--accent);box-shadow:0 0 0 2px var(--accent-glow)}}'
    + '.ab{{font-size:10px;background:var(--success);color:#000;padding:1px 6px;border-radius:8px;font-weight:700;margin-left:2px}}'
    + '.logo{{color:var(--accent);font-weight:800;font-size:13px;letter-spacing:-0.5px;margin:0 4px}}'
    + '#atb-menu-dropdown{{display:none;position:absolute;right:6px;top:40px;min-width:188px;flex-direction:column;padding:4px;background:var(--bg-primary,var(--bg-secondary,#0a0e14));border:1px solid var(--border,#1a2332);border-radius:var(--radius,8px);box-shadow:0 8px 28px rgba(0,0,0,0.5);z-index:20}}'
    + '#atb-menu-dropdown.open{{display:flex}}'
    + '#atb-menu-dropdown .mi{{background:none;border:none;color:var(--text-primary,#e0e6f0);text-align:left;padding:9px 12px;border-radius:6px;cursor:pointer;font:13px var(--font-family,system-ui,sans-serif)}}'
    + '#atb-menu-dropdown .mi:hover{{background:var(--accent-glow);color:var(--accent)}}';
}}
function tabLabel(t){{
  var title = (t && t.title) ? String(t.title) : '';
  if (title && title !== 'New Tab' && title !== 'Private Tab') return title;
  try {{ var u = new URL(t.url || ''); return u.hostname || t.url || 'Tab'; }} catch(_) {{ return (t && t.url) || 'Tab'; }}
}}
function paintTabs(list){{
  var host = document.getElementById('__atb_host');
  var root = host && host.shadowRoot;
  if (!root) return;
  var strip = root.getElementById('_tabs');
  if (!strip) return;
  var tabs = Array.isArray(list) ? list : (typeof list === 'string' ? (function(){{try{{return JSON.parse(list)}}catch(_){{return[]}}}})() : []);
  if (tabs && tabs.length) window.__AMNI_TAB_SEED = tabs;
  strip.innerHTML = '';
  tabs.forEach(function(t){{
    var el = document.createElement('button');
    el.type = 'button';
    el.className = 'tab' + (t.is_active ? ' active' : '');
    el.title = t.url || '';
    el.setAttribute('data-id', t.id || '');
    var ttl = document.createElement('span');
    ttl.className = 'ttl';
    ttl.textContent = tabLabel(t);
    var x = document.createElement('span');
    x.className = 'x';
    x.textContent = '×';
    x.title = 'Close';
    x.onclick = function(e){{ e.stopPropagation(); if (t.id) ipc({{ type:'close_tab', id:t.id }}); }};
    el.appendChild(ttl);
    el.appendChild(x);
    el.onclick = function(){{ if (t.id) ipc({{ type:'switch_tab', id:t.id }}); }};
    strip.appendChild(el);
    if (t.is_active) try {{ el.scrollIntoView({{ inline:'nearest', block:'nearest' }}); }} catch(_){{}}
  }});
  var plus = document.createElement('button');
  plus.type = 'button';
  plus.id = '_newtab';
  plus.title = 'New tab';
  plus.textContent = '+';
  plus.onclick = function(){{ ipc({{ type:'new_tab', url:'amnibrowse://newtab' }}); }};
  strip.appendChild(plus);
}}
function paintBookmarks(list){{
  var host = document.getElementById('__atb_host');
  var root = host && host.shadowRoot;
  if (!root) return;
  var strip = root.getElementById('_bookmarks');
  if (!strip) return;
  var bmks = Array.isArray(list) ? list : (typeof list === 'string' ? (function(){{try{{return JSON.parse(list)}}catch(_){{return[]}}}})() : []);
  strip.innerHTML = '';
  bmks.forEach(function(b){{
    var el = document.createElement('button');
    el.type = 'button';
    el.className = 'bmk';
    el.title = b.url || '';
    el.textContent = b.title || b.url || 'Bookmark';
    el.onclick = function(){{ ipc({{ type:'navigate', url:b.url }}); }};
    strip.appendChild(el);
  }});
}}
function wireHandlers(host){{
  var root = host && host.shadowRoot;
  if (!root) return;
  var q = function(id){{ return root.getElementById(id); }};
  var u = q('_au');
  if (u) {{
    u.value = location.href;
    u.onkeydown = function(e){{
      if (e.key !== 'Enter') return;
      var v = (this.value || '').trim();
      if (!v) return;
      var msg = /^https?:\/\//.test(v) ? {{ type:'navigate', url:v }} : (v.indexOf('.') > -1 && v.indexOf(' ') < 0 ? {{ type:'navigate', url:'https://' + v }} : {{ type:'search', query:v }});
      ipc(msg);
    }};
  }}
  var bind = function(id, fn){{ var el = q(id); if (el) el.onclick = fn; }};
  bind('_ab', function(){{ ipc({{ type:'back' }}); }});
  bind('_af', function(){{ ipc({{ type:'forward' }}); }});
  bind('_ar', function(){{ ipc({{ type:'refresh' }}); }});
  bind('_ah', function(){{ ipc({{ type:'navigate', url:'amnibrowse://newtab' }}); }});
  bind('_abk', function(){{ ipc({{ type:'bookmark_add', title:document.title || location.href, url:location.href }}); }});
  var menuBtn = q('atb-menu-btn');
  var menu = q('atb-menu-dropdown');
  var setMenu = function(open){{
    if (!menu || !menuBtn) return;
    menu.classList.toggle('open', !!open);
    menuBtn.setAttribute('aria-expanded', open ? 'true' : 'false');
  }};
  if (menuBtn && menu) {{
    menuBtn.onclick = function(e){{ e.stopPropagation(); setMenu(!menu.classList.contains('open')); }};
    menu.querySelectorAll('.mi').forEach(function(btn){{
      btn.onclick = function(e){{
        e.stopPropagation();
        var act = btn.getAttribute('data-act') || '';
        setMenu(false);
        act === 'apps' ? ipc({{ type:'navigate', url:'https://amni-scient.com' }})
          : act === 'themes' ? ipc({{ type:'navigate', url:'amnibrowse://newtab' }})
          : act === 'history' ? ipc({{ type:'navigate', url:'amnibrowse://newtab' }})
          : act === 'downloads' ? ipc({{ type:'navigate', url:'amnibrowse://newtab' }})
          : null;
      }};
    }});
  }}
  if (!host.__amniMenuWired) {{
    host.__amniMenuWired = true;
    host.addEventListener('click', function(e){{
      if (!menu || !menu.classList.contains('open')) return;
      var t = e.target;
      if ((menuBtn && (t === menuBtn || menuBtn.contains(t))) || (menu && (t === menu || menu.contains(t)))) return;
      setMenu(false);
    }});
    window.addEventListener('keydown', function(e){{
      if (!((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K'))) return;
      e.preventDefault();
      if (menu) setMenu(!menu.classList.contains('open'));
    }}, true);
  }}
}}
function ensureToolbar(){{
  try {{
    var d = document;
    if (!d.documentElement || !d.head || !d.body) return false;
    var host = d.getElementById('__atb_host');
    if (!host || !host.shadowRoot) {{
      host = d.createElement('div');
      host.id = '__atb_host';
      host.style.cssText = 'position:fixed;top:0;left:0;right:0;height:{chrome_h}px;z-index:2147483647;pointer-events:auto;';
      var root = host.attachShadow({{ mode:'open' }});
      var style = d.createElement('style');
      style.id = '_ath_css';
      style.textContent = chromeCss();
      root.appendChild(style);
      var bar = d.createElement('div');
      bar.id = '__atb';
      bar.innerHTML = '<div id="_tabs"></div>'
        + '<div id="_nav">'
        + '<span class="logo">A</span>'
        + '<button class="nav" title="Back" id="_ab">◀</button>'
        + '<button class="nav" title="Forward" id="_af">▶</button>'
        + '<button class="nav" title="Reload" id="_ar">⟳</button>'
        + '<button class="nav" title="Home" id="_ah">⌂</button>'
        + '<input id="_au" value="" placeholder="Search or enter URL…"/>'
        + '<button class="nav" title="Bookmark" id="_abk">★</button>'
        + '<button class="nav" title="Menu" id="atb-menu-btn" aria-haspopup="true" aria-expanded="false">☰</button>'
        + '<span class="ab" id="_as" title="Ads blocked">🛡</span>'
        + '<div id="atb-menu-dropdown" role="menu">'
        + '<button type="button" class="mi" data-act="apps" role="menuitem">Amni Apps</button>'
        + '<button type="button" class="mi" data-act="themes" role="menuitem">Themes</button>'
        + '<button type="button" class="mi" data-act="history" role="menuitem">History</button>'
        + '<button type="button" class="mi" data-act="downloads" role="menuitem">Downloads</button>'
        + '</div>'
        + '</div>'
        + '<div id="_bookmarks"></div>';
      root.appendChild(bar);
      d.body.prepend(host);
      applyTheme(T);
      var seed = (window.__AMNI_TAB_SEED && window.__AMNI_TAB_SEED.length) ? window.__AMNI_TAB_SEED : SEED;
      paintTabs(Array.isArray(seed) && seed.length ? seed : SEED);
    }}
    if (!d.getElementById('__amni_push_style')) {{
      var s = d.createElement('style');
      s.id = '__amni_push_style';
      s.textContent = 'html{{margin-top:{push_h}px!important}}';
      d.head.appendChild(s);
    }}
    wireHandlers(host);
    ipc({{ type:'get_tabs' }});
    ipc({{ type:'get_stats' }});
    ipc({{ type:'theme_get_active' }});
    ipc({{ type:'bookmark_list' }});
    return true;
  }} catch(_) {{ return false; }}
}}
window.__AMNI_SYNC_THEME = function(th){{
  try {{
    var obj = (typeof th === 'string') ? JSON.parse(th) : th;
    applyTheme(obj);
  }} catch(_) {{}}
}};
window.__AMNI_SYNC_TABS = function(list){{
  try {{ paintTabs(list); }} catch(_) {{}}
}};
window.__amni_receive = function(msg){{
  if (!msg) return;
  var host = document.getElementById('__atb_host');
  var root = host && host.shadowRoot;
  if (!root) return;
  var u = root.getElementById('_au');
  var sh = root.getElementById('_as');
  if (msg.type === 'stats' && sh) sh.textContent = '🛡 ' + msg.ads_blocked;
  if (msg.type === 'navigate_to' && u) u.value = msg.url;
  if (msg.type === 'tabs_updated') {{
    var raw = msg.tabs;
    var list = Array.isArray(raw) ? raw : (typeof raw === 'string' ? (function(){{try{{return JSON.parse(raw)}}catch(_){{return[]}}}})() : []);
    paintTabs(list);
  }}
  if (msg.type === 'bookmarks') {{
    paintBookmarks(msg.data);
  }}
  if (msg.type === 'active_theme' && msg.data) {{
    try {{
      var th = (typeof msg.data === 'string') ? JSON.parse(msg.data) : msg.data;
      applyTheme(th);
    }} catch(_) {{}}
  }}
}};
function start(){{
  if (!ensureToolbar()) {{
    var tries = 0;
    var tid = setInterval(function(){{ tries++; if (ensureToolbar() || tries > 80) clearInterval(tid); }}, 50);
  }}
  var observer = new MutationObserver(function(){{ ensureToolbar(); }});
  observer.observe(document.documentElement || document, {{ childList:true, subtree:true }});
  window.addEventListener('pageshow', function(){{ ensureToolbar(); }});
}}
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, {{ once:true }});
else start();
}})();"#, theme = t0, seed = seed, tab_h = tab_h, nav_h = nav_h, book_h = book_h, chrome_h = chrome_h, push_h = push_h)
}
