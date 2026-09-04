mod ui;
mod net;
mod storage;
mod crypto;
mod media;
mod platform;
mod engine;
mod app;
use log::info;
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn,egui_wgpu=warn")).init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _guard = rt.enter();
    info!("Amni Browse v{} starting up...", storage::config::APP_VERSION);
    info!("  by Amni-Scient | Privacy: ALWAYS ON | Telemetry: DISABLED");
    info!("  Vault: AES-256-GCM/PBKDF2-SHA256 | DoH: Ready | Extensions: Ready");
    let mut state = app::BrowserState::new();
    if let Some(arg) = std::env::args().skip(1).find(|a| !a.starts_with('-')) {
        let url = if arg.contains("://") { arg } else { format!("https://{}", arg) };
        info!("CLI start url: {}", url);
        if let Some(t) = state.tabs.active_tab_mut() { t.navigate(&url); } else { state.tabs.new_tab(&url); }
    }
    #[cfg(feature = "servo-real")]
    { info!("  Backend: Real Servo (libservo)"); platform::servo_real::run(state); return; }
    #[cfg(all(feature = "webview", not(feature = "servo-real")))]
    { info!("  Backend: {}", match cfg!(windows) { true => "Chromium (WebView2 via wry/tao)", false => "WebKitGTK (wry/tao)" }); platform::chromium::run(state); }
    #[cfg(all(feature = "servo-engine", not(feature = "webview"), not(feature = "servo-real")))]
    { info!("  Backend: Servo Engine (winit/wgpu/egui)"); platform::servo::run(state); }
}
