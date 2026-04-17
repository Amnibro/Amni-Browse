pub mod audio;
pub mod video;
pub mod codecs;
pub mod stream;
pub mod mse;

pub use audio::{AudioEngine, AudioTrack, PlaybackState, AudioInfo};
pub use video::{VideoEngine, VideoTrack, VideoState};

pub struct MediaManager {
    pub audio: AudioEngine,
    pub video: VideoEngine,
    pub codec_registry: codecs::CodecRegistry,
    pub stream_manager: stream::MediaStreamManager,
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            audio: AudioEngine::new(),
            video: VideoEngine::new(),
            codec_registry: codecs::CodecRegistry::new(),
            stream_manager: stream::MediaStreamManager::new(),
        }
    }

    pub fn stop_all(&mut self) {
        let audio_ids: Vec<String> = self.audio.active_tracks().iter().map(|t| t.id.clone()).collect();
        for id in audio_ids { self.audio.stop(&id); }
    }

    pub fn is_media_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        let audio_exts = ["mp3", "ogg", "wav", "flac", "aac", "m4a", "opus"];
        let video_exts = ["mp4", "webm", "mkv", "avi", "mov"];
        audio_exts.iter().any(|e| lower.ends_with(e)) || video_exts.iter().any(|e| lower.ends_with(e))
    }

    pub fn capabilities_json() -> String {
        serde_json::json!({
            "audio_available": AudioEngine::is_available(),
            "audio_formats": ["mp3", "ogg", "wav", "flac", "aac"],
            "video_containers": video::VideoEngine::supported_containers(),
            "image_formats": crate::engine::image_decode::ImageCache::supported_formats(),
        }).to_string()
    }
}
