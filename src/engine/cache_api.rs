use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub cached_at: u64,
}

pub struct Cache {
    pub name: String,
    entries: HashMap<String, CachedResponse>,
}

impl Cache {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), entries: HashMap::new() }
    }

    pub fn put(&mut self, url: &str, status: u16, headers: HashMap<String, String>, body: Vec<u8>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.entries.insert(url.to_string(), CachedResponse {
            url: url.to_string(), status, headers, body, cached_at: now,
        });
    }

    pub fn match_request(&self, url: &str) -> Option<&CachedResponse> {
        self.entries.get(url)
    }

    pub fn match_all(&self, url: &str) -> Vec<&CachedResponse> {
        self.entries.get(url).into_iter().collect()
    }

    pub fn delete(&mut self, url: &str) -> bool {
        self.entries.remove(url).is_some()
    }

    pub fn keys(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }
}

pub struct CacheStorage {
    caches: HashMap<String, Cache>,
    origin_access: HashMap<String, Vec<String>>,
}

impl CacheStorage {
    pub fn new() -> Self {
        Self { caches: HashMap::new(), origin_access: HashMap::new() }
    }

    pub fn open(&mut self, origin: &str, name: &str) -> &mut Cache {
        let access = self.origin_access.entry(origin.to_string()).or_default();
        if !access.contains(&name.to_string()) {
            access.push(name.to_string());
        }
        self.caches.entry(name.to_string()).or_insert_with(|| Cache::new(name))
    }

    pub fn has(&self, origin: &str, name: &str) -> bool {
        self.origin_access.get(origin)
            .map(|names| names.contains(&name.to_string()))
            .unwrap_or(false)
            && self.caches.contains_key(name)
    }

    pub fn delete(&mut self, origin: &str, name: &str) -> bool {
        if let Some(access) = self.origin_access.get_mut(origin) {
            access.retain(|n| n != name);
        }
        self.caches.remove(name).is_some()
    }

    pub fn keys(&self, origin: &str) -> Vec<String> {
        self.origin_access.get(origin)
            .map(|names| names.iter()
                .filter(|n| self.caches.contains_key(n.as_str()))
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    pub fn match_request(&self, origin: &str, url: &str) -> Option<&CachedResponse> {
        let names = self.origin_access.get(origin)?;
        for name in names {
            if let Some(cache) = self.caches.get(name) {
                if let Some(resp) = cache.match_request(url) {
                    return Some(resp);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_and_match() {
        let mut cache = Cache::new("v1");
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        cache.put("https://example.com/", 200, headers, b"<html>hi</html>".to_vec());
        let resp = cache.match_request("https://example.com/").unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"<html>hi</html>");
        assert!(cache.match_request("https://other.com/").is_none());
    }

    #[test]
    fn test_cache_delete_and_keys() {
        let mut cache = Cache::new("v1");
        cache.put("https://a.com/", 200, HashMap::new(), vec![]);
        cache.put("https://b.com/", 200, HashMap::new(), vec![]);
        assert_eq!(cache.keys().len(), 2);
        assert!(cache.delete("https://a.com/"));
        assert!(!cache.delete("https://a.com/"));
        assert_eq!(cache.keys().len(), 1);
    }

    #[test]
    fn test_cache_storage_open_and_has() {
        let mut storage = CacheStorage::new();
        storage.open("https://example.com", "v1");
        assert!(storage.has("https://example.com", "v1"));
        assert!(!storage.has("https://other.com", "v1"));
        assert!(!storage.has("https://example.com", "v2"));
    }

    #[test]
    fn test_cache_storage_delete_and_keys() {
        let mut storage = CacheStorage::new();
        storage.open("https://example.com", "v1");
        storage.open("https://example.com", "v2");
        assert_eq!(storage.keys("https://example.com").len(), 2);
        assert!(storage.delete("https://example.com", "v1"));
        assert!(!storage.has("https://example.com", "v1"));
        assert_eq!(storage.keys("https://example.com").len(), 1);
    }

    #[test]
    fn test_cache_storage_match_request() {
        let mut storage = CacheStorage::new();
        let cache = storage.open("https://example.com", "static");
        cache.put("https://cdn.example.com/style.css", 200,
            HashMap::new(), b"body{}".to_vec());
        let resp = storage.match_request("https://example.com", "https://cdn.example.com/style.css").unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"body{}");
        assert!(storage.match_request("https://other.com", "https://cdn.example.com/style.css").is_none());
    }

    #[test]
    fn test_cache_match_all() {
        let mut cache = Cache::new("v1");
        cache.put("https://example.com/", 200, HashMap::new(), vec![1, 2, 3]);
        let results = cache.match_all("https://example.com/");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].body, vec![1, 2, 3]);
        let empty = cache.match_all("https://nope.com/");
        assert!(empty.is_empty());
    }
}
