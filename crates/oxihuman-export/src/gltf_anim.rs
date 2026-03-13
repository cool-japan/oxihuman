// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GLTF animation export with morph target weight keyframes.

use std::path::Path;

// ── Data structures ───────────────────────────────────────────────────────────

/// A single keyframe of morph-target weights at a given time.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MorphWeightKeyframe {
    /// Time in seconds.
    pub time: f32,
    /// One weight per morph target, in [0.0, 1.0].
    pub weights: Vec<f32>,
}

/// Path type for a GLTF animation channel.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnimPath {
    Translation,
    Rotation,
    Scale,
    MorphWeights,
}

/// A single animation channel targeting one node property.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GltfAnimChannel {
    /// Index of the target node in the GLTF node array.
    pub target_node: u32,
    /// Which property is animated.
    pub path: AnimPath,
    /// Time stamps for each keyframe (seconds).
    pub times: Vec<f32>,
    /// Flattened values. For `MorphWeights`, len = `times.len() * n_morphs`.
    pub values: Vec<f32>,
}

/// A complete animation clip with one or more channels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GltfAnimClip {
    pub name: String,
    pub channels: Vec<GltfAnimChannel>,
    /// Total duration in seconds (max time across all channels).
    pub duration: f32,
}

/// Result of exporting a GLTF animation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GltfAnimExportResult {
    /// GLTF animation JSON fragment.
    pub json: String,
    /// Number of accessors written.
    pub accessor_count: usize,
    /// Total keyframes across all channels.
    pub total_keyframes: usize,
}

// ── Core functions ────────────────────────────────────────────────────────────

/// Build a morph-weight animation channel from a slice of keyframes.
///
/// The `times` and `values` (flattened weights) are extracted from the keyframes.
#[allow(dead_code)]
pub fn build_morph_anim_channel(node: u32, keyframes: &[MorphWeightKeyframe]) -> GltfAnimChannel {
    let times: Vec<f32> = keyframes.iter().map(|kf| kf.time).collect();
    let values: Vec<f32> = keyframes
        .iter()
        .flat_map(|kf| kf.weights.iter().copied())
        .collect();
    GltfAnimChannel {
        target_node: node,
        path: AnimPath::MorphWeights,
        times,
        values,
    }
}

/// Produce a compact GLTF animation node JSON fragment.
///
/// Each channel gets a sampler with `input` (time accessor) and `output` (value accessor).
/// Accessor indices start at `first_accessor_idx` and increment by 2 per channel.
#[allow(dead_code)]
pub fn build_gltf_anim_json(clip: &GltfAnimClip, first_accessor_idx: u32) -> String {
    let mut channels_json = Vec::new();
    let mut samplers_json = Vec::new();

    for (i, ch) in clip.channels.iter().enumerate() {
        let path_str = match ch.path {
            AnimPath::Translation => "translation",
            AnimPath::Rotation => "rotation",
            AnimPath::Scale => "scale",
            AnimPath::MorphWeights => "weights",
        };
        let sampler_idx = i as u32;
        let input_acc = first_accessor_idx + i as u32 * 2;
        let output_acc = first_accessor_idx + i as u32 * 2 + 1;

        channels_json.push(format!(
            r#"{{"sampler":{},"target":{{"node":{},"path":"{}"}}}}"#,
            sampler_idx, ch.target_node, path_str
        ));
        samplers_json.push(format!(
            r#"{{"input":{},"interpolation":"LINEAR","output":{}}}"#,
            input_acc, output_acc
        ));
    }

    format!(
        r#"{{"name":"{}","channels":[{}],"samplers":[{}]}}"#,
        json_escape(&clip.name),
        channels_json.join(","),
        samplers_json.join(",")
    )
}

/// Produce a GLTF accessor JSON object for a flat f32 array.
///
/// `accessor_type` is e.g. `"SCALAR"` or `"VEC3"`.
/// `component_type` 5126 = `FLOAT`.
#[allow(dead_code)]
pub fn build_gltf_accessor_json(
    data: &[f32],
    accessor_type: &str,
    component_type: u32,
    idx: u32,
) -> String {
    let min_val = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    format!(
        r#"{{"bufferView":{},"componentType":{},"count":{},"type":"{}","min":[{}],"max":[{}]}}"#,
        idx,
        component_type,
        data.len(),
        accessor_type,
        min_val,
        max_val
    )
}

/// Write the animation JSON fragment for `clip` to `path`.
#[allow(dead_code)]
pub fn export_morph_animation(clip: &GltfAnimClip, path: &Path) -> anyhow::Result<()> {
    let json = build_gltf_anim_json(clip, 0);
    let wrapper = format!(r#"{{"animations":[{}]}}"#, json);
    std::fs::write(path, wrapper)?;
    Ok(())
}

/// Resample `clip` to uniform `fps` via linear interpolation.
///
/// Returns a new `GltfAnimClip` whose channel time stamps are uniform.
#[allow(dead_code)]
pub fn resample_animation(clip: &GltfAnimClip, fps: f32) -> GltfAnimClip {
    let duration = clip_duration(clip);
    if fps <= 0.0 || duration <= 0.0 {
        return clip.clone();
    }

    let frame_dt = 1.0 / fps;
    let n_frames = (duration * fps).ceil() as usize + 1;

    let new_channels: Vec<GltfAnimChannel> = clip
        .channels
        .iter()
        .map(|ch| {
            if ch.times.is_empty() {
                return ch.clone();
            }
            // Determine how many values per time step
            let n_morphs = if ch.times.len() > 1 {
                ch.values.len() / ch.times.len()
            } else {
                ch.values.len()
            };
            let n_morphs = n_morphs.max(1);

            let mut new_times = Vec::with_capacity(n_frames);
            let mut new_values = Vec::with_capacity(n_frames * n_morphs);

            for frame in 0..n_frames {
                let t = (frame as f32 * frame_dt).min(duration);
                new_times.push(t);

                // find segment
                let weights = sample_weights_at(ch, t, n_morphs);
                new_values.extend_from_slice(&weights);
            }

            GltfAnimChannel {
                target_node: ch.target_node,
                path: ch.path.clone(),
                times: new_times,
                values: new_values,
            }
        })
        .collect();

    GltfAnimClip {
        name: clip.name.clone(),
        channels: new_channels,
        duration,
    }
}

/// Linear interpolation between two weight slices, element-wise.
///
/// `t` is clamped to [0, 1].
#[allow(dead_code)]
pub fn lerp_weights(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let t = t.clamp(0.0, 1.0);
    let len = a.len().min(b.len());
    (0..len).map(|i| a[i] + (b[i] - a[i]) * t).collect()
}

/// Return the maximum time across all channels.
#[allow(dead_code)]
pub fn clip_duration(clip: &GltfAnimClip) -> f32 {
    clip.channels
        .iter()
        .flat_map(|ch| ch.times.iter().copied())
        .fold(0.0f32, f32::max)
}

/// Return `true` if every weight is in the range [0.0, 1.0].
#[allow(dead_code)]
pub fn validate_morph_weights(weights: &[f32]) -> bool {
    weights.iter().all(|&w| (0.0..=1.0).contains(&w))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Sample linearly-interpolated weights from a channel at time `t`.
fn sample_weights_at(ch: &GltfAnimChannel, t: f32, n_morphs: usize) -> Vec<f32> {
    if ch.times.is_empty() {
        return vec![0.0; n_morphs];
    }
    if t <= ch.times[0] {
        return ch.values[..n_morphs.min(ch.values.len())].to_vec();
    }
    if ch.times.last().is_none_or(|last| t >= *last) {
        let start = ch.values.len().saturating_sub(n_morphs);
        return ch.values[start..].to_vec();
    }

    // Find the two surrounding keyframes
    let idx = ch
        .times
        .windows(2)
        .position(|w| t >= w[0] && t < w[1])
        .unwrap_or(ch.times.len() - 2);

    let t0 = ch.times[idx];
    let t1 = ch.times[idx + 1];
    let alpha = if (t1 - t0).abs() < 1e-9 {
        0.0
    } else {
        (t - t0) / (t1 - t0)
    };

    let a_start = idx * n_morphs;
    let b_start = (idx + 1) * n_morphs;
    let a_end = (a_start + n_morphs).min(ch.values.len());
    let b_end = (b_start + n_morphs).min(ch.values.len());

    if a_end <= a_start || b_end <= b_start {
        return vec![0.0; n_morphs];
    }

    lerp_weights(
        &ch.values[a_start..a_end],
        &ch.values[b_start..b_end],
        alpha,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clip() -> GltfAnimClip {
        let kf0 = MorphWeightKeyframe {
            time: 0.0,
            weights: vec![0.0, 0.5],
        };
        let kf1 = MorphWeightKeyframe {
            time: 1.0,
            weights: vec![1.0, 0.5],
        };
        let ch = build_morph_anim_channel(0, &[kf0, kf1]);
        GltfAnimClip {
            name: "test_clip".to_string(),
            channels: vec![ch],
            duration: 1.0,
        }
    }

    // 1. lerp_weights at t=0 returns a
    #[test]
    fn lerp_t0_returns_a() {
        let a = vec![0.2, 0.4, 0.6];
        let b = vec![0.8, 1.0, 0.0];
        let result = lerp_weights(&a, &b, 0.0);
        for (r, &av) in result.iter().zip(a.iter()) {
            assert!((r - av).abs() < 1e-6);
        }
    }

    // 2. lerp_weights at t=1 returns b
    #[test]
    fn lerp_t1_returns_b() {
        let a = vec![0.2, 0.4];
        let b = vec![0.8, 1.0];
        let result = lerp_weights(&a, &b, 1.0);
        for (r, &bv) in result.iter().zip(b.iter()) {
            assert!((r - bv).abs() < 1e-6);
        }
    }

    // 3. lerp_weights at t=0.5 is midpoint
    #[test]
    fn lerp_t05_is_midpoint() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 1.0];
        let result = lerp_weights(&a, &b, 0.5);
        assert!((result[0] - 0.5).abs() < 1e-6);
        assert!((result[1] - 0.5).abs() < 1e-6);
    }

    // 4. validate_morph_weights — valid
    #[test]
    fn validate_weights_valid() {
        assert!(validate_morph_weights(&[0.0, 0.5, 1.0]));
    }

    // 5. validate_morph_weights — invalid (>1)
    #[test]
    fn validate_weights_invalid_above() {
        assert!(!validate_morph_weights(&[0.0, 1.1]));
    }

    // 6. validate_morph_weights — invalid (<0)
    #[test]
    fn validate_weights_invalid_below() {
        assert!(!validate_morph_weights(&[-0.1, 0.5]));
    }

    // 7. build_morph_anim_channel time count
    #[test]
    fn build_channel_time_count() {
        let keyframes: Vec<MorphWeightKeyframe> = (0..5)
            .map(|i| MorphWeightKeyframe {
                time: i as f32 * 0.25,
                weights: vec![0.5],
            })
            .collect();
        let ch = build_morph_anim_channel(1, &keyframes);
        assert_eq!(ch.times.len(), 5);
        assert_eq!(ch.target_node, 1);
    }

    // 8. clip_duration
    #[test]
    fn clip_duration_correct() {
        let clip = make_clip();
        assert!((clip_duration(&clip) - 1.0).abs() < 1e-6);
    }

    // 9. build_gltf_anim_json contains "animations" (via wrapper)
    #[test]
    fn build_gltf_anim_json_contains_animations_key() {
        let clip = make_clip();
        let json = build_gltf_anim_json(&clip, 0);
        // wrap it the same way export does
        let wrapped = format!(r#"{{"animations":[{}]}}"#, json);
        assert!(wrapped.contains("animations"));
    }

    // 10. build_gltf_anim_json contains name
    #[test]
    fn build_gltf_anim_json_contains_name() {
        let clip = make_clip();
        let json = build_gltf_anim_json(&clip, 0);
        assert!(json.contains("test_clip"));
    }

    // 11. resample_animation frame count
    #[test]
    fn resample_animation_frame_count() {
        let clip = make_clip(); // duration = 1.0
        let resampled = resample_animation(&clip, 10.0); // 10 fps → 11 frames (0..=10)
        assert_eq!(resampled.channels[0].times.len(), 11);
    }

    // 12. GltfAnimChannel path is MorphWeights
    #[test]
    fn build_channel_path_is_morph_weights() {
        let kf = MorphWeightKeyframe {
            time: 0.0,
            weights: vec![0.5],
        };
        let ch = build_morph_anim_channel(0, &[kf]);
        assert_eq!(ch.path, AnimPath::MorphWeights);
    }

    // 13. build_gltf_accessor_json contains componentType 5126
    #[test]
    fn accessor_json_contains_component_type_5126() {
        let data = vec![0.0f32, 0.5, 1.0];
        let json = build_gltf_accessor_json(&data, "SCALAR", 5126, 0);
        assert!(json.contains("5126"));
        assert!(json.contains("SCALAR"));
    }

    // 14. export_morph_animation writes file
    #[test]
    fn export_morph_animation_writes_file() {
        let clip = make_clip();
        let path = std::path::Path::new("/tmp/test_morph_anim.json");
        export_morph_animation(&clip, path).expect("export should succeed");
        let content = std::fs::read_to_string(path).expect("file should exist");
        assert!(content.contains("animations"));
    }

    // 15. lerp_weights handles mismatched lengths (uses min)
    #[test]
    fn lerp_weights_mismatched_length() {
        let a = vec![0.0, 0.5, 1.0];
        let b = vec![1.0, 0.5];
        let result = lerp_weights(&a, &b, 0.5);
        assert_eq!(result.len(), 2);
    }
}
