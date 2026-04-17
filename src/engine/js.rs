#[cfg(feature = "js-engine")]
use boa_engine::{Context, Source, JsValue, JsResult, property::Attribute, JsNativeError};
use std::collections::HashMap;

pub struct JsRuntime {
    #[cfg(feature = "js-engine")]
    context: Context,
    console_log: Vec<ConsoleEntry>,
    globals: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct JsEvalResult {
    pub value: String,
    pub error: Option<String>,
    pub console_output: Vec<ConsoleEntry>,
}

impl JsRuntime {
    pub fn new() -> Self {
        let mut rt = Self {
            #[cfg(feature = "js-engine")]
            context: Context::default(),
            console_log: Vec::new(),
            globals: HashMap::new(),
        };
        rt.setup_builtins();
        rt
    }

    fn setup_builtins(&mut self) {
        #[cfg(feature = "js-engine")]
        {
            let _ = self.context.eval(Source::from_bytes(r#"
                var __amni_console_log = [];
                var console = {
                    log: function() { __amni_console_log.push({level:'log', msg: Array.from(arguments).join(' ')}); },
                    warn: function() { __amni_console_log.push({level:'warn', msg: Array.from(arguments).join(' ')}); },
                    error: function() { __amni_console_log.push({level:'error', msg: Array.from(arguments).join(' ')}); },
                    info: function() { __amni_console_log.push({level:'info', msg: Array.from(arguments).join(' ')}); },
                };
                var __amni_timers = {};
                var setTimeout = function(fn, ms) { var id = Math.random(); __amni_timers[id] = {fn:fn, ms:ms}; return id; };
                var clearTimeout = function(id) { delete __amni_timers[id]; };
            "#));
        }
    }

    pub fn eval(&mut self, script: &str) -> JsEvalResult {
        #[cfg(feature = "js-engine")]
        {
            let result = self.context.eval(Source::from_bytes(script));
            let (value, error) = match result {
                Ok(val) => (val.display().to_string(), None),
                Err(e) => (String::new(), Some(e.to_string())),
            };
            let console_output = self.drain_console();
            JsEvalResult { value, error, console_output }
        }
        #[cfg(not(feature = "js-engine"))]
        {
            let _ = script;
            JsEvalResult {
                value: String::new(),
                error: Some("JS engine not enabled (compile with --features js-engine)".into()),
                console_output: vec![],
            }
        }
    }

    pub fn eval_dom_script(&mut self, script: &str, document_json: &str) -> JsEvalResult {
        #[cfg(feature = "js-engine")]
        {
            let setup = format!(
                "var __amni_doc = {};\n\
                 var document = {{\n\
                   title: __amni_doc.title || '',\n\
                   getElementById: function(id) {{ return __amni_doc.elements ? __amni_doc.elements[id] : null; }},\n\
                   querySelector: function(sel) {{ return null; }},\n\
                   querySelectorAll: function(sel) {{ return []; }},\n\
                   createElement: function(tag) {{ return {{tagName: tag, children: [], style: {{}}}}; }},\n\
                   body: {{ innerHTML: '', appendChild: function(n) {{}} }},\n\
                 }};\n\
                 var window = {{ document: document, location: {{ href: '' }}, navigator: {{ userAgent: 'AmniBrowse/0.6' }} }};\n\
                 {}",
                document_json, script
            );
            self.eval(&setup)
        }
        #[cfg(not(feature = "js-engine"))]
        {
            let _ = (script, document_json);
            JsEvalResult {
                value: String::new(),
                error: Some("JS engine not enabled".into()),
                console_output: vec![],
            }
        }
    }

    fn drain_console(&mut self) -> Vec<ConsoleEntry> {
        #[cfg(feature = "js-engine")]
        {
            let mut entries = Vec::new();
            let result = self.context.eval(Source::from_bytes(
                "JSON.stringify(__amni_console_log)"
            ));
            if let Ok(val) = result {
                let json_str = val.display().to_string();
                let clean = json_str.trim_matches('"').replace("\\\"", "\"");
                if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&clean) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                    for item in arr {
                        entries.push(ConsoleEntry {
                            level: item["level"].as_str().unwrap_or("log").to_string(),
                            message: item["msg"].as_str().unwrap_or("").to_string(),
                            timestamp: now,
                        });
                    }
                }
            }
            let _ = self.context.eval(Source::from_bytes("__amni_console_log = [];"));
            entries
        }
        #[cfg(not(feature = "js-engine"))]
        { vec![] }
    }

    pub fn set_global(&mut self, name: &str, value: &str) {
        self.globals.insert(name.to_string(), value.to_string());
        #[cfg(feature = "js-engine")]
        {
            let code = format!("var {} = {};", name, value);
            let _ = self.context.eval(Source::from_bytes(&code));
        }
    }

    pub fn reset(&mut self) {
        #[cfg(feature = "js-engine")]
        { self.context = Context::default(); }
        self.console_log.clear();
        self.globals.clear();
        self.setup_builtins();
    }

    pub fn is_available() -> bool {
        cfg!(feature = "js-engine")
    }
}
