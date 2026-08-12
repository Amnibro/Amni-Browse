use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use wry::WebView;
use crate::engine::drm_fallback::is_drm_required;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineKind { Servo, Media }
impl Default for EngineKind { fn default() -> Self { Self::Servo } }
pub const MSE_PATTERNS: &[&str] = &[
    "youtube.com/watch", "youtu.be/", "youtube.com/embed", "m.youtube.com/watch", "music.youtube.com", "youtube.com/shorts", "www.youtube.com/",
    "twitch.tv/", "clips.twitch.tv/",
    "vimeo.com/", "player.vimeo.com/", "dailymotion.com/video",
    "soundcloud.com/",
];
pub const MEDIA_PATTERNS: &[&str] = MSE_PATTERNS;
pub fn is_embed_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("/embed/") || lower.contains("/embed?") || lower.contains("youtube.com/embed")
}
pub fn route(url: &str) -> EngineKind {
    let lower = url.to_lowercase();
    let mse = MSE_PATTERNS.iter().any(|p| lower.contains(p));
    let drm = is_drm_required(url);
    (mse || drm).then_some(EngineKind::Media).unwrap_or(EngineKind::Servo)
}
pub fn wants_media_window(url: &str) -> bool {
    route(url) == EngineKind::Media && !is_embed_url(url)
}
pub struct MediaWindow {
    pub window: Window,
    pub webview: WebView,
    pub url: String,
    pub close_req: Arc<AtomicBool>,
}
const MEDIA_UA: &str = concat!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 AmniBrowse/", env!("CARGO_PKG_VERSION"));
fn media_chrome_js() -> String {
    r#"(function(){
try{if(window.self!==window.top)return;}catch(_){return;}
function ipc(s){try{window.ipc&&window.ipc.postMessage(s);}catch(_){}}
function ensure(){
  try{
    var d=document;
    if(!d.documentElement)return false;
    var host=d.getElementById('__amni_media_bar');
    if(!host){
      host=d.createElement('div');
      host.id='__amni_media_bar';
      host.setAttribute('data-amni','media-chrome');
      host.style.cssText='all:initial;position:fixed!important;top:0!important;left:0!important;right:0!important;height:44px!important;z-index:2147483647!important;display:flex!important;align-items:center!important;gap:8px!important;padding:0 12px!important;box-sizing:border-box!important;background:#0f1419!important;color:#e0e6f0!important;font:13px system-ui,sans-serif!important;border-bottom:1px solid #1a2332!important;box-shadow:0 2px 12px rgba(0,0,0,.45)!important;pointer-events:auto!important;';
      var mk=function(tag,css,txt){var e=d.createElement(tag);e.style.cssText=css;if(txt!=null)e.textContent=txt;return e;};
      var back=mk('button','all:unset;cursor:pointer;padding:6px 12px;border-radius:8px;background:#1a1f2e;color:#00d4ff;font:600 12px system-ui,sans-serif;border:1px solid #1a2332;','← Back to Amni');
      back.title='Close media window and return to browser tabs';
      back.onclick=function(e){e.preventDefault();e.stopPropagation();ipc('amni_media_close');};
      var home=mk('button','all:unset;cursor:pointer;padding:6px 12px;border-radius:8px;background:transparent;color:#e0e6f0;font:12px system-ui,sans-serif;border:1px solid #1a2332;','Home');
      home.onclick=function(e){e.preventDefault();e.stopPropagation();ipc('amni_media_close');};
      var label=mk('span','flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#6b7d99;font:12px system-ui,sans-serif;','Amni Media · streaming/DRM · close this window to return to tabs');
      try{label.textContent='Amni Media · '+(location.hostname||'stream')+' · close to return to tabs';}catch(_){}
      var x=mk('button','all:unset;cursor:pointer;padding:6px 10px;border-radius:8px;background:transparent;color:#ff4757;font:14px system-ui,sans-serif;','✕');
      x.title='Close media window';
      x.onclick=function(e){e.preventDefault();e.stopPropagation();ipc('amni_media_close');};
      host.appendChild(back);host.appendChild(home);host.appendChild(label);host.appendChild(x);
      (d.body||d.documentElement).prepend(host);
    } else if(host.parentNode!==(d.body||d.documentElement)){
      (d.body||d.documentElement).prepend(host);
    }
    if(!d.getElementById('__amni_media_push')){
      var s=d.createElement('style');
      s.id='__amni_media_push';
      s.textContent='html{margin-top:44px!important}body{margin-top:0!important}';
      (d.head||d.documentElement).appendChild(s);
    }
    return true;
  }catch(_){return false;}
}
function start(){
  ensure();
  setInterval(ensure,400);
  try{
    var obs=new MutationObserver(function(){ensure();});
    obs.observe(document.documentElement||document,{childList:true,subtree:true});
  }catch(_){}
  window.addEventListener('pageshow',function(){ensure();});
  window.addEventListener('keydown',function(e){
    if(e.key==='Escape'&&(e.ctrlKey||e.metaKey)){e.preventDefault();ipc('amni_media_close');}
  },true);
}
if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',start,{once:true});
else start();
})();"#.into()
}
pub fn spawn_media_window(event_loop: &ActiveEventLoop, url: &str) -> Option<(WindowId, MediaWindow)> {
    configure_privacy_env();
    let close_req = Arc::new(AtomicBool::new(false));
    let cr = close_req.clone();
    let attrs = Window::default_attributes()
        .with_title(format!("Amni Media \u{2014} {}", display_title(url)))
        .with_decorations(true)
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 400.0));
    let window = match event_loop.create_window(attrs) {
        Ok(w) => w,
        Err(e) => { warn!("media_engine: window create failed: {}", e); return None; }
    };
    let id = window.id();
    let builder = wry::WebViewBuilder::new()
        .with_url(url)
        .with_user_agent(MEDIA_UA)
        .with_initialization_script(&media_chrome_js())
        .with_ipc_handler(move |msg| {
            let body = msg.body();
            if body.contains("amni_media_close") { cr.store(true, Ordering::SeqCst); }
        })
        .with_devtools(cfg!(debug_assertions));
    let webview = match builder.build(&window) {
        Ok(w) => w,
        Err(e) => { warn!("media_engine: webview build failed: {}", e); return None; }
    };
    info!("media_engine: spawned media window {:?} for {} via {}", id, url, platform_label());
    Some((id, MediaWindow { window, webview, url: url.into(), close_req }))
}
pub fn drain_close_requests(windows: &mut std::collections::HashMap<WindowId, MediaWindow>) -> Vec<WindowId> {
    let ids: Vec<WindowId> = windows.iter().filter(|(_, m)| m.close_req.load(Ordering::SeqCst)).map(|(id, _)| *id).collect();
    for id in &ids { windows.remove(id); info!("media_engine: closed media window {:?}", id); }
    ids
}
fn display_title(url: &str) -> String {
    url::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| url.chars().take(40).collect())
}
#[cfg(target_os = "windows")]
fn configure_privacy_env() {
    let args = "--disable-features=msEdgeSmartScreen,AutoUpgradeAllUpgradableMixedContent,OptimizationHints,InterestGroupStorage,BrowsingTopics --disable-background-networking --disable-sync --disable-breakpad --no-default-browser-check --no-first-run";
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);
    if let Some(dir) = dirs::config_dir() {
        let ud = dir.join("amni-browse").join("webview2-data");
        std::fs::create_dir_all(&ud).ok();
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", ud);
    }
}
#[cfg(target_os = "macos")]
fn configure_privacy_env() { info!("media_engine: WKWebView with app-sandboxed data store"); }
#[cfg(target_os = "linux")]
fn configure_privacy_env() {
    if let Some(dir) = dirs::config_dir() {
        let ud = dir.join("amni-browse").join("webkit-data");
        std::fs::create_dir_all(&ud).ok();
    }
    if widevine_installed() { std::env::set_var("WEBKIT_FORCE_WIDEVINE_ENABLED", "1"); }
}
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn configure_privacy_env() {}
#[cfg(target_os = "linux")]
pub fn widevine_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("amni-browse").join("widevine").join("libwidevinecdm.so"))
}
#[cfg(target_os = "linux")]
pub fn widevine_installed() -> bool {
    widevine_path().map(|p| p.exists()).unwrap_or(false)
}
#[cfg(target_os = "linux")]
pub fn install_widevine() -> Result<String, String> {
    let target = widevine_path().ok_or_else(|| "no config dir".to_string())?;
    if target.exists() { return Ok(format!("Widevine already installed at {}", target.display())); }
    Err(format!("Manual install required: download libwidevinecdm.so from a Chrome/Chromium build and copy to {}. This is opt-in because Widevine is proprietary (Google TOS).", target.display()))
}
#[cfg(not(target_os = "linux"))]
pub fn widevine_installed() -> bool { true }
#[cfg(not(target_os = "linux"))]
pub fn install_widevine() -> Result<String, String> { Ok("Widevine provided by system WebView runtime".into()) }
pub fn platform_label() -> &'static str {
    if cfg!(target_os = "windows") { "WebView2 (Chromium/Edge)" }
    else if cfg!(target_os = "macos") { "WKWebView (Safari/WebKit)" }
    else if cfg!(target_os = "linux") { "WebKitGTK" }
    else { "wry (generic)" }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn servo_for_normal_sites() {
        assert_eq!(route("https://duckduckgo.com/"), EngineKind::Servo);
        assert_eq!(route("https://en.wikipedia.org/wiki/Rust"), EngineKind::Servo);
        assert_eq!(route("https://github.com/servo/servo"), EngineKind::Servo);
        assert_eq!(route("https://amni-scient.com/"), EngineKind::Servo);
    }
    #[test]
    fn media_for_drm_domains() {
        assert_eq!(route("https://www.netflix.com/"), EngineKind::Media);
        assert_eq!(route("https://www.netflix.com/watch/80057281"), EngineKind::Media);
        assert_eq!(route("https://www.disneyplus.com/video/abc"), EngineKind::Media);
        assert_eq!(route("https://play.max.com/video/xyz"), EngineKind::Media);
        assert_eq!(route("https://www.spotify.com/"), EngineKind::Media);
    }
    #[test]
    fn media_for_mse_streams() {
        assert_eq!(route("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), EngineKind::Media);
        assert_eq!(route("https://youtu.be/dQw4w9WgXcQ"), EngineKind::Media);
        assert_eq!(route("https://www.twitch.tv/somechannel"), EngineKind::Media);
    }
    #[test]
    fn embed_urls_do_not_spawn_window() {
        assert!(is_embed_url("https://www.youtube.com/embed/xyz"));
        assert!(!wants_media_window("https://www.youtube.com/embed/xyz"));
        assert!(wants_media_window("https://www.youtube.com/watch?v=xyz"));
        assert!(wants_media_window("https://www.netflix.com/browse"));
    }
}
