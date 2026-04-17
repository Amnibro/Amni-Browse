use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceWorkerState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistration {
    pub scope: String,
    pub script_url: String,
    pub state: ServiceWorkerState,
    pub registered_at: u64,
}

#[derive(Debug, Clone)]
pub struct ServiceWorkerResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct ServiceWorkerManager {
    registrations: HashMap<String, ServiceWorkerRegistration>,
}

impl ServiceWorkerManager {
    pub fn new() -> Self { Self { registrations: HashMap::new() } }

    pub fn register(&mut self, scope: &str, script_url: &str) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let reg = ServiceWorkerRegistration {
            scope: scope.to_string(),
            script_url: script_url.to_string(),
            state: ServiceWorkerState::Installing,
            registered_at: now,
        };
        self.registrations.insert(scope.to_string(), reg);
        scope.to_string()
    }

    pub fn unregister(&mut self, scope: &str) -> bool {
        self.registrations.remove(scope).is_some()
    }

    pub fn get_registration(&self, scope: &str) -> Option<&ServiceWorkerRegistration> {
        self.registrations.get(scope)
    }

    pub fn get_registrations(&self) -> Vec<&ServiceWorkerRegistration> {
        self.registrations.values().collect()
    }

    pub fn matches_scope(&self, url: &str) -> Option<&ServiceWorkerRegistration> {
        let mut best: Option<&ServiceWorkerRegistration> = None;
        let mut best_len = 0;
        for reg in self.registrations.values() {
            if url.starts_with(&reg.scope) && reg.scope.len() > best_len {
                best = Some(reg);
                best_len = reg.scope.len();
            }
        }
        best
    }

    pub fn intercept_fetch(&self, _url: &str, _method: &str) -> Option<ServiceWorkerResponse> {
        None
    }

    pub fn registration_count(&self) -> usize { self.registrations.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut mgr = ServiceWorkerManager::new();
        let id = mgr.register("https://example.com/", "sw.js");
        assert_eq!(id, "https://example.com/");
        let reg = mgr.get_registration("https://example.com/").unwrap();
        assert_eq!(reg.script_url, "sw.js");
        assert_eq!(reg.state, ServiceWorkerState::Installing);
        assert_eq!(mgr.registration_count(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut mgr = ServiceWorkerManager::new();
        mgr.register("https://example.com/", "sw.js");
        assert!(mgr.unregister("https://example.com/"));
        assert!(!mgr.unregister("https://example.com/"));
        assert!(mgr.get_registration("https://example.com/").is_none());
        assert_eq!(mgr.registration_count(), 0);
    }

    #[test]
    fn test_get_registrations() {
        let mut mgr = ServiceWorkerManager::new();
        mgr.register("https://a.com/", "a.js");
        mgr.register("https://b.com/", "b.js");
        let regs = mgr.get_registrations();
        assert_eq!(regs.len(), 2);
    }

    #[test]
    fn test_matches_scope() {
        let mut mgr = ServiceWorkerManager::new();
        mgr.register("https://example.com/", "root.js");
        mgr.register("https://example.com/app/", "app.js");
        let m1 = mgr.matches_scope("https://example.com/app/page").unwrap();
        assert_eq!(m1.script_url, "app.js");
        let m2 = mgr.matches_scope("https://example.com/other").unwrap();
        assert_eq!(m2.script_url, "root.js");
        assert!(mgr.matches_scope("https://other.com/").is_none());
    }

    #[test]
    fn test_intercept_fetch_stub() {
        let mgr = ServiceWorkerManager::new();
        assert!(mgr.intercept_fetch("https://example.com/api", "GET").is_none());
    }

    #[test]
    fn test_register_overwrites() {
        let mut mgr = ServiceWorkerManager::new();
        mgr.register("https://example.com/", "old.js");
        mgr.register("https://example.com/", "new.js");
        let reg = mgr.get_registration("https://example.com/").unwrap();
        assert_eq!(reg.script_url, "new.js");
        assert_eq!(mgr.registration_count(), 1);
    }
}
