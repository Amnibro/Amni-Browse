use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
pub const GH_API: &str = "https://api.github.com/repos/Amnibro/Amni-Browse/releases/latest";
pub const SITE_FEED: &str = "https://amni-scient.com/browse/latest.json";
pub const GH_REPO: &str = "Amnibro/Amni-Browse";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseInfo {
    pub version: String,
    pub url: String,
    pub notes: String,
    pub source: String,
}
pub fn install_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| std::env::temp_dir()).join("AmniBrowse")
}
pub fn is_installed_copy() -> bool {
    let Ok(exe) = std::env::current_exe() else { return false };
    let Some(parent) = exe.parent() else { return false };
    parent == install_dir()
}
pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let t = s.trim().trim_start_matches('v');
    let mut it = t.split('.').filter_map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok());
    Some((it.next()?, it.next().unwrap_or(0), it.next().unwrap_or(0)))
}
pub fn is_newer(remote: &str, local: &str) -> bool {
    match (parse_version(remote), parse_version(local)) {
        (Some(r), Some(l)) => r > l,
        _ => remote.trim_start_matches('v') != local.trim_start_matches('v'),
    }
}
fn ua() -> String { format!("AmniBrowse/{} (+https://amni-scient.com)", env!("CARGO_PKG_VERSION")) }
fn http_get(url: &str) -> Result<String, String> {
    reqwest::blocking::Client::builder().user_agent(ua()).build().map_err(|e| e.to_string())?.get(url).send().map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?.text().map_err(|e| e.to_string())
}
fn from_site() -> Result<ReleaseInfo, String> {
    let raw = http_get(SITE_FEED)?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let version = v.get("version").and_then(|x| x.as_str()).ok_or("site feed missing version")?.to_string();
    let url = v.get("url").and_then(|x| x.as_str()).ok_or("site feed missing url")?.to_string();
    let notes = v.get("notes").and_then(|x| x.as_str()).unwrap_or("").to_string();
    Ok(ReleaseInfo { version, url, notes, source: "site".into() })
}
fn from_github() -> Result<ReleaseInfo, String> {
    let raw = http_get(GH_API)?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let tag = v.get("tag_name").and_then(|x| x.as_str()).unwrap_or("").trim_start_matches('v').to_string();
    let notes = v.get("body").and_then(|x| x.as_str()).unwrap_or("").chars().take(280).collect();
    let assets = v.get("assets").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let want = format!("amni-browse-v{}-win64.zip", tag);
    let url = assets.iter().find_map(|a| {
        let name = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
        (name == want || name.ends_with("-win64.zip")).then(|| a.get("browser_download_url").and_then(|u| u.as_str()).map(|s| s.to_string()))
    }).flatten().ok_or_else(|| format!("no win64 zip on GitHub release {}", tag))?;
    Ok(ReleaseInfo { version: tag, url, notes, source: "github".into() })
}
pub fn check_latest(feed_override: Option<&str>) -> Result<ReleaseInfo, String> {
    if let Some(u) = feed_override.filter(|s| !s.is_empty()) {
        let raw = http_get(u)?;
        let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        let version = v.get("version").and_then(|x| x.as_str()).ok_or("feed missing version")?.to_string();
        let url = v.get("url").and_then(|x| x.as_str()).ok_or("feed missing url")?.to_string();
        let notes = v.get("notes").and_then(|x| x.as_str()).unwrap_or("").to_string();
        return Ok(ReleaseInfo { version, url, notes, source: "feed".into() });
    }
    match from_site() {
        Ok(r) => Ok(r),
        Err(e) => { info!("site feed skipped: {}", e); from_github() }
    }
}
pub fn check_for_update(local: &str, feed_override: Option<&str>) -> Result<Option<ReleaseInfo>, String> {
    let rel = check_latest(feed_override)?;
    Ok(is_newer(&rel.version, local).then_some(rel))
}
pub fn apply_update(rel: &ReleaseInfo) -> Result<String, String> {
    let dest = install_dir();
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let zip_path = dest.join("update.zip");
    let bytes = reqwest::blocking::Client::builder().user_agent(ua()).build().map_err(|e| e.to_string())?.get(&rel.url).send().map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
    std::fs::write(&zip_path, &bytes).map_err(|e| e.to_string())?;
    let pending = dest.join("pending");
    let _ = std::fs::remove_dir_all(&pending);
    std::fs::create_dir_all(&pending).map_err(|e| e.to_string())?;
    let expand = format!("Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force", zip_path.display(), pending.display());
    let st = Command::new("powershell").args(["-NoProfile", "-Command", &expand]).status().map_err(|e| e.to_string())?;
    if !st.success() { return Err("Expand-Archive failed".into()); }
    let helper = dest.join("apply-update.cmd");
    let exe_name = "amni-browse.exe";
    let body = format!("@echo off\r\nping 127.0.0.1 -n 2 >nul\r\nif exist \"{dest}\\{exe}.bak\" del /f /q \"{dest}\\{exe}.bak\"\r\nif exist \"{dest}\\{exe}\" move /y \"{dest}\\{exe}\" \"{dest}\\{exe}.bak\" >nul\r\nxcopy /e /y /q \"{pend}\\*\" \"{dest}\\\" >nul\r\nstart \"\" \"{dest}\\{exe}\"\r\n", dest = dest.display(), exe = exe_name, pend = pending.display());
    std::fs::write(&helper, body).map_err(|e| e.to_string())?;
    Command::new("cmd").args(["/C", "start", "", "/MIN", helper.to_string_lossy().as_ref()]).spawn().map_err(|e| e.to_string())?;
    Ok(format!("Applying {} from {}", rel.version, rel.source))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions_compare() {
        assert!(is_newer("0.12.1", "0.12.0"));
        assert!(is_newer("v1.0.0", "0.12.9"));
        assert!(!is_newer("0.12.0", "0.12.0"));
        assert!(!is_newer("0.11.13", "0.12.0"));
        assert_eq!(parse_version("v0.12.1-win"), Some((0, 12, 1)));
    }
}
