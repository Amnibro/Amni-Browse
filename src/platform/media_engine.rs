use log::{info, warn};
use serde::{Deserialize, Serialize};
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
pub struct MediaWindow { pub window: Window, pub webview: WebView, pub url: String }
const MEDIA_UA: &str = concat!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 AmniBrowse/", env!("CARGO_PKG_VERSION"));
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
    info!("media_engine: spawned media window {:?} for {} via {}", id, url, platform_label());
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
