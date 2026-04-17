use super::js::{JsRuntime, JsEvalResult};
use super::paint::RenderTree;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub event_type: String,
    pub target_node_id: usize,
    pub x: f32,
    pub y: f32,
    pub key: String,
    pub prevent_default: bool,
}

#[derive(Debug, Clone)]
pub struct DomMutation {
    pub mutation_type: MutationType,
    pub node_id: usize,
}

#[derive(Debug, Clone)]
pub enum MutationType {
    SetText(String),
    SetAttribute(String, String),
    RemoveAttribute(String),
    SetStyle(String, String),
    AppendChild(usize),
    RemoveChild(usize),
    SetInnerHtml(String),
}

#[derive(Debug, Clone)]
pub struct TimerEntry {
    pub id: u32,
    pub callback_code: String,
    pub interval_ms: u64,
    pub is_interval: bool,
    pub next_fire_ms: u64,
}

#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub id: u32,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
struct DomSnapshot {
    nodes: HashMap<usize, SnapshotNode>,
    body_id: Option<usize>,
    head_id: Option<usize>,
    parent_map: HashMap<usize, usize>,
}

#[derive(Debug, Clone)]
struct SnapshotNode {
    tag: String,
    text: String,
    attrs: HashMap<String, String>,
    styles: HashMap<String, String>,
    children: Vec<usize>,
    classes: Vec<String>,
}

pub struct JsDomBridge {
    runtime: JsRuntime,
    event_handlers: HashMap<(usize, String), Vec<String>>,
    pending_mutations: Vec<DomMutation>,
    timers: HashMap<u32, TimerEntry>,
    next_timer_id: u32,
    fetch_queue: Vec<FetchRequest>,
    next_fetch_id: usize,
    pending_fetches: Vec<FetchRequest>,
    dom_snapshot: Option<DomSnapshot>,
    local_storage: HashMap<String, HashMap<String, String>>,
    session_storage: HashMap<String, HashMap<String, String>>,
    current_origin: String,
}

impl JsDomBridge {
    pub fn new() -> Self {
        Self {
            runtime: JsRuntime::new(),
            event_handlers: HashMap::new(),
            pending_mutations: Vec::new(),
            timers: HashMap::new(),
            next_timer_id: 1,
            fetch_queue: Vec::new(),
            next_fetch_id: 1,
            pending_fetches: Vec::new(),
            dom_snapshot: None,
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
            current_origin: String::new(),
        }
    }

    pub fn set_origin(&mut self, origin: &str) {
        self.current_origin = origin.to_string();
    }

    pub fn snapshot_dom(&mut self, tree: &RenderTree) {
        let mut nodes = HashMap::new();
        let mut body_id = None;
        let mut head_id = None;
        let mut parent_map = HashMap::new();
        for (id, node) in &tree.nodes {
            let tag_lower = node.tag.to_lowercase();
            if tag_lower == "body" { body_id = Some(*id); }
            if tag_lower == "head" { head_id = Some(*id); }
            let classes: Vec<String> = node.style.props.get("class")
                .or_else(|| None)
                .unwrap_or(&String::new())
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            for child_id in &node.children {
                parent_map.insert(*child_id, *id);
            }
            nodes.insert(*id, SnapshotNode {
                tag: node.tag.clone(),
                text: node.text.clone(),
                attrs: HashMap::new(),
                styles: node.style.props.clone(),
                children: node.children.clone(),
                classes,
            });
        }
        self.dom_snapshot = Some(DomSnapshot { nodes, body_id, head_id, parent_map });
        self.inject_dom_api();
    }

    fn build_nodes_json(snap: &DomSnapshot) -> String {
        let mut nodes_map = serde_json::Map::new();
        for (id, node) in &snap.nodes {
            let children_json: Vec<serde_json::Value> = node.children.iter()
                .map(|c| serde_json::json!(c))
                .collect();
            let parent_id = snap.parent_map.get(id).copied().unwrap_or(0);
            let sibling_list = if let Some(pid) = snap.parent_map.get(id) {
                snap.nodes.get(pid).map(|p| &p.children[..]).unwrap_or(&[])
            } else {
                &[]
            };
            let my_idx = sibling_list.iter().position(|c| c == id);
            let prev_sibling = my_idx.and_then(|i| if i > 0 { sibling_list.get(i - 1).copied() } else { None });
            let next_sibling = my_idx.and_then(|i| sibling_list.get(i + 1).copied());
            let el = serde_json::json!({
                "_nodeId": id,
                "tagName": node.tag.to_uppercase(),
                "textContent": node.text,
                "innerHTML": node.text,
                "children": children_json,
                "style": node.styles,
                "attrs": node.attrs,
                "classes": node.classes,
                "parentId": parent_id,
                "prevSibling": prev_sibling,
                "nextSibling": next_sibling,
            });
            nodes_map.insert(id.to_string(), el);
        }
        serde_json::Value::Object(nodes_map).to_string()
    }

    fn build_storage_json(storage: &HashMap<String, HashMap<String, String>>, origin: &str) -> String {
        let data = storage.get(origin).cloned().unwrap_or_default();
        serde_json::to_string(&data).unwrap_or_else(|_| "{}".into())
    }

    fn inject_dom_api(&mut self) {
        let snap = match &self.dom_snapshot {
            Some(s) => s,
            None => return,
        };
        let nodes_json = Self::build_nodes_json(snap);
        let body_id = snap.body_id.unwrap_or(0);
        let head_id = snap.head_id.unwrap_or(0);

        let ls_json = Self::build_storage_json(&self.local_storage, &self.current_origin);
        let ss_json = Self::build_storage_json(&self.session_storage, &self.current_origin);

        let api = format!(r#"
var __amni_nodes = {nodes_json};
var __amni_mutations = [];
var __amni_fetch_queue = [];
var __amni_timer_id = {timer_id};
var __amni_storage_mutations = [];

function __amni_make_element(data) {{
    if (!data) return null;
    var el = {{
        nodeType: 1,
        tagName: data.tagName || '',
        _nodeId: data._nodeId || 0,
        _attrs: data.attrs || {{}},
        _classes: (data.classes || []).slice(),
        getAttribute: function(name) {{ return el._attrs[name] !== undefined ? el._attrs[name] : null; }},
        setAttribute: function(name, value) {{
            el._attrs[name] = String(value);
            __amni_mutations.push({{type:'setAttribute', nodeId: el._nodeId, key: name, value: String(value)}});
        }},
        removeAttribute: function(name) {{
            delete el._attrs[name];
            __amni_mutations.push({{type:'removeAttribute', nodeId: el._nodeId, key: name}});
        }},
        classList: {{
            _el: null,
            add: function() {{
                for (var i = 0; i < arguments.length; i++) {{
                    if (el._classes.indexOf(arguments[i]) === -1) el._classes.push(arguments[i]);
                }}
                __amni_mutations.push({{type:'setAttribute', nodeId: el._nodeId, key:'class', value: el._classes.join(' ')}});
            }},
            remove: function() {{
                for (var i = 0; i < arguments.length; i++) {{
                    var idx = el._classes.indexOf(arguments[i]);
                    if (idx !== -1) el._classes.splice(idx, 1);
                }}
                __amni_mutations.push({{type:'setAttribute', nodeId: el._nodeId, key:'class', value: el._classes.join(' ')}});
            }},
            contains: function(c) {{ return el._classes.indexOf(c) !== -1; }},
            toggle: function(c, force) {{
                var has = el._classes.indexOf(c) !== -1;
                if (force !== undefined) {{
                    if (force && !has) {{ el._classes.push(c); }}
                    else if (!force && has) {{ el._classes.splice(el._classes.indexOf(c), 1); }}
                }} else {{
                    if (has) {{ el._classes.splice(el._classes.indexOf(c), 1); }}
                    else {{ el._classes.push(c); }}
                }}
                __amni_mutations.push({{type:'setAttribute', nodeId: el._nodeId, key:'class', value: el._classes.join(' ')}});
                return el._classes.indexOf(c) !== -1;
            }},
        }},
        style: new Proxy(data.style || {{}}, {{
            set: function(obj, prop, value) {{
                obj[prop] = value;
                __amni_mutations.push({{type:'setStyle', nodeId: el._nodeId, prop: prop, value: String(value)}});
                return true;
            }}
        }}),
        appendChild: function(child) {{
            __amni_mutations.push({{type:'appendChild', nodeId: el._nodeId, childId: child._nodeId || -1}});
        }},
        removeChild: function(child) {{
            __amni_mutations.push({{type:'removeChild', nodeId: el._nodeId, childId: child._nodeId || -1}});
        }},
        addEventListener: function(type, fn) {{
            __amni_mutations.push({{type:'addEventListener', nodeId: el._nodeId, eventType: type, handler: fn.toString()}});
        }},
    }};
    var _childIds = data.children || [];
    Object.defineProperty(el, 'children', {{
        get: function() {{
            var result = [];
            for (var i = 0; i < _childIds.length; i++) {{
                var cdata = __amni_nodes[String(_childIds[i])];
                if (cdata) result.push(__amni_make_element(cdata));
            }}
            return result;
        }}
    }});
    Object.defineProperty(el, 'parentElement', {{
        get: function() {{
            var pid = data.parentId;
            if (!pid && pid !== 0) return null;
            var pdata = __amni_nodes[String(pid)];
            return pdata ? __amni_make_element(pdata) : null;
        }}
    }});
    Object.defineProperty(el, 'nextSibling', {{
        get: function() {{
            var nid = data.nextSibling;
            if (nid === null || nid === undefined) return null;
            var ndata = __amni_nodes[String(nid)];
            return ndata ? __amni_make_element(ndata) : null;
        }}
    }});
    Object.defineProperty(el, 'previousSibling', {{
        get: function() {{
            var pid = data.prevSibling;
            if (pid === null || pid === undefined) return null;
            var pdata = __amni_nodes[String(pid)];
            return pdata ? __amni_make_element(pdata) : null;
        }}
    }});
    var _text = data.textContent || '';
    Object.defineProperty(el, 'textContent', {{
        get: function() {{ return _text; }},
        set: function(v) {{ _text = v; __amni_mutations.push({{type:'setText', nodeId: el._nodeId, text: v}}); }}
    }});
    var _html = data.innerHTML || '';
    Object.defineProperty(el, 'innerHTML', {{
        get: function() {{ return _html; }},
        set: function(v) {{ _html = v; __amni_mutations.push({{type:'setInnerHtml', nodeId: el._nodeId, html: v}}); }}
    }});
    return el;
}}

function __amni_find_by_id(id) {{
    for (var k in __amni_nodes) {{
        var n = __amni_nodes[k];
        if (n.attrs && n.attrs.id === id) return n;
    }}
    return null;
}}

function __amni_match_selector(node, sel) {{
    sel = sel.trim();
    if (sel.charAt(0) === '#') {{
        return node.attrs && node.attrs.id === sel.substring(1);
    }}
    if (sel.charAt(0) === '.') {{
        var cls = sel.substring(1);
        return (node.classes || []).indexOf(cls) !== -1;
    }}
    if (sel.charAt(0) === '[') {{
        var inner = sel.substring(1, sel.length - 1);
        var eqIdx = inner.indexOf('=');
        if (eqIdx !== -1) {{
            var aname = inner.substring(0, eqIdx);
            var aval = inner.substring(eqIdx + 1).replace(/['"]/g, '');
            return node.attrs && node.attrs[aname] === aval;
        }}
        return node.attrs && node.attrs[inner] !== undefined;
    }}
    return (node.tagName || '').toUpperCase() === sel.toUpperCase();
}}

function __amni_query(sel) {{
    var results = [];
    for (var k in __amni_nodes) {{
        if (__amni_match_selector(__amni_nodes[k], sel)) results.push(__amni_nodes[k]);
    }}
    return results;
}}

function __amni_make_storage(initData, storageType) {{
    var _data = {{}};
    for (var k in initData) _data[k] = initData[k];
    var _keys = function() {{ var r = []; for (var k in _data) r.push(k); return r; }};
    return {{
        getItem: function(k) {{ return _data.hasOwnProperty(k) ? _data[k] : null; }},
        setItem: function(k, v) {{
            _data[k] = String(v);
            __amni_storage_mutations.push({{storageType: storageType, op: 'set', key: k, value: String(v)}});
        }},
        removeItem: function(k) {{
            delete _data[k];
            __amni_storage_mutations.push({{storageType: storageType, op: 'remove', key: k}});
        }},
        clear: function() {{
            _data = {{}};
            __amni_storage_mutations.push({{storageType: storageType, op: 'clear'}});
        }},
        key: function(index) {{
            var keys = _keys();
            return index >= 0 && index < keys.length ? keys[index] : null;
        }},
        get length() {{ return _keys().length; }},
    }};
}}

var localStorage = __amni_make_storage({ls_json}, 'local');
var sessionStorage = __amni_make_storage({ss_json}, 'session');

var __amni_doc_title = '';
var document = {{
    get title() {{ return __amni_doc_title; }},
    set title(v) {{ __amni_doc_title = v; }},
    getElementById: function(id) {{
        var n = __amni_find_by_id(id);
        return n ? __amni_make_element(n) : null;
    }},
    querySelector: function(sel) {{
        var r = __amni_query(sel);
        return r.length > 0 ? __amni_make_element(r[0]) : null;
    }},
    querySelectorAll: function(sel) {{
        var r = __amni_query(sel);
        var out = [];
        for (var i = 0; i < r.length; i++) out.push(__amni_make_element(r[i]));
        out.item = function(i) {{ return out[i] || null; }};
        out.forEach = function(fn) {{ for (var i = 0; i < out.length; i++) fn(out[i], i, out); }};
        return out;
    }},
    getElementsByClassName: function(name) {{
        var r = [];
        for (var k in __amni_nodes) {{
            if ((__amni_nodes[k].classes || []).indexOf(name) !== -1) r.push(__amni_make_element(__amni_nodes[k]));
        }}
        r.item = function(i) {{ return r[i] || null; }};
        return r;
    }},
    getElementsByTagName: function(name) {{
        var upper = name.toUpperCase();
        var r = [];
        for (var k in __amni_nodes) {{
            if ((__amni_nodes[k].tagName || '').toUpperCase() === upper) r.push(__amni_make_element(__amni_nodes[k]));
        }}
        r.item = function(i) {{ return r[i] || null; }};
        return r;
    }},
    createElement: function(tag) {{
        return __amni_make_element({{tagName: tag.toUpperCase(), textContent: '', innerHTML: '', children: [], style: {{}}, _nodeId: -1, attrs: {{}}, classes: [], parentId: null, prevSibling: null, nextSibling: null}});
    }},
    createTextNode: function(text) {{
        return {{textContent: text, nodeType: 3}};
    }},
    get body() {{
        var bd = __amni_nodes[String({body_id})];
        if (!bd) bd = {{tagName:'BODY', textContent:'', innerHTML:'', children:[], style:{{}}, _nodeId:{body_id}, attrs:{{}}, classes:[], parentId:null, prevSibling:null, nextSibling:null}};
        return __amni_make_element(bd);
    }},
    get head() {{
        var hd = __amni_nodes[String({head_id})];
        if (!hd) hd = {{tagName:'HEAD', textContent:'', innerHTML:'', children:[], style:{{}}, _nodeId:{head_id}, attrs:{{}}, classes:[], parentId:null, prevSibling:null, nextSibling:null}};
        return __amni_make_element(hd);
    }},
}};

var window = {{
    document: document,
    location: {{ href: '', pathname: '/', search: '', hash: '' }},
    navigator: {{ userAgent: 'AmniBrowse/0.6 AmniShunt/1.0' }},
    innerWidth: 1280,
    innerHeight: 720,
    localStorage: localStorage,
    sessionStorage: sessionStorage,
}};

function setTimeout(fn, ms) {{
    var id = ++__amni_timer_id;
    __amni_mutations.push({{type:'setTimeout', id: id, code: fn.toString(), ms: ms || 0}});
    return id;
}}
function setInterval(fn, ms) {{
    var id = ++__amni_timer_id;
    __amni_mutations.push({{type:'setInterval', id: id, code: fn.toString(), ms: ms || 0}});
    return id;
}}
function clearTimeout(id) {{ __amni_mutations.push({{type:'clearTimer', id: id}}); }}
function clearInterval(id) {{ __amni_mutations.push({{type:'clearTimer', id: id}}); }}

function fetch(url, opts) {{
    opts = opts || {{}};
    var id = ++__amni_timer_id;
    var hdrs = {{}};
    if (opts.headers) {{
        if (typeof opts.headers.forEach === 'function') {{
            opts.headers.forEach(function(v, k) {{ hdrs[k] = v; }});
        }} else {{
            for (var k in opts.headers) hdrs[k] = opts.headers[k];
        }}
    }}
    __amni_fetch_queue.push({{id: id, url: url, method: opts.method || 'GET', headers: hdrs, body: opts.body || null}});
    return new Promise(function(resolve) {{
        window.__amni_pending_fetches = window.__amni_pending_fetches || {{}};
        window.__amni_pending_fetches[id] = resolve;
    }});
}}
"#,
            nodes_json = nodes_json,
            timer_id = self.next_timer_id,
            ls_json = ls_json,
            ss_json = ss_json,
            body_id = body_id,
            head_id = head_id,
        );

        self.runtime.eval(&api);
    }

    pub fn exec_script(&mut self, script: &str) -> JsExecResult {
        let eval_result = self.runtime.eval(script);
        let mutations = self.collect_mutations();
        let timers = self.collect_timers();
        let fetches = self.collect_fetches();
        self.collect_storage_mutations();

        JsExecResult {
            value: eval_result.value,
            error: eval_result.error,
            console: eval_result.console_output,
            mutations,
            new_timers: timers,
            fetch_requests: fetches,
        }
    }

    fn collect_mutations(&mut self) -> Vec<DomMutation> {
        let result = self.runtime.eval("JSON.stringify(__amni_mutations)");
        let _ = self.runtime.eval("__amni_mutations = []");
        let mut mutations = Vec::new();

        let json_str = result.value.trim_matches('"').replace("\\\"", "\"");
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
            for item in arr {
                let node_id = item["nodeId"].as_u64().unwrap_or(0) as usize;
                let mt = item["type"].as_str().unwrap_or("");
                match mt {
                    "setText" => {
                        if let Some(text) = item["text"].as_str() {
                            mutations.push(DomMutation { mutation_type: MutationType::SetText(text.into()), node_id });
                        }
                    }
                    "setInnerHtml" => {
                        if let Some(html) = item["html"].as_str() {
                            mutations.push(DomMutation { mutation_type: MutationType::SetInnerHtml(html.into()), node_id });
                        }
                    }
                    "setAttribute" => {
                        let key = item["key"].as_str().unwrap_or("").to_string();
                        let value = item["value"].as_str().unwrap_or("").to_string();
                        mutations.push(DomMutation { mutation_type: MutationType::SetAttribute(key, value), node_id });
                    }
                    "removeAttribute" => {
                        let key = item["key"].as_str().unwrap_or("").to_string();
                        mutations.push(DomMutation { mutation_type: MutationType::RemoveAttribute(key), node_id });
                    }
                    "setStyle" => {
                        let prop = item["prop"].as_str().unwrap_or("").to_string();
                        let value = item["value"].as_str().unwrap_or("").to_string();
                        mutations.push(DomMutation { mutation_type: MutationType::SetStyle(prop, value), node_id });
                    }
                    "appendChild" => {
                        let child_id = item["childId"].as_u64().unwrap_or(0) as usize;
                        mutations.push(DomMutation { mutation_type: MutationType::AppendChild(child_id), node_id });
                    }
                    "removeChild" => {
                        let child_id = item["childId"].as_u64().unwrap_or(0) as usize;
                        mutations.push(DomMutation { mutation_type: MutationType::RemoveChild(child_id), node_id });
                    }
                    "addEventListener" => {
                        let event_type = item["eventType"].as_str().unwrap_or("").to_string();
                        let handler = item["handler"].as_str().unwrap_or("").to_string();
                        self.event_handlers.entry((node_id, event_type)).or_default().push(handler);
                    }
                    _ => {}
                }
            }
        }
        mutations
    }

    fn collect_timers(&mut self) -> Vec<TimerEntry> {
        let result = self.runtime.eval("JSON.stringify(__amni_mutations.filter(function(m) { return m.type === 'setTimeout' || m.type === 'setInterval'; }))");
        let mut timers = Vec::new();
        let json_str = result.value.trim_matches('"').replace("\\\"", "\"");
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            for item in arr {
                let id = item["id"].as_u64().unwrap_or(0) as u32;
                let code = item["code"].as_str().unwrap_or("").to_string();
                let ms = item["ms"].as_u64().unwrap_or(0);
                let is_interval = item["type"].as_str() == Some("setInterval");
                let entry = TimerEntry { id, callback_code: code, interval_ms: ms, is_interval, next_fire_ms: now + ms };
                self.timers.insert(id, entry.clone());
                timers.push(entry);
            }
        }
        timers
    }

    fn collect_fetches(&mut self) -> Vec<FetchRequest> {
        let result = self.runtime.eval("var __r = JSON.stringify(__amni_fetch_queue); __amni_fetch_queue = []; __r");
        let mut fetches = Vec::new();
        let json_str = result.value.trim_matches('"').replace("\\\"", "\"");
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
            for item in arr {
                let mut headers = HashMap::new();
                if let Some(hmap) = item["headers"].as_object() {
                    for (k, v) in hmap {
                        headers.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                    }
                }
                fetches.push(FetchRequest {
                    id: item["id"].as_u64().unwrap_or(0) as u32,
                    url: item["url"].as_str().unwrap_or("").into(),
                    method: item["method"].as_str().unwrap_or("GET").into(),
                    headers,
                    body: item["body"].as_str().map(|s| s.into()),
                });
            }
        }
        self.fetch_queue.extend(fetches.clone());
        self.pending_fetches.extend(fetches.clone());
        fetches
    }

    fn collect_storage_mutations(&mut self) {
        let result = self.runtime.eval("var __sr = JSON.stringify(__amni_storage_mutations); __amni_storage_mutations = []; __sr");
        let json_str = result.value.trim_matches('"').replace("\\\"", "\"");
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
            for item in arr {
                let stype = item["storageType"].as_str().unwrap_or("");
                let op = item["op"].as_str().unwrap_or("");
                let storage = match stype {
                    "local" => &mut self.local_storage,
                    "session" => &mut self.session_storage,
                    _ => continue,
                };
                let origin_map = storage.entry(self.current_origin.clone()).or_default();
                match op {
                    "set" => {
                        let key = item["key"].as_str().unwrap_or("").to_string();
                        let value = item["value"].as_str().unwrap_or("").to_string();
                        origin_map.insert(key, value);
                    }
                    "remove" => {
                        let key = item["key"].as_str().unwrap_or("").to_string();
                        origin_map.remove(&key);
                    }
                    "clear" => { origin_map.clear(); }
                    _ => {}
                }
            }
        }
    }

    pub fn dispatch_event(&mut self, event: &DomEvent) -> Vec<DomMutation> {
        let key = (event.target_node_id, event.event_type.clone());
        let handlers = self.event_handlers.get(&key).cloned().unwrap_or_default();
        let mut all_mutations = Vec::new();

        let event_json = serde_json::json!({
            "type": event.event_type,
            "target": { "_nodeId": event.target_node_id },
            "clientX": event.x, "clientY": event.y,
            "key": event.key,
            "preventDefault": "function() {}",
            "stopPropagation": "function() {}",
        });

        for handler in &handlers {
            let code = format!("({})({})", handler, event_json);
            let result = self.exec_script(&code);
            all_mutations.extend(result.mutations);
        }
        all_mutations
    }

    pub fn tick_timers(&mut self) -> Vec<DomMutation> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        let mut to_fire = Vec::new();
        let mut to_remove = Vec::new();

        for (id, timer) in &self.timers {
            if now >= timer.next_fire_ms {
                to_fire.push(timer.clone());
                if !timer.is_interval { to_remove.push(*id); }
            }
        }

        for id in to_remove { self.timers.remove(&id); }
        for timer in &mut to_fire {
            if timer.is_interval {
                if let Some(t) = self.timers.get_mut(&timer.id) {
                    t.next_fire_ms = now + t.interval_ms;
                }
            }
        }

        let mut all_mutations = Vec::new();
        for timer in &to_fire {
            let result = self.exec_script(&format!("({})()", timer.callback_code));
            all_mutations.extend(result.mutations);
        }
        all_mutations
    }

    pub fn resolve_fetch(&mut self, id: u32, status: u16, headers: HashMap<String, String>, body: String) {
        let headers_json = serde_json::to_string(&headers).unwrap_or_else(|_| "{}".into());
        let body_json = serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".into());
        let code = format!(
            r#"if (window.__amni_pending_fetches && window.__amni_pending_fetches[{id}]) {{
                var __hdrs = {headers_json};
                window.__amni_pending_fetches[{id}]({{
                    ok: {ok},
                    status: {status},
                    headers: {{
                        get: function(k) {{ return __hdrs[k] || null; }},
                        has: function(k) {{ return __hdrs.hasOwnProperty(k); }},
                        entries: function() {{ var r = []; for (var k in __hdrs) r.push([k, __hdrs[k]]); return r; }},
                    }},
                    text: function() {{ return Promise.resolve({body_json}); }},
                    json: function() {{ return Promise.resolve(JSON.parse({body_json})); }},
                    blob: function() {{ return Promise.resolve(new Blob([{body_json}])); }},
                    arrayBuffer: function() {{ return Promise.resolve(new ArrayBuffer(0)); }},
                    clone: function() {{ return this; }},
                }});
                delete window.__amni_pending_fetches[{id}];
            }}"#,
            id = id, ok = status < 400, status = status,
            headers_json = headers_json, body_json = body_json,
        );
        self.runtime.eval(&code);
    }

    pub fn apply_mutations(tree: &mut RenderTree, mutations: &[DomMutation]) -> bool {
        let mut changed = false;
        for m in mutations {
            if let Some(node) = tree.nodes.get_mut(&m.node_id) {
                match &m.mutation_type {
                    MutationType::SetText(text) => { node.text = text.clone(); changed = true; }
                    MutationType::SetInnerHtml(html) => { node.text = html.clone(); changed = true; }
                    MutationType::SetStyle(prop, value) => {
                        let css = format!("{}: {}", prop, value);
                        let sheet = super::style::StyleSheet::parse(&format!("_m {{ {} }}", css));
                        for rule in &sheet.rules { node.style.apply_declarations(&rule.declarations); }
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
        changed
    }

    pub fn pending_fetch_count(&self) -> usize { self.fetch_queue.len() }
    pub fn timer_count(&self) -> usize { self.timers.len() }

    pub fn drain_fetch_queue(&mut self) -> Vec<FetchRequest> {
        std::mem::take(&mut self.fetch_queue)
    }

    pub fn drain_pending_fetches(&mut self) -> Vec<FetchRequest> {
        std::mem::take(&mut self.pending_fetches)
    }

    pub fn get_local_storage(&self, origin: &str) -> &HashMap<String, String> {
        static EMPTY: std::sync::LazyLock<HashMap<String, String>> = std::sync::LazyLock::new(HashMap::new);
        self.local_storage.get(origin).unwrap_or(&EMPTY)
    }

    pub fn get_session_storage(&self, origin: &str) -> &HashMap<String, String> {
        static EMPTY: std::sync::LazyLock<HashMap<String, String>> = std::sync::LazyLock::new(HashMap::new);
        self.session_storage.get(origin).unwrap_or(&EMPTY)
    }

    pub fn clear_session_storage(&mut self) {
        self.session_storage.clear();
    }

    pub fn reset(&mut self) {
        self.runtime.reset();
        self.event_handlers.clear();
        self.pending_mutations.clear();
        self.timers.clear();
        self.fetch_queue.clear();
        self.pending_fetches.clear();
        self.dom_snapshot = None;
        self.session_storage.clear();
    }
}

#[derive(Debug)]
pub struct JsExecResult {
    pub value: String,
    pub error: Option<String>,
    pub console: Vec<super::js::ConsoleEntry>,
    pub mutations: Vec<DomMutation>,
    pub new_timers: Vec<TimerEntry>,
    pub fetch_requests: Vec<FetchRequest>,
}
