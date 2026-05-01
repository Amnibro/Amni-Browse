use log::{info, warn};
use serde::{Deserialize, Serialize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use wry::WebView;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineKind { Servo, Media }
impl Default for EngineKind { fn default() -> Self { Self::Servo } }
pub const MEDIA_PATTERNS: &[&str] = &[
    "youtube.com/watch", "youtu.be/", "youtube.com/embed", "m.youtube.com/watch", "music.youtube.com",
    "twitch.tv/", "clips.twitch.tv/",
    "vimeo.com/", "player.vimeo.com/", "dailymotion.com/video",
    "netflix.com/watch", "netflix.com/title",
    "hulu.com/watch", "hbomax.com/", "max.com/video", "disneyplus.com/video", "peacocktv.com/watch",
    "primevideo.com/", "amazon.com/gp/video", "paramountplus.com/video",
    "crunchyroll.com/watch", "funimation.com/v/",
    "appletv.apple.com/", "tv.apple.com/",
    "open.spotify.com/embed", "tidal.com/browse", "soundcloud.com/",
    "discoveryplus.com/video", "espnplus.com/",
];
pub fn route(url: &str) -> EngineKind {
    let lower = url.to_lowercase();
    MEDIA_PATTERNS.iter().any(|p| lower.contains(p)).then_some(EngineKind::Media).unwrap_or(EngineKind::Servo)
}
pub struct MediaWindow { pub window: Window, pub webview: WebView, pub url: String }
const MEDIA_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 AmniBrowse/0.9";
pub fn spawn_media_window(event_loop: &ActiveEventLoop, url: &str) -> Option<(WindowId, MediaWindow)> {
    configure_privacy_env();
    let attrs = Window::default_attributes()
        .with_title(format!("Amni Media \u{2014} {}", display_title(url)))
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 400.0));
    let window = match event_loop.create_window(attrs) {
        Ok(w) => w,
        Err(e) => { warn!("media_engine: window create failed: {}", e); return None; }
    };
    let id = window.id();
    let builder = wry::WebViewBuilder::new().with_url(url).with_user_agent(MEDIA_UA).with_devtools(cfg!(debug_assertions));
    let webview = match builder.build(&window) {
        Ok(w) => w,
        Err(e) => { warn!("media_engine: webview build failed: {}", e); return None; }
    };
    info!("media_engine: spawned media window {:?} for {}", id, url);
    Some((id, MediaWindow { window, webview, url: url.into() }))
}
fn display_title(url: &str) -> String {
    url::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| url.chars().take(40).collect())
}
#[cfg(target_os = "windows")]
fn configure_privacy_env() {
    let args = "--disable-features=msEdgeSmartScreen,AutoUpgradeAllUpgradableMixedContent,OptimizationHints --disable-background-networking --disable-sync --disable-breakpad --no-default-browser-check --no-first-run";
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
