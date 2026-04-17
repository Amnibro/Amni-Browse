use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::HashMap;
pub struct AmniDom {
    pub dom: RcDom,
}
pub struct DomNode<'a> {
    pub handle: &'a Handle,
}
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub charset: String,
    pub lang: String,
    pub links: Vec<String>,
    pub scripts: Vec<String>,
    pub stylesheets: Vec<String>,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub audios: Vec<String>,
    pub canvases: usize,
    pub headings: Vec<(u8, String)>,
    pub text_content: String,
    pub meta_tags: HashMap<String, String>,
    pub iframes: Vec<String>,
}
impl AmniDom {
    pub fn parse(html: &str) -> Self {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .expect("parse");
        Self { dom }
    }
    pub fn root(&self) -> &Handle { &self.dom.document }
    pub fn extract_meta(&self) -> PageMeta {
        let mut meta = PageMeta {
            title: String::new(), description: String::new(),
            charset: "UTF-8".to_string(), lang: String::new(),
            links: Vec::new(), scripts: Vec::new(), stylesheets: Vec::new(),
            images: Vec::new(), videos: Vec::new(), audios: Vec::new(), canvases: 0,
            headings: Vec::new(), text_content: String::new(),
            meta_tags: HashMap::new(), iframes: Vec::new(),
        };
        self.walk_meta(&self.dom.document, &mut meta);
        meta
    }
    fn walk_meta(&self, handle: &Handle, meta: &mut PageMeta) {
        self.walk_meta_inner(handle, meta, "");
    }
    fn walk_meta_inner(&self, handle: &Handle, meta: &mut PageMeta, parent_tag: &str) {
        match &handle.data {
            NodeData::Element { name, attrs, .. } => {
                let tag = name.local.to_string();
                let attrs_map: HashMap<String, String> = attrs.borrow().iter()
                    .map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
                match tag.as_str() {
                    "title" => meta.title = self.text_of(handle),
                    "meta" => {
                        if let Some(cs) = attrs_map.get("charset") { meta.charset = cs.clone(); }
                        let name_attr = attrs_map.get("name").cloned().unwrap_or_default();
                        let content = attrs_map.get("content").cloned().unwrap_or_default();
                        if !name_attr.is_empty() && !content.is_empty() {
                            if name_attr == "description" { meta.description = content.clone(); }
                            meta.meta_tags.insert(name_attr, content);
                        }
                    }
                    "link" => {
                        let rel = attrs_map.get("rel").cloned().unwrap_or_default();
                        if let Some(href) = attrs_map.get("href") {
                            if rel == "stylesheet" { meta.stylesheets.push(href.clone()); }
                            meta.links.push(href.clone());
                        }
                    }
                    "script" => { if let Some(src) = attrs_map.get("src") { meta.scripts.push(src.clone()); } }
                    "img" => { if let Some(src) = attrs_map.get("src") { meta.images.push(src.clone()); } }
                    "a" => { if let Some(href) = attrs_map.get("href") { meta.links.push(href.clone()); } }
                    "html" => { if let Some(l) = attrs_map.get("lang") { meta.lang = l.clone(); } }
                    "video" => { if let Some(src) = attrs_map.get("src") { meta.videos.push(src.clone()); } }
                    "audio" => { if let Some(src) = attrs_map.get("src") { meta.audios.push(src.clone()); } }
                    "source" => {
                        if let Some(src) = attrs_map.get("src") {
                            match parent_tag {
                                "video" => meta.videos.push(src.clone()),
                                "audio" => meta.audios.push(src.clone()),
                                _ => {}
                            }
                        }
                    }
                    "canvas" => { meta.canvases += 1; }
                    "iframe" => { if let Some(src) = attrs_map.get("src") { meta.iframes.push(src.clone()); } }
                    t if t.starts_with('h') && t.len() == 2 => {
                        if let Some(level) = t.chars().nth(1).and_then(|c| c.to_digit(10)) {
                            if (1..=6).contains(&level) { meta.headings.push((level as u8, self.text_of(handle))); }
                        }
                    }
                    _ => {}
                }
                let cur_tag = tag.as_str();
                let pass_tag = match cur_tag { "video" | "audio" => cur_tag, _ => parent_tag };
                for child in handle.children.borrow().iter() { self.walk_meta_inner(child, meta, pass_tag); }
                return;
            }
            NodeData::Text { contents } => { meta.text_content.push_str(&contents.borrow()); meta.text_content.push(' '); }
            _ => {}
        }
        for child in handle.children.borrow().iter() { self.walk_meta_inner(child, meta, parent_tag); }
    }
    fn text_of(&self, handle: &Handle) -> String {
        let mut out = String::new();
        self.collect_text(handle, &mut out);
        out.trim().to_string()
    }
    fn collect_text(&self, handle: &Handle, out: &mut String) {
        match &handle.data {
            NodeData::Text { contents } => out.push_str(&contents.borrow()),
            _ => {}
        }
        for child in handle.children.borrow().iter() { self.collect_text(child, out); }
    }
    pub fn extract_reader_content(&self) -> (String, String) {
        let meta = self.extract_meta();
        let mut content = String::new();
        self.extract_article(&self.dom.document, &mut content, 0);
        let clean = content.trim().to_string();
        (meta.title, if clean.is_empty() { meta.text_content.trim().to_string() } else { clean })
    }
    fn extract_article(&self, handle: &Handle, out: &mut String, depth: usize) {
        if depth > 100 { return; }
        match &handle.data {
            NodeData::Element { name, attrs, .. } => {
                let tag = name.local.to_string();
                let attrs_map: HashMap<String, String> = attrs.borrow().iter()
                    .map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
                let dominated = matches!(tag.as_str(), "nav"|"header"|"footer"|"aside"|"script"|"style"|"noscript"|"iframe"|"form");
                if dominated { return; }
                let id = attrs_map.get("id").cloned().unwrap_or_default();
                let class = attrs_map.get("class").cloned().unwrap_or_default();
                let is_article = tag == "article" || tag == "main" || id.contains("content") || id.contains("article") || class.contains("article") || class.contains("post-content") || class.contains("entry-content");
                if is_article || tag == "body" {
                    for child in handle.children.borrow().iter() { self.extract_article(child, out, depth + 1); }
                    return;
                }
                match tag.as_str() {
                    "p" => { out.push_str("<p>"); self.serialize_inline(handle, out); out.push_str("</p>\n"); }
                    t if t.starts_with('h') && t.len() == 2 => {
                        out.push_str(&format!("<{}>", t)); self.serialize_inline(handle, out); out.push_str(&format!("</{}>\n", t));
                    }
                    "blockquote" => { out.push_str("<blockquote>"); self.serialize_inline(handle, out); out.push_str("</blockquote>\n"); }
                    "pre" | "code" => { out.push_str("<pre><code>"); self.serialize_inline(handle, out); out.push_str("</code></pre>\n"); }
                    "ul" | "ol" => { out.push_str(&format!("<{}>", tag)); for child in handle.children.borrow().iter() { self.extract_article(child, out, depth + 1); } out.push_str(&format!("</{}>\n", tag)); }
                    "li" => { out.push_str("<li>"); self.serialize_inline(handle, out); out.push_str("</li>\n"); }
                    "img" => {
                        let src = attrs_map.get("src").cloned().unwrap_or_default();
                        let alt = attrs_map.get("alt").cloned().unwrap_or_default();
                        if !src.is_empty() { out.push_str(&format!("<figure><img src=\"{}\" alt=\"{}\"></figure>\n", src, alt)); }
                    }
                    _ => { for child in handle.children.borrow().iter() { self.extract_article(child, out, depth + 1); } }
                }
            }
            NodeData::Text { contents } => out.push_str(&contents.borrow()),
            _ => { for child in handle.children.borrow().iter() { self.extract_article(child, out, depth + 1); } }
        }
    }
    fn serialize_inline(&self, handle: &Handle, out: &mut String) {
        for child in handle.children.borrow().iter() {
            match &child.data {
                NodeData::Text { contents } => out.push_str(&contents.borrow()),
                NodeData::Element { name, attrs, .. } => {
                    let tag = name.local.to_string();
                    match tag.as_str() {
                        "a" => {
                            let href = attrs.borrow().iter().find(|a| a.name.local.to_string() == "href").map(|a| a.value.to_string()).unwrap_or_default();
                            out.push_str(&format!("<a href=\"{}\">", href));
                            self.serialize_inline(child, out);
                            out.push_str("</a>");
                        }
                        "strong" | "b" => { out.push_str("<strong>"); self.serialize_inline(child, out); out.push_str("</strong>"); }
                        "em" | "i" => { out.push_str("<em>"); self.serialize_inline(child, out); out.push_str("</em>"); }
                        "code" => { out.push_str("<code>"); self.serialize_inline(child, out); out.push_str("</code>"); }
                        "br" => out.push_str("<br>"),
                        _ => self.serialize_inline(child, out),
                    }
                }
                _ => {}
            }
        }
    }
    pub fn query_by_tag(&self, tag: &str) -> Vec<Handle> {
        let mut results = Vec::new();
        self.find_by_tag(&self.dom.document, tag, &mut results);
        results
    }
    fn find_by_tag(&self, handle: &Handle, tag: &str, results: &mut Vec<Handle>) {
        if let NodeData::Element { name, .. } = &handle.data {
            if name.local.to_string() == tag { results.push(handle.clone()); }
        }
        for child in handle.children.borrow().iter() { self.find_by_tag(child, tag, results); }
    }
    pub fn query_by_id(&self, id: &str) -> Option<Handle> {
        self.find_by_id(&self.dom.document, id)
    }
    fn find_by_id(&self, handle: &Handle, id: &str) -> Option<Handle> {
        if let NodeData::Element { attrs, .. } = &handle.data {
            if attrs.borrow().iter().any(|a| a.name.local.to_string() == "id" && a.value.to_string() == id) { return Some(handle.clone()); }
        }
        for child in handle.children.borrow().iter() { if let Some(found) = self.find_by_id(child, id) { return Some(found); } }
        None
    }
    pub fn query_selector(&self, selector: &str) -> Option<Handle> {
        self.find_first_simple(&self.dom.document, selector.trim())
    }
    pub fn query_selector_all(&self, selector: &str) -> Vec<Handle> {
        let mut results = Vec::new();
        let parts: Vec<&str> = selector.split_whitespace().collect();
        if parts.len() == 1 {
            self.find_all_simple(&self.dom.document, parts[0], &mut results);
        } else if parts.len() >= 2 {
            let ancestor_sel = parts[..parts.len() - 1].join(" ");
            let final_sel = parts[parts.len() - 1];
            let ancestors = self.query_selector_all(&ancestor_sel);
            for anc in &ancestors {
                for child in anc.children.borrow().iter() {
                    self.find_all_simple(child, final_sel, &mut results);
                }
            }
        }
        results
    }
    pub fn query_by_class(&self, class: &str) -> Vec<Handle> {
        let mut results = Vec::new();
        self.find_by_class(&self.dom.document, class, &mut results);
        results
    }
    pub fn query_by_attr(&self, attr: &str, value: &str) -> Vec<Handle> {
        let mut results = Vec::new();
        self.find_by_attr_val(&self.dom.document, attr, value, &mut results);
        results
    }
    fn find_first_simple(&self, handle: &Handle, sel: &str) -> Option<Handle> {
        if Self::matches_simple(handle, sel) { return Some(handle.clone()); }
        for child in handle.children.borrow().iter() {
            if let Some(found) = self.find_first_simple(child, sel) { return Some(found); }
        }
        None
    }
    fn find_all_simple(&self, handle: &Handle, sel: &str, results: &mut Vec<Handle>) {
        if Self::matches_simple(handle, sel) { results.push(handle.clone()); }
        for child in handle.children.borrow().iter() { self.find_all_simple(child, sel, results); }
    }
    fn matches_simple(handle: &Handle, sel: &str) -> bool {
        if let NodeData::Element { name, attrs, .. } = &handle.data {
            let tag = name.local.to_string();
            let attrs_map: HashMap<String, String> = attrs.borrow().iter()
                .map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
            let id_attr = attrs_map.get("id").cloned().unwrap_or_default();
            let class_attr = attrs_map.get("class").cloned().unwrap_or_default();
            let classes: Vec<&str> = class_attr.split_whitespace().collect();
            if sel == "*" { return true; }
            if sel.starts_with('#') { return &sel[1..] == id_attr; }
            if sel.starts_with('.') { return classes.contains(&&sel[1..]); }
            if sel.contains('#') {
                let p: Vec<&str> = sel.splitn(2, '#').collect();
                return (p[0].is_empty() || p[0] == tag) && p[1] == id_attr;
            }
            if sel.contains('.') {
                let p: Vec<&str> = sel.splitn(2, '.').collect();
                return (p[0].is_empty() || p[0] == tag) && classes.contains(&p[1]);
            }
            if sel.contains('[') && sel.contains(']') {
                let inner = &sel[sel.find('[').unwrap() + 1..sel.find(']').unwrap()];
                let tag_part = &sel[..sel.find('[').unwrap()];
                let tag_ok = tag_part.is_empty() || tag_part == tag;
                if inner.contains('=') {
                    let eq: Vec<&str> = inner.splitn(2, '=').collect();
                    let a_val = eq[1].trim_matches(|c| c == '"' || c == '\'');
                    return tag_ok && attrs_map.get(eq[0]).map_or(false, |v| v == a_val);
                }
                return tag_ok && attrs_map.contains_key(inner);
            }
            return tag == sel;
        }
        false
    }
    fn find_by_class(&self, handle: &Handle, class: &str, results: &mut Vec<Handle>) {
        if let NodeData::Element { attrs, .. } = &handle.data {
            let ca = attrs.borrow().iter().find(|a| a.name.local.to_string() == "class")
                .map(|a| a.value.to_string()).unwrap_or_default();
            if ca.split_whitespace().any(|c| c == class) { results.push(handle.clone()); }
        }
        for child in handle.children.borrow().iter() { self.find_by_class(child, class, results); }
    }
    fn find_by_attr_val(&self, handle: &Handle, attr: &str, value: &str, results: &mut Vec<Handle>) {
        if let NodeData::Element { attrs, .. } = &handle.data {
            if attrs.borrow().iter().any(|a| a.name.local.to_string() == attr && a.value.to_string() == value) {
                results.push(handle.clone());
            }
        }
        for child in handle.children.borrow().iter() { self.find_by_attr_val(child, attr, value, results); }
    }
}
