use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct WorkerHandle {
    pub id: u32,
    pub script_url: String,
    pub message_queue: Vec<String>,
}
pub struct WorkerManager {
    workers: HashMap<u32, WorkerHandle>,
    next_id: u32,
}
impl WorkerManager {
    pub fn new() -> Self { Self { workers: HashMap::new(), next_id: 1 } }
    pub fn create_worker(&mut self, script_url: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.workers.insert(id, WorkerHandle { id, script_url: script_url.to_string(), message_queue: Vec::new() });
        id
    }
    pub fn post_message(&mut self, worker_id: u32, message: &str) {
        if let Some(w) = self.workers.get_mut(&worker_id) {
            w.message_queue.push(message.to_string());
        }
    }
    pub fn terminate(&mut self, worker_id: u32) { self.workers.remove(&worker_id); }
    pub fn drain_messages(&mut self, worker_id: u32) -> Vec<String> {
        self.workers.get_mut(&worker_id).map(|w| std::mem::take(&mut w.message_queue)).unwrap_or_default()
    }
    pub fn worker_count(&self) -> usize { self.workers.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_and_count() {
        let mut mgr = WorkerManager::new();
        assert_eq!(mgr.worker_count(), 0);
        let id1 = mgr.create_worker("worker.js");
        let id2 = mgr.create_worker("worker2.js");
        assert_ne!(id1, id2);
        assert_eq!(mgr.worker_count(), 2);
    }
    #[test]
    fn test_post_and_drain() {
        let mut mgr = WorkerManager::new();
        let id = mgr.create_worker("w.js");
        mgr.post_message(id, "hello");
        mgr.post_message(id, "world");
        let msgs = mgr.drain_messages(id);
        assert_eq!(msgs, vec!["hello", "world"]);
        assert_eq!(mgr.drain_messages(id), Vec::<String>::new());
    }
    #[test]
    fn test_terminate() {
        let mut mgr = WorkerManager::new();
        let id = mgr.create_worker("w.js");
        mgr.post_message(id, "msg");
        mgr.terminate(id);
        assert_eq!(mgr.worker_count(), 0);
        assert_eq!(mgr.drain_messages(id), Vec::<String>::new());
    }
    #[test]
    fn test_post_to_nonexistent() {
        let mut mgr = WorkerManager::new();
        mgr.post_message(999, "nope");
        assert_eq!(mgr.drain_messages(999), Vec::<String>::new());
    }
}
