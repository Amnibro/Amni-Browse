use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VideoState { Idle, Loading, Playing, Paused, Ended, Error(String) }

#[derive(Debug, Clone)]
pub struct VideoTrack {
    pub id: String,
    pub url: String,
    pub state: VideoState,
    pub width: u32,
    pub height: u32,
    pub duration_secs: f64,
    pub position_secs: f64,
    pub volume: f32,
    pub muted: bool,
    pub frame_data: Option<Vec<u8>>,
}

pub struct VideoEngine {
    tracks: HashMap<String, VideoTrack>,
}

impl VideoEngine {
    pub fn new() -> Self { Self { tracks: HashMap::new() } }

    pub fn load_track(&mut self, url: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.tracks.insert(id.clone(), VideoTrack {
            id: id.clone(), url: url.to_string(),
            state: VideoState::Idle,
            width: 0, height: 0,
            duration_secs: 0.0, position_secs: 0.0,
            volume: 1.0, muted: false,
            frame_data: None,
        });
        id
    }

    pub fn play(&mut self, id: &str) -> bool {
        if let Some(t) = self.tracks.get_mut(id) { t.state = VideoState::Playing; true } else { false }
    }

    pub fn pause(&mut self, id: &str) -> bool {
        if let Some(t) = self.tracks.get_mut(id) { t.state = VideoState::Paused; true } else { false }
    }

    pub fn stop(&mut self, id: &str) -> bool {
        if let Some(t) = self.tracks.get_mut(id) {
            t.state = VideoState::Idle; t.position_secs = 0.0; t.frame_data = None; true
        } else { false }
    }

    pub fn seek(&mut self, id: &str, pos: f64) -> bool {
        if let Some(t) = self.tracks.get_mut(id) {
            t.position_secs = pos.clamp(0.0, t.duration_secs); true
        } else { false }
    }

    pub fn set_volume(&mut self, id: &str, vol: f32) {
        if let Some(t) = self.tracks.get_mut(id) { t.volume = vol.clamp(0.0, 1.0); }
    }

    pub fn set_muted(&mut self, id: &str, muted: bool) {
        if let Some(t) = self.tracks.get_mut(id) { t.muted = muted; }
    }

    pub fn get_track(&self, id: &str) -> Option<&VideoTrack> { self.tracks.get(id) }

    pub fn remove_track(&mut self, id: &str) { self.tracks.remove(id); }

    pub fn extract_poster_frame(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
        // For image-based poster frames (e.g., from thumbnail URLs)
        if let Ok(img) = image::load_from_memory(data) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            Some((w, h, rgba.into_raw()))
        } else { None }
    }

    pub fn supported_containers() -> &'static [&'static str] {
        &["mp4", "webm", "ogg", "mkv", "avi"]
    }

    pub fn is_video_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        Self::supported_containers().iter().any(|ext| lower.ends_with(ext))
            || lower.contains("video") || lower.contains("/watch")
    }
}
