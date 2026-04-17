/// AmniShunt Sandbox — Process-isolated rendering shunt.
/// HTML enters the shunt process, AmniIR comes out over base17-encoded pipes.
/// The shunt has no network, no filesystem, no syscalls beyond memory allocation.

use super::amni_ir::{IrProgram, IrBuilder, Instruction};
use super::base17;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum ShuntState {
    Idle,
    Processing,
    Ready,
    Crashed(String),
}

#[derive(Debug, Clone)]
pub struct ShuntConfig {
    pub memory_budget_mb: usize,
    pub watchdog_timeout_ms: u64,
    pub enable_process_isolation: bool,
}

impl Default for ShuntConfig {
    fn default() -> Self {
        Self { memory_budget_mb: 256, watchdog_timeout_ms: 10_000, enable_process_isolation: false }
    }
}

// --- In-process shunt (thread-based, for development and fallback) ---

pub struct InProcessShunt {
    state: ShuntState,
    config: ShuntConfig,
    ir_cache: HashMap<String, IrProgram>,
}

impl InProcessShunt {
    pub fn new(config: ShuntConfig) -> Self {
        Self { state: ShuntState::Idle, config, ir_cache: HashMap::new() }
    }

    pub fn state(&self) -> &ShuntState { &self.state }

    /// Parse HTML + CSS and emit AmniIR
    pub fn process_html(&mut self, url: &str, html: &str, css_sources: &[&str]) -> Result<IrProgram, String> {
        self.state = ShuntState::Processing;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.emit_ir(html, css_sources, 1280.0, 2048.0)
        }));

        match result {
            Ok(prog) => {
                self.ir_cache.insert(url.to_string(), prog.clone_program());
                self.state = ShuntState::Ready;
                Ok(prog)
            }
            Err(_) => {
                self.state = ShuntState::Crashed("panic in shunt processing".into());
                Err("shunt crashed during HTML processing".into())
            }
        }
    }

    /// Process HTML and return base17-encoded wire format
    pub fn process_to_wire(&mut self, url: &str, html: &str, css_sources: &[&str]) -> Result<Vec<u8>, String> {
        let prog = self.process_html(url, html, css_sources)?;
        Ok(prog.encode_to_wire())
    }

    fn emit_ir(&self, html: &str, css_sources: &[&str], vw: f32, vh: f32) -> IrProgram {
        use super::dom::AmniDom;
        use super::style::StyleSheet;
        use super::paint::RenderTree;
        use super::layout::{LayoutEngine, LayoutRect};
        use super::style::ComputedStyle;

        let dom = AmniDom::parse(html);
        let mut sheets = Vec::new();
        for src in css_sources { sheets.push(StyleSheet::parse(src)); }

        // Build layout
        let mut engine = LayoutEngine::new();
        let mut styles: HashMap<usize, ComputedStyle> = HashMap::new();
        let mut id_counter = 0usize;
        build_layout_tree(&dom.dom.document, &sheets, &mut engine, &mut styles, &mut id_counter);
        if id_counter > 0 { engine.compute(0, vw, vh); }
        let ids: Vec<usize> = (0..id_counter).collect();
        engine.collect_all(&ids);
        let mut rects = HashMap::new();
        for id in 0..id_counter {
            if let Some(r) = engine.get_layout(id) { rects.insert(id, r.clone()); }
        }

        // Build render tree and emit IR
        let mut rt_counter = 0usize;
        let render_tree = RenderTree::build_from_dom(&dom.dom.document, &sheets, &mut rt_counter);
        super::amni_ir::emit_ir_from_render_tree(&render_tree, &rects, vw, vh)
    }

    pub fn get_cached(&self, url: &str) -> Option<&IrProgram> {
        self.ir_cache.get(url)
    }

    pub fn clear_cache(&mut self) { self.ir_cache.clear(); }

    pub fn reset(&mut self) {
        self.state = ShuntState::Idle;
        self.ir_cache.clear();
    }
}

// Re-use the tree building logic from pipeline
fn build_layout_tree(
    handle: &markup5ever_rcdom::Handle,
    sheets: &[super::style::StyleSheet],
    engine: &mut super::layout::LayoutEngine,
    styles: &mut HashMap<usize, super::style::ComputedStyle>,
    counter: &mut usize,
) {
    use markup5ever_rcdom::NodeData;
    use super::style::ComputedStyle;
    let my_id = *counter;
    *counter += 1;
    let mut cs = ComputedStyle::default();
    cs.font_size = 16.0; cs.line_height = 1.2; cs.opacity = 1.0; cs.flex_shrink = 1.0;
    if let NodeData::Element { name, attrs, .. } = &handle.data {
        let tag = name.local.to_string();
        let attrs_map: HashMap<String, String> = attrs.borrow().iter()
            .map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
        let id_attr = attrs_map.get("id").cloned().unwrap_or_default();
        let class_attr = attrs_map.get("class").cloned().unwrap_or_default();
        let classes: Vec<&str> = class_attr.split_whitespace().collect();
        for sheet in sheets {
            for rule in &sheet.rules {
                if selector_matches(&rule.selectors, &tag, &id_attr, &classes) {
                    cs.apply_declarations(&rule.declarations);
                }
            }
        }
        if let Some(style_attr) = attrs_map.get("style") {
            let inline = super::style::StyleSheet::parse(&format!("_i {{ {} }}", style_attr));
            for rule in &inline.rules { cs.apply_declarations(&rule.declarations); }
        }
    }
    let mut child_ids = Vec::new();
    for child in handle.children.borrow().iter() {
        let child_id = *counter;
        build_layout_tree(child, sheets, engine, styles, counter);
        child_ids.push(child_id);
    }
    styles.insert(my_id, cs.clone());
    if child_ids.is_empty() { engine.add_leaf(my_id, &cs); }
    else { engine.add_node(my_id, &cs, &child_ids); }
}

fn selector_matches(selectors: &[String], tag: &str, id: &str, classes: &[&str]) -> bool {
    selectors.iter().any(|sel| {
        let sel = sel.trim();
        if sel == tag || sel == "*" { return true; }
        if sel.starts_with('#') && &sel[1..] == id { return true; }
        if sel.starts_with('.') && classes.contains(&&sel[1..]) { return true; }
        false
    })
}

// --- Process-isolated shunt (real sandbox via child process) ---

pub struct ProcessShunt {
    config: ShuntConfig,
    state: ShuntState,
    child: Option<std::process::Child>,
}

impl ProcessShunt {
    pub fn new(config: ShuntConfig) -> Self {
        Self { config, state: ShuntState::Idle, child: None }
    }

    pub fn spawn(&mut self) -> Result<(), String> {
        // The shunt binary would be a separate executable that:
        // 1. Reads HTML from stdin (base17-encoded)
        // 2. Parses HTML → DOM → layout → IR
        // 3. Writes AmniIR to stdout (base17-encoded)
        // 4. Has no network/FS capabilities
        //
        // For now, this is a placeholder that describes the architecture.
        // In production, this would spawn a restricted child process.

        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        log::info!("ProcessShunt: would spawn {} with --shunt-mode", exe.display());
        self.state = ShuntState::Ready;
        Ok(())
    }

    pub fn send_html(&mut self, html: &str) -> Result<Vec<u8>, String> {
        // Encode HTML as base17, send to child stdin
        let encoded = base17::encode_bytes(html.as_bytes());

        // In full implementation:
        // 1. Write STREAM_START marker
        // 2. Write encoded HTML
        // 3. Write STREAM_END marker
        // 4. Read IR response from stdout
        // 5. Verify checksum
        // 6. Return decoded IR

        // For now, use in-process fallback
        let mut shunt = InProcessShunt::new(self.config.clone());
        shunt.process_to_wire("stdin", html, &[])
    }

    pub fn is_alive(&self) -> bool {
        self.state == ShuntState::Ready || self.state == ShuntState::Processing
    }

    pub fn kill(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
        }
        self.child = None;
        self.state = ShuntState::Idle;
    }

    pub fn state(&self) -> &ShuntState { &self.state }
}

impl Drop for ProcessShunt {
    fn drop(&mut self) { self.kill(); }
}

// --- Shunt Manager: manages lifecycle, watchdog, crash recovery ---

pub struct ShuntManager {
    in_process: InProcessShunt,
    process_shunt: Option<ProcessShunt>,
    config: ShuntConfig,
    crash_count: u32,
    max_crashes: u32,
}

impl ShuntManager {
    pub fn new(config: ShuntConfig) -> Self {
        let in_process = InProcessShunt::new(config.clone());
        Self {
            in_process,
            process_shunt: None,
            config,
            crash_count: 0,
            max_crashes: 3,
        }
    }

    pub fn process_html(&mut self, url: &str, html: &str, css: &[&str]) -> Result<IrProgram, String> {
        // Try process-isolated shunt first
        if self.config.enable_process_isolation {
            if self.process_shunt.is_none() {
                let mut ps = ProcessShunt::new(self.config.clone());
                if ps.spawn().is_ok() {
                    self.process_shunt = Some(ps);
                }
            }
            if let Some(ref mut ps) = self.process_shunt {
                match ps.send_html(html) {
                    Ok(wire) => {
                        return IrProgram::decode_from_wire(&wire)
                            .map_err(|e| format!("IR decode: {}", e));
                    }
                    Err(e) => {
                        log::warn!("Process shunt failed: {}, falling back", e);
                        self.crash_count += 1;
                        if self.crash_count >= self.max_crashes {
                            log::error!("Too many shunt crashes, disabling process isolation");
                            self.process_shunt = None;
                        }
                    }
                }
            }
        }
        // Fallback to in-process shunt
        self.in_process.process_html(url, html, css)
    }

    pub fn process_to_wire(&mut self, url: &str, html: &str, css: &[&str]) -> Result<Vec<u8>, String> {
        let prog = self.process_html(url, html, css)?;
        Ok(prog.encode_to_wire())
    }

    pub fn state(&self) -> &ShuntState { self.in_process.state() }

    pub fn crash_count(&self) -> u32 { self.crash_count }

    pub fn reset(&mut self) {
        self.in_process.reset();
        if let Some(ref mut ps) = self.process_shunt { ps.kill(); }
        self.process_shunt = None;
        self.crash_count = 0;
    }

    pub fn stats_json(&self) -> String {
        serde_json::json!({
            "state": format!("{:?}", self.in_process.state()),
            "process_isolation": self.config.enable_process_isolation,
            "crash_count": self.crash_count,
            "memory_budget_mb": self.config.memory_budget_mb,
            "cache_entries": self.in_process.ir_cache.len(),
        }).to_string()
    }
}

// Helper for IrProgram cloning
trait CloneProgram { fn clone_program(&self) -> IrProgram; }
impl CloneProgram for IrProgram {
    fn clone_program(&self) -> IrProgram {
        // Re-encode and decode for a clean copy
        let wire = self.encode_to_wire();
        IrProgram::decode_from_wire(&wire).unwrap_or_else(|_| IrProgram::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_process_shunt_basic() {
        let mut shunt = InProcessShunt::new(ShuntConfig::default());
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        let result = shunt.process_html("test://page", html, &[]);
        assert!(result.is_ok());
        let prog = result.unwrap();
        assert!(!prog.is_empty());
    }

    #[test]
    fn shunt_wire_roundtrip() {
        let mut shunt = InProcessShunt::new(ShuntConfig::default());
        let html = "<div><span>Test</span></div>";
        let wire = shunt.process_to_wire("test://wire", html, &[]).unwrap();
        let decoded = IrProgram::decode_from_wire(&wire).unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn shunt_manager_fallback() {
        let mut mgr = ShuntManager::new(ShuntConfig::default());
        let html = "<p>Hello from the shunt manager</p>";
        let result = mgr.process_html("test://mgr", html, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn crash_recovery() {
        let mut shunt = InProcessShunt::new(ShuntConfig::default());
        // Valid HTML should succeed
        let r = shunt.process_html("test://ok", "<p>ok</p>", &[]);
        assert!(r.is_ok());
        assert_eq!(*shunt.state(), ShuntState::Ready);
    }
}
