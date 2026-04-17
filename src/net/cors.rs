use std::collections::HashMap;
use std::time::{Duration, Instant};
#[derive(Debug, Clone, PartialEq)]
pub enum RequestType { Simple, Preflight, Actual }
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
    pub allow_credentials: bool,
}
impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".into(), "HEAD".into(), "POST".into()],
            allowed_headers: Vec::new(),
            max_age: 86400,
            allow_credentials: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct CorsResult {
    pub allowed: bool,
    pub exposed_headers: Vec<String>,
}
pub fn is_simple_method(method: &str) -> bool {
    matches!(method.to_uppercase().as_str(), "GET" | "HEAD" | "POST")
}
pub fn is_simple_header(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
        "accept" | "accept-language" | "content-language" | "content-type")
}
pub fn is_simple_content_type(value: &str) -> bool {
    let ct = value.split(';').next().unwrap_or("").trim().to_lowercase();
    matches!(ct.as_str(),
        "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain")
}
pub fn classify_request(method: &str, headers: &[(String, String)], origin: Option<&str>) -> RequestType {
    if origin.is_none() { return RequestType::Simple; }
    if method.eq_ignore_ascii_case("OPTIONS") {
        let has_acrm = headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("access-control-request-method"));
        if has_acrm { return RequestType::Preflight; }
    }
    if !is_simple_method(method) { return RequestType::Actual; }
    let has_non_simple = headers.iter().any(|(k, v)| {
        if !is_simple_header(k) { return true; }
        if k.eq_ignore_ascii_case("content-type") && !is_simple_content_type(v) { return true; }
        false
    });
    if has_non_simple { RequestType::Actual } else { RequestType::Simple }
}
pub struct PreflightRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
}
pub fn build_preflight_request(url: &str, method: &str, request_headers: &[(String, String)], origin: &str) -> PreflightRequest {
    let non_simple: Vec<String> = request_headers.iter()
        .filter(|(k, _)| !is_simple_header(k))
        .map(|(k, _)| k.clone())
        .collect();
    let mut headers = vec![
        ("Origin".to_string(), origin.to_string()),
        ("Access-Control-Request-Method".to_string(), method.to_string()),
    ];
    if !non_simple.is_empty() {
        headers.push(("Access-Control-Request-Headers".to_string(), non_simple.join(", ")));
    }
    PreflightRequest { method: "OPTIONS".to_string(), url: url.to_string(), headers }
}
pub fn validate_response(origin: &str, response_headers: &[(String, String)]) -> CorsResult {
    let acao = response_headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("access-control-allow-origin"))
        .map(|(_, v)| v.as_str());
    let allowed = match acao {
        Some("*") => true,
        Some(v) => v == origin,
        None => false,
    };
    let exposed_headers = response_headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("access-control-expose-headers"))
        .map(|(_, v)| v.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    CorsResult { allowed, exposed_headers }
}
pub fn check_method_allowed(method: &str, response_headers: &[(String, String)]) -> bool {
    let acam = response_headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("access-control-allow-methods"))
        .map(|(_, v)| v.as_str());
    match acam {
        Some(v) => v.split(',').any(|m| m.trim().eq_ignore_ascii_case(method)),
        None => is_simple_method(method),
    }
}
pub fn check_headers_allowed(request_headers: &[(String, String)], response_headers: &[(String, String)]) -> bool {
    let acah = response_headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("access-control-allow-headers"))
        .map(|(_, v)| v.as_str());
    let allowed: Vec<String> = match acah {
        Some(v) => v.split(',').map(|s| s.trim().to_lowercase()).collect(),
        None => Vec::new(),
    };
    request_headers.iter().all(|(k, _)| {
        is_simple_header(k) || allowed.contains(&k.to_lowercase())
    })
}
#[derive(Debug, Clone)]
struct CorsEntry {
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    inserted: Instant,
    max_age: Duration,
}
pub struct CorsCache {
    entries: HashMap<(String, String), CorsEntry>,
}
impl CorsCache {
    pub fn new() -> Self { Self { entries: HashMap::new() } }
    pub fn check(&self, origin: &str, url: &str) -> Option<(&[String], &[String])> {
        let key = (origin.to_string(), url.to_string());
        let entry = self.entries.get(&key)?;
        if entry.inserted.elapsed() > entry.max_age { return None; }
        Some((&entry.allowed_methods, &entry.allowed_headers))
    }
    pub fn store(&mut self, origin: &str, url: &str, response_headers: &[(String, String)]) {
        let max_age_secs = response_headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("access-control-max-age"))
            .and_then(|(_, v)| v.parse::<u64>().ok())
            .unwrap_or(5);
        let allowed_methods = response_headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("access-control-allow-methods"))
            .map(|(_, v)| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let allowed_headers = response_headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("access-control-allow-headers"))
            .map(|(_, v)| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let key = (origin.to_string(), url.to_string());
        self.entries.insert(key, CorsEntry {
            allowed_methods, allowed_headers,
            inserted: Instant::now(), max_age: Duration::from_secs(max_age_secs),
        });
    }
    pub fn evict_expired(&mut self) {
        self.entries.retain(|_, entry| entry.inserted.elapsed() <= entry.max_age);
    }
    pub fn clear(&mut self) { self.entries.clear(); }
}
pub struct CorsEnforcer {
    pub config: CorsConfig,
    pub cache: CorsCache,
}
impl CorsEnforcer {
    pub fn new(config: CorsConfig) -> Self { Self { config, cache: CorsCache::new() } }
    pub fn default_permissive() -> Self { Self::new(CorsConfig::default()) }
    pub fn should_allow(&self, origin: &str, url: &str, method: &str, headers: &[(String, String)]) -> CorsResult {
        if self.config.allowed_origins.contains(&"*".to_string()) || self.config.allowed_origins.iter().any(|o| o == origin) {
            if let Some((methods, _hdrs)) = self.cache.check(origin, url) {
                if methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                    return CorsResult { allowed: true, exposed_headers: Vec::new() };
                }
            }
            let method_ok = self.config.allowed_methods.iter().any(|m| m.eq_ignore_ascii_case(method)) || is_simple_method(method);
            let headers_ok = headers.iter().all(|(k, _)| {
                is_simple_header(k) || self.config.allowed_headers.iter().any(|h| h.eq_ignore_ascii_case(k))
            });
            CorsResult { allowed: method_ok && headers_ok, exposed_headers: Vec::new() }
        } else {
            CorsResult { allowed: false, exposed_headers: Vec::new() }
        }
    }
}
