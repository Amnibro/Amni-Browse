use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TrackKind { Audio, Video }

#[derive(Debug, Clone, PartialEq)]
pub enum TrackState { Live, Ended }

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceKind { AudioInput, AudioOutput, VideoInput }

#[derive(Debug, Clone)]
pub struct MediaStreamTrack {
    pub id: String,
    pub kind: TrackKind,
    pub label: String,
    pub enabled: bool,
    pub muted: bool,
    pub ready_state: TrackState,
}

#[derive(Debug, Clone)]
pub struct MediaStream {
    pub id: String,
    pub tracks: Vec<MediaStreamTrack>,
}

#[derive(Debug, Clone)]
pub struct MediaDeviceInfo {
    pub device_id: String,
    pub kind: DeviceKind,
    pub label: String,
    pub group_id: String,
}

pub struct MediaStreamManager {
    pub streams: HashMap<String, MediaStream>,
}

impl MediaStreamManager {
    pub fn new() -> Self { Self { streams: HashMap::new() } }

    pub fn create_stream(&mut self) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.streams.insert(id.clone(), MediaStream { id: id.clone(), tracks: Vec::new() });
        id
    }

    pub fn add_track(&mut self, stream_id: &str, kind: TrackKind, label: &str) -> Option<String> {
        let stream = self.streams.get_mut(stream_id)?;
        let track_id = uuid::Uuid::new_v4().to_string();
        stream.tracks.push(MediaStreamTrack {
            id: track_id.clone(), kind, label: label.to_string(),
            enabled: true, muted: false, ready_state: TrackState::Live,
        });
        Some(track_id)
    }

    pub fn remove_track(&mut self, stream_id: &str, track_id: &str) -> bool {
        if let Some(stream) = self.streams.get_mut(stream_id) {
            let before = stream.tracks.len();
            stream.tracks.retain(|t| t.id != track_id);
            stream.tracks.len() < before
        } else { false }
    }

    pub fn get_stream(&self, id: &str) -> Option<&MediaStream> { self.streams.get(id) }

    pub fn get_user_media(&mut self, audio: bool, video: bool) -> Result<String, String> {
        if !audio && !video { return Err("At least one of audio or video must be requested".into()); }
        let sid = self.create_stream();
        if audio { self.add_track(&sid, TrackKind::Audio, "Default Microphone"); }
        if video { self.add_track(&sid, TrackKind::Video, "Default Camera"); }
        Ok(sid)
    }

    pub fn get_display_media(&mut self) -> Result<String, String> {
        let sid = self.create_stream();
        self.add_track(&sid, TrackKind::Video, "Screen Capture");
        self.add_track(&sid, TrackKind::Audio, "System Audio");
        Ok(sid)
    }

    pub fn stop_stream(&mut self, id: &str) {
        if let Some(stream) = self.streams.get_mut(id) {
            for track in &mut stream.tracks {
                track.ready_state = TrackState::Ended;
                track.enabled = false;
            }
        }
    }

    pub fn enumerate_devices(&self) -> Vec<MediaDeviceInfo> {
        vec![
            MediaDeviceInfo {
                device_id: "audio-input-default".into(), kind: DeviceKind::AudioInput,
                label: "Default Microphone".into(), group_id: "default".into(),
            },
            MediaDeviceInfo {
                device_id: "audio-output-default".into(), kind: DeviceKind::AudioOutput,
                label: "Default Speaker".into(), group_id: "default".into(),
            },
            MediaDeviceInfo {
                device_id: "video-input-default".into(), kind: DeviceKind::VideoInput,
                label: "Default Camera".into(), group_id: "default".into(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_stream_and_add_tracks() {
        let mut mgr = MediaStreamManager::new();
        let sid = mgr.create_stream();
        let tid = mgr.add_track(&sid, TrackKind::Audio, "mic").unwrap();
        let stream = mgr.get_stream(&sid).unwrap();
        assert_eq!(stream.tracks.len(), 1);
        assert_eq!(stream.tracks[0].id, tid);
        assert_eq!(stream.tracks[0].kind, TrackKind::Audio);
    }

    #[test]
    fn test_remove_track() {
        let mut mgr = MediaStreamManager::new();
        let sid = mgr.create_stream();
        let tid = mgr.add_track(&sid, TrackKind::Video, "cam").unwrap();
        assert!(mgr.remove_track(&sid, &tid));
        assert_eq!(mgr.get_stream(&sid).unwrap().tracks.len(), 0);
        assert!(!mgr.remove_track(&sid, "nonexistent"));
    }

    #[test]
    fn test_get_user_media() {
        let mut mgr = MediaStreamManager::new();
        let sid = mgr.get_user_media(true, true).unwrap();
        let stream = mgr.get_stream(&sid).unwrap();
        assert_eq!(stream.tracks.len(), 2);
        assert!(stream.tracks.iter().any(|t| t.kind == TrackKind::Audio));
        assert!(stream.tracks.iter().any(|t| t.kind == TrackKind::Video));
        assert!(mgr.get_user_media(false, false).is_err());
    }

    #[test]
    fn test_stop_stream_and_enumerate() {
        let mut mgr = MediaStreamManager::new();
        let sid = mgr.get_user_media(true, false).unwrap();
        mgr.stop_stream(&sid);
        let stream = mgr.get_stream(&sid).unwrap();
        assert!(stream.tracks.iter().all(|t| t.ready_state == TrackState::Ended));
        assert!(stream.tracks.iter().all(|t| !t.enabled));
        let devices = mgr.enumerate_devices();
        assert_eq!(devices.len(), 3);
        assert!(devices.iter().any(|d| d.kind == DeviceKind::AudioInput));
        assert!(devices.iter().any(|d| d.kind == DeviceKind::AudioOutput));
        assert!(devices.iter().any(|d| d.kind == DeviceKind::VideoInput));
    }
}
