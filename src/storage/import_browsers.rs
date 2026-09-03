use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct DetectedBrowser {
    pub id: String,
    pub name: String,
    pub path: String,
}
#[derive(Debug, Clone, Default, serde::Serialize, PartialEq)]
pub struct ImportReport {
    pub source: String,
    pub bookmarks: usize,
    pub history: usize,
    pub passwords: usize,
    pub password_skip: usize,
    pub notes: Vec<String>,
}
pub fn local_app() -> PathBuf { dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")) }
pub fn roaming_app() -> PathBuf { dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")) }
pub fn chrome_user_data() -> PathBuf { local_app().join("Google").join("Chrome").join("User Data") }
pub fn edge_user_data() -> PathBuf { local_app().join("Microsoft").join("Edge").join("User Data") }
pub fn brave_user_data() -> PathBuf { local_app().join("BraveSoftware").join("Brave-Browser").join("User Data") }
pub fn detect() -> Vec<DetectedBrowser> {
    let mut out = vec![];
    for (id, name, root) in [("chrome","Google Chrome",chrome_user_data()),("edge","Microsoft Edge",edge_user_data()),("brave","Brave",brave_user_data())] {
        let bm = root.join("Default").join("Bookmarks");
        if bm.exists() { out.push(DetectedBrowser { id: id.into(), name: name.into(), path: bm.display().to_string() }); }
    }
    if let Some(p) = firefox_places() { out.push(DetectedBrowser { id: "firefox".into(), name: "Firefox".into(), path: p.display().to_string() }); }
    out
}
pub fn firefox_places() -> Option<PathBuf> {
    let ini = roaming_app().join("Mozilla").join("Firefox").join("profiles.ini");
    let text = fs::read_to_string(ini).ok()?;
    let mut path: Option<String> = None;
    let mut def = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') { if def { if let Some(p) = path.take() { return Some(resolve_ff_path(&p)); } } def = false; path = None; }
        else if l.eq_ignore_ascii_case("default=1") || l.eq_ignore_ascii_case("default=true") { def = true; }
        else if let Some(v) = l.strip_prefix("Path=") { path = Some(v.replace('/', std::path::MAIN_SEPARATOR_STR)); }
    }
    path.map(|p| resolve_ff_path(&p)).or_else(|| {
        let root = roaming_app().join("Mozilla").join("Firefox").join("Profiles");
        fs::read_dir(root).ok()?.flatten().map(|e| e.path().join("places.sqlite")).find(|p| p.exists())
    })
}
fn resolve_ff_path(p: &str) -> PathBuf {
    let pb = PathBuf::from(p);
    let full = if pb.is_absolute() { pb } else { roaming_app().join("Mozilla").join("Firefox").join(pb) };
    if full.ends_with("places.sqlite") { full } else { full.join("places.sqlite") }
}
pub fn parse_chromium_bookmarks(json: &str) -> Vec<(String, String, Option<String>)> {
    let Ok(v) = serde_json::from_str::<Value>(json) else { return vec![] };
    let mut out = vec![];
    if let Some(roots) = v.get("roots").and_then(|r| r.as_object()) {
        for (folder, node) in roots {
            walk_bm(node, folder, &mut out);
        }
    }
    out
}
fn walk_bm(node: &Value, folder: &str, out: &mut Vec<(String, String, Option<String>)>) {
    match node.get("type").and_then(|t| t.as_str()).unwrap_or("") {
        "url" => {
            let url = node.get("url").and_then(|u| u.as_str()).unwrap_or("");
            if url.starts_with("http") {
                let title = node.get("name").and_then(|n| n.as_str()).unwrap_or(url);
                out.push((title.into(), url.into(), Some(folder.into())));
            }
        }
        "folder" => {
            let name = node.get("name").and_then(|n| n.as_str()).unwrap_or(folder);
            if let Some(kids) = node.get("children").and_then(|c| c.as_array()) {
                for k in kids { walk_bm(k, name, out); }
            }
        }
        _ => {
            if let Some(kids) = node.get("children").and_then(|c| c.as_array()) {
                for k in kids { walk_bm(k, folder, out); }
            }
        }
    }
}
pub fn chromium_profile(root: &Path) -> PathBuf { root.join("Default") }
pub fn import_chromium_bookmarks_file(path: &Path) -> Vec<(String, String, Option<String>)> {
    fs::read_to_string(path).ok().map(|s| parse_chromium_bookmarks(&s)).unwrap_or_default()
}
pub fn copy_unlocked(src: &Path) -> Option<PathBuf> {
    if !src.exists() { return None; }
    let dst = std::env::temp_dir().join(format!("amni-imp-{}-{}", src.file_name()?.to_string_lossy(), std::process::id()));
    fs::copy(src, &dst).ok()?;
    Some(dst)
}
pub fn chromium_history_rows(db: &Path) -> Vec<(String, String, u32)> {
    let Ok(conn) = rusqlite::Connection::open(db) else { return vec![] };
    let Ok(mut stmt) = conn.prepare("SELECT url, IFNULL(title,''), visit_count FROM urls WHERE url LIKE 'http%' ORDER BY last_visit_time DESC LIMIT 800") else { return vec![] };
    stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? as u32))).ok().map(|rows| rows.filter_map(|x| x.ok()).collect()).unwrap_or_default()
}
pub fn firefox_bookmark_rows(db: &Path) -> Vec<(String, String, Option<String>)> {
    let Ok(conn) = rusqlite::Connection::open(db) else { return vec![] };
    let Ok(mut stmt) = conn.prepare("SELECT IFNULL(b.title, p.title), p.url FROM moz_bookmarks b JOIN moz_places p ON b.fk = p.id WHERE p.url LIKE 'http%' AND b.type = 1") else { return vec![] };
    stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, Some("firefox".into())))).ok().map(|rows| rows.filter_map(|x| x.ok()).collect()).unwrap_or_default()
}
pub fn firefox_history_rows(db: &Path) -> Vec<(String, String, u32)> {
    let Ok(conn) = rusqlite::Connection::open(db) else { return vec![] };
    let Ok(mut stmt) = conn.prepare("SELECT url, IFNULL(title,''), visit_count FROM moz_places WHERE url LIKE 'http%' ORDER BY last_visit_date DESC LIMIT 800") else { return vec![] };
    stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? as u32))).ok().map(|rows| rows.filter_map(|x| x.ok()).collect()).unwrap_or_default()
}
#[cfg(windows)]
fn dpapi_unprotect(data: &[u8]) -> Option<Vec<u8>> {
    #[repr(C)]
    struct Blob { cb: u32, pb: *mut u8 }
    #[link(name = "crypt32")]
    extern "system" { fn CryptUnprotectData(i: *mut Blob, n: *mut *mut u16, e: *mut Blob, r: *mut core::ffi::c_void, p: *mut core::ffi::c_void, f: u32, o: *mut Blob) -> i32; }
    #[link(name = "kernel32")]
    extern "system" { fn LocalFree(h: *mut core::ffi::c_void) -> *mut core::ffi::c_void; }
    let mut raw = data.to_vec();
    let mut input = Blob { cb: raw.len() as u32, pb: raw.as_mut_ptr() };
    let mut output = Blob { cb: 0, pb: std::ptr::null_mut() };
    let ok = unsafe { CryptUnprotectData(&mut input, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), 0, &mut output) };
    if ok == 0 || output.pb.is_null() { return None; }
    let sl = unsafe { std::slice::from_raw_parts(output.pb, output.cb as usize) }.to_vec();
    unsafe { LocalFree(output.pb as *mut _); }
    Some(sl)
}
#[cfg(not(windows))]
fn dpapi_unprotect(_: &[u8]) -> Option<Vec<u8>> { None }
pub fn chromium_master_key(local_state: &Path) -> Option<Vec<u8>> {
    let v: Value = serde_json::from_str(&fs::read_to_string(local_state).ok()?).ok()?;
    let b64 = v.pointer("/os_crypt/encrypted_key")?.as_str()?;
    let raw = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64).ok()?;
    let data = raw.strip_prefix(b"DPAPI").unwrap_or(&raw);
    dpapi_unprotect(data)
}
pub fn decrypt_chrome_secret(master: &[u8], blob: &[u8]) -> Option<String> {
    if blob.starts_with(b"v10") || blob.starts_with(b"v20") {
        if blob.len() < 3 + 12 + 16 { return None; }
        let nonce = &blob[3..15];
        let rest = &blob[15..];
        let cipher = aes_gcm::Aes256Gcm::new_from_slice(master).ok()?;
        use aes_gcm::aead::{Aead, KeyInit};
        let pt = cipher.decrypt(aes_gcm::Nonce::from_slice(nonce), rest).ok()?;
        return String::from_utf8(pt).ok();
    }
    String::from_utf8(dpapi_unprotect(blob)?).ok()
}
pub fn chromium_logins(db: &Path, master: &[u8]) -> Vec<(String, String, String)> {
    let Ok(conn) = rusqlite::Connection::open(db) else { return vec![] };
    let Ok(mut stmt) = conn.prepare("SELECT origin_url, username_value, password_value FROM logins") else { return vec![] };
    stmt.query_map([], |r| {
        let url: String = r.get(0)?;
        let user: String = r.get(1)?;
        let blob: Vec<u8> = r.get(2)?;
        Ok((url, user, blob))
    }).ok().map(|rows| rows.filter_map(|x| x.ok()).filter_map(|(u, user, blob)| decrypt_chrome_secret(master, &blob).map(|p| (u, user, p))).collect()).unwrap_or_default()
}
pub fn apply_import(id: &str, bookmarks: &mut crate::storage::bookmarks::BookmarkManager, history: &mut crate::storage::history::HistoryManager, vault: &mut crate::crypto::vault::PasswordManager) -> ImportReport {
    let mut r = ImportReport { source: id.into(), ..Default::default() };
    if id == "all" {
        let mut acc = ImportReport { source: "all".into(), ..Default::default() };
        for d in detect() {
            let one = apply_import(&d.id, bookmarks, history, vault);
            acc.bookmarks += one.bookmarks; acc.history += one.history; acc.passwords += one.passwords; acc.password_skip += one.password_skip;
            acc.notes.extend(one.notes);
        }
        return acc;
    }
    if id == "firefox" {
        let Some(places) = firefox_places().or_else(|| detect().into_iter().find(|d| d.id == "firefox").map(|d| PathBuf::from(d.path))) else { r.notes.push("Firefox not found".into()); return r; };
        let tmp = copy_unlocked(&places).unwrap_or(places.clone());
        for (t, u, f) in firefox_bookmark_rows(&tmp) {
            if bookmarks.find_by_url(&u).is_none() { bookmarks.add(&t, &u, f.as_deref()); r.bookmarks += 1; }
        }
        for (u, t, c) in firefox_history_rows(&tmp) { history.import_visit(&u, &t, c); r.history += 1; }
        if tmp != places { let _ = fs::remove_file(tmp); }
        r.notes.push("Firefox passwords stay in Firefox (NSS). Bookmarks + history imported.".into());
        return r;
    }
    let Some(root) = source_root(id) else { r.notes.push("unknown browser".into()); return r; };
    let prof = chromium_profile(&root);
    let bm = prof.join("Bookmarks");
    for (t, u, f) in import_chromium_bookmarks_file(&bm) {
        if bookmarks.find_by_url(&u).is_none() { bookmarks.add(&t, &u, f.as_deref()); r.bookmarks += 1; }
    }
    if let Some(tmp) = copy_unlocked(&prof.join("History")) {
        for (u, t, c) in chromium_history_rows(&tmp) { history.import_visit(&u, &t, c); r.history += 1; }
        let _ = fs::remove_file(&tmp);
    }
    if vault.is_unlocked() {
        if let Some(key) = chromium_master_key(&root.join("Local State")) {
            if let Some(tmp) = copy_unlocked(&prof.join("Login Data")) {
                let logins = chromium_logins(&tmp, &key);
                for (url, user, pass) in logins {
                    if pass.is_empty() { r.password_skip += 1; continue; }
                    match vault.add_credential(&url, &user, &pass, None, Some(id)) { Ok(_) => r.passwords += 1, Err(_) => r.password_skip += 1 }
                }
                let _ = fs::remove_file(tmp);
            }
        } else { r.notes.push("Could not unwrap Chromium OS key (close the other browser and retry).".into()); }
    } else { r.notes.push("Unlock the Amni vault to import saved passwords.".into()); }
    r
}
pub fn source_root(id: &str) -> Option<PathBuf> {
    match id {
        "chrome" => Some(chrome_user_data()),
        "edge" => Some(edge_user_data()),
        "brave" => Some(brave_user_data()),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chrome_bookmark_json_walks() {
        let j = r#"{"roots":{"bookmark_bar":{"type":"folder","name":"Bar","children":[{"type":"url","name":"GH","url":"https://github.com"},{"type":"folder","name":"News","children":[{"type":"url","name":"DDG","url":"https://duckduckgo.com"}]}]}}}"#;
        let v = parse_chromium_bookmarks(j);
        assert_eq!(v.len(), 2);
        assert!(v.iter().any(|x| x.1 == "https://github.com"));
        assert!(v.iter().any(|x| x.0 == "DDG" && x.2.as_deref() == Some("News")));
    }
    #[test]
    fn skips_non_http() {
        let j = r#"{"roots":{"other":{"type":"folder","name":"o","children":[{"type":"url","name":"x","url":"javascript:alert(1)"}]}}}"#;
        assert!(parse_chromium_bookmarks(j).is_empty());
    }
}
