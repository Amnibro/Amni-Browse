use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState { Stopped, Playing, Paused }

#[derive(Debug, Clone)]
pub struct AudioTrack {
    pub id: String,
    pub url: String,
    pub state: PlaybackState,
    pub duration_secs: f64,
    pub position_secs: f64,
    pub volume: f32,
    pub muted: bool,
}

pub struct AudioEngine {
    tracks: HashMap<String, AudioTrack>,
    master_volume: f32,
    master_muted: bool,
    #[cfg(feature = "media-playback")]
    _host: Option<cpal::Host>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
            master_volume: 1.0,
            master_muted: false,
            #[cfg(feature = "media-playback")]
            _host: Self::init_audio_host(),
        }
    }

    #[cfg(feature = "media-playback")]
    fn init_audio_host() -> Option<cpal::Host> {
        use cpal::traits::HostTrait;
        let host = cpal::default_host();
        if host.default_output_device().is_some() {
            log::info!("Audio output device available");
            Some(host)
        } else {
            log::warn!("No audio output device found");
            None
        }
    }

    pub fn load_track(&mut self, url: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.tracks.insert(id.clone(), AudioTrack {
            id: id.clone(), url: url.to_string(),
            state: PlaybackState::Stopped,
            duration_secs: 0.0, position_secs: 0.0,
            volume: 1.0, muted: false,
        });
        id
    }

    pub fn play(&mut self, id: &str) -> bool {
        if let Some(track) = self.tracks.get_mut(id) {
            track.state = PlaybackState::Playing;
            self.start_playback(id);
            true
        } else { false }
    }

    pub fn pause(&mut self, id: &str) -> bool {
        if let Some(track) = self.tracks.get_mut(id) {
            track.state = PlaybackState::Paused;
            true
        } else { false }
    }

    pub fn stop(&mut self, id: &str) -> bool {
        if let Some(track) = self.tracks.get_mut(id) {
            track.state = PlaybackState::Stopped;
            track.position_secs = 0.0;
            true
        } else { false }
    }

    pub fn seek(&mut self, id: &str, position_secs: f64) -> bool {
        if let Some(track) = self.tracks.get_mut(id) {
            track.position_secs = position_secs.clamp(0.0, track.duration_secs);
            true
        } else { false }
    }

    pub fn set_volume(&mut self, id: &str, volume: f32) {
        if let Some(track) = self.tracks.get_mut(id) {
            track.volume = volume.clamp(0.0, 1.0);
        }
    }

    pub fn set_muted(&mut self, id: &str, muted: bool) {
        if let Some(track) = self.tracks.get_mut(id) { track.muted = muted; }
    }

    pub fn set_master_volume(&mut self, vol: f32) { self.master_volume = vol.clamp(0.0, 1.0); }
    pub fn set_master_muted(&mut self, muted: bool) { self.master_muted = muted; }
    pub fn master_volume(&self) -> f32 { self.master_volume }
    pub fn master_muted(&self) -> bool { self.master_muted }

    pub fn get_track(&self, id: &str) -> Option<&AudioTrack> { self.tracks.get(id) }
    pub fn remove_track(&mut self, id: &str) { self.stop(id); self.tracks.remove(id); }
    pub fn active_tracks(&self) -> Vec<&AudioTrack> {
        self.tracks.values().filter(|t| t.state == PlaybackState::Playing).collect()
    }

    fn start_playback(&self, _id: &str) {
        #[cfg(feature = "media-playback")]
        {
            log::info!("Audio playback requested for track {}", _id);
        }
    }

    pub fn decode_audio_info(data: &[u8]) -> Option<AudioInfo> {
        #[cfg(feature = "media-playback")]
        {
            use symphonia::core::io::MediaSourceStream;
            use symphonia::core::probe::Hint;
            let cursor = std::io::Cursor::new(data.to_vec());
            let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
            let hint = Hint::new();
            let probed = symphonia::default::get_probe().format(&hint, mss, &Default::default(), &Default::default());
            if let Ok(mut probed) = probed {
                let format = probed.format;
                if let Some(track) = format.tracks().first() {
                    let codec = track.codec_params.codec.to_string();
                    let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
                    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(0);
                    let duration = track.codec_params.n_frames
                        .map(|f| f as f64 / sample_rate.max(1) as f64)
                        .unwrap_or(0.0);
                    return Some(AudioInfo { codec, sample_rate, channels, duration_secs: duration });
                }
            }
        }
        let _ = data;
        None
    }

    pub fn is_available() -> bool { cfg!(feature = "media-playback") }
}

#[derive(Debug, Clone)]
pub struct AudioInfo {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_secs: f64,
}
