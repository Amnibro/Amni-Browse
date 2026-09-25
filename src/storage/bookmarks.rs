use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;
use crate::storage::config::BrowserConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
    pub created_at: DateTime<Utc>,
    pub favicon: Option<String>,
}

impl Bookmark {
    pub fn new(title: &str, url: &str, folder: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            url: url.to_string(),
            folder: folder.map(|f| f.to_string()),
            created_at: Utc::now(),
            favicon: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkManager {
    pub bookmarks: Vec<Bookmark>,
    #[serde(default)]
    pub folders: Vec<String>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self::load()
    }

    fn file_path() -> std::path::PathBuf {
        BrowserConfig::config_dir().join("bookmarks.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(&path, data).ok();
        }
    }

    pub fn add(&mut self, title: &str, url: &str, folder: Option<&str>) -> Bookmark {
        let bookmark = Bookmark::new(title, url, folder);
        self.bookmarks.push(bookmark.clone());
        self.save();
        bookmark
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.bookmarks.len();
        self.bookmarks.retain(|b| b.id != id);
        let removed = self.bookmarks.len() < len_before;
        if removed {
            self.save();
        }
        removed
    }

    pub fn find_by_url(&self, url: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.url == url)
    }

    pub fn list_folder(&self, folder: Option<&str>) -> Vec<&Bookmark> {
        self.bookmarks
            .iter()
            .filter(|b| b.folder.as_deref() == folder)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Bookmark> {
        let q = query.to_lowercase();
        self.bookmarks
            .iter()
            .filter(|b| b.title.to_lowercase().contains(&q) || b.url.to_lowercase().contains(&q))
            .collect()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.bookmarks).unwrap_or_else(|_| "[]".to_string())
    }
    pub fn all_folders(&self) -> Vec<String> {
        let mut v: Vec<String> = self.folders.iter().cloned().chain(self.bookmarks.iter().filter_map(|b| b.folder.clone())).flat_map(|f| { let parts: Vec<&str> = f.split('/').collect(); (1..=parts.len()).map(move |n| parts[..n].join("/")).collect::<Vec<_>>() }).filter(|f| !f.trim().is_empty()).collect();
        v.sort_by_key(|f| f.to_lowercase());
        v.dedup();
        v
    }
    pub fn manager_json(&self) -> String { serde_json::json!({ "bookmarks": self.bookmarks, "folders": self.all_folders() }).to_string() }
    fn clean_folder(f: &str) -> Option<String> { let c = f.split('/').map(str::trim).filter(|p| !p.is_empty()).collect::<Vec<_>>().join("/"); (!c.is_empty()).then_some(c) }
    pub fn update(&mut self, id: &str, title: Option<&str>, url: Option<&str>, folder: Option<&str>) {
        if let Some(b) = self.bookmarks.iter_mut().find(|b| b.id == id) { if let Some(t) = title { b.title = t.to_string(); } if let Some(u) = url.filter(|u| !u.trim().is_empty()) { b.url = u.trim().to_string(); } if let Some(f) = folder { b.folder = Self::clean_folder(f); } }
        self.save();
    }
    pub fn move_to(&mut self, ids: &[&str], folder: &str) { let f = Self::clean_folder(folder); self.bookmarks.iter_mut().filter(|b| ids.contains(&b.id.as_str())).for_each(|b| b.folder = f.clone()); self.save(); }
    pub fn remove_many(&mut self, ids: &[&str]) { self.bookmarks.retain(|b| !ids.contains(&b.id.as_str())); self.save(); }
    pub fn reorder(&mut self, id: &str, before: Option<&str>) {
        let Some(from) = self.bookmarks.iter().position(|b| b.id == id) else { return };
        let b = self.bookmarks.remove(from);
        let to = before.and_then(|x| self.bookmarks.iter().position(|b| b.id == x)).unwrap_or(self.bookmarks.len());
        self.bookmarks.insert(to, b);
        self.save();
    }
    pub fn create_folder(&mut self, path: &str) { if let Some(f) = Self::clean_folder(path) { if !self.folders.contains(&f) { self.folders.push(f); } } self.save(); }
    fn renamed(p: &str, from: &str, to: &str) -> Option<String> { (p == from).then(|| to.to_string()).or_else(|| p.strip_prefix(&format!("{}/", from)).map(|r| format!("{}/{}", to, r))) }
    pub fn rename_folder(&mut self, from: &str, to: &str) {
        let Some(to) = Self::clean_folder(to) else { return };
        self.bookmarks.iter_mut().for_each(|b| if let Some(n) = b.folder.as_deref().and_then(|f| Self::renamed(f, from, &to)) { b.folder = Some(n); });
        self.folders = self.folders.iter().map(|f| Self::renamed(f, from, &to).unwrap_or_else(|| f.clone())).collect();
        self.save();
    }
    pub fn delete_folder(&mut self, path: &str, with_contents: bool) {
        let (parent, pre) = (path.rsplit_once('/').map(|(p, _)| p.to_string()), format!("{}/", path));
        let lift = |f: &str| -> Option<String> { match f.strip_prefix(&pre) { Some(rest) => Some(parent.as_ref().map(|p| format!("{}/{}", p, rest)).unwrap_or_else(|| rest.to_string())), None => (f == path).then(|| parent.clone()).flatten().or_else(|| (f != path).then(|| f.to_string())) } };
        if with_contents { self.bookmarks.retain(|b| !b.folder.as_deref().map(|f| f == path || f.starts_with(&pre)).unwrap_or(false)); } else { self.bookmarks.iter_mut().for_each(|b| if let Some(f) = b.folder.clone() { if f == path || f.starts_with(&pre) { b.folder = lift(&f); } }); }
        self.folders = self.folders.iter().filter(|f| *f != path && !(with_contents && f.starts_with(&pre))).filter_map(|f| lift(f)).collect();
        self.save();
    }
    pub fn export_html(&self) -> String {
        fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;") }
        fn emit(out: &mut String, m: &BookmarkManager, folder: Option<&str>, depth: usize) {
            let pad = "    ".repeat(depth);
            for f in m.all_folders().iter().filter(|f| f.rsplit_once('/').map(|(p, _)| Some(p)).unwrap_or(None) == folder) {
                out.push_str(&format!("{}<DT><H3>{}</H3>\n{}<DL><p>\n", pad, esc(f.rsplit('/').next().unwrap_or(f)), pad));
                emit(out, m, Some(f), depth + 1);
                out.push_str(&format!("{}</DL><p>\n", pad));
            }
            for b in m.bookmarks.iter().filter(|b| b.folder.as_deref() == folder) { out.push_str(&format!("{}<DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n", pad, esc(&b.url), b.created_at.timestamp(), esc(&b.title))); }
        }
        let mut out = String::from("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n<TITLE>Bookmarks</TITLE>\n<H1>Bookmarks</H1>\n<DL><p>\n");
        emit(&mut out, self, None, 1);
        out.push_str("</DL><p>\n");
        out
    }
    pub fn import_html(&mut self, html: &str) -> usize {
        let tag = regex::Regex::new(r#"(?is)<DT>\s*<H3[^>]*>(.*?)</H3>|<DT>\s*<A\s[^>]*?HREF="([^"]*)"[^>]*?(?:ADD_DATE="(\d+)")?[^>]*>(.*?)</A>|</DL>"#).unwrap();
        let un = |s: &str| s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&");
        let (mut stack, mut added): (Vec<String>, usize) = (Vec::new(), 0);
        for c in tag.captures_iter(html) {
            if let Some(h) = c.get(1) { stack.push(un(h.as_str().trim()).replace('/', "-")); let f = stack.join("/"); if !self.folders.contains(&f) { self.folders.push(f); } continue; }
            let Some(u) = c.get(2) else { stack.pop(); continue };
            let url = un(u.as_str());
            if !url.starts_with("http") && !url.starts_with("file:") { continue; }
            let folder = (!stack.is_empty()).then(|| stack.join("/"));
            if self.bookmarks.iter().any(|b| b.url == url && b.folder == folder) { continue; }
            let mut b = Bookmark::new(&un(c.get(4).map(|t| t.as_str()).unwrap_or("")), &url, folder.as_deref());
            if let Some(t) = c.get(3).and_then(|d| d.as_str().parse::<i64>().ok()).and_then(|d| DateTime::from_timestamp(d, 0)) { b.created_at = t; }
            self.bookmarks.push(b);
            added += 1;
        }
        self.save();
        added
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn m() -> BookmarkManager { let mut m = BookmarkManager::default(); m.bookmarks = vec![Bookmark::new("a", "https://a.test/", Some("Work")), Bookmark::new("b", "https://b.test/", Some("Work/Docs")), Bookmark::new("c", "https://c.test/", None)]; m }
    #[test]
    fn folders_include_parents_and_empty() { let mut m = m(); m.folders.push("Fun/Games".into()); assert_eq!(m.all_folders(), vec!["Fun", "Fun/Games", "Work", "Work/Docs"]); }
    #[test]
    fn rename_moves_subfolders() { let mut m = m(); m.rename_folder("Work", "Job"); assert_eq!(m.bookmarks[1].folder.as_deref(), Some("Job/Docs")); assert_eq!(m.bookmarks[0].folder.as_deref(), Some("Job")); }
    #[test]
    fn delete_folder_keeps_items_by_default() { let mut m = m(); m.delete_folder("Work", false); assert_eq!(m.bookmarks.len(), 3); assert_eq!(m.bookmarks[0].folder, None); assert_eq!(m.bookmarks[1].folder.as_deref(), Some("Docs")); }
    #[test]
    fn delete_folder_with_contents() { let mut m = m(); m.delete_folder("Work", true); assert_eq!(m.bookmarks.len(), 1); assert_eq!(m.bookmarks[0].title, "c"); }
    #[test]
    fn reorder_and_move() { let mut m = m(); let (a, c) = (m.bookmarks[0].id.clone(), m.bookmarks[2].id.clone()); m.reorder(&c, Some(&a)); assert_eq!(m.bookmarks[0].title, "c"); m.move_to(&[a.as_str()], " Fun / Stuff "); assert_eq!(m.bookmarks[1].folder.as_deref(), Some("Fun/Stuff")); }
    #[test]
    fn html_roundtrip() { let m1 = m(); let html = m1.export_html(); let mut m2 = BookmarkManager::default(); assert_eq!(m2.import_html(&html), 3); assert_eq!(m2.bookmarks.iter().find(|b| b.title == "b").and_then(|b| b.folder.clone()).as_deref(), Some("Work/Docs")); assert_eq!(m2.import_html(&html), 0); }
    #[test]
    fn imports_chrome_export() { let mut m = BookmarkManager::default(); let html = "<DL><p><DT><H3 ADD_DATE=\"1\">Bookmarks bar</H3><DL><p><DT><A HREF=\"https://x.test/?a=1&amp;b=2\" ADD_DATE=\"1700000000\" ICON=\"data:x\">X &amp; Co</A></DL><p><DT><A HREF=\"javascript:alert(1)\">bad</A></DL>"; assert_eq!(m.import_html(html), 1); assert_eq!(m.bookmarks[0].url, "https://x.test/?a=1&b=2"); assert_eq!(m.bookmarks[0].title, "X & Co"); assert_eq!(m.bookmarks[0].folder.as_deref(), Some("Bookmarks bar")); }
}
