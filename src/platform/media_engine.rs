use log::{info, warn};
use serde::{Deserialize, Serialize};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::raw_window_handle::HasWindowHandle;
use wry::{Rect, WebView, WebViewBuilder};
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
    is_drm_required(url).then_some(EngineKind::Media).unwrap_or(EngineKind::Servo)
}
pub fn wants_media_window(url: &str) -> bool {
    route(url) == EngineKind::Media && !is_embed_url(url)
}
pub struct MediaWindow {
    pub webview: WebView,
    pub url: String,
}
/// Kill HTML media / speech before tearing a pane or Servo tab down.
/// Hide alone does not stop WebView2 or Servo audio — ghost tabs keep playing.
/// Do NOT navigate to about:blank here: that allocates another document and leaks
/// if Drop is deferred; silence + hide + Drop is the teardown.
pub const SILENCE_JS: &str = "(function(){try{window.stop()}catch(e){}\
try{document.querySelectorAll('video,audio').forEach(function(m){try{m.pause();m.removeAttribute('src');m.srcObject=null;m.load()}catch(e){}})}catch(e){}\
try{if(window.speechSynthesis)speechSynthesis.cancel()}catch(e){}})()";
pub fn trash_pane(pane: MediaWindow) {
    let _ = pane.webview.evaluate_script(SILENCE_JS);
    let _ = pane.webview.set_visible(false);
    drop(pane);
    info!("media_engine: trashed DRM pane");
}
pub fn content_bounds(win_w: u32, win_h: u32, chrome_px: u32) -> Rect {
    let y = chrome_px.min(win_h.saturating_sub(1));
    let h = win_h.saturating_sub(y).max(1);
    Rect { position: PhysicalPosition::new(0i32, y as i32).into(), size: PhysicalSize::new(win_w.max(1), h).into() }
}
const MEDIA_UA: &str = concat!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 AmniBrowse/", env!("CARGO_PKG_VERSION"));
pub fn spawn_media_pane<W: HasWindowHandle>(parent: &W, url: &str, bounds: Rect) -> Option<MediaWindow> {
    configure_privacy_env();
    let builder = WebViewBuilder::new().with_url(url).with_user_agent(MEDIA_UA).with_bounds(bounds).with_devtools(cfg!(debug_assertions));
    let webview = match builder.build_as_child(parent) {
        Ok(w) => w,
        Err(e) => { warn!("media_engine: child webview failed: {}", e); return None; }
    };
    info!("media_engine: in-tab DRM pane for {} via {}", url, platform_label());
    Some(MediaWindow { webview, url: url.into() })
}
pub fn apply_pane_bounds(pane: &MediaWindow, bounds: Rect, visible: bool) {
    let _ = pane.webview.set_bounds(bounds);
    let _ = pane.webview.set_visible(visible);
}
pub fn display_title(url: &str) -> String {
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
    fn mse_hosts_stay_on_servo() {
        assert_eq!(route("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), EngineKind::Servo);
        assert_eq!(route("https://music.youtube.com/"), EngineKind::Servo);
        assert_eq!(route("https://www.twitch.tv/foo"), EngineKind::Servo);
        assert_eq!(route("https://youtu.be/dQw4w9WgXcQ"), EngineKind::Servo);
        assert_eq!(route("https://www.twitch.tv/somechannel"), EngineKind::Servo);
        assert!(!wants_media_window("https://www.youtube.com/watch?v=xyz"));
    }
    #[test]
    fn silence_js_kills_media_elements() {
        assert!(SILENCE_JS.contains("pause()"));
        assert!(SILENCE_JS.contains("video,audio"));
        assert!(SILENCE_JS.contains("speechSynthesis"));
    }
    #[test]
    fn embed_urls_do_not_spawn_window() {
        assert!(is_embed_url("https://www.youtube.com/embed/xyz"));
        assert!(!wants_media_window("https://www.youtube.com/embed/xyz"));
        assert!(wants_media_window("https://www.netflix.com/browse"));
    }
    #[test]
    fn content_bounds_sit_under_chrome() {
        let b = content_bounds(1280, 800, 84);
        let y = match b.position { wry::dpi::Position::Physical(p) => p.y, wry::dpi::Position::Logical(p) => p.y as i32 };
        let h = match b.size { wry::dpi::Size::Physical(s) => s.height, wry::dpi::Size::Logical(s) => s.height as u32 };
        assert_eq!(y, 84);
        assert_eq!(h, 716);
    }
}
