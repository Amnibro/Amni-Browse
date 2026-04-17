use hyper::body::Incoming;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use http_body_util::{BodyExt, Full};
use bytes::Bytes;
use rustls::ClientConfig;
use std::sync::Arc;
use hyper_rustls::HttpsConnectorBuilder;
use log::info;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use rustls::RootCertStore;
use serde::de::DeserializeOwned;
struct CacheEntry {
    body: Bytes,
    headers: Vec<(String, String)>,
    status: u16,
    inserted: Instant,
    ttl: Duration,
}
pub struct AmniClient {
    client: Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    dnt: bool,
    block_3pc: bool,
    user_agent: String,
    default_auth: Option<String>,
}
pub struct AmniResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Bytes,
    pub url: String,
    pub from_cache: bool,
}
impl AmniClient {
    pub fn new() -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        let client = Client::builder(TokioExecutor::new())
            .build(connector);
        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            dnt: true,
            block_3pc: true,
            user_agent: format!("AmniBrowse/{} (Privacy-First; +https://amniscient.dev)", env!("CARGO_PKG_VERSION")),
            default_auth: None,
        }
    }
    pub async fn get(&self, url: &str) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(cached) = self.check_cache(url) { return Ok(cached); }
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "identity")
            .header("Connection", "keep-alive");
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        let req = builder.body(Full::new(Bytes::new()))?;
        info!("AmniNet GET {}", url);
        let resp: Response<Incoming> = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        let ttl = self.extract_cache_ttl(&headers);
        if ttl > Duration::ZERO {
            if let Ok(mut c) = self.cache.write() {
                c.insert(url.to_string(), CacheEntry { body: body.clone(), headers: headers.clone(), status, inserted: Instant::now(), ttl });
            }
        }
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }
    pub async fn post(&self, url: &str, content_type: &str, body: Bytes) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("User-Agent", &self.user_agent)
            .header("Content-Type", content_type);
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        let req = builder.body(Full::new(body))?;
        info!("AmniNet POST {}", url);
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }
    fn check_cache(&self, url: &str) -> Option<AmniResponse> {
        let c = self.cache.read().ok()?;
        let entry = c.get(url)?;
        (entry.inserted.elapsed() < entry.ttl).then(|| AmniResponse {
            status: entry.status, headers: entry.headers.clone(),
            body: entry.body.clone(), url: url.to_string(), from_cache: true,
        })
    }
    fn extract_cache_ttl(&self, headers: &[(String, String)]) -> Duration {
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("cache-control") {
                if v.contains("no-store") || v.contains("no-cache") { return Duration::ZERO; }
                if let Some(max_age) = v.split(',').find_map(|d| {
                    let d = d.trim();
                    d.strip_prefix("max-age=").and_then(|s| s.trim().parse::<u64>().ok())
                }) {
                    return Duration::from_secs(max_age.min(3600));
                }
            }
        }
        Duration::ZERO
    }
    pub fn clear_cache(&self) {
        if let Ok(mut c) = self.cache.write() { c.clear(); }
    }
    pub fn cache_size(&self) -> usize {
        self.cache.read().map_or(0, |c| c.len())
    }
    pub fn set_dnt(&mut self, enabled: bool) { self.dnt = enabled; }
    pub fn set_block_3pc(&mut self, enabled: bool) { self.block_3pc = enabled; }

    pub async fn get_with_headers(&self, url: &str, extra_headers: &[(&str, &str)]) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(cached) = self.check_cache(url) { return Ok(cached); }
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "identity")
            .header("Connection", "keep-alive");
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        for (k, v) in extra_headers { builder = builder.header(*k, *v); }
        let req = builder.body(Full::new(Bytes::new()))?;
        info!("AmniNet GET {}", url);
        let resp: Response<Incoming> = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        let ttl = self.extract_cache_ttl(&headers);
        if ttl > Duration::ZERO {
            if let Ok(mut c) = self.cache.write() {
                c.insert(url.to_string(), CacheEntry { body: body.clone(), headers: headers.clone(), status, inserted: Instant::now(), ttl });
            }
        }
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }

    // --- Additional HTTP Methods ---

    pub async fn put(&self, url: &str, content_type: &str, body: Bytes) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("User-Agent", &self.user_agent)
            .header("Content-Type", content_type);
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        if let Some(ref auth) = self.default_auth { builder = builder.header("Authorization", auth.as_str()); }
        let req = builder.body(Full::new(body))?;
        info!("AmniNet PUT {}", url);
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }

    pub async fn delete(&self, url: &str) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("User-Agent", &self.user_agent);
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        if let Some(ref auth) = self.default_auth { builder = builder.header("Authorization", auth.as_str()); }
        let req = builder.body(Full::new(Bytes::new()))?;
        info!("AmniNet DELETE {}", url);
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }

    pub async fn patch(&self, url: &str, content_type: &str, body: Bytes) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("User-Agent", &self.user_agent)
            .header("Content-Type", content_type);
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        if let Some(ref auth) = self.default_auth { builder = builder.header("Authorization", auth.as_str()); }
        let req = builder.body(Full::new(body))?;
        info!("AmniNet PATCH {}", url);
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }

    pub async fn head(&self, url: &str) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("HEAD")
            .uri(&uri)
            .header("User-Agent", &self.user_agent);
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        if let Some(ref auth) = self.default_auth { builder = builder.header("Authorization", auth.as_str()); }
        let req = builder.body(Full::new(Bytes::new()))?;
        info!("AmniNet HEAD {}", url);
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        // HEAD responses have no body
        Ok(AmniResponse { status, headers, body: Bytes::new(), url: url.to_string(), from_cache: false })
    }

    pub async fn options(&self, url: &str) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = url.parse()?;
        let mut builder = Request::builder()
            .method("OPTIONS")
            .uri(&uri)
            .header("User-Agent", &self.user_agent);
        if self.dnt { builder = builder.header("DNT", "1").header("Sec-GPC", "1"); }
        if let Some(ref auth) = self.default_auth { builder = builder.header("Authorization", auth.as_str()); }
        let req = builder.body(Full::new(Bytes::new()))?;
        info!("AmniNet OPTIONS {}", url);
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok(AmniResponse { status, headers, body, url: url.to_string(), from_cache: false })
    }

    // --- Request Builder Pattern ---

    /// Create a new request builder for the given URL.
    pub fn request(&self, url: &str) -> AmniRequestBuilder {
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), self.user_agent.clone());
        if self.dnt {
            default_headers.insert("DNT".to_string(), "1".to_string());
            default_headers.insert("Sec-GPC".to_string(), "1".to_string());
        }
        if let Some(ref auth) = self.default_auth {
            default_headers.insert("Authorization".to_string(), auth.clone());
        }
        AmniRequestBuilder {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: default_headers,
            body: None,
            timeout: None,
            client: &self.client,
        }
    }

    // --- Auth Support ---

    /// Set a Bearer token for all subsequent requests.
    pub fn set_bearer_token(&mut self, token: &str) {
        self.default_auth = Some(format!("Bearer {}", token));
    }

    /// Set Basic auth credentials for all subsequent requests.
    pub fn set_basic_auth(&mut self, user: &str, pass: &str) {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", user, pass));
        self.default_auth = Some(format!("Basic {}", encoded));
    }
}

/// Builder pattern for constructing HTTP requests with fine-grained control.
pub struct AmniRequestBuilder<'a> {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    timeout: Option<Duration>,
    client: &'a Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>,
}

impl<'a> AmniRequestBuilder<'a> {
    /// Set the HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS).
    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_uppercase();
        self
    }

    /// Set a request header. Overwrites any existing header with the same key.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the request body.
    pub fn body(mut self, data: Vec<u8>) -> Self {
        self.body = Some(data);
        self
    }

    /// Set a request timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Send the request and return the response.
    pub async fn send(self) -> Result<AmniResponse, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = self.url.parse()?;
        let mut builder = Request::builder()
            .method(self.method.as_str())
            .uri(&uri);
        for (k, v) in &self.headers {
            builder = builder.header(k.as_str(), v.as_str());
        }
        let body_bytes = self.body.map(Bytes::from).unwrap_or_else(Bytes::new);
        let req = builder.body(Full::new(body_bytes))?;
        info!("AmniNet {} {}", self.method, self.url);

        let resp: Response<Incoming> = if let Some(timeout_dur) = self.timeout {
            tokio::time::timeout(timeout_dur, self.client.request(req)).await
                .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "request timed out"))
                })??
        } else {
            self.client.request(req).await?
        };

        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp.headers().iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok(AmniResponse { status, headers, body, url: self.url, from_cache: false })
    }
}
impl AmniResponse {
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.to_vec())
    }

    /// Find a response header by name (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Get the Content-Type header value, if present.
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Check if the response Content-Type indicates JSON.
    pub fn is_json(&self) -> bool {
        self.content_type().map_or(false, |ct| ct.contains("application/json"))
    }

    /// Check if the response Content-Type indicates HTML.
    pub fn is_html(&self) -> bool {
        self.content_type().map_or(false, |ct| ct.contains("text/html"))
    }

    /// Deserialize the response body as JSON into the given type.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let text = String::from_utf8(self.body.to_vec())?;
        let parsed = serde_json::from_str(&text)?;
        Ok(parsed)
    }

    pub fn is_ok(&self) -> bool { self.status >= 200 && self.status < 300 }
    pub fn is_redirect(&self) -> bool { self.status >= 300 && self.status < 400 }
    pub fn redirect_url(&self) -> Option<String> {
        self.is_redirect().then(|| self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("location")).map(|(_, v)| v.clone())).flatten()
    }
}
