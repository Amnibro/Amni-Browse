use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
#[derive(Debug, Clone)]
pub struct GeolocationResult {
    pub success: bool,
    pub latitude: f64,
    pub longitude: f64,
    pub error: String,
}
pub struct GeolocationApi;
impl GeolocationApi {
    pub fn get_position() -> GeolocationResult {
        GeolocationResult { success: false, latitude: 0.0, longitude: 0.0, error: "permission denied".to_string() }
    }
}
pub struct NotificationsApi;
impl NotificationsApi {
    pub fn request_permission() -> String { "denied".to_string() }
    pub fn show_notification(_title: &str, _body: &str) -> bool { false }
}
pub struct PerformanceApi {
    timing: HashMap<String, f64>,
    start: Instant,
    navigation_start_ms: f64,
}
impl PerformanceApi {
    pub fn new() -> Self {
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0);
        Self { timing: HashMap::new(), start: Instant::now(), navigation_start_ms: now_ms }
    }
    pub fn mark(&mut self, name: &str) {
        let ts = self.now_internal();
        self.timing.insert(name.to_string(), ts);
    }
    pub fn measure(&self, _name: &str, start_mark: &str, end_mark: &str) -> Option<f64> {
        let s = self.timing.get(start_mark)?;
        let e = self.timing.get(end_mark)?;
        Some(e - s)
    }
    pub fn now() -> f64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0)
    }
    pub fn navigation_start(&self) -> f64 { self.navigation_start_ms }
    fn now_internal(&self) -> f64 { self.start.elapsed().as_secs_f64() * 1000.0 }
}
#[derive(Debug, Clone)]
pub struct IntersectionEntry {
    pub target_id: usize,
    pub is_intersecting: bool,
    pub ratio: f32,
}
pub struct IntersectionObserverStub {
    pub entries: Vec<IntersectionEntry>,
    observed: Vec<usize>,
}
impl IntersectionObserverStub {
    pub fn new() -> Self { Self { entries: Vec::new(), observed: Vec::new() } }
    pub fn observe(&mut self, target_id: usize) {
        if !self.observed.contains(&target_id) { self.observed.push(target_id); }
    }
    pub fn unobserve(&mut self, target_id: usize) { self.observed.retain(|&id| id != target_id); }
    pub fn check_intersections(&self, viewport_rect: &(f32, f32, f32, f32), layouts: &HashMap<usize, crate::engine::layout::LayoutRect>) -> Vec<IntersectionEntry> {
        let (vx, vy, vw, vh) = *viewport_rect;
        let mut results = Vec::new();
        for &tid in &self.observed {
            if let Some(rect) = layouts.get(&tid) {
                let ix0 = rect.x.max(vx);
                let iy0 = rect.y.max(vy);
                let ix1 = (rect.x + rect.w).min(vx + vw);
                let iy1 = (rect.y + rect.h).min(vy + vh);
                let inter_area = (ix1 - ix0).max(0.0) * (iy1 - iy0).max(0.0);
                let target_area = rect.w * rect.h;
                let ratio = if target_area > 0.0 { (inter_area / target_area).clamp(0.0, 1.0) } else { 0.0 };
                results.push(IntersectionEntry { target_id: tid, is_intersecting: ratio > 0.0, ratio });
            } else {
                results.push(IntersectionEntry { target_id: tid, is_intersecting: false, ratio: 0.0 });
            }
        }
        results
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::layout::LayoutRect;
    #[test]
    fn test_geolocation_denied() {
        let res = GeolocationApi::get_position();
        assert!(!res.success);
        assert_eq!(res.error, "permission denied");
    }
    #[test]
    fn test_notifications_denied() {
        assert_eq!(NotificationsApi::request_permission(), "denied");
        assert!(!NotificationsApi::show_notification("test", "body"));
    }
    #[test]
    fn test_performance_marks() {
        let mut perf = PerformanceApi::new();
        perf.mark("start");
        std::thread::sleep(std::time::Duration::from_millis(10));
        perf.mark("end");
        let dur = perf.measure("test", "start", "end");
        assert!(dur.is_some());
        assert!(dur.unwrap() >= 5.0);
    }
    #[test]
    fn test_performance_measure_missing() {
        let perf = PerformanceApi::new();
        assert!(perf.measure("x", "nonexistent", "also_nonexistent").is_none());
    }
    #[test]
    fn test_performance_now() {
        let t = PerformanceApi::now();
        assert!(t > 0.0);
    }
    #[test]
    fn test_performance_navigation_start() {
        let perf = PerformanceApi::new();
        assert!(perf.navigation_start() > 0.0);
    }
    #[test]
    fn test_intersection_observe_unobserve() {
        let mut obs = IntersectionObserverStub::new();
        obs.observe(1);
        obs.observe(2);
        obs.observe(1);
        obs.unobserve(1);
        let mut layouts = HashMap::new();
        layouts.insert(2, LayoutRect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 });
        let results = obs.check_intersections(&(0.0, 0.0, 800.0, 600.0), &layouts);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target_id, 2);
        assert!(results[0].is_intersecting);
    }
    #[test]
    fn test_intersection_partial() {
        let mut obs = IntersectionObserverStub::new();
        obs.observe(1);
        let mut layouts = HashMap::new();
        layouts.insert(1, LayoutRect { x: -50.0, y: 0.0, w: 100.0, h: 100.0 });
        let results = obs.check_intersections(&(0.0, 0.0, 800.0, 600.0), &layouts);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_intersecting);
        assert!(results[0].ratio > 0.0 && results[0].ratio < 1.0);
    }
    #[test]
    fn test_intersection_no_layout() {
        let mut obs = IntersectionObserverStub::new();
        obs.observe(99);
        let layouts = HashMap::new();
        let results = obs.check_intersections(&(0.0, 0.0, 800.0, 600.0), &layouts);
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_intersecting);
        assert_eq!(results[0].ratio, 0.0);
    }
}
