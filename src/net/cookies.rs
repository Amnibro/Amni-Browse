use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::info;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<DateTime<Utc>>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SameSite { None, Lax, Strict }
pub struct CookieJar {
    cookies: HashMap<String, Vec<Cookie>>,
    block_third_party: bool,
    allow_list: Vec<String>,
    deny_list: Vec<String>,
}
impl CookieJar {
    pub fn new(block_third_party: bool) -> Self {
        Self { cookies: HashMap::new(), block_third_party, allow_list: Vec::new(), deny_list: Vec::new() }
    }
    pub fn set_cookie(&mut self, cookie: Cookie, request_domain: &str) -> bool {
        if self.deny_list.iter().any(|d| cookie.domain.contains(d)) { return false; }
        if self.block_third_party && !self.is_same_site(&cookie.domain, request_domain) && !self.allow_list.iter().any(|d| cookie.domain.contains(d)) { return false; }
        if let Some(exp) = &cookie.expires { if *exp < Utc::now() { self.remove_cookie(&cookie.domain, &cookie.name); return false; } }
        let domain_cookies = self.cookies.entry(cookie.domain.clone()).or_default();
        domain_cookies.retain(|c| c.name != cookie.name || c.path != cookie.path);
        domain_cookies.push(cookie);
        true
    }
    pub fn get_cookies(&self, domain: &str, path: &str, is_secure: bool) -> Vec<&Cookie> {
        let mut result = Vec::new();
        let now = Utc::now();
        for (d, cookies) in &self.cookies {
            if !self.domain_matches(d, domain) { continue; }
            for c in cookies {
                if !path.starts_with(&c.path) { continue; }
                if c.secure && !is_secure { continue; }
                if let Some(exp) = &c.expires { if *exp < now { continue; } }
                result.push(c);
            }
        }
        result
    }
    pub fn cookie_header(&self, domain: &str, path: &str, is_secure: bool) -> Option<String> {
        let cookies = self.get_cookies(domain, path, is_secure);
        (!cookies.is_empty()).then(|| cookies.iter().map(|c| format!("{}={}", c.name, c.value)).collect::<Vec<_>>().join("; "))
    }
    pub fn remove_cookie(&mut self, domain: &str, name: &str) {
        if let Some(cookies) = self.cookies.get_mut(domain) { cookies.retain(|c| c.name != name); }
    }
    pub fn clear_domain(&mut self, domain: &str) { self.cookies.remove(domain); }
    pub fn clear_all(&mut self) { self.cookies.clear(); info!("All cookies cleared"); }
    pub fn domain_count(&self) -> usize { self.cookies.len() }
    pub fn total_count(&self) -> usize { self.cookies.values().map(|v| v.len()).sum() }
    pub fn set_block_third_party(&mut self, block: bool) { self.block_third_party = block; }
    pub fn add_allow(&mut self, domain: String) { self.allow_list.push(domain); }
    pub fn add_deny(&mut self, domain: String) { self.deny_list.push(domain); }
    pub fn parse_set_cookie(header: &str, request_domain: &str) -> Option<Cookie> {
        let mut parts = header.split(';');
        let nv = parts.next()?.trim();
        let (name, value) = nv.split_once('=')?;
        let mut cookie = Cookie {
            name: name.trim().to_string(), value: value.trim().to_string(),
            domain: request_domain.to_string(), path: "/".to_string(),
            expires: None, secure: false, http_only: false, same_site: SameSite::Lax,
        };
        for attr in parts {
            let attr = attr.trim();
            let (k, v) = attr.split_once('=').map_or((attr, ""), |(a, b)| (a.trim(), b.trim()));
            match k.to_lowercase().as_str() {
                "domain" => cookie.domain = v.trim_start_matches('.').to_string(),
                "path" => cookie.path = v.to_string(),
                "secure" => cookie.secure = true,
                "httponly" => cookie.http_only = true,
                "samesite" => cookie.same_site = match v.to_lowercase().as_str() { "strict" => SameSite::Strict, "none" => SameSite::None, _ => SameSite::Lax },
                "max-age" => if let Ok(secs) = v.parse::<i64>() { cookie.expires = Some(Utc::now() + chrono::Duration::seconds(secs)); },
                _ => {}
            }
        }
        Some(cookie)
    }
    fn is_same_site(&self, cookie_domain: &str, request_domain: &str) -> bool {
        let cd = cookie_domain.trim_start_matches('.');
        let rd = request_domain.trim_start_matches('.');
        cd == rd || rd.ends_with(&format!(".{}", cd))
    }
    fn domain_matches(&self, cookie_domain: &str, request_domain: &str) -> bool {
        let cd = cookie_domain.trim_start_matches('.');
        let rd = request_domain.trim_start_matches('.');
        cd == rd || rd.ends_with(&format!(".{}", cd))
    }
    pub fn save(&self) -> String { serde_json::to_string(&self.cookies).unwrap_or_default() }
    pub fn load(data: &str, block_third_party: bool) -> Self {
        let cookies: HashMap<String, Vec<Cookie>> = serde_json::from_str(data).unwrap_or_default();
        Self { cookies, block_third_party, allow_list: Vec::new(), deny_list: Vec::new() }
    }
}
