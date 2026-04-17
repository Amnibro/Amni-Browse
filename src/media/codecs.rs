#[derive(Debug, Clone, PartialEq)]
pub enum CodecType { Audio, Video }

#[derive(Debug, Clone)]
pub struct CodecProfile {
    pub id: String,
    pub codec_type: CodecType,
    pub name: String,
    pub mime_type: String,
    pub supported: bool,
}

pub struct CodecRegistry {
    pub codecs: Vec<CodecProfile>,
}

impl CodecRegistry {
    pub fn new() -> Self {
        let mut codecs = Vec::new();
        let video = [
            ("h264", "H.264/AVC", "video/mp4; codecs=\"avc1\""),
            ("vp8", "VP8", "video/webm; codecs=\"vp8\""),
            ("vp9", "VP9", "video/webm; codecs=\"vp9\""),
            ("av1", "AV1", "video/mp4; codecs=\"av01\""),
            ("hevc", "H.265/HEVC", "video/mp4; codecs=\"hvc1\""),
            ("theora", "Theora", "video/ogg; codecs=\"theora\""),
        ];
        let audio = [
            ("aac", "AAC", "audio/mp4; codecs=\"mp4a\""),
            ("opus", "Opus", "audio/webm; codecs=\"opus\""),
            ("vorbis", "Vorbis", "audio/ogg; codecs=\"vorbis\""),
            ("mp3", "MP3", "audio/mpeg"),
            ("flac", "FLAC", "audio/flac"),
            ("pcm", "PCM", "audio/wav"),
        ];
        for (id, name, mime) in video {
            codecs.push(CodecProfile {
                id: id.to_string(), codec_type: CodecType::Video,
                name: name.to_string(), mime_type: mime.to_string(), supported: true,
            });
        }
        for (id, name, mime) in audio {
            codecs.push(CodecProfile {
                id: id.to_string(), codec_type: CodecType::Audio,
                name: name.to_string(), mime_type: mime.to_string(), supported: true,
            });
        }
        Self { codecs }
    }

    pub fn is_type_supported(&self, mime: &str) -> bool {
        let lower = mime.to_lowercase();
        self.codecs.iter().any(|c| c.supported && c.mime_type.to_lowercase().contains(&lower))
            || self.codecs.iter().any(|c| c.supported && lower.contains(&c.mime_type.to_lowercase().split(';').next().unwrap_or("")))
    }

    pub fn can_play_type(&self, mime: &str) -> &str {
        let lower = mime.to_lowercase();
        let has_codecs = lower.contains("codecs=") || lower.contains("codecs=\"");
        let base_match = self.codecs.iter().any(|c| {
            let base = c.mime_type.to_lowercase();
            let base_type = base.split(';').next().unwrap_or("");
            lower.starts_with(base_type)
        });
        if !base_match { return ""; }
        let exact_match = self.codecs.iter().any(|c| {
            c.supported && c.mime_type.to_lowercase() == lower
        });
        if exact_match { return "probably"; }
        if has_codecs {
            let codec_match = self.codecs.iter().any(|c| {
                if !c.supported { return false; }
                let ct = c.mime_type.to_lowercase();
                if let Some(codec_part) = ct.split("codecs=").nth(1) {
                    let codec_clean = codec_part.trim_matches('"');
                    lower.contains(codec_clean)
                } else { false }
            });
            if codec_match { "probably" } else { "" }
        } else {
            "maybe"
        }
    }

    pub fn list_video_codecs(&self) -> Vec<&CodecProfile> {
        self.codecs.iter().filter(|c| c.codec_type == CodecType::Video).collect()
    }

    pub fn list_audio_codecs(&self) -> Vec<&CodecProfile> {
        self.codecs.iter().filter(|c| c.codec_type == CodecType::Audio).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_populated() {
        let reg = CodecRegistry::new();
        assert_eq!(reg.list_video_codecs().len(), 6);
        assert_eq!(reg.list_audio_codecs().len(), 6);
        assert_eq!(reg.codecs.len(), 12);
    }

    #[test]
    fn test_can_play_type() {
        let reg = CodecRegistry::new();
        assert_eq!(reg.can_play_type("video/mp4; codecs=\"avc1\""), "probably");
        assert_eq!(reg.can_play_type("video/mp4"), "maybe");
        assert_eq!(reg.can_play_type("application/pdf"), "");
        assert_eq!(reg.can_play_type("audio/webm; codecs=\"opus\""), "probably");
    }

    #[test]
    fn test_is_type_supported() {
        let reg = CodecRegistry::new();
        assert!(reg.is_type_supported("video/mp4"));
        assert!(reg.is_type_supported("audio/mpeg"));
        assert!(!reg.is_type_supported("application/json"));
    }
}
