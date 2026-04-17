use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CspDirective {
    DefaultSrc, ScriptSrc, StyleSrc, ImgSrc, FontSrc, ConnectSrc,
    MediaSrc, FrameSrc, ObjectSrc, BaseUri, FormAction, FrameAncestors, ReportUri,
}
impl CspDirective {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "default-src" => Some(Self::DefaultSrc),
            "script-src" => Some(Self::ScriptSrc),
            "style-src" => Some(Self::StyleSrc),
            "img-src" => Some(Self::ImgSrc),
            "font-src" => Some(Self::FontSrc),
            "connect-src" => Some(Self::ConnectSrc),
            "media-src" => Some(Self::MediaSrc),
            "frame-src" => Some(Self::FrameSrc),
            "object-src" => Some(Self::ObjectSrc),
            "base-uri" => Some(Self::BaseUri),
            "form-action" => Some(Self::FormAction),
            "frame-ancestors" => Some(Self::FrameAncestors),
            "report-uri" => Some(Self::ReportUri),
            _ => None,
        }
    }
    pub fn resource_type_to_directive(resource_type: &str) -> Self {
        match resource_type {
            "script" => Self::ScriptSrc,
            "style" => Self::StyleSrc,
            "image" | "img" => Self::ImgSrc,
            "font" => Self::FontSrc,
            "connect" | "xhr" | "fetch" => Self::ConnectSrc,
            "media" => Self::MediaSrc,
            "frame" | "iframe" => Self::FrameSrc,
            "object" => Self::ObjectSrc,
            _ => Self::DefaultSrc,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum CspSource {
    Self_,
    None_,
    UnsafeInline,
    UnsafeEval,
    Https,
    Data,
    Blob,
    Host(String),
    Nonce(String),
    Hash(String),
}
impl CspSource {
    pub fn parse(s: &str) -> Self {
        match s {
            "'self'" => Self::Self_,
            "'none'" => Self::None_,
            "'unsafe-inline'" => Self::UnsafeInline,
            "'unsafe-eval'" => Self::UnsafeEval,
            "https:" => Self::Https,
            "data:" => Self::Data,
            "blob:" => Self::Blob,
            _ if s.starts_with("'nonce-") && s.ends_with('\'') => {
                Self::Nonce(s[7..s.len()-1].to_string())
            }
            _ if (s.starts_with("'sha256-") || s.starts_with("'sha384-") || s.starts_with("'sha512-")) && s.ends_with('\'') => {
                Self::Hash(s[1..s.len()-1].to_string())
            }
            _ => Self::Host(s.to_string()),
        }
    }
    pub fn matches(&self, url: &str, page_origin: &str, nonce: Option<&str>) -> bool {
        match self {
            Self::None_ => false,
            Self::Self_ => url_matches_origin(url, page_origin),
            Self::UnsafeInline => false,
            Self::UnsafeEval => false,
            Self::Https => url.starts_with("https://") || url.starts_with("https:"),
            Self::Data => url.starts_with("data:"),
            Self::Blob => url.starts_with("blob:"),
            Self::Host(pattern) => host_matches(url, pattern),
            Self::Nonce(n) => nonce.map_or(false, |provided| provided == n),
            Self::Hash(_) => false,
        }
    }
}
fn url_matches_origin(url: &str, origin: &str) -> bool {
    let url_origin = extract_origin(url);
    url_origin.eq_ignore_ascii_case(&extract_origin(origin))
}
fn extract_origin(url: &str) -> String {
    if let Some(idx) = url.find("://") {
        let after = &url[idx + 3..];
        let end = after.find('/').unwrap_or(after.len());
        format!("{}{}", &url[..idx + 3], &after[..end])
    } else { url.to_string() }
}
fn host_matches(url: &str, pattern: &str) -> bool {
    let url_host = extract_origin(url);
    let url_host = url_host.split("://").nth(1).unwrap_or(&url_host);
    if pattern.starts_with("*.") {
        let suffix = &pattern[1..];
        url_host.ends_with(suffix) || url_host == &pattern[2..]
    } else if pattern.contains("://") {
        url.starts_with(pattern)
    } else {
        url_host == pattern || url_host.starts_with(&format!("{}:", pattern))
    }
}
#[derive(Debug, Clone)]
pub struct CspPolicy {
    pub directives: HashMap<CspDirective, Vec<CspSource>>,
    pub page_origin: String,
}
impl CspPolicy {
    pub fn new(page_origin: &str) -> Self {
        Self { directives: HashMap::new(), page_origin: page_origin.to_string() }
    }
    pub fn allows(&self, directive: &CspDirective, url: &str, nonce: Option<&str>) -> bool {
        let sources = self.directives.get(directive)
            .or_else(|| self.directives.get(&CspDirective::DefaultSrc));
        match sources {
            None => true,
            Some(srcs) => {
                if srcs.iter().any(|s| matches!(s, CspSource::None_)) { return false; }
                srcs.iter().any(|s| s.matches(url, &self.page_origin, nonce))
            }
        }
    }
    pub fn allows_inline_script(&self, nonce: Option<&str>) -> bool {
        let sources = self.directives.get(&CspDirective::ScriptSrc)
            .or_else(|| self.directives.get(&CspDirective::DefaultSrc));
        match sources {
            None => true,
            Some(srcs) => {
                if srcs.iter().any(|s| matches!(s, CspSource::UnsafeInline)) { return true; }
                if let Some(n) = nonce {
                    return srcs.iter().any(|s| matches!(s, CspSource::Nonce(ref sn) if sn == n));
                }
                false
            }
        }
    }
    pub fn allows_inline_style(&self, nonce: Option<&str>) -> bool {
        let sources = self.directives.get(&CspDirective::StyleSrc)
            .or_else(|| self.directives.get(&CspDirective::DefaultSrc));
        match sources {
            None => true,
            Some(srcs) => {
                if srcs.iter().any(|s| matches!(s, CspSource::UnsafeInline)) { return true; }
                if let Some(n) = nonce {
                    return srcs.iter().any(|s| matches!(s, CspSource::Nonce(ref sn) if sn == n));
                }
                false
            }
        }
    }
    pub fn allows_eval(&self) -> bool {
        let sources = self.directives.get(&CspDirective::ScriptSrc)
            .or_else(|| self.directives.get(&CspDirective::DefaultSrc));
        match sources {
            None => true,
            Some(srcs) => srcs.iter().any(|s| matches!(s, CspSource::UnsafeEval)),
        }
    }
}
pub fn parse_policy(header: &str, page_origin: &str) -> CspPolicy {
    let mut policy = CspPolicy::new(page_origin);
    for directive_str in header.split(';') {
        let directive_str = directive_str.trim();
        if directive_str.is_empty() { continue; }
        let mut parts = directive_str.split_whitespace();
        let name = match parts.next() { Some(n) => n, None => continue };
        let directive = match CspDirective::from_str(name) { Some(d) => d, None => continue };
        let sources: Vec<CspSource> = parts.map(CspSource::parse).collect();
        policy.directives.insert(directive, sources);
    }
    policy
}
#[derive(Debug, Clone)]
pub struct CspResult {
    pub allowed: bool,
    pub report: Option<CspViolation>,
}
#[derive(Debug, Clone)]
pub struct CspViolation {
    pub directive: String,
    pub blocked_uri: String,
    pub document_uri: String,
}
pub struct CspEnforcer {
    pub enforce_policy: Option<CspPolicy>,
    pub report_only_policy: Option<CspPolicy>,
    pub document_uri: String,
}
impl CspEnforcer {
    pub fn new(document_uri: &str) -> Self {
        Self { enforce_policy: None, report_only_policy: None, document_uri: document_uri.to_string() }
    }
    pub fn set_enforce(&mut self, header: &str) {
        self.enforce_policy = Some(parse_policy(header, &self.document_uri));
    }
    pub fn set_report_only(&mut self, header: &str) {
        self.report_only_policy = Some(parse_policy(header, &self.document_uri));
    }
    pub fn from_headers(document_uri: &str, headers: &[(String, String)]) -> Self {
        let mut enforcer = Self::new(document_uri);
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("content-security-policy") { enforcer.set_enforce(v); }
            if k.eq_ignore_ascii_case("content-security-policy-report-only") { enforcer.set_report_only(v); }
        }
        enforcer
    }
    pub fn check_resource(&self, resource_type: &str, url: &str) -> CspResult {
        let directive = CspDirective::resource_type_to_directive(resource_type);
        let directive_name = resource_type.to_string();
        let enforce_allowed = self.enforce_policy.as_ref().map_or(true, |p| p.allows(&directive, url, None));
        let report_violation = if let Some(ref rp) = self.report_only_policy {
            if !rp.allows(&directive, url, None) {
                Some(CspViolation {
                    directive: directive_name.clone(),
                    blocked_uri: url.to_string(),
                    document_uri: self.document_uri.clone(),
                })
            } else { None }
        } else { None };
        if !enforce_allowed {
            CspResult {
                allowed: false,
                report: Some(CspViolation {
                    directive: directive_name,
                    blocked_uri: url.to_string(),
                    document_uri: self.document_uri.clone(),
                }),
            }
        } else {
            CspResult { allowed: true, report: report_violation }
        }
    }
    pub fn check_inline_script(&self, nonce: Option<&str>) -> CspResult {
        let enforce_allowed = self.enforce_policy.as_ref().map_or(true, |p| p.allows_inline_script(nonce));
        if !enforce_allowed {
            CspResult {
                allowed: false,
                report: Some(CspViolation {
                    directive: "script-src".to_string(),
                    blocked_uri: "inline".to_string(),
                    document_uri: self.document_uri.clone(),
                }),
            }
        } else { CspResult { allowed: true, report: None } }
    }
    pub fn check_inline_style(&self, nonce: Option<&str>) -> CspResult {
        let enforce_allowed = self.enforce_policy.as_ref().map_or(true, |p| p.allows_inline_style(nonce));
        if !enforce_allowed {
            CspResult {
                allowed: false,
                report: Some(CspViolation {
                    directive: "style-src".to_string(),
                    blocked_uri: "inline".to_string(),
                    document_uri: self.document_uri.clone(),
                }),
            }
        } else { CspResult { allowed: true, report: None } }
    }
    pub fn check_eval(&self) -> CspResult {
        let enforce_allowed = self.enforce_policy.as_ref().map_or(true, |p| p.allows_eval());
        if !enforce_allowed {
            CspResult {
                allowed: false,
                report: Some(CspViolation {
                    directive: "script-src".to_string(),
                    blocked_uri: "eval".to_string(),
                    document_uri: self.document_uri.clone(),
                }),
            }
        } else { CspResult { allowed: true, report: None } }
    }
}
