use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MediaSourceState { Closed, Open, Ended }

#[derive(Debug, Clone, PartialEq)]
pub enum AppendMode { Segments, Sequence }

#[derive(Debug, Clone)]
pub struct SourceBuffer {
    pub id: String,
    pub mime_type: String,
    pub mode: AppendMode,
    pub buffered_ranges: Vec<(f64, f64)>,
    pub pending_data: Vec<Vec<u8>>,
    pub updating: bool,
    pub timestamp_offset: f64,
}

impl SourceBuffer {
    pub fn append_buffer(&mut self, data: Vec<u8>) {
        self.pending_data.push(data);
        self.updating = true;
    }

    pub fn abort(&mut self) {
        self.pending_data.clear();
        self.updating = false;
    }

    pub fn remove(&mut self, start: f64, end: f64) {
        self.buffered_ranges = self.buffered_ranges.iter().filter_map(|&(s, e)| {
            if e <= start || s >= end { Some((s, e)) }
            else if s < start && e > end { Some((s, start)) }
            else if s < start { Some((s, start)) }
            else if e > end { Some((end, e)) }
            else { None }
        }).filter(|(s, e)| e > s).collect();
    }

    pub fn buffered_duration(&self) -> f64 {
        self.buffered_ranges.iter().map(|(s, e)| e - s).sum()
    }

    pub fn finish_update(&mut self) {
        if !self.pending_data.is_empty() {
            let total_bytes: usize = self.pending_data.iter().map(|d| d.len()).sum();
            let chunk_duration = total_bytes as f64 / 1000.0;
            let start = if let Some(&(_, last_end)) = self.buffered_ranges.last() {
                last_end
            } else {
                self.timestamp_offset
            };
            self.buffered_ranges.push((start, start + chunk_duration));
            self.merge_ranges();
            self.pending_data.clear();
        }
        self.updating = false;
    }

    fn merge_ranges(&mut self) {
        if self.buffered_ranges.len() < 2 { return; }
        self.buffered_ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut merged: Vec<(f64, f64)> = vec![self.buffered_ranges[0]];
        for &(s, e) in &self.buffered_ranges[1..] {
            let last = merged.last_mut().unwrap();
            if s <= last.1 { last.1 = last.1.max(e); }
            else { merged.push((s, e)); }
        }
        self.buffered_ranges = merged;
    }
}

#[derive(Debug, Clone)]
pub struct MediaSource {
    pub id: String,
    pub state: MediaSourceState,
    pub duration: f64,
    pub source_buffers: Vec<SourceBuffer>,
}

impl MediaSource {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: MediaSourceState::Closed,
            duration: 0.0,
            source_buffers: Vec::new(),
        }
    }

    pub fn add_source_buffer(&mut self, mime_type: &str) -> Result<usize, String> {
        if self.state != MediaSourceState::Open {
            return Err("MediaSource is not open".to_string());
        }
        let buf = SourceBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            mime_type: mime_type.to_string(),
            mode: AppendMode::Segments,
            buffered_ranges: Vec::new(),
            pending_data: Vec::new(),
            updating: false,
            timestamp_offset: 0.0,
        };
        self.source_buffers.push(buf);
        Ok(self.source_buffers.len() - 1)
    }

    pub fn remove_source_buffer(&mut self, index: usize) -> bool {
        if index < self.source_buffers.len() {
            self.source_buffers.remove(index);
            true
        } else {
            false
        }
    }

    pub fn end_of_stream(&mut self) {
        self.state = MediaSourceState::Ended;
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
    }

    pub fn active_source_buffers(&self) -> Vec<&SourceBuffer> {
        self.source_buffers.iter().filter(|b| !b.buffered_ranges.is_empty() || b.updating).collect()
    }
}

pub struct MseManager {
    sources: HashMap<String, MediaSource>,
    video_attachments: HashMap<String, String>,
}

impl MseManager {
    pub fn new() -> Self {
        Self { sources: HashMap::new(), video_attachments: HashMap::new() }
    }

    pub fn create_source(&mut self) -> String {
        let mut src = MediaSource::new();
        src.state = MediaSourceState::Open;
        let id = src.id.clone();
        self.sources.insert(id.clone(), src);
        id
    }

    pub fn get_source(&self, id: &str) -> Option<&MediaSource> {
        self.sources.get(id)
    }

    pub fn get_source_mut(&mut self, id: &str) -> Option<&mut MediaSource> {
        self.sources.get_mut(id)
    }

    pub fn attach_to_video(&mut self, source_id: &str, video_id: &str) -> bool {
        if self.sources.contains_key(source_id) {
            self.video_attachments.insert(video_id.to_string(), source_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn detach(&mut self, source_id: &str) {
        if let Some(src) = self.sources.get_mut(source_id) {
            src.state = MediaSourceState::Closed;
        }
        self.video_attachments.retain(|_, v| v != source_id);
    }

    pub fn is_type_supported(mime: &str) -> bool {
        let supported = [
            "video/mp4", "video/webm", "audio/mp4", "audio/webm",
            "video/mp4; codecs=\"avc1.42E01E\"",
            "video/mp4; codecs=\"avc1.42E01E, mp4a.40.2\"",
            "video/webm; codecs=\"vp8\"",
            "video/webm; codecs=\"vp8, vorbis\"",
            "video/webm; codecs=\"vp9\"",
            "video/webm; codecs=\"vp9, opus\"",
            "audio/webm; codecs=\"opus\"",
            "audio/webm; codecs=\"vorbis\"",
            "audio/mp4; codecs=\"mp4a.40.2\"",
        ];
        let lower = mime.to_lowercase();
        supported.iter().any(|s| lower.starts_with(&s.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_source() {
        let mut mgr = MseManager::new();
        let id = mgr.create_source();
        let src = mgr.get_source(&id).unwrap();
        assert_eq!(src.state, MediaSourceState::Open);
        assert!(src.source_buffers.is_empty());
    }

    #[test]
    fn test_add_remove_source_buffer() {
        let mut mgr = MseManager::new();
        let id = mgr.create_source();
        let src = mgr.get_source_mut(&id).unwrap();
        let idx = src.add_source_buffer("video/mp4").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(src.source_buffers.len(), 1);
        assert!(src.remove_source_buffer(0));
        assert!(src.source_buffers.is_empty());
        assert!(!src.remove_source_buffer(5));
    }

    #[test]
    fn test_source_buffer_append_and_finish() {
        let mut buf = SourceBuffer {
            id: "test".to_string(),
            mime_type: "video/mp4".to_string(),
            mode: AppendMode::Segments,
            buffered_ranges: Vec::new(),
            pending_data: Vec::new(),
            updating: false,
            timestamp_offset: 0.0,
        };
        buf.append_buffer(vec![0; 1000]);
        assert!(buf.updating);
        assert_eq!(buf.pending_data.len(), 1);
        buf.finish_update();
        assert!(!buf.updating);
        assert!(buf.pending_data.is_empty());
        assert_eq!(buf.buffered_ranges.len(), 1);
        assert!((buf.buffered_duration() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_source_buffer_remove_range() {
        let mut buf = SourceBuffer {
            id: "test".to_string(),
            mime_type: "video/mp4".to_string(),
            mode: AppendMode::Segments,
            buffered_ranges: vec![(0.0, 10.0)],
            pending_data: Vec::new(),
            updating: false,
            timestamp_offset: 0.0,
        };
        buf.remove(3.0, 7.0);
        assert_eq!(buf.buffered_ranges.len(), 1);
        assert_eq!(buf.buffered_ranges[0], (0.0, 3.0));
        assert!((buf.buffered_duration() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_is_type_supported() {
        assert!(MseManager::is_type_supported("video/mp4"));
        assert!(MseManager::is_type_supported("video/webm; codecs=\"vp9\""));
        assert!(MseManager::is_type_supported("audio/webm; codecs=\"opus\""));
        assert!(!MseManager::is_type_supported("video/avi"));
        assert!(!MseManager::is_type_supported("text/plain"));
    }

    #[test]
    fn test_attach_detach() {
        let mut mgr = MseManager::new();
        let id = mgr.create_source();
        assert!(mgr.attach_to_video(&id, "video-1"));
        assert!(!mgr.attach_to_video("nonexistent", "video-2"));
        mgr.detach(&id);
        let src = mgr.get_source(&id).unwrap();
        assert_eq!(src.state, MediaSourceState::Closed);
    }

    #[test]
    fn test_end_of_stream_and_duration() {
        let mut mgr = MseManager::new();
        let id = mgr.create_source();
        let src = mgr.get_source_mut(&id).unwrap();
        src.set_duration(120.5);
        assert_eq!(src.duration, 120.5);
        src.end_of_stream();
        assert_eq!(src.state, MediaSourceState::Ended);
    }
}
