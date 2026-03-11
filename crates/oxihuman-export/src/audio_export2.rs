//! Audio clip export utilities.
#![allow(dead_code)]

/// An audio clip descriptor.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AudioClip2 {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub frame_count: u64,
}

/// Audio export container.
#[allow(dead_code)]
pub struct AudioExport2 {
    pub clips: Vec<AudioClip2>,
}

/// Create a new audio clip.
#[allow(dead_code)]
pub fn new_audio_clip2(name: &str, sample_rate: u32, channels: u32, frame_count: u64) -> AudioClip2 {
    AudioClip2 { name: name.to_string(), sample_rate, channels, frame_count }
}

/// Get duration in seconds.
#[allow(dead_code)]
pub fn audio2_duration_secs(clip: &AudioClip2) -> f64 {
    if clip.sample_rate == 0 { 0.0 } else { clip.frame_count as f64 / clip.sample_rate as f64 }
}

/// Get sample rate.
#[allow(dead_code)]
pub fn audio2_sample_rate(clip: &AudioClip2) -> u32 { clip.sample_rate }

/// Get channel count.
#[allow(dead_code)]
pub fn audio2_channel_count(clip: &AudioClip2) -> u32 { clip.channels }

/// Export audio to JSON (stub).
#[allow(dead_code)]
pub fn export_audio2_stub(clip: &AudioClip2) -> Vec<u8> {
    let s = format!(r#"{{"name":"{}","sample_rate":{},"channels":{},"frames":{}}}"#,
        clip.name, clip.sample_rate, clip.channels, clip.frame_count);
    s.into_bytes()
}

/// Convert audio clip to JSON string.
#[allow(dead_code)]
pub fn audio2_to_json(clip: &AudioClip2) -> String {
    format!(r#"{{"name":"{}","sample_rate":{},"channels":{},"frames":{}}}"#,
        clip.name, clip.sample_rate, clip.channels, clip.frame_count)
}

/// Get frame count.
#[allow(dead_code)]
pub fn audio2_frame_count(clip: &AudioClip2) -> u64 { clip.frame_count }

/// Get clip name.
#[allow(dead_code)]
pub fn audio2_clip_name(clip: &AudioClip2) -> &str { &clip.name }

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_clip() -> AudioClip2 {
        new_audio_clip2("bgm", 44100, 2, 44100)
    }

    #[test]
    fn test_audio_duration_secs() {
        let c = sample_clip();
        assert!((audio2_duration_secs(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_audio_sample_rate() {
        let c = sample_clip();
        assert_eq!(audio2_sample_rate(&c), 44100);
    }

    #[test]
    fn test_audio_channel_count() {
        let c = sample_clip();
        assert_eq!(audio2_channel_count(&c), 2);
    }

    #[test]
    fn test_export_audio_stub() {
        let c = sample_clip();
        let b = export_audio2_stub(&c);
        assert!(!b.is_empty());
    }

    #[test]
    fn test_audio_to_json() {
        let c = sample_clip();
        let j = audio2_to_json(&c);
        assert!(j.contains("bgm"));
    }

    #[test]
    fn test_audio_frame_count() {
        let c = sample_clip();
        assert_eq!(audio2_frame_count(&c), 44100);
    }

    #[test]
    fn test_audio_clip_name() {
        let c = sample_clip();
        assert_eq!(audio2_clip_name(&c), "bgm");
    }

    #[test]
    fn test_audio_duration_zero_rate() {
        let c = new_audio_clip2("x", 0, 1, 100);
        assert!((audio2_duration_secs(&c)).abs() < 1e-10);
    }

    #[test]
    fn test_audio_export2_struct() {
        let ae = AudioExport2 { clips: vec![sample_clip()] };
        assert_eq!(ae.clips.len(), 1);
    }
}
