use crate::crypto::vault::PasswordManager;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginMatch {
    pub id: String,
    pub username: String,
    pub source: String,
    pub site: String,
}
#[derive(Debug, Clone)]
pub struct PmState {
    pub kind: String,
    pub session: Option<String>,
    pub cli: Option<String>,
    pub keepass_db: Option<String>,
    pub last: Vec<LoginMatch>,
}
impl PmState {
    pub fn from_config(kind: &str, cli: Option<String>, db: Option<String>) -> Self {
        Self { kind: normalize_kind(kind), session: None, cli, keepass_db: db, last: vec![] }
    }
    pub fn unlocked(&self, vault: &PasswordManager) -> bool {
        match self.kind.as_str() {
            "amni" => vault.is_unlocked(),
            "bitwarden" => self.session.is_some(),
            "onepassword" => which(self.cli.as_deref().unwrap_or("op")).is_some(),
            "keepassxc" => self.session.is_some() && self.keepass_db.is_some(),
            _ => false,
        }
    }
    pub fn label(&self) -> &'static str {
        match self.kind.as_str() { "bitwarden" => "Bitwarden", "onepassword" => "1Password", "keepassxc" => "KeePassXC", _ => "Amni vault" }
    }
}
pub fn normalize_kind(k: &str) -> String {
    match k.trim().to_lowercase().as_str() {
        "bw" | "bitwarden" => "bitwarden".into(),
        "op" | "1password" | "onepassword" | "one-password" => "onepassword".into(),
        "keepass" | "keepassxc" | "kpxc" => "keepassxc".into(),
        _ => "amni".into(),
    }
}
pub fn which(name: &str) -> Option<String> {
    let key = if cfg!(windows) { "where" } else { "which" };
    let out = Command::new(key).arg(name).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("").trim().to_string()).filter(|s| !s.is_empty())
}
fn bin_for(state: &PmState) -> String {
    if let Some(p) = state.cli.as_ref().filter(|s| !s.is_empty()) { return p.clone(); }
    match state.kind.as_str() {
        "bitwarden" => which("bw").unwrap_or_else(|| "bw".into()),
        "onepassword" => which("op").unwrap_or_else(|| "op".into()),
        "keepassxc" => which("keepassxc-cli").unwrap_or_else(|| "keepassxc-cli".into()),
        _ => String::new(),
    }
}
fn run_cmd(bin: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Result<String, String> {
    let mut c = Command::new(bin);
    c.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if stdin.is_some() { c.stdin(Stdio::piped()); }
    for (k, v) in env { c.env(k, v); }
    let mut child = c.spawn().map_err(|e| format!("{}: {}", bin, e))?;
    if let Some(s) = stdin {
        if let Some(mut inn) = child.stdin.take() { let _ = inn.write_all(s.as_bytes()); }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("{} failed: {}", bin, String::from_utf8_lossy(&out.stderr).chars().take(200).collect::<String>()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
pub fn unlock(state: &mut PmState, secret: &str, vault: &mut PasswordManager) -> Result<String, String> {
    match state.kind.as_str() {
        "amni" => {
            let r = if vault.is_initialized() { vault.unlock(secret) } else { vault.initialize(secret) };
            r.map(|_| "Amni vault unlocked".into())
        }
        "bitwarden" => {
            let bin = bin_for(state);
            let session = run_cmd(&bin, &["unlock", "--raw"], &[], Some(&format!("{}\n", secret)))?;
            let s = session.trim().to_string();
            if s.is_empty() { return Err("Bitwarden returned empty session".into()); }
            state.session = Some(s);
            Ok("Bitwarden unlocked".into())
        }
        "onepassword" => {
            let bin = bin_for(state);
            let _ = run_cmd(&bin, &["account", "list", "--format", "json"], &[], None)?;
            state.session = Some("desktop".into());
            Ok("1Password CLI ready (uses signed-in desktop session)".into())
        }
        "keepassxc" => {
            if state.keepass_db.as_ref().map(|s| s.is_empty()).unwrap_or(true) { return Err("Set KeePass database path first".into()); }
            state.session = Some(secret.to_string());
            Ok("KeePassXC password held in memory".into())
        }
        _ => Err("unknown provider".into()),
    }
}
pub fn matches_for_url(state: &mut PmState, vault: &PasswordManager, url: &str) -> Vec<LoginMatch> {
    let host = url::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string())).unwrap_or_default();
    let found = match state.kind.as_str() {
        "amni" => vault.matches_for_url(url).into_iter().map(|c| LoginMatch { id: c.id, username: c.username, source: "amni".into(), site: c.site }).collect(),
        "bitwarden" => bw_list(state, url, &host).unwrap_or_default(),
        "onepassword" => op_list(state, &host).unwrap_or_default(),
        "keepassxc" => kpxc_list(state, &host).unwrap_or_default(),
        _ => vec![],
    };
    state.last = found.clone();
    found
}
pub fn secret_for(state: &PmState, vault: &PasswordManager, id: &str) -> Result<(String, String), String> {
    match state.kind.as_str() {
        "amni" => {
            let user = vault.list_credentials().into_iter().find(|c| c.id == id).map(|c| c.username).unwrap_or_default();
            Ok((user, vault.get_password(id)?))
        }
        "bitwarden" => bw_get(state, id),
        "onepassword" => op_get(state, id),
        "keepassxc" => kpxc_get(state, id),
        _ => Err("unknown provider".into()),
    }
}
fn bw_list(state: &PmState, url: &str, host: &str) -> Result<Vec<LoginMatch>, String> {
    let session = state.session.as_deref().ok_or("Bitwarden locked")?;
    let bin = bin_for(state);
    let raw = run_cmd(&bin, &["list", "items", "--url", url, "--session", session], &[], None)?;
    Ok(parse_bw_items(&raw, host))
}
pub fn parse_bw_items(raw: &str, host: &str) -> Vec<LoginMatch> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) else { return vec![] };
    let Some(arr) = v.as_array() else { return vec![] };
    arr.iter().filter_map(|it| {
        let id = it.get("id").and_then(|x| x.as_str())?.to_string();
        let login = it.get("login")?;
        let username = login.get("username").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let uris = login.get("uris").and_then(|u| u.as_array()).cloned().unwrap_or_default();
        let site = uris.iter().find_map(|u| u.get("uri").and_then(|x| x.as_str())).unwrap_or(host).to_string();
        Some(LoginMatch { id, username, source: "bitwarden".into(), site })
    }).collect()
}
fn bw_get(state: &PmState, id: &str) -> Result<(String, String), String> {
    let session = state.session.as_deref().ok_or("Bitwarden locked")?;
    let bin = bin_for(state);
    let raw = run_cmd(&bin, &["get", "item", id, "--session", session], &[], None)?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let login = v.get("login").ok_or("no login")?;
    Ok((login.get("username").and_then(|x| x.as_str()).unwrap_or("").into(), login.get("password").and_then(|x| x.as_str()).unwrap_or("").into()))
}
fn op_list(state: &PmState, host: &str) -> Result<Vec<LoginMatch>, String> {
    let bin = bin_for(state);
    let raw = run_cmd(&bin, &["item", "list", "--categories", "Login", "--format", "json"], &[], None)?;
    Ok(parse_op_items(&raw, host))
}
pub fn parse_op_items(raw: &str, host: &str) -> Vec<LoginMatch> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) else { return vec![] };
    let Some(arr) = v.as_array() else { return vec![] };
    let h = host.to_lowercase();
    arr.iter().filter_map(|it| {
        let id = it.get("id").and_then(|x| x.as_str())?.to_string();
        let title = it.get("title").and_then(|x| x.as_str()).unwrap_or("");
        let urls = it.get("urls").and_then(|u| u.as_array()).cloned().unwrap_or_default();
        let hit = urls.iter().any(|u| u.get("href").and_then(|x| x.as_str()).unwrap_or("").to_lowercase().contains(&h)) || title.to_lowercase().contains(&h);
        if !hit && !h.is_empty() { return None; }
        Some(LoginMatch { id, username: title.into(), source: "onepassword".into(), site: host.into() })
    }).collect()
}
fn op_get(state: &PmState, id: &str) -> Result<(String, String), String> {
    let bin = bin_for(state);
    let raw = run_cmd(&bin, &["item", "get", id, "--fields", "username,password", "--format", "json"], &[], None)?;
    parse_op_secret(&raw).ok_or_else(|| "1Password field parse failed".into())
}
pub fn parse_op_secret(raw: &str) -> Option<(String, String)> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    if let Some(arr) = v.as_array() {
        let mut user = String::new();
        let mut pass = String::new();
        for f in arr {
            let id = f.get("id").or_else(|| f.get("label")).and_then(|x| x.as_str()).unwrap_or("").to_lowercase();
            let val = f.get("value").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if id.contains("user") { user = val; } else if id.contains("pass") { pass = val; }
        }
        return Some((user, pass));
    }
    if v.is_object() {
        return Some((v.get("username").and_then(|x| x.as_str()).unwrap_or("").into(), v.get("password").and_then(|x| x.as_str()).unwrap_or("").into()));
    }
    None
}
fn kpxc_list(state: &PmState, host: &str) -> Result<Vec<LoginMatch>, String> {
    let db = state.keepass_db.as_deref().ok_or("no keepass db")?;
    let pw = state.session.as_deref().ok_or("KeePass locked")?;
    let bin = bin_for(state);
    let raw = run_cmd(&bin, &["search", db, host], &[], Some(&format!("{}\n", pw)))?;
    Ok(raw.lines().filter(|l| !l.trim().is_empty() && !l.contains("Insert password")).map(|l| LoginMatch { id: l.trim().to_string(), username: l.trim().to_string(), source: "keepassxc".into(), site: host.into() }).collect())
}
fn kpxc_get(state: &PmState, id: &str) -> Result<(String, String), String> {
    let db = state.keepass_db.as_deref().ok_or("no keepass db")?;
    let pw = state.session.as_deref().ok_or("KeePass locked")?;
    let bin = bin_for(state);
    let user = run_cmd(&bin, &["show", "-a", "UserName", "--quiet", db, id], &[], Some(&format!("{}\n", pw)))?;
    let pass = run_cmd(&bin, &["show", "-a", "Password", "--quiet", db, id], &[], Some(&format!("{}\n", pw)))?;
    Ok((user.trim().into(), pass.trim().into()))
}
pub fn import_chrome_csv(vault: &mut PasswordManager, csv: &str) -> Result<usize, String> {
    if !vault.is_unlocked() { return Err("Unlock Amni vault first".into()); }
    let mut n = 0usize;
    for (i, line) in csv.lines().enumerate() {
        if i == 0 && line.to_lowercase().contains("password") { continue; }
        let cols: Vec<&str> = parse_csv_line(line);
        if cols.len() < 4 { continue; }
        let (url, user, pass) = if cols[1].starts_with("http") { (cols[1], cols[2], cols[3]) } else { (cols[0], cols[1], cols[2]) };
        if pass.is_empty() { continue; }
        vault.add_credential(url, user, pass, None, Some("import")).ok();
        n += 1;
    }
    Ok(n)
}
pub fn parse_csv_line(line: &str) -> Vec<&str> {
    let mut out = vec![];
    let mut start = 0usize;
    let b = line.as_bytes();
    let mut i = 0usize;
    let mut q = false;
    while i < b.len() {
        match b[i] {
            b'"' => q = !q,
            b',' if !q => { out.push(line[start..i].trim_matches('"')); start = i + 1; }
            _ => {}
        }
        i += 1;
    }
    out.push(line[start..].trim_matches('"'));
    out
}
pub fn summaries_json(list: &[LoginMatch]) -> String {
    serde_json::to_string(list).unwrap_or_else(|_| "[]".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kinds_normalize() {
        assert_eq!(normalize_kind("BW"), "bitwarden");
        assert_eq!(normalize_kind("1Password"), "onepassword");
        assert_eq!(normalize_kind("keepassxc"), "keepassxc");
        assert_eq!(normalize_kind("nope"), "amni");
    }
    #[test]
    fn bw_json_parses() {
        let raw = r#"[{"id":"abc","login":{"username":"ada","uris":[{"uri":"https://github.com"}]}}]"#;
        let v = parse_bw_items(raw, "github.com");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].username, "ada");
        assert_eq!(v[0].source, "bitwarden");
    }
    #[test]
    fn op_json_filters_host() {
        let raw = r#"[{"id":"1","title":"GitHub","urls":[{"href":"https://github.com"}]},{"id":"2","title":"Bank","urls":[{"href":"https://bank.test"}]}]"#;
        let v = parse_op_items(raw, "github.com");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, "1");
    }
    #[test]
    fn op_secret_array() {
        let raw = r#"[{"id":"username","value":"ada"},{"id":"password","value":"s3"}]"#;
        assert_eq!(parse_op_secret(raw), Some(("ada".into(), "s3".into())));
    }
    #[test]
    fn csv_chrome() {
        let cols = parse_csv_line(r#"GitHub,https://github.com,ada,s3cret"#);
        assert_eq!(cols[1], "https://github.com");
        assert_eq!(cols[3], "s3cret");
    }
}
