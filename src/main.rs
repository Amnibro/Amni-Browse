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
    #[cfg(feature = "servo-real")]
    { info!("  Backend: Real Servo (libservo)"); platform::servo_real::run(app::BrowserState::new()); return; }
    #[cfg(feature = "webview")]
    { info!("  Backend: WebView (wry/tao)"); platform::webview::Browser::new().run(); }
    #[cfg(all(feature = "servo-engine", not(feature = "webview"), not(feature = "servo-real")))]
    { info!("  Backend: Servo Engine (winit/wgpu/egui)"); platform::servo::run(app::BrowserState::new()); }
}
