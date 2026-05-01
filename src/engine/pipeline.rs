use crate::net::http::AmniClient;
use crate::net::cookies::CookieJar;
use crate::net::cors::CorsEnforcer;
use crate::net::csp::{CspEnforcer, CspDirective};
use crate::engine::dom::AmniDom;
use crate::engine::style::{StyleSheet, ComputedStyle};
use crate::engine::layout::{LayoutEngine, LayoutRect, TextInfo};
use crate::engine::paint::{RenderTree, DisplayList, SoftwareRenderer, build_display_list};
use crate::engine::image_decode::ImageCache;
use crate::engine::events::{EventDispatcher, FocusManager, HitTester, DomEvent, EventType};
use crate::engine::forms::FormState;
use markup5ever_rcdom::{Handle, NodeData};
use std::collections::HashMap;
use std::sync::RwLock;
pub struct RenderPipeline {
    pub client: AmniClient,
    pub cookies: RwLock<CookieJar>,
    pub image_cache: ImageCache,
    pub cors: CorsEnforcer,
    pub csp: Option<CspEnforcer>,
    pub canvas_registry: HashMap<usize, crate::engine::canvas::Canvas2D>,
}
pub struct PageResult {
    pub url: String,
    pub title: String,
    pub html: String,
    pub meta: PageSummary,
    pub css_sources: Vec<String>,
}
pub struct PageSummary {
    pub description: String,
    pub lang: String,
    pub charset: String,
    pub link_count: usize,
    pub image_count: usize,
    pub script_count: usize,
    pub stylesheet_count: usize,
    pub heading_count: usize,
    pub text_length: usize,
}
pub struct LayoutResult {
    pub rects: HashMap<usize, LayoutRect>,
    pub node_count: usize,
    pub viewport_w: f32,
    pub viewport_h: f32,
}
impl RenderPipeline {
    pub fn new() -> Self {
        Self {
            client: AmniClient::new(),
            cookies: RwLock::new(CookieJar::new(true)),
            image_cache: ImageCache::new(200),
            cors: CorsEnforcer::default_permissive(),
            csp: None,
            canvas_registry: HashMap::new(),
        }
    }
    pub async fn fetch_and_parse(&self, url: &str) -> Result<PageResult, String> {
        let parsed_url = url::Url::parse(url).map_err(|e| format!("url parse: {}", e))?;
        let domain = parsed_url.host_str().unwrap_or("").to_string();
        let path = parsed_url.path().to_string();
        let is_secure = parsed_url.scheme() == "https";
        let cookie_hdr = self.cookies.read().ok().and_then(|jar| jar.cookie_header(&domain, &path, is_secure));
        let extra: Vec<(&str, &str)> = cookie_hdr.as_ref().map(|h| vec![("Cookie", h.as_str())]).unwrap_or_default();
        let resp = self.client.get_with_headers(url, &extra).await.map_err(|e| format!("fetch: {}", e))?;
        let status = resp.status;
        let resp_headers: Vec<(String, String)> = resp.headers.clone();
        for (k, v) in &resp_headers {
            if k.eq_ignore_ascii_case("set-cookie") {
                if let Some(cookie) = CookieJar::parse_set_cookie(v, &domain) {
                    if let Ok(mut jar) = self.cookies.write() {
                        jar.set_cookie(cookie, &domain);
                    }
                }
            }
        }
        if resp.is_redirect() {
            if let Some(loc) = resp.redirect_url() {
                return Box::pin(self.fetch_and_parse(&loc)).await;
            }
        }
        if status >= 400 {
            return Err(format!("HTTP {}", status));
        }
        let csp_enforcer = CspEnforcer::from_headers(url, &resp_headers);
        let body = resp.text().map_err(|e| format!("decode: {}", e))?;
        let (summary, title, stylesheet_hrefs) = {
            let dom = AmniDom::parse(&body);
            let meta_raw = dom.extract_meta();
            let s = PageSummary {
                description: meta_raw.description.clone(),
                lang: meta_raw.lang.clone(),
                charset: meta_raw.charset.clone(),
                link_count: meta_raw.links.len(),
                image_count: meta_raw.images.len(),
                script_count: meta_raw.scripts.len(),
                stylesheet_count: meta_raw.stylesheets.len(),
                heading_count: meta_raw.headings.len(),
                text_length: meta_raw.text_content.len(),
            };
            let t = if meta_raw.title.is_empty() { url.to_string() } else { meta_raw.title.clone() };
            let hrefs = meta_raw.stylesheets.clone();
            (s, t, hrefs)
        };
        let base = url::Url::parse(url).ok();
        let mut css_sources = Vec::new();
        for href in &stylesheet_hrefs {
            let resolved = base.as_ref()
                .and_then(|b| b.join(href).ok())
                .map(|u| u.to_string())
                .unwrap_or_else(|| href.clone());
            let csp_check = csp_enforcer.check_resource("style", &resolved);
            if !csp_check.allowed {
                log::warn!("CSP blocked style: {}", resolved);
                continue;
            }
            match self.client.get(&resolved).await {
                Ok(css_resp) => {
                    if let Ok(css_text) = css_resp.text() { css_sources.push(css_text); }
                }
                Err(e) => log::warn!("CSS fetch failed {}: {}", resolved, e),
            }
        }
        Ok(PageResult { url: url.to_string(), title, html: body, meta: summary, css_sources })
    }
    pub async fn fetch_reader(&self, url: &str) -> Result<(String, String), String> {
        let resp = self.client.get(url).await.map_err(|e| format!("fetch: {}", e))?;
        if resp.is_redirect() {
            if let Some(loc) = resp.redirect_url() {
                return Box::pin(self.fetch_reader(&loc)).await;
            }
        }
        let body = resp.text().map_err(|e| format!("decode: {}", e))?;
        Ok(AmniDom::parse(&body).extract_reader_content())
    }
    pub fn parse_and_layout(html: &str, css_sources: &[&str], vw: f32, vh: f32) -> LayoutResult {
        let dom = AmniDom::parse(html);
        let mut sheets = Vec::new();
        for src in css_sources { sheets.push(StyleSheet::parse(src)); }
        let mut engine = LayoutEngine::new();
        let mut styles: HashMap<usize, ComputedStyle> = HashMap::new();
        let mut id_counter = 0usize;
        Self::build_tree(&dom.dom.document, &sheets, &mut engine, &mut styles, &mut id_counter, None);
        if id_counter > 0 { engine.compute(0, vw, vh); }
        let ids: Vec<usize> = (0..id_counter).collect();
        engine.collect_all(&ids);
        let mut rects = HashMap::new();
        for id in 0..id_counter {
            if let Some(r) = engine.get_layout(id) { rects.insert(id, r.clone()); }
        }
        LayoutResult { rects, node_count: id_counter, viewport_w: vw, viewport_h: vh }
    }
    fn build_tree(handle: &Handle, sheets: &[StyleSheet], engine: &mut LayoutEngine, styles: &mut HashMap<usize, ComputedStyle>, counter: &mut usize, parent_cs: Option<&ComputedStyle>) {
        let my_id = *counter;
        *counter += 1;
        let mut cs = ComputedStyle::default();
        cs.font_size = parent_cs.map(|p| p.font_size).unwrap_or(16.0);
        cs.line_height = parent_cs.map(|p| p.line_height).unwrap_or(1.2);
        cs.color = parent_cs.map(|p| p.color.clone()).unwrap_or(crate::engine::style::Color { r: 0, g: 0, b: 0, a: 1.0 });
        cs.opacity = 1.0;
        cs.flex_shrink = 1.0;
        let mut leaf_text = String::new();
        if let NodeData::Element { name, attrs, .. } = &handle.data {
            let tag = name.local.to_string();
            if matches!(tag.as_str(), "head" | "style" | "script" | "meta" | "link" | "title" | "noscript" | "template" | "base") {
                cs.display = crate::engine::style::Display::None;
            }
            let attrs_map: HashMap<String, String> = attrs.borrow().iter()
                .map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
            let id_attr = attrs_map.get("id").cloned().unwrap_or_default();
            let class_attr = attrs_map.get("class").cloned().unwrap_or_default();
            let classes: Vec<&str> = class_attr.split_whitespace().collect();
            for sheet in sheets {
                for rule in &sheet.rules {
                    if Self::selector_matches(&rule.selectors, &tag, &id_attr, &classes) {
                        cs.apply_declarations(&rule.declarations);
                    }
                }
            }
            if let Some(style_attr) = attrs_map.get("style") {
                let inline_sheet = StyleSheet::parse(&format!("_inline {{ {} }}", style_attr));
                for rule in &inline_sheet.rules {
                    cs.apply_declarations(&rule.declarations);
                }
            }
            Self::apply_tag_font_defaults(&tag, &mut cs);
        } else if let NodeData::Text { contents } = &handle.data {
            leaf_text = contents.borrow().trim().to_string();
        }
        let mut child_ids = Vec::new();
        for child in handle.children.borrow().iter() {
            let child_id = *counter;
            Self::build_tree(child, sheets, engine, styles, counter, Some(&cs));
            child_ids.push(child_id);
        }
        styles.insert(my_id, cs.clone());
        if child_ids.is_empty() {
            if !leaf_text.is_empty() {
                engine.add_leaf_with_text(my_id, &cs, TextInfo { text: leaf_text, font_size: cs.font_size, line_height: cs.line_height });
            } else {
                engine.add_leaf(my_id, &cs);
            }
        } else {
            engine.add_node(my_id, &cs, &child_ids);
        }
    }
    fn apply_tag_font_defaults(tag: &str, cs: &mut ComputedStyle) {
        match tag {
            "h1" => { cs.font_size = 32.0; cs.font_weight = 700; }
            "h2" => { cs.font_size = 24.0; cs.font_weight = 700; }
            "h3" => { cs.font_size = 20.0; cs.font_weight = 700; }
            "h4" => { cs.font_size = 16.0; cs.font_weight = 700; }
            "h5" => { cs.font_size = 14.0; cs.font_weight = 700; }
            "h6" => { cs.font_size = 12.0; cs.font_weight = 700; }
            "strong" | "b" => { cs.font_weight = 700; }
            _ => {}
        }
    }
    fn selector_matches(selectors: &[String], tag: &str, id: &str, classes: &[&str]) -> bool {
        selectors.iter().any(|sel| {
            let sel = sel.trim();
            if sel == tag || sel == "*" { return true; }
            if sel.starts_with('#') && &sel[1..] == id { return true; }
            if sel.starts_with('.') && classes.contains(&&sel[1..]) { return true; }
            if sel.contains('.') {
                let parts: Vec<&str> = sel.splitn(2, '.').collect();
                if parts.len() == 2 && (parts[0].is_empty() || parts[0] == tag) && classes.contains(&parts[1]) { return true; }
            }
            false
        })
    }
    pub fn render_to_semantic_html(html: &str) -> String {
        let dom = AmniDom::parse(html);
        let (title, content) = dom.extract_reader_content();
        format!("<html><head><title>{}</title></head><body>{}</body></html>", title, content)
    }
    pub fn extract_page_meta(html: &str) -> PageSummary {
        let dom = AmniDom::parse(html);
        let m = dom.extract_meta();
        PageSummary {
            description: m.description, lang: m.lang, charset: m.charset,
            link_count: m.links.len(), image_count: m.images.len(),
            script_count: m.scripts.len(), stylesheet_count: m.stylesheets.len(),
            heading_count: m.headings.len(), text_length: m.text_content.len(),
        }
    }
    pub async fn execute_fetch_requests(&self, requests: Vec<crate::engine::js_bridge::FetchRequest>) -> Vec<(u32, u16, HashMap<String, String>, String)> {
        let mut results = Vec::new();
        for req in requests {
            let origin = url::Url::parse(&req.url)
                .ok()
                .map(|u| format!("{}://{}", u.scheme(), u.host_str().unwrap_or("")))
                .unwrap_or_default();
            let req_headers: Vec<(String, String)> = req.headers.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let cors_result = self.cors.should_allow(&origin, &req.url, &req.method, &req_headers);
            if !cors_result.allowed {
                let mut h = HashMap::new();
                h.insert("x-amni-cors-blocked".to_string(), "true".to_string());
                results.push((req.id, 0, h, String::new()));
                continue;
            }
            let parsed = url::Url::parse(&req.url).ok();
            let domain = parsed.as_ref().and_then(|u| u.host_str()).unwrap_or("").to_string();
            let path = parsed.as_ref().map(|u| u.path().to_string()).unwrap_or_else(|| "/".to_string());
            let is_secure = parsed.as_ref().map(|u| u.scheme() == "https").unwrap_or(false);
            let cookie_hdr = self.cookies.read().ok().and_then(|jar| jar.cookie_header(&domain, &path, is_secure));
            let mut extra: Vec<(&str, &str)> = req.headers.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let cookie_val;
            if let Some(ref c) = cookie_hdr {
                cookie_val = c.clone();
                extra.push(("Cookie", &cookie_val));
            }
            let resp = match req.method.to_uppercase().as_str() {
                "POST" => {
                    let ct = req.headers.get("Content-Type").or_else(|| req.headers.get("content-type"))
                        .cloned().unwrap_or_else(|| "application/x-www-form-urlencoded".to_string());
                    let body_bytes = bytes::Bytes::from(req.body.unwrap_or_default());
                    self.client.post(&req.url, &ct, body_bytes).await
                }
                _ => self.client.get_with_headers(&req.url, &extra).await,
            };
            match resp {
                Ok(r) => {
                    for (k, v) in &r.headers {
                        if k.eq_ignore_ascii_case("set-cookie") {
                            if let Some(cookie) = CookieJar::parse_set_cookie(v, &domain) {
                                if let Ok(mut jar) = self.cookies.write() {
                                    jar.set_cookie(cookie, &domain);
                                }
                            }
                        }
                    }
                    let mut resp_headers = HashMap::new();
                    for (k, v) in &r.headers {
                        resp_headers.insert(k.clone(), v.clone());
                    }
                    let body_text = r.text().unwrap_or_default();
                    results.push((req.id, r.status, resp_headers, body_text));
                }
                Err(_) => {
                    results.push((req.id, 0, HashMap::new(), String::new()));
                }
            }
        }
        results
    }
    pub async fn fetch_full_layout(&self, url: &str, vw: f32, vh: f32) -> Result<(PageResult, LayoutResult), String> {
        let page = self.fetch_and_parse(url).await?;
        let css_refs: Vec<&str> = page.css_sources.iter().map(|s| s.as_str()).collect();
        let layout = Self::parse_and_layout(&page.html, &css_refs, vw, vh);
        Ok((page, layout))
    }

    pub fn render_to_pixels(&mut self, html: &str, css_sources: &[&str], vw: f32, vh: f32) -> RenderedPage {
        let dom = AmniDom::parse(html);
        let mut sheets = Vec::new();
        for src in css_sources { sheets.push(StyleSheet::parse(src)); }
        let mut engine = LayoutEngine::new();
        let mut styles: HashMap<usize, ComputedStyle> = HashMap::new();
        let mut id_counter = 0usize;
        Self::build_tree(&dom.dom.document, &sheets, &mut engine, &mut styles, &mut id_counter, None);
        if id_counter > 0 { engine.compute(0, vw, vh); }
        let ids: Vec<usize> = (0..id_counter).collect();
        engine.collect_all(&ids);
        let mut rects = HashMap::new();
        for id in 0..id_counter {
            if let Some(r) = engine.get_layout(id) { rects.insert(id, r.clone()); }
        }
        let mut rt_counter = 0usize;
        let render_tree = RenderTree::build_from_dom(&dom.dom.document, &sheets, &mut rt_counter);
        let content_h = rects.values().map(|r| r.y + r.h).fold(vh, f32::max).min(16384.0).max(vh);
        let w = vw as u32;
        let h = content_h as u32;
        let mut dl = DisplayList::new(w, h);
        build_display_list(&render_tree, &rects, &self.image_cache, render_tree.root_id, 0.0, 0.0, &mut dl);
        let mut renderer = SoftwareRenderer::new(w, h);
        renderer.render_with_resources(&dl, &self.canvas_registry);
        RenderedPage {
            pixels: renderer.pixels().to_vec(),
            width: w,
            height: h,
            node_count: id_counter,
            command_count: dl.commands.len(),
            layouts: rects,
        }
    }

    pub async fn fetch_and_render(&mut self, url: &str, vw: f32, vh: f32) -> Result<(PageResult, RenderedPage), String> {
        let page = self.fetch_and_parse(url).await?;
        let base = url::Url::parse(url).ok();
        let meta = AmniDom::parse(&page.html).extract_meta();
        for img_url in &meta.images {
            let resolved = base.as_ref()
                .and_then(|b| b.join(img_url).ok())
                .map(|u| u.to_string())
                .unwrap_or_else(|| img_url.clone());
            if !self.image_cache.contains(&resolved) {
                match self.client.get(&resolved).await {
                    Ok(resp) => {
                        let _ = self.image_cache.decode_and_resize(&resolved, &resp.body, 1024);
                    }
                    Err(e) => log::warn!("Image fetch failed {}: {}", resolved, e),
                }
            }
        }
        let css_refs: Vec<&str> = page.css_sources.iter().map(|s| s.as_str()).collect();
        let rendered = self.render_to_pixels(&page.html, &css_refs, vw, vh);
        Ok((page, rendered))
    }
}

pub struct RenderedPage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub node_count: usize,
    pub command_count: usize,
    pub layouts: HashMap<usize, LayoutRect>,
}

pub struct PageInteractor {
    pub event_dispatcher: EventDispatcher,
    pub focus_manager: FocusManager,
    pub form_state: FormState,
    pub js_bridge: crate::engine::js_bridge::JsDomBridge,
    pub current_layouts: HashMap<usize, LayoutRect>,
}
impl PageInteractor {
    pub fn new() -> Self {
        Self {
            event_dispatcher: EventDispatcher::new(),
            focus_manager: FocusManager::new(),
            form_state: FormState::new(),
            js_bridge: crate::engine::js_bridge::JsDomBridge::new(),
            current_layouts: HashMap::new(),
        }
    }
    pub fn dispatch_click(&mut self, x: f32, y: f32) -> Option<usize> {
        let target = HitTester::hit_test(x, y, &self.current_layouts)?;
        let mut event = DomEvent::mouse(EventType::Click, target, x, y, 0);
        self.event_dispatcher.dispatch(&mut event);
        Some(target)
    }
    pub fn dispatch_key(&mut self, key: &str, code: u32, shift: bool, ctrl: bool, alt: bool) {
        if let Some(focused) = self.focus_manager.current_focus {
            let mut event = DomEvent::keyboard(EventType::KeyDown, focused, key, code, shift, ctrl, alt, false);
            self.event_dispatcher.dispatch(&mut event);
            let fid = focused.to_string();
            self.form_state.handle_key_input(&fid, key, shift, ctrl);
        }
    }
    pub fn focus_node(&mut self, node_id: usize) {
        self.focus_manager.focus(node_id, &mut self.event_dispatcher);
    }
    pub fn tab_focus_next(&mut self) {
        self.focus_manager.tab_next(&mut self.event_dispatcher);
    }
    pub fn submit_form(&self, form_id: &str) -> HashMap<String, String> {
        self.form_state.submit_form(form_id)
    }
    pub fn set_page_origin(&mut self, url: &str) {
        if let Ok(parsed) = url::Url::parse(url) {
            let origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));
            self.js_bridge.set_origin(&origin);
        }
    }
    pub fn drain_js_fetches(&mut self) -> Vec<crate::engine::js_bridge::FetchRequest> {
        self.js_bridge.drain_fetch_queue()
    }
}

pub fn extract_scripts(html: &str) -> Vec<String> {
    let dom = AmniDom::parse(html);
    let mut scripts = Vec::new();
    collect_inline_scripts(&dom.dom.document, &mut scripts);
    scripts
}

pub fn extract_external_script_urls(html: &str) -> Vec<String> {
    let dom = AmniDom::parse(html);
    let mut urls = Vec::new();
    collect_external_scripts(&dom.dom.document, &mut urls);
    urls
}

fn collect_external_scripts(handle: &Handle, out: &mut Vec<String>) {
    if let NodeData::Element { name, attrs, .. } = &handle.data {
        if name.local.to_string() == "script" {
            if let Some(src) = attrs.borrow().iter().find(|a| a.name.local.to_string() == "src") {
                let url = src.value.to_string();
                if !url.is_empty() { out.push(url); }
            }
        }
    }
    for child in handle.children.borrow().iter() { collect_external_scripts(child, out); }
}

fn collect_inline_scripts(handle: &Handle, out: &mut Vec<String>) {
    if let NodeData::Element { name, attrs, .. } = &handle.data {
        if name.local.to_string() == "script" {
            let has_src = attrs.borrow().iter().any(|a| a.name.local.to_string() == "src");
            if !has_src {
                let mut text = String::new();
                for child in handle.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        text.push_str(&contents.borrow());
                    }
                }
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    out.push(trimmed);
                }
            }
        }
    }
    for child in handle.children.borrow().iter() {
        collect_inline_scripts(child, out);
    }
}
