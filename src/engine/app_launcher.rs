use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use log::info;
#[derive(Debug, Clone, Serialize)]
pub struct AmniApp {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub emoji: &'static str,
    pub launch: LaunchType,
    pub category: AppCategory,
}
#[derive(Debug, Clone, Serialize)]
pub enum LaunchType {
    Bat(&'static str),
    Cargo(&'static str),
    Web(&'static str),
}
#[derive(Debug, Clone, Serialize)]
pub enum AppCategory { Local, Web }
#[derive(Debug, Clone, Serialize)]
pub struct AmniAppPayload {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub emoji: &'static str,
    pub launch: LaunchType,
    pub category: AppCategory,
    pub icon_src: Option<String>,
}
const WORKSPACE: &str = r"C:\Users\antho\Documents\ai";
pub static AMNI_APPS: &[AmniApp] = &[
    AmniApp { id: "amni-ai", name: "Amni AI", desc: "Qwen3.5-122B AI assistant with Gradio UI", emoji: "rocket", launch: LaunchType::Bat(r"Amni-Ai\run.bat"), category: AppCategory::Local },
    AmniApp { id: "azno-v2", name: "Azno v2", desc: "GPU-accelerated trading platform", emoji: "chart", launch: LaunchType::Bat(r"Azno - v2\Run.bat"), category: AppCategory::Local },
    AmniApp { id: "amni-mail", name: "Amni Mail", desc: "Privacy-first email client", emoji: "inbox", launch: LaunchType::Bat(r"Amni-Mail\run.bat"), category: AppCategory::Local },
    AmniApp { id: "amni-gen", name: "Amni Gen", desc: "AI image generation with ROCm/ZLUDA", emoji: "palette", launch: LaunchType::Bat(r"Amni-gen\run.bat"), category: AppCategory::Local },
    AmniApp { id: "amni-calc", name: "Amni Calc", desc: "Septidecimal WASM calculator", emoji: "diamond", launch: LaunchType::Bat(r"Amni-Calc\run.bat"), category: AppCategory::Local },
    AmniApp { id: "amni-explore", name: "Amni Explore", desc: "3D exoplanet exploration", emoji: "globe", launch: LaunchType::Bat(r"Amni-Explore\run.bat"), category: AppCategory::Local },
    AmniApp { id: "amni-miner", name: "Amni Miner", desc: "Data mining dashboard", emoji: "bolt", launch: LaunchType::Bat(r"Amni-miner\run_dashboard.bat"), category: AppCategory::Local },
    AmniApp { id: "amni-game", name: "Amni Game", desc: "Rust game engine — galactic exploration", emoji: "xr", launch: LaunchType::Cargo(r"Amni-Game"), category: AppCategory::Local },
    AmniApp { id: "amni-coder", name: "Amni Coder", desc: "AI-powered code editor", emoji: "wrench", launch: LaunchType::Web("https://www.example.com/coder"), category: AppCategory::Web },
    AmniApp { id: "amni-scient", name: "Amni-Scient", desc: "Main website — all Amni products", emoji: "crown", launch: LaunchType::Web("https://www.example.com"), category: AppCategory::Web },
];
pub fn list_apps_json() -> String {
    let mail_icon_src = detect_mail_icon_src();
    let apps: Vec<AmniAppPayload> = AMNI_APPS.iter().map(|a| AmniAppPayload {
        id: a.id,
        name: a.name,
        desc: a.desc,
        emoji: a.emoji,
        launch: a.launch.clone(),
        category: a.category.clone(),
        icon_src: (a.id == "amni-mail").then(|| mail_icon_src.clone()).flatten(),
    }).collect();
    serde_json::to_string(&apps).unwrap_or_else(|_| "[]".into())
}
fn browse_root() -> PathBuf {
    Path::new(WORKSPACE).join("Amni-Browse")
}
fn assets_dir() -> PathBuf {
    browse_root().join("assets")
}
fn as_file_url(path: &Path) -> String {
    format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
}
fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| matches!(x.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}
fn detect_mail_icon_src() -> Option<String> {
    let assets = assets_dir();
    if !assets.exists() { return None; }
    for file_name in ["amni-mail.png", "amni-mail.jpg", "amni-mail.jpeg"] {
        let direct = assets.join(file_name);
        if direct.exists() { return Some(as_file_url(&direct)); }
    }
    let mut named: Vec<PathBuf> = Vec::new();
    let mut fallback: Vec<(SystemTime, PathBuf)> = Vec::new();
    let entries = fs::read_dir(&assets).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_supported_image(&path) { continue; }
        let file_name = path.file_name().and_then(|x| x.to_str()).map(|x| x.to_ascii_lowercase()).unwrap_or_default();
        if file_name.contains("mail") {
            named.push(path);
            continue;
        }
        if file_name.contains("browse") { continue; }
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(SystemTime::UNIX_EPOCH);
        fallback.push((modified, path));
    }
    if let Some(p) = named.into_iter().next() { return Some(as_file_url(&p)); }
    fallback.sort_by_key(|(t, _)| *t);
    fallback.pop().map(|(_, p)| as_file_url(&p))
}
pub fn launch_app(id: &str) -> Result<String, String> {
    let app = AMNI_APPS.iter().find(|a| a.id == id).ok_or_else(|| format!("Unknown app: {}", id))?;
    match &app.launch {
        LaunchType::Bat(rel) => {
            let full = format!("{}\\{}", WORKSPACE, rel);
            let path = std::path::Path::new(&full);
            if !path.exists() { return Err(format!("Not found: {}", full)); }
            let dir = path.parent().unwrap_or(path);
            info!("Launching {}: {}", app.name, full);
            Command::new("cmd").args(["/C", "start", "", &full]).current_dir(dir).spawn().map_err(|e| format!("Spawn failed: {}", e))?;
            Ok(format!("{} launched", app.name))
        }
        LaunchType::Cargo(rel) => {
            let dir = format!("{}\\{}", WORKSPACE, rel);
            let path = std::path::Path::new(&dir);
            if !path.exists() { return Err(format!("Not found: {}", dir)); }
            info!("Launching {} via cargo run", app.name);
            Command::new("cmd").args(["/C", "start", "cmd", "/K", "cd", &dir, "&&", "cargo", "run", "--release"]).spawn().map_err(|e| format!("Spawn failed: {}", e))?;
            Ok(format!("{} building & launching", app.name))
        }
        LaunchType::Web(url) => {
            info!("Web app: {} -> {}", app.name, url);
            Ok(url.to_string())
        }
    }
}
