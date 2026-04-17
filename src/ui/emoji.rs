use std::collections::HashMap;
use std::sync::LazyLock;
static ATLAS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("back", "\u{25C0}");
    m.insert("forward", "\u{25B6}");
    m.insert("refresh", "\u{21BB}");
    m.insert("home", "\u{2302}");
    m.insert("star_empty", "\u{2606}");
    m.insert("star_filled", "\u{2733}");
    m.insert("star_solid", "\u{2605}");
    m.insert("shield", "\u{1F6E1}\u{FE0F}");
    m.insert("split", "\u{2AFF}");
    m.insert("key", "\u{1F510}");
    m.insert("palette", "\u{1F3A8}");
    m.insert("download", "\u{1F4E5}");
    m.insert("clock", "\u{1F550}");
    m.insert("book", "\u{1F4D6}");
    m.insert("menu", "\u{2630}");
    m.insert("close", "\u{00D7}");
    m.insert("up", "\u{25B2}");
    m.insert("down", "\u{25BC}");
    m.insert("plus", "+");
    m.insert("lock", "\u{1F512}");
    m.insert("unlock", "\u{1F513}");
    m.insert("search", "\u{1F50D}");
    m.insert("gear", "\u{2699}\u{FE0F}");
    m.insert("trash", "\u{1F5D1}\u{FE0F}");
    m.insert("copy", "\u{1F4CB}");
    m.insert("clipboard", "\u{1F4CB}");
    m.insert("check", "\u{2705}");
    m.insert("cross", "\u{274C}");
    m.insert("warning", "\u{26A0}\u{FE0F}");
    m.insert("no_entry", "\u{1F6AB}");
    m.insert("private", "\u{1F576}\u{FE0F}");
    m.insert("wrench", "\u{1F527}");
    m.insert("puzzle", "\u{1F9E9}");
    m.insert("person", "\u{1F464}");
    m.insert("memo", "\u{1F4DD}");
    m.insert("floppy", "\u{1F4BE}");
    m.insert("chart", "\u{1F4CA}");
    m.insert("globe", "\u{1F310}");
    m.insert("link", "\u{1F517}");
    m.insert("pin", "\u{1F4CC}");
    m.insert("sparkles", "\u{2728}");
    m.insert("fire", "\u{1F525}");
    m.insert("rocket", "\u{1F680}");
    m.insert("bolt", "\u{26A1}");
    m.insert("diamond", "\u{1F48E}");
    m.insert("crown", "\u{1F451}");
    m.insert("broom", "\u{1F9F9}");
    m.insert("inbox", "\u{1F4E5}");
    m.insert("outbox", "\u{1F4E4}");
    m.insert("folder", "\u{1F4C1}");
    m.insert("file", "\u{1F4C4}");
    m.insert("xr", "\u{1F97D}");
    m.insert("middot", "\u{00B7}");
    m.insert("emdash", "\u{2014}");
    m.insert("arrow_left", "\u{2190}");
    m.insert("arrow_right", "\u{2192}");
    m.insert("pause", "\u{23F9}");
    m.insert("play", "\u{25B6}");
    m.insert("stop", "\u{23F9}");
    m.insert("new_doc", "\u{1F4C4}");
    m.insert("reset", "\u{21BA}");
    m
});
static CUSTOM: LazyLock<std::sync::RwLock<HashMap<String, String>>> = LazyLock::new(|| std::sync::RwLock::new(HashMap::new()));
pub fn e(name: &str) -> &str {
    ATLAS.get(name).copied().unwrap_or(name)
}
pub fn eh(name: &str) -> String {
    let s = e(name);
    s.chars().map(|c| if c as u32 > 127 { format!("&#{};", c as u32) } else { c.to_string() }).collect()
}
pub fn register(name: String, glyph: String) {
    if let Ok(mut w) = CUSTOM.write() { w.insert(name, glyph); }
}
pub fn resolve(name: &str) -> String {
    if let Ok(r) = CUSTOM.read() { if let Some(v) = r.get(name) { return v.clone(); } }
    e(name).to_string()
}
pub fn resolve_html(name: &str) -> String {
    let s = resolve(name);
    s.chars().map(|c| if c as u32 > 127 { format!("&#{};", c as u32) } else { c.to_string() }).collect()
}
pub fn all_names() -> Vec<&'static str> {
    ATLAS.keys().copied().collect()
}
