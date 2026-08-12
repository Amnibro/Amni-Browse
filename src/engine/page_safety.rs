use serde::{Deserialize, Serialize};
use url::Url;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel { Safe, Low, Medium, High, Critical }
impl RiskLevel {
    pub fn as_str(self) -> &'static str {
        match self { Self::Safe => "safe", Self::Low => "low", Self::Medium => "medium", Self::High => "high", Self::Critical => "critical" }
    }
    pub fn rank(self) -> u8 {
        match self { Self::Safe => 0, Self::Low => 1, Self::Medium => 2, Self::High => 3, Self::Critical => 4 }
    }
    fn max(self, other: Self) -> Self { if other.rank() > self.rank() { other } else { self } }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    pub url: String,
    pub host: String,
    pub level: RiskLevel,
    pub score: u32,
    pub reasons: Vec<String>,
    pub tips: Vec<String>,
    pub scheme: String,
    pub is_https: bool,
    pub is_local: bool,
}
const SUSPICIOUS_TLDS: &[&str] = &["zip", "mov", "tk", "ml", "ga", "cf", "gq", "top", "xyz", "work", "click", "country", "stream", "download", "racing"];
const LOOKALIKE: &[(&str, &str)] = &[
    ("paypa1.", "paypal"), ("paypai.", "paypal"), ("appleid-", "apple"), ("micros0ft.", "microsoft"),
    ("g00gle.", "google"), ("faceb00k.", "facebook"), ("amaz0n.", "amazon"), ("netf1ix.", "netflix"),
    ("bankofamerica-", "bank"), ("secure-login.", "phish"), ("account-verify.", "phish"),
    ("login-update.", "phish"), ("signin-secure.", "phish"),
];
const SENSITIVE_PATH: &[&str] = &["login", "signin", "sign-in", "password", "passwd", "wallet", "crypto", "seed", "mnemonic", "verify", "account/update", "confirm"];
pub fn assess(raw: &str) -> SafetyReport {
    let mut level = RiskLevel::Safe;
    let mut score: u32 = 0;
    let mut reasons: Vec<String> = Vec::new();
    let mut tips: Vec<String> = Vec::new();
    let url_s = raw.trim().to_string();
    let parsed = Url::parse(&url_s).ok();
    let scheme = parsed.as_ref().map(|u| u.scheme().to_string()).unwrap_or_else(|| "unknown".into());
    let host = parsed.as_ref().and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase())).unwrap_or_default();
    let is_https = scheme == "https";
    let is_local = scheme == "amnibrowse" || scheme == "file" || scheme == "data" || host == "localhost" || host == "127.0.0.1" || host.ends_with(".local");
    if is_local {
        tips.push("Local or built-in page — Amni chrome still applies.".into());
        return SafetyReport { url: url_s, host, level: RiskLevel::Safe, score: 0, reasons, tips, scheme, is_https, is_local: true };
    }
    if scheme == "http" {
        level = level.max(RiskLevel::Medium);
        score += 25;
        reasons.push("Connection is not encrypted (HTTP).".into());
        tips.push("Prefer HTTPS before entering passwords or payment data.".into());
    }
    if scheme == "data" || scheme == "blob" {
        level = level.max(RiskLevel::Medium);
        score += 20;
        reasons.push("Non-standard scheme may hide the real origin.".into());
    }
    if host.is_empty() && parsed.is_some() {
        level = level.max(RiskLevel::High);
        score += 40;
        reasons.push("URL has no hostname.".into());
    }
    if host.parse::<std::net::Ipv4Addr>().is_ok() || host.parse::<std::net::Ipv6Addr>().is_ok() || host.starts_with('[') {
        level = level.max(RiskLevel::High);
        score += 35;
        reasons.push("Site is served from a raw IP address.".into());
        tips.push("Banks and major brands almost never use bare IPs for login.".into());
    }
    let labels: Vec<&str> = host.split('.').filter(|s| !s.is_empty()).collect();
    if labels.len() >= 5 {
        level = level.max(RiskLevel::Medium);
        score += 15;
        reasons.push(format!("Unusually deep subdomain chain ({} labels).", labels.len()));
    }
    if let Some(tld) = labels.last() {
        if SUSPICIOUS_TLDS.iter().any(|t| t == tld) {
            level = level.max(RiskLevel::Medium);
            score += 18;
            reasons.push(format!("TLD '.{}' is frequently abused in phishing campaigns.", tld));
        }
    }
    for (pat, brand) in LOOKALIKE {
        if host.contains(pat) {
            level = level.max(RiskLevel::High);
            score += 40;
            reasons.push(format!("Hostname looks like a spoof of {}.", brand));
            tips.push("Check the spelling carefully; close the tab if you did not type this URL yourself.".into());
        }
    }
    if host.chars().filter(|c| *c == '-').count() >= 3 {
        level = level.max(RiskLevel::Low);
        score += 8;
        reasons.push("Many hyphens in the hostname (common in disposable phishing domains).".into());
    }
    if let Some(u) = &parsed {
        let path = u.path().to_ascii_lowercase();
        let q = u.query().unwrap_or("").to_ascii_lowercase();
        let blob = format!("{} {}", path, q);
        for p in SENSITIVE_PATH {
            if blob.contains(p) && !is_https {
                level = level.max(RiskLevel::High);
                score += 20;
                reasons.push(format!("Sensitive path fragment '{}' on a non-HTTPS page.", p));
            } else if blob.contains(p) && score > 0 {
                tips.push("Login/password-related path — verify the padlock and domain before typing secrets.".into());
            }
        }
        if u.username().len() > 0 || u.password().is_some() {
            level = level.max(RiskLevel::High);
            score += 30;
            reasons.push("Credentials embedded in the URL (user:pass@host).".into());
            tips.push("Never follow links that include passwords in the address bar.".into());
        }
        if host.contains("xn--") {
            level = level.max(RiskLevel::Medium);
            score += 20;
            reasons.push("Internationalized (punycode) domain — may spoof familiar brands.".into());
        }
    }
    if score >= 70 { level = level.max(RiskLevel::Critical); }
    else if score >= 45 { level = level.max(RiskLevel::High); }
    else if score >= 25 { level = level.max(RiskLevel::Medium); }
    else if score >= 8 { level = level.max(RiskLevel::Low); }
    if reasons.is_empty() {
        tips.push("No automated red flags. Stay alert for unexpected password prompts.".into());
    } else if tips.is_empty() {
        tips.push("If this is not a site you trust, leave without signing in.".into());
    }
    SafetyReport { url: url_s, host, level, score: score.min(100), reasons, tips, scheme, is_https, is_local }
}
pub fn assess_json(raw: &str) -> String {
    serde_json::to_string(&assess(raw)).unwrap_or_else(|_| "{}".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn https_known_ok() {
        let r = assess("https://www.wikipedia.org/wiki/Rust");
        assert!(r.level.rank() <= RiskLevel::Low.rank());
        assert!(r.is_https);
    }
    #[test]
    fn http_login_flagged() {
        let r = assess("http://example.com/login");
        assert!(r.level.rank() >= RiskLevel::Medium.rank());
    }
    #[test]
    fn ip_host_high() {
        let r = assess("https://185.199.108.153/login");
        assert!(r.level.rank() >= RiskLevel::High.rank());
    }
    #[test]
    fn lookalike_high() {
        let r = assess("https://paypa1.example-secure.tk/signin");
        assert!(r.level.rank() >= RiskLevel::High.rank());
    }
}
