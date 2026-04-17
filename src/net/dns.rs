use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub addresses: Vec<String>,
    pub ttl: u64,
    #[serde(skip)]
    #[serde(default = "default_instant")]
    pub cached_at_epoch: u64,
}
fn default_instant() -> u64 { 0 }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DohProvider {
    Cloudflare,
    Google,
    Quad9,
    Custom(String),
}
impl DohProvider {
    pub fn url(&self) -> &str {
        match self {
            Self::Cloudflare => "https://cloudflare-dns.com/dns-query",
            Self::Google => "https://dns.google/resolve",
            Self::Quad9 => "https://dns.quad9.net:5053/dns-query",
            Self::Custom(u) => u.as_str(),
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Cloudflare => "Cloudflare (1.1.1.1)",
            Self::Google => "Google (8.8.8.8)",
            Self::Quad9 => "Quad9 (9.9.9.9)",
            Self::Custom(_) => "Custom",
        }
    }
}
impl Default for DohProvider {
    fn default() -> Self { Self::Cloudflare }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsAnswer {
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: u16,
    #[serde(rename = "TTL")]
    pub ttl: Option<u64>,
    pub data: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResponse {
    #[serde(rename = "Status")]
    pub status: u32,
    #[serde(rename = "Answer")]
    pub answer: Option<Vec<DnsAnswer>>,
}
pub struct DohResolver {
    pub enabled: bool,
    pub provider: DohProvider,
    cache: HashMap<String, DnsRecord>,
    last_clean: Instant,
}
impl DohResolver {
    pub fn new(enabled: bool, provider: DohProvider) -> Self {
        Self { enabled, provider, cache: HashMap::new(), last_clean: Instant::now() }
    }
    pub fn resolve(&mut self, hostname: &str) -> Option<Vec<String>> {
        if !self.enabled { return None; }
        if self.last_clean.elapsed() > Duration::from_secs(300) {
            self.clean_cache();
            self.last_clean = Instant::now();
        }
        let now_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        if let Some(cached) = self.cache.get(hostname) {
            if now_epoch < cached.cached_at_epoch + cached.ttl {
                debug!("🔒 DoH cache hit: {}", hostname);
                return Some(cached.addresses.clone());
            }
        }
        match self.query_doh(hostname) {
            Ok(addrs) => {
                let record = DnsRecord { addresses: addrs.clone(), ttl: 300, cached_at_epoch: now_epoch };
                self.cache.insert(hostname.to_string(), record);
                info!("🔒 DoH resolved: {} -> {:?}", hostname, addrs);
                Some(addrs)
            }
            Err(e) => {
                error!("🔒 DoH failed for {}: {}", hostname, e);
                None
            }
        }
    }
    fn query_doh(&self, hostname: &str) -> Result<Vec<String>, String> {
        let url = format!("{}?name={}&type=A", self.provider.url(), hostname);
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("HTTP client err: {}", e))?;
        let resp = client.get(&url)
            .header("Accept", "application/dns-json")
            .send()
            .map_err(|e| format!("DoH request err: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("DoH HTTP {}", resp.status()));
        }
        let dns_resp: DnsResponse = resp.json().map_err(|e| format!("DoH parse err: {}", e))?;
        if dns_resp.status != 0 {
            return Err(format!("DoH status: {}", dns_resp.status));
        }
        let addrs: Vec<String> = dns_resp.answer.unwrap_or_default().iter()
            .filter(|a| a.record_type == 1 || a.record_type == 28)
            .filter_map(|a| a.data.clone())
            .collect();
        if addrs.is_empty() { Err("No addresses returned".into()) } else { Ok(addrs) }
    }
    fn clean_cache(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        self.cache.retain(|_, v| now < v.cached_at_epoch + v.ttl);
    }
    pub fn cache_size(&self) -> usize { self.cache.len() }
    pub fn clear_cache(&mut self) { self.cache.clear(); }
    pub fn set_provider(&mut self, provider: DohProvider) {
        self.provider = provider;
        self.cache.clear();
    }
    pub fn provider_json(&self) -> String {
        serde_json::to_string(&self.provider).unwrap_or_else(|_| "\"Cloudflare\"".into())
    }
}
