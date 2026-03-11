//! Generic keyframe animation export for poses and morphs.

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Interpolation method for an animation key.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnimInterpolation {
    Step,
    Linear,
    Bezier,
    Hermite,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for animation export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimExportConfig {
    pub frame_rate: f32,
    pub loop_mode: bool,
    pub bake_to_frames: bool,
}

/// A single keyframe in an animation channel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimKey {
    pub time: f32,
    pub value: f32,
    pub interp: AnimInterpolation,
}

/// A named animation channel containing keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimChannel {
    pub name: String,
    pub keys: Vec<AnimKey>,
}

/// A named animation clip containing channels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimClipExport {
    pub name: String,
    pub channels: Vec<AnimChannel>,
    pub duration: f32,
}

/// Result of exporting one or more animation clips.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimExportResult {
    pub clips: Vec<String>,
    pub total_keys: usize,
    pub duration_sec: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a default `AnimExportConfig`.
#[allow(dead_code)]
pub fn default_anim_export_config() -> AnimExportConfig {
    AnimExportConfig {
        frame_rate: 30.0,
        loop_mode: false,
        bake_to_frames: false,
    }
}

/// Creates a new `AnimClipExport` with the given name and duration.
#[allow(dead_code)]
pub fn new_anim_clip(name: &str, duration: f32) -> AnimClipExport {
    AnimClipExport {
        name: name.to_string(),
        channels: Vec::new(),
        duration,
    }
}

/// Adds a channel to a clip.
#[allow(dead_code)]
pub fn add_channel(clip: &mut AnimClipExport, ch: AnimChannel) {
    clip.channels.push(ch);
}

/// Creates a new `AnimChannel` with the given name.
#[allow(dead_code)]
pub fn new_anim_channel(name: &str) -> AnimChannel {
    AnimChannel {
        name: name.to_string(),
        keys: Vec::new(),
    }
}

/// Adds a keyframe to a channel.
#[allow(dead_code)]
pub fn add_anim_key_to_channel(ch: &mut AnimChannel, key: AnimKey) {
    ch.keys.push(key);
}

/// Exports a single clip to a JSON-like string.
#[allow(dead_code)]
pub fn export_anim_clip(clip: &AnimClipExport, cfg: &AnimExportConfig) -> String {
    let total_keys: usize = clip.channels.iter().map(|c| c.keys.len()).sum();
    format!(
        "{{\"name\":\"{}\",\"duration\":{:.4},\"frame_rate\":{:.2},\"loop\":{},\"channels\":{},\"total_keys\":{}}}",
        clip.name,
        clip.duration,
        cfg.frame_rate,
        cfg.loop_mode,
        clip.channels.len(),
        total_keys,
    )
}

/// Exports multiple clips and returns an `AnimExportResult`.
#[allow(dead_code)]
pub fn export_anim_clips(clips: &[AnimClipExport], cfg: &AnimExportConfig) -> AnimExportResult {
    let mut total_keys = 0usize;
    let mut duration_sec = 0.0f32;
    let mut names = Vec::with_capacity(clips.len());
    for clip in clips {
        let keys: usize = clip.channels.iter().map(|c| c.keys.len()).sum();
        total_keys += keys;
        if clip.duration > duration_sec {
            duration_sec = clip.duration;
        }
        names.push(export_anim_clip(clip, cfg));
    }
    AnimExportResult {
        clips: names,
        total_keys,
        duration_sec,
    }
}

/// Returns the number of keys in a channel.
#[allow(dead_code)]
pub fn channel_key_count(ch: &AnimChannel) -> usize {
    ch.keys.len()
}

/// Returns the number of channels in a clip.
#[allow(dead_code)]
pub fn clip_channel_count(clip: &AnimClipExport) -> usize {
    clip.channels.len()
}

/// Returns the interpolation name for a keyframe.
#[allow(dead_code)]
pub fn anim_interp_name(key: &AnimKey) -> &'static str {
    match key.interp {
        AnimInterpolation::Step => "step",
        AnimInterpolation::Linear => "linear",
        AnimInterpolation::Bezier => "bezier",
        AnimInterpolation::Hermite => "hermite",
    }
}

/// Serialises an `AnimExportResult` to a JSON string.
#[allow(dead_code)]
pub fn anim_export_result_to_json(r: &AnimExportResult) -> String {
    let clips_json = r
        .clips
        .iter()
        .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"clip_count\":{},\"total_keys\":{},\"duration_sec\":{:.4},\"clips\":[{}]}}",
        r.clips.len(),
        r.total_keys,
        r.duration_sec,
        clips_json,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_anim_export_config();
        assert!((cfg.frame_rate - 30.0).abs() < 1e-6);
        assert!(!cfg.loop_mode);
        assert!(!cfg.bake_to_frames);
    }

    #[test]
    fn new_clip_has_correct_fields() {
        let clip = new_anim_clip("idle", 2.5);
        assert_eq!(clip.name, "idle");
        assert!((clip.duration - 2.5).abs() < 1e-6);
        assert!(clip.channels.is_empty());
    }

    #[test]
    fn add_channel_grows_clip() {
        let mut clip = new_anim_clip("run", 1.0);
        let ch = new_anim_channel("position_x");
        add_channel(&mut clip, ch);
        assert_eq!(clip_channel_count(&clip), 1);
    }

    #[test]
    fn add_keys_to_channel() {
        let mut ch = new_anim_channel("rot_y");
        let key = AnimKey {
            time: 0.0,
            value: 0.0,
            interp: AnimInterpolation::Linear,
        };
        add_anim_key_to_channel(&mut ch, key);
        assert_eq!(channel_key_count(&ch), 1);
    }

    #[test]
    fn interp_names_correct() {
        let cases = [
            (AnimInterpolation::Step, "step"),
            (AnimInterpolation::Linear, "linear"),
            (AnimInterpolation::Bezier, "bezier"),
            (AnimInterpolation::Hermite, "hermite"),
        ];
        for (interp, expected) in cases {
            let key = AnimKey { time: 0.0, value: 0.0, interp };
            assert_eq!(anim_interp_name(&key), expected);
        }
    }

    #[test]
    fn export_single_clip_contains_name() {
        let cfg = default_anim_export_config();
        let clip = new_anim_clip("walk", 1.2);
        let s = export_anim_clip(&clip, &cfg);
        assert!(s.contains("walk"));
    }

    #[test]
    fn export_multiple_clips_result() {
        let cfg = default_anim_export_config();
        let clip1 = new_anim_clip("a", 1.0);
        let clip2 = new_anim_clip("b", 3.0);
        let result = export_anim_clips(&[clip1, clip2], &cfg);
        assert_eq!(result.clips.len(), 2);
        assert!((result.duration_sec - 3.0).abs() < 1e-6);
        assert_eq!(result.total_keys, 0);
    }

    #[test]
    fn result_to_json_contains_clip_count() {
        let r = AnimExportResult {
            clips: vec!["c1".to_string(), "c2".to_string()],
            total_keys: 10,
            duration_sec: 5.0,
        };
        let json = anim_export_result_to_json(&r);
        assert!(json.contains("\"clip_count\":2"));
        assert!(json.contains("\"total_keys\":10"));
    }
}
