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
        #[cfg(target_os = "windows")]
        {
            let args = "--disable-features=msEdgeSmartScreen,InterestGroupStorage,BrowsingTopics --disable-background-networking --disable-sync --disable-breakpad --no-default-browser-check --no-first-run";
            std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);
            if let Some(dir) = dirs::config_dir() {
                let ud = dir.join("amni-browse").join("webview2-data");
                std::fs::create_dir_all(&ud).ok();
                std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", ud);
            }
        }
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
            .with_title(format!("{} v{} — Local Profile", APP_NAME, APP_VERSION))
            .with_decorations(true)
            .with_inner_size(tao::dpi::LogicalSize::new(1400.0, 900.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(640.0, 400.0))
            .build(&event_loop).expect("window");
        let boot_theme = state.borrow().themes.active_theme();
        let boot_tabs = state.borrow().tabs.to_json();
        let last_tabs = Rc::new(RefCell::new(boot_tabs.clone()));
        let last_theme = Rc::new(RefCell::new(state.borrow().themes.active_theme_json()));
        let s1 = Rc::clone(&state);
        let a1 = Rc::clone(&acts);
        let lu1 = Rc::clone(&loaded_url);
        let lt1 = Rc::clone(&last_tabs);
        let lth1 = Rc::clone(&last_theme);
        let px1 = proxy.clone();
        let s_proto = Rc::clone(&state);
        let a_load = Rc::clone(&acts);
        let lt_load = Rc::clone(&last_tabs);
        let lth_load = Rc::clone(&last_theme);
        let px_load = proxy.clone();
        let webview = WebViewBuilder::new()
            .with_custom_protocol("amnibrowse".to_string(), move |_, request| {
                let theme = s_proto.borrow().themes.active_theme();
                let uri = request.uri().to_string().to_ascii_lowercase();
                let html = if uri.contains("developer") || uri.contains("dev") {
                    crate::ui::developer::developer_html(&theme)
                } else {
                    spa::browser_html(&theme)
                }.into_bytes();
                wry::http::Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .header("Pragma", "no-cache")
                    .header("Expires", "0")
                    .body(Cow::Owned(html))
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
                let th = lth_load.borrow().clone();
                let tabs = if t.is_empty() { "[]".to_string() } else { t };
                let theme = if th.is_empty() { "{}".to_string() } else { th };
                let safety = crate::engine::page_safety::assess_json(&url);
                a_load.borrow_mut().push(Act::Js(format!(
                    "(function(){{try{{window.__AMNI_ENSURE&&window.__AMNI_ENSURE();var tabs={0};window.__AMNI_TAB_SEED=tabs;window.__AMNI_SYNC_TABS&&window.__AMNI_SYNC_TABS(tabs);var th={1};window.__AMNI_SYNC_THEME&&window.__AMNI_SYNC_THEME(th);window.__AMNI_SAFETY&&window.__AMNI_SAFETY({2});try{{var tt=document.title||location.hostname||location.href;if(tt&&window.ipc)window.ipc.postMessage(JSON.stringify({{type:'update_title',title:String(tt).slice(0,80)}}));}}catch(_t){{}}window.__AMNI_ENSURE&&window.__AMNI_ENSURE();}}catch(_){{}}}})()",
                    tabs, theme, safety
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
                            let theme_json = s.themes.active_theme_json();
                            *lth1.borrow_mut() = theme_json.clone();
                            let active_url = s.tabs.active_tab().map(|t| t.url.clone());
                            drop(s);
                            match &resp {
                                IpcResponse::NavigateTo { url } => {
                                    a1.borrow_mut().push(Act::Js(resp.to_js_call()));
                                    a1.borrow_mut().push(Act::Js(format!(
                                        "window.__AMNI_TAB_SEED={0};window.__AMNI_SYNC_TABS&&window.__AMNI_SYNC_TABS({0});window.__AMNI_SYNC_THEME&&window.__AMNI_SYNC_THEME({1})",
                                        tabs_json, theme_json
                                    )));
                                    a1.borrow_mut().push(Act::Nav(url.clone()));
                                    a1.borrow_mut().push(Act::Title(url.clone()));
                                }
                                IpcResponse::TabsUpdated { .. } => {
                                    a1.borrow_mut().push(Act::Js(resp.to_js_call()));
                                    a1.borrow_mut().push(Act::Js(format!(
                                        "window.__AMNI_TAB_SEED={0};window.__AMNI_SYNC_TABS&&window.__AMNI_SYNC_TABS({0});window.__AMNI_SYNC_THEME&&window.__AMNI_SYNC_THEME({1})",
                                        tabs_json, theme_json
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
                                    *lth1.borrow_mut() = data.clone();
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
                                    let (base, frag) = match rest.find('#') { Some(i) => (&rest[..i], &rest[i..]), None => (rest, "") };
                                    let (host, path) = match base.find('/') { Some(i) => (&base[..i], &base[i..]), None => (base, "/") };
                                    let target = format!("http://amnibrowse.{}{}{}", host, path, frag);
                                    if let Err(e) = webview.load_url(&target) {
                                        error!("Failed to load internal page '{}' (from '{}'): {}", target, raw, e);
                                    }
                                }
                                if nav_url.starts_with("amnibrowse://") {
                                    window.set_title(&format!("{} v{} — Local Profile", APP_NAME, APP_VERSION));
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
                Event::WindowEvent { event: WindowEvent::Focused(true) | WindowEvent::Resized(_), .. } => {
                    let th = last_theme.borrow().clone();
                    let tabs = last_tabs.borrow().clone();
                    let th_js = if th.is_empty() || th == "{}" { "null".into() } else { th };
                    let tabs_js = if tabs.is_empty() { "[]".into() } else { tabs };
                    webview.evaluate_script(&format!(
                        "try{{window.__AMNI_ENSURE&&window.__AMNI_ENSURE();window.__AMNI_SYNC_TABS&&window.__AMNI_SYNC_TABS({tabs});if({th})window.__AMNI_SYNC_THEME&&window.__AMNI_SYNC_THEME({th})}}catch(_){{}}",
                        tabs = tabs_js, th = th_js
                    )).ok();
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
        "warning": theme.warning,
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
function c(v, fb){{
  v = (v == null ? '' : String(v)).trim();
  if (!v || v === 'transparent' || v === 'inherit' || v === 'initial' || v === 'currentColor') return fb;
  return v;
}}
function palette(){{
  return {{
    bg: c(T.bg_secondary, '#0D0F12'),
    bg2: c(T.bg_primary, '#08090B'),
    bg3: c(T.bg_tertiary || T.bg_hover, '#111418'),
    border: c(T.border, '#20242B'),
    text: c(T.text_primary, '#EDEFF2'),
    muted: c(T.text_secondary, '#A7ADB6'),
    accent: c(T.accent, '#C89B4E'),
    accentH: c(T.accent_hover, '#E2BC7C'),
    glow: c(T.accent_glow, 'rgba(200,155,78,0.18)'),
    ok: c(T.success, '#4ADE80'),
    danger: c(T.danger, '#FF6B6B'),
    warn: c(T.warning, '#E8B04B'),
    tabA: c(T.tab_active || T.bg_primary, '#111418'),
    tabI: c(T.tab_inactive || T.bg_secondary, '#0D0F12'),
    font: c(T.font_family, 'Segoe UI Variable Display,Segoe UI,system-ui,sans-serif'),
    radius: c(T.border_radius, '4px')
  }};
}}
function applyTheme(th){{
  if (th && typeof th === 'object') T = Object.assign({{}}, T, th);
  var root = (document.getElementById('__atb_host') && (document.getElementById('__atb_host').shadowRoot || window.__AMNI_SR)) || window.__AMNI_SR;
  if (!root) return;
  var st = root.getElementById('_ath_css');
  if (st) st.textContent = chromeCss();
}}
function chromeCss(){{
  var p = palette();
  return ':host{{all:initial !important;position:fixed !important;top:0 !important;left:0 !important;right:0 !important;width:100% !important;height:{chrome_h}px !important;z-index:2147483647 !important;font-family:' + p.font + ' !important;color:' + p.text + ' !important;background:' + p.bg + ' !important;pointer-events:auto !important}}'
    + '*{{box-sizing:border-box !important;color:inherit}}'
    + '#__atb{{position:fixed !important;top:0 !important;left:0 !important;right:0 !important;height:{chrome_h}px !important;display:flex !important;flex-direction:column !important;background:' + p.bg + ' !important;color:' + p.text + ' !important;z-index:2147483647 !important;box-shadow:0 2px 16px rgba(0,0,0,0.55) !important;border-bottom:1px solid ' + p.border + ' !important}}'
    + '#_tabs{{height:{tab_h}px !important;display:flex !important;align-items:center !important;gap:4px !important;padding:0 8px !important;overflow-x:auto !important;overflow-y:hidden !important;scrollbar-width:thin !important;flex-wrap:nowrap !important;background:' + p.bg + ' !important;color:' + p.text + ' !important;border-bottom:1px solid ' + p.border + ' !important;pointer-events:auto !important}}'
    + '.grp{{flex:0 0 auto !important;max-width:72px !important;font-size:10px !important;font-weight:700 !important;letter-spacing:.4px !important;text-transform:uppercase !important;color:' + p.accent + ' !important;padding:0 6px !important;border-left:2px solid ' + p.accent + ' !important;margin-left:4px !important;opacity:.95 !important;white-space:nowrap !important;overflow:hidden !important;text-overflow:ellipsis !important;user-select:none !important}}'
    + '#_tabs::-webkit-scrollbar{{height:3px}}'
    + '.tab{{flex:0 0 148px !important;flex-grow:0 !important;flex-shrink:0 !important;width:148px !important;min-width:148px !important;max-width:148px !important;height:30px !important;display:flex !important;align-items:center !important;gap:6px !important;padding:0 8px !important;margin:0 !important;border:1px solid transparent !important;border-bottom:none !important;border-radius:' + p.radius + ' ' + p.radius + ' 0 0 !important;background:' + p.tabI + ' !important;color:' + p.muted + ' !important;font-size:12px !important;font-family:' + p.font + ' !important;cursor:pointer !important;white-space:nowrap !important;overflow:hidden !important;text-overflow:ellipsis !important;pointer-events:auto !important;touch-action:manipulation !important}}'
    + '.tab:hover{{background:' + p.bg3 + ' !important;color:' + p.text + ' !important}}'
    + '.tab.active{{background:' + p.tabA + ' !important;color:' + p.text + ' !important;border-color:' + p.border + ' !important;box-shadow:0 2px 0 ' + p.accent + ' inset,0 -1px 4px rgba(0,0,0,0.18) !important;z-index:2 !important}}'
    + '.tab:focus-visible,.tab.kbd-focus{{outline:2px solid ' + p.accent + ' !important;outline-offset:-2px !important;z-index:3 !important;box-shadow:0 0 0 1px ' + p.glow + ' !important}}'
    + '.tab.priv{{box-shadow:inset 0 0 0 1px ' + p.accent + '33 !important}}'
    + '.tab .ttl{{flex:1 1 auto !important;min-width:0 !important;overflow:hidden !important;text-overflow:ellipsis !important;white-space:nowrap !important;color:inherit !important;pointer-events:none !important}}'
    + '.tab .pb{{flex:0 0 auto !important;font-size:9px !important;font-weight:700 !important;letter-spacing:.2px !important;padding:1px 4px !important;border-radius:3px !important;background:' + p.accent + ' !important;color:' + p.bg2 + ' !important;line-height:1.2 !important;pointer-events:none !important}}'
    + '.tab .x{{flex:0 0 18px !important;width:18px !important;min-width:18px !important;height:18px !important;display:inline-flex !important;align-items:center !important;justify-content:center !important;border-radius:50% !important;opacity:0.9 !important;font-size:14px !important;padding:0 !important;border:none !important;background:transparent !important;color:' + p.muted + ' !important;cursor:pointer !important;line-height:1 !important;pointer-events:auto !important}}'
    + '.tab .x:hover{{opacity:1 !important;background:' + p.danger + ' !important;color:color-mix(in srgb,' + p.danger + ' 12%,#fff) !important}}'
    + '#_newtab{{flex:0 0 28px !important;width:28px !important;min-width:28px !important;height:28px !important;border:none !important;background:transparent !important;color:' + p.muted + ' !important;font-size:18px !important;cursor:pointer !important;border-radius:' + p.radius + ' !important;margin-left:4px !important;font-family:' + p.font + ' !important;pointer-events:auto !important}}'
    + '#_newtab:hover{{background:' + p.bg3 + ' !important;color:' + p.accent + ' !important}}'
    + '#_nav{{position:relative !important;height:{nav_h}px !important;display:flex !important;align-items:center !important;padding:5px 10px !important;gap:4px !important;background:' + p.bg + ' !important;color:' + p.text + ' !important}}'
    + '#_bookmarks{{height:{book_h}px !important;display:flex !important;align-items:center !important;gap:4px !important;padding:3px 12px !important;background:' + p.bg + ' !important;color:' + p.muted + ' !important;border-bottom:1px solid ' + p.border + ' !important;overflow-x:auto !important;font-size:12px !important}}'
    + '.bmk{{padding:3px 10px !important;border-radius:' + p.radius + ' !important;background:transparent !important;border:none !important;color:' + p.muted + ' !important;font-size:11px !important;cursor:pointer !important;white-space:nowrap !important;font-family:' + p.font + ' !important}}'
    + '.bmk:hover{{background:' + p.bg3 + ' !important;color:' + p.text + ' !important}}'
    + 'button.nav{{background:transparent !important;border:none !important;color:' + p.muted + ' !important;cursor:pointer !important;width:32px !important;height:32px !important;padding:0 !important;font-size:15px !important;border-radius:' + p.radius + ' !important;line-height:1 !important;display:inline-flex !important;align-items:center !important;justify-content:center !important;font-family:' + p.font + ' !important}}'
    + 'button.nav:hover{{background:' + p.bg3 + ' !important;color:' + p.text + ' !important}}'
    + 'input{{flex:1 !important;height:32px !important;background:' + p.bg2 + ' !important;border:1px solid ' + p.border + ' !important;color:' + p.text + ' !important;caret-color:' + p.accent + ' !important;padding:0 16px !important;border-radius:20px !important;font-size:13px !important;outline:none !important;min-width:0 !important;font-family:' + p.font + ' !important}}'
    + 'input::placeholder{{color:' + p.muted + ' !important;opacity:1 !important}}'
    + 'input:focus{{border-color:' + p.accent + ' !important;box-shadow:0 0 0 3px ' + p.glow + ' !important}}'
    + '.ab{{font-size:10px !important;background:' + p.ok + ' !important;color:color-mix(in srgb,' + p.ok + ' 18%,#000) !important;padding:1px 6px !important;border-radius:8px !important;font-weight:700 !important;margin-left:2px !important}}'
    + '.secchip{{font-size:10px !important;font-weight:700 !important;padding:2px 8px !important;border-radius:999px !important;margin-left:4px !important;letter-spacing:.3px !important;cursor:pointer !important;border:1px solid ' + p.border + ' !important;background:' + p.bg3 + ' !important;color:' + p.text + ' !important}}'
    + '.secchip.safe{{background:color-mix(in srgb,' + p.ok + ' 22%,' + p.bg + ') !important;color:' + p.ok + ' !important;border-color:color-mix(in srgb,' + p.ok + ' 42%,' + p.border + ') !important}}'
    + '.secchip.low{{background:color-mix(in srgb,' + p.warn + ' 22%,' + p.bg + ') !important;color:' + p.warn + ' !important;border-color:color-mix(in srgb,' + p.warn + ' 42%,' + p.border + ') !important}}'
    + '.secchip.medium{{background:color-mix(in srgb,color-mix(in srgb,' + p.warn + ' 50%,' + p.danger + ') 28%,' + p.bg + ') !important;color:color-mix(in srgb,' + p.warn + ' 55%,' + p.danger + ') !important;border-color:color-mix(in srgb,' + p.danger + ' 35%,' + p.border + ') !important}}'
    + '.secchip.high,.secchip.critical{{background:color-mix(in srgb,' + p.danger + ' 22%,' + p.bg + ') !important;color:' + p.danger + ' !important;border-color:color-mix(in srgb,' + p.danger + ' 42%,' + p.border + ') !important}}'
    + '#_safebar{{display:none;padding:6px 12px;font-size:12px;background:color-mix(in srgb,' + p.danger + ' 18%,' + p.bg + ');color:color-mix(in srgb,' + p.danger + ' 55%,' + p.text + ');border-bottom:1px solid color-mix(in srgb,' + p.danger + ' 45%,' + p.border + ')}}'
    + '#_safebar.show{{display:block}}'
    + '#_safebar button{{margin-left:10px;background:' + p.danger + ';color:color-mix(in srgb,' + p.danger + ' 12%,#fff);border:none;border-radius:6px;padding:3px 8px;cursor:pointer;font:600 11px ' + p.font + '}}'
    + '#atb-menu-dropdown{{display:none !important;position:absolute !important;right:6px !important;top:40px !important;min-width:188px !important;flex-direction:column !important;padding:4px !important;background:' + p.bg2 + ' !important;color:' + p.text + ' !important;border:1px solid ' + p.border + ' !important;border-radius:' + p.radius + ' !important;box-shadow:0 8px 28px rgba(0,0,0,0.55) !important;z-index:20 !important}}'
    + '#atb-menu-dropdown.open{{display:flex !important}}'
    + '#atb-menu-dropdown .mi{{background:transparent !important;border:none !important;color:' + p.text + ' !important;text-align:left !important;padding:9px 12px !important;border-radius:6px !important;cursor:pointer !important;font:13px ' + p.font + ' !important}}'
    + '#atb-menu-dropdown .mi:hover{{background:' + p.glow + ' !important;color:' + p.accent + ' !important}}';
}}
function tabLabel(t){{
  var title = (t && t.title) ? String(t.title) : '';
  var url = (t && t.url) ? String(t.url) : '';
  if (!title || title === 'New Tab' || title === 'Private Tab' || title === 'Private') {{
    if (url.indexOf('amnibrowse://developer') === 0) title = 'Developer';
    else if (url.indexOf('amnibrowse://') === 0) title = 'Home';
    else {{
      try {{ title = new URL(url).hostname.replace(/^www\\./,'') || url; }} catch(_) {{ title = url || 'Tab'; }}
    }}
  }}
  title = title.replace(/\\s+/g, ' ').trim();
  var chars = Array.from(title);
  if (chars.length > 22) title = chars.slice(0, 20).join('') + '…';
  return title || 'Tab';
}}
function groupLabel(g){{
  g = String(g || '').replace(/\\s+/g, ' ').trim();
  var chars = Array.from(g);
  if (chars.length > 12) g = chars.slice(0, 10).join('') + '…';
  return g;
}}
function orderedTabs(tabs){{
  return (Array.isArray(tabs) ? tabs : []).map(function(t,i){{ return {{ t:t, i:i }}; }}).sort(function(a,b){{
    var ga = (a.t && a.t.panel_group) ? String(a.t.panel_group) : '~~~~';
    var gb = (b.t && b.t.panel_group) ? String(b.t.panel_group) : '~~~~';
    if (ga < gb) return -1; if (ga > gb) return 1; return a.i - b.i;
  }}).map(function(x){{ return x.t; }});
}}
function markKbdTab(id){{
  window.__AMNI_KBD_TAB = id || null;
  if (window.__AMNI_KBD_TAB_T) clearTimeout(window.__AMNI_KBD_TAB_T);
  window.__AMNI_KBD_TAB_T = setTimeout(function(){{
    window.__AMNI_KBD_TAB = null;
    var root = amniRoot();
    if (!root) return;
    root.querySelectorAll('.tab.kbd-focus').forEach(function(e){{ e.classList.remove('kbd-focus'); }});
  }}, 900);
}}
function amniRoot(){{
  var host = document.getElementById('__atb_host');
  return (host && (host.shadowRoot || window.__AMNI_SR)) || window.__AMNI_SR || null;
}}
function clearKids(el){{
  if (!el) return;
  while (el.firstChild) el.removeChild(el.firstChild);
}}
function el(tag, props, kids){{
  var n = document.createElement(tag);
  if (props) {{
    Object.keys(props).forEach(function(k){{
      var v = props[k];
      if (k === 'className') n.className = v;
      else if (k === 'text') n.textContent = v;
      else if (k === 'html') n.textContent = v;
      else if (k.indexOf('on') === 0 && typeof v === 'function') n[k] = v;
      else if (k === 'style' && typeof v === 'string') n.setAttribute('style', v);
      else if (v === true) n.setAttribute(k, '');
      else if (v !== false && v != null) n.setAttribute(k, String(v));
    }});
  }}
  if (kids) kids.forEach(function(c){{ if (c) n.appendChild(c); }});
  return n;
}}
function stopHit(e){{
  try {{ e.preventDefault(); e.stopPropagation(); if (e.stopImmediatePropagation) e.stopImmediatePropagation(); }} catch(_){{}}
}}
function bindHit(node, fn){{
  if (!node || !fn) return;
  node.onpointerdown = function(e){{
    if (e.button != null && e.button !== 0) return;
    stopHit(e);
    fn(e);
  }};
}}
function tabsFingerprint(tabs){{
  try {{
    return (tabs || []).map(function(t){{ return (t.id||'') + '|' + (t.is_active?1:0) + '|' + (t.is_private?1:0) + '|' + tabLabel(t) + '|' + (t.url||'') + '|' + (t.panel_group||''); }}).join(';;');
  }} catch(_) {{ return String(Math.random()); }}
}}
function paintTabs(list){{
  var root = amniRoot();
  if (!root) return;
  var strip = root.getElementById('_tabs');
  if (!strip) return;
  var raw = Array.isArray(list) ? list : (typeof list === 'string' ? (function(){{try{{return JSON.parse(list)}}catch(_){{return[]}}}})() : []);
  var tabs = orderedTabs(raw);
  if (tabs && tabs.length) window.__AMNI_TAB_SEED = tabs;
  var fp = tabsFingerprint(tabs);
  if (window.__AMNI_TABS_FP === fp && strip.childNodes.length) {{
    var kidEarly = window.__AMNI_KBD_TAB;
    if (kidEarly) {{
      var ke = strip.querySelector('.tab[data-id="' + String(kidEarly).replace(/"/g,'') + '"]');
      if (ke) {{ ke.classList.add('kbd-focus'); }}
    }}
    return;
  }}
  window.__AMNI_TABS_FP = fp;
  clearKids(strip);
  var lastG = null;
  tabs.forEach(function(t){{
    var g = (t && t.panel_group) ? String(t.panel_group) : '';
    var priv = !!(t && t.is_private);
    if (g && g !== lastG) {{
      lastG = g;
      var gl = el('span', {{ className:'grp', text: groupLabel(g) }});
      gl.title = 'Group: ' + g;
      strip.appendChild(gl);
    }} else if (!g) {{
      lastG = null;
    }}
    var kids = [el('span', {{ className:'ttl', text: tabLabel(t) }})];
    if (priv) kids.push(el('span', {{ className:'pb', text:'P', title:'Private' }}));
    var x = el('span', {{ className:'x', role:'button', title:'Close tab', tabIndex:'0', text:'×' }});
    kids.push(x);
    bindHit(x, function(){{ if (t.id) ipc({{ type:'close_tab', id:t.id }}); }});
    var node = el('button', {{ type:'button', className:'tab' + (t.is_active ? ' active' : '') + (priv ? ' priv' : ''), title: (t.title || t.url || 'Tab') + (g ? (' · ' + g) : '') + (priv ? ' · Private' : '') + String.fromCharCode(10) + 'Right-click to set group', 'data-id': t.id || '', tabIndex: t.is_active ? '0' : '-1' }}, kids);
    bindHit(node, function(e){{
      if (e && e.target && (e.target === x || (e.target.closest && e.target.closest('.x')))) return;
      if (t.id) ipc({{ type:'switch_tab', id:t.id }});
    }});
    node.onauxclick = function(e){{
      if (e.button !== 1 || !t.id) return;
      stopHit(e);
      ipc({{ type:'close_tab', id:t.id }});
    }};
    node.oncontextmenu = function(e){{
      stopHit(e);
      if (!t.id) return;
      var name = prompt('Tab group name (empty to clear)', g || '');
      if (name === null) return;
      ipc({{ type:'tab_set_group', id:t.id, group: name.trim() ? name.trim() : null }});
    }};
    strip.appendChild(node);
    if (t.is_active) try {{ node.scrollIntoView({{ inline:'nearest', block:'nearest' }}); }} catch(_){{}}
  }});
  var plus = el('button', {{ type:'button', id:'_newtab', title:'New tab', text:'+' }});
  bindHit(plus, function(){{ ipc({{ type:'new_tab', url:'amnibrowse://newtab' }}); }});
  strip.appendChild(plus);
  strip.ondblclick = function(e){{
    if (e.target === strip) {{ stopHit(e); ipc({{ type:'new_tab', url:'amnibrowse://newtab' }}); }}
  }};
  var kid = window.__AMNI_KBD_TAB;
  if (kid) {{
    var kn = strip.querySelector('.tab[data-id="' + String(kid).replace(/"/g,'') + '"]');
    if (kn) {{ kn.classList.add('kbd-focus'); kn.tabIndex = 0; }}
  }}
}}
function paintBookmarks(list){{
  var root = amniRoot();
  if (!root) return;
  var strip = root.getElementById('_bookmarks');
  if (!strip) return;
  var bmks = Array.isArray(list) ? list : (typeof list === 'string' ? (function(){{try{{return JSON.parse(list)}}catch(_){{return[]}}}})() : []);
  clearKids(strip);
  bmks.forEach(function(b){{
    var node = el('button', {{ type:'button', className:'bmk', title: b.url || '', text: b.title || b.url || 'Bookmark' }});
    node.onclick = function(){{ ipc({{ type:'navigate', url:b.url }}); }};
    strip.appendChild(node);
  }});
}}
function wireHandlers(host){{
  var root = (host && (host.shadowRoot || window.__AMNI_SR)) || amniRoot();
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
          : act === 'developer' ? ipc({{ type:'navigate', url:'amnibrowse://developer' }})
          : act === 'themes' ? ipc({{ type:'navigate', url:'amnibrowse://developer#themes' }})
          : act === 'extensions' ? ipc({{ type:'navigate', url:'amnibrowse://developer#ext' }})
          : act === 'report' ? ipc({{ type:'navigate', url:'amnibrowse://developer#bug' }})
          : act === 'history' ? ipc({{ type:'navigate', url:'amnibrowse://newtab#history' }})
          : act === 'downloads' ? ipc({{ type:'navigate', url:'amnibrowse://newtab#downloads' }})
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
function buildChromeBar(root){{
  var bar = el('div', {{ id:'__atb' }});
  bar.appendChild(el('div', {{ id:'_tabs' }}));
  var nav = el('div', {{ id:'_nav' }});
  nav.appendChild(el('span', {{ className:'logo', text:'A' }}));
  nav.appendChild(el('button', {{ type:'button', className:'nav', title:'Back', id:'_ab', text:'◀' }}));
  nav.appendChild(el('button', {{ type:'button', className:'nav', title:'Forward', id:'_af', text:'▶' }}));
  nav.appendChild(el('button', {{ type:'button', className:'nav', title:'Reload', id:'_ar', text:'⟳' }}));
  nav.appendChild(el('button', {{ type:'button', className:'nav', title:'Home', id:'_ah', text:'⌂' }}));
  var input = el('input', {{ id:'_au', placeholder:'Search or enter URL…' }});
  input.setAttribute('type', 'text');
  input.value = '';
  try {{ input.value = location.href; }} catch(_){{}}
  nav.appendChild(input);
  nav.appendChild(el('button', {{ type:'button', className:'nav', title:'Bookmark', id:'_abk', text:'★' }}));
  nav.appendChild(el('button', {{ type:'button', className:'nav', title:'Menu', id:'atb-menu-btn', 'aria-haspopup':'true', 'aria-expanded':'false', text:'☰' }}));
  nav.appendChild(el('span', {{ className:'ab', id:'_as', title:'Ads blocked', text:'🛡' }}));
  nav.appendChild(el('span', {{ className:'secchip', id:'_sec', title:'Page security', text:'SAFE' }}));
  var menu = el('div', {{ id:'atb-menu-dropdown', role:'menu' }});
  [['apps','Amni Apps'],['developer','Developer'],['themes','Themes'],['extensions','Extensions'],['report','Report bug'],['history','History'],['downloads','Downloads']].forEach(function(pair){{
    menu.appendChild(el('button', {{ type:'button', className:'mi', 'data-act':pair[0], role:'menuitem', text:pair[1] }}));
  }});
  nav.appendChild(menu);
  bar.appendChild(nav);
  bar.appendChild(el('div', {{ id:'_bookmarks' }}));
  var safe = el('div', {{ id:'_safebar' }});
  safe.appendChild(el('span', {{ id:'_safetxt', text:'' }}));
  var det = el('button', {{ type:'button', text:'Details' }});
  det.onclick = function(e){{ e.preventDefault(); ipc({{ type:'navigate', url:'amnibrowse://developer#sec' }}); }};
  safe.appendChild(det);
  bar.appendChild(safe);
  root.appendChild(bar);
}}
function pinHost(host){{
  if (!host) return;
  var hostCss = 'all:initial;position:fixed!important;top:0!important;left:0!important;right:0!important;width:100%!important;height:{chrome_h}px!important;max-height:{chrome_h}px!important;z-index:2147483647!important;pointer-events:auto!important;display:block!important;visibility:visible!important;opacity:1!important;transform:none!important;isolation:isolate!important;margin:0!important;border:none!important;padding:0!important;overflow:visible!important;inset:auto!important;';
  try {{ host.style.cssText = hostCss; }} catch(_) {{ try {{ host.setAttribute('style', hostCss); }} catch(__){{}} }}
  try {{
    if (!host.hasAttribute('popover')) host.setAttribute('popover', 'manual');
    if (typeof host.showPopover === 'function' && !host.matches(':popover-open')) host.showPopover();
  }} catch(_){{}}
}}
function ensureToolbar(){{
  try {{
    var d = document;
    if (!d.documentElement) return false;
    var mount = d.documentElement;
    var host = d.getElementById('__atb_host');
    var needBuild = !host || !(host.shadowRoot || window.__AMNI_SR) || !((host.shadowRoot || window.__AMNI_SR).getElementById('__atb'));
    if (needBuild) {{
      if (host) try {{ host.remove(); }} catch(_){{}}
      host = d.createElement('div');
      host.id = '__atb_host';
      host.setAttribute('data-amni','chrome');
      pinHost(host);
      var root;
      try {{ root = host.attachShadow({{ mode:'closed' }}); }} catch(_) {{ root = host.attachShadow({{ mode:'open' }}); }}
      window.__AMNI_SR = root;
      var style = d.createElement('style');
      style.id = '_ath_css';
      style.textContent = chromeCss();
      root.appendChild(style);
      buildChromeBar(root);
      mount.appendChild(host);
      pinHost(host);
      applyTheme(T);
      window.__AMNI_TABS_FP = '';
      var seed = (window.__AMNI_TAB_SEED && window.__AMNI_TAB_SEED.length) ? window.__AMNI_TAB_SEED : SEED;
      paintTabs(Array.isArray(seed) && seed.length ? seed : SEED);
      window.__AMNI_WIRED = false;
    }} else {{
      pinHost(host);
      if (!mount.contains(host)) mount.appendChild(host);
    }}
    applyContentPush(host);
    if (!window.__AMNI_WIRED) {{ wireHandlers(host); window.__AMNI_WIRED = true; }}
    if (!window.__AMNI_CHROME_POLL) {{
      window.__AMNI_CHROME_POLL = true;
      ipc({{ type:'get_tabs' }});
      ipc({{ type:'get_stats' }});
      ipc({{ type:'theme_get_active' }});
      ipc({{ type:'bookmark_list' }});
    }}
    return true;
  }} catch(err) {{
    try {{ console.warn('[amni-chrome]', err && err.message ? err.message : err); }} catch(_){{}}
    return false;
  }}
}}
function contentPushPx(host){{
  try {{
    var h = host && host.offsetHeight ? host.offsetHeight : {chrome_h};
    return Math.max({chrome_h}, h) + 4;
  }} catch(_) {{ return {push_h}; }}
}}
function applyContentPush(host){{
  try {{
    var d = document;
    if (!d.documentElement) return;
    var ph = contentPushPx(host || d.getElementById('__atb_host'));
    if (window.__AMNI_PUSH_H === ph && d.getElementById('__amni_push_style')) return;
    window.__AMNI_PUSH_H = ph;
    var head = d.head || d.documentElement;
    var push = d.getElementById('__amni_push_style');
    if (!push) {{
      push = d.createElement('style');
      push.id = '__amni_push_style';
      head.appendChild(push);
    }}
    push.textContent = ''
      + ':root{{--amni-chrome-h:' + ph + 'px!important}}'
      + 'html{{padding-top:' + ph + 'px!important;scroll-padding-top:' + ph + 'px!important;box-sizing:border-box!important;min-height:100%!important}}'
      + 'html,body{{margin-top:0!important}}'
      + 'body{{padding-top:0!important;box-sizing:border-box!important}}'
      + '#__atb_host,#__atb_host:popover-open{{position:fixed!important;top:0!important;left:0!important;right:0!important;width:100%!important;height:{chrome_h}px!important;max-height:{chrome_h}px!important;z-index:2147483647!important;margin:0!important;border:none!important;padding:0!important;overflow:visible!important;inset:auto!important;pointer-events:auto!important;background:transparent!important}}'
      + '#masthead-container,ytd-masthead,#masthead,#header-bar,header#header,#header,tp-yt-app-header,#gb,header[role="banner"],.ytSearchboxComponentHost,.ytSearchboxComponentInputBox{{top:' + ph + 'px!important}}'
      + '#masthead-container{{position:fixed!important;left:0!important;right:0!important;width:100%!important;z-index:2020!important}}'
      + 'ytd-mini-guide-renderer,ytd-guide-renderer#guide,tp-yt-app-drawer#guide{{top:' + ph + 'px!important;height:calc(100vh - ' + ph + 'px)!important;max-height:calc(100vh - ' + ph + 'px)!important}}'
      + '#guide-button,ytd-mini-guide-renderer{{margin-top:0!important}}'
      + 'ytd-app{{--ytd-masthead-height:56px;min-height:calc(100vh - ' + ph + 'px)!important}}'
      + '#page-manager,ytd-page-manager{{box-sizing:border-box!important}}'
      + '#content.ytd-app,ytd-app #content{{padding-top:0!important}}'
      + 'ytd-watch-flexy,ytd-browse,ytd-search{{scroll-margin-top:' + ph + 'px!important}}'
      + '@supports (height:100dvh){{ytd-mini-guide-renderer,ytd-guide-renderer#guide{{height:calc(100dvh - ' + ph + 'px)!important;max-height:calc(100dvh - ' + ph + 'px)!important}}}}';
    try {{
      d.documentElement.style.setProperty('padding-top', ph + 'px', 'important');
      d.documentElement.style.setProperty('--amni-chrome-h', ph + 'px', 'important');
    }} catch(_){{}}
    try {{
      if (d.body) {{
        d.body.style.setProperty('margin-top', '0px', 'important');
        d.body.style.setProperty('padding-top', '0px', 'important');
      }}
    }} catch(_){{}}
  }} catch(_){{}}
}}
window.__AMNI_ENSURE = ensureToolbar;
window.__AMNI_SAFETY = function(rep){{
  try {{
    ensureToolbar();
    var root = amniRoot();
    if (!root || !rep) return;
    var chip = root.getElementById('_sec');
    var bar = root.getElementById('_safebar');
    var txt = root.getElementById('_safetxt');
    var lvl = String(rep.level || 'safe').toLowerCase();
    if (chip) {{
      chip.className = 'secchip ' + lvl;
      chip.textContent = lvl.toUpperCase();
      chip.title = (rep.reasons && rep.reasons.length) ? rep.reasons.join(' · ') : ('Page security: ' + lvl);
      chip.onclick = function(){{ ipc({{ type:'navigate', url:'amnibrowse://developer#sec' }}); }};
    }}
    if (bar && txt) {{
      if (lvl === 'medium' || lvl === 'high' || lvl === 'critical') {{
        bar.className = 'show';
        txt.textContent = 'Caution (' + lvl + '): ' + ((rep.reasons && rep.reasons[0]) || 'Review this page before signing in.');
      }} else {{
        bar.className = '';
        txt.textContent = '';
      }}
    }}
  }} catch(_){{}}
}};
window.__AMNI_SYNC_THEME = function(th){{
  try {{
    ensureToolbar();
    var obj = (typeof th === 'string') ? JSON.parse(th) : th;
    applyTheme(obj);
  }} catch(_) {{}}
}};
window.__AMNI_SYNC_TABS = function(list){{
  try {{
    ensureToolbar();
    paintTabs(list);
  }} catch(_) {{}}
}};
window.__amni_receive = function(msg){{
  if (!msg) return;
  ensureToolbar();
  var root = amniRoot();
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
  ensureToolbar();
  if (!window.__AMNI_CHROME_WATCH) {{
    window.__AMNI_CHROME_WATCH = true;
    setInterval(function(){{
      var h = document.getElementById('__atb_host');
      if (!h || !document.documentElement.contains(h) || !(h.shadowRoot || window.__AMNI_SR)) ensureToolbar();
      else {{
        pinHost(h);
        try {{ if (typeof h.showPopover === 'function' && !h.matches(':popover-open')) h.showPopover(); }} catch(_){{}}
      }}
    }}, 800);
    try {{
      var observer = new MutationObserver(function(){{
        var h = document.getElementById('__atb_host');
        if (!h || !document.documentElement.contains(h)) ensureToolbar();
      }});
      observer.observe(document.documentElement || document, {{ childList:true, subtree:false }});
    }} catch(_) {{}}
    window.addEventListener('pageshow', function(){{ ensureToolbar(); }});
    window.addEventListener('focus', function(){{ ensureToolbar(); }});
    document.addEventListener('fullscreenchange', function(){{
      if (!document.fullscreenElement) ensureToolbar();
    }});
    window.addEventListener('keydown', function(e){{
      var mod = e.ctrlKey || e.metaKey;
      if (!mod) return;
      var k = (e.key || '').toLowerCase();
      var tabs = orderedTabs(window.__AMNI_TAB_SEED || []);
      if (k === 'w') {{ stopHit(e); var cur = tabs.filter(function(t){{ return t.is_active; }})[0]; if (cur && cur.id) ipc({{ type:'close_tab', id:cur.id }}); }}
      else if (k === 't') {{ stopHit(e); ipc({{ type:'new_tab', url:'amnibrowse://newtab' }}); }}
      else if (k === 'l') {{ stopHit(e); var r0 = amniRoot(); var u0 = r0 && r0.getElementById('_au'); if (u0) {{ u0.focus(); u0.select(); }} }}
      else if (k === 'd') {{ stopHit(e); ipc({{ type:'bookmark_add', title:document.title || location.href, url:location.href }}); }}
      else if (k === 'h') {{ stopHit(e); ipc({{ type:'navigate', url:'amnibrowse://newtab#history' }}); }}
      else if (k === 'j') {{ stopHit(e); ipc({{ type:'navigate', url:'amnibrowse://newtab#downloads' }}); }}
      else if (k === 'tab') {{
        stopHit(e);
        if (!tabs.length) return;
        var i = 0; for (; i < tabs.length; i++) if (tabs[i].is_active) break;
        var next = tabs[(i + (e.shiftKey ? tabs.length - 1 : 1)) % tabs.length];
        if (next && next.id) {{ markKbdTab(next.id); ipc({{ type:'switch_tab', id:next.id }}); }}
      }}
      else if (k >= '1' && k <= '9') {{
        stopHit(e);
        if (!tabs.length) return;
        var idx = k === '9' ? tabs.length - 1 : Math.min(parseInt(k, 10) - 1, tabs.length - 1);
        var jt = tabs[idx];
        if (jt && jt.id) {{ markKbdTab(jt.id); ipc({{ type:'switch_tab', id:jt.id }}); }}
      }}
    }}, true);
  }}
}}
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, {{ once:true }});
else start();
}})();"#, theme = t0, seed = seed, tab_h = tab_h, nav_h = nav_h, book_h = book_h, chrome_h = chrome_h, push_h = push_h)
}
