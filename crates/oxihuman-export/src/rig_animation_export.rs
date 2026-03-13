// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export a rigged animation (bone transforms over time) as JSON keyframes.

#![allow(dead_code)]

/// Configuration for rig animation export.
#[derive(Debug, Clone)]
pub struct RigAnimExportConfig {
    /// Target frames-per-second for the exported animation.
    pub frame_rate: f32,
    /// Float precision in JSON.
    pub precision: usize,
    /// Whether to include bone scale in output.
    pub include_scale: bool,
}

/// A single bone keyframe: position, rotation quaternion, optional scale.
#[derive(Debug, Clone)]
pub struct BoneKeyframe {
    /// Bone name.
    pub bone_name: String,
    /// Time in seconds.
    pub time_sec: f32,
    /// Translation [x, y, z].
    pub translation: [f64; 3],
    /// Rotation quaternion [x, y, z, w].
    pub rotation: [f64; 4],
    /// Scale [x, y, z].
    pub scale: [f64; 3],
}

/// Accumulated rig animation export state.
#[derive(Debug, Clone)]
pub struct RigAnimExportResult {
    /// All keyframes.
    pub keyframes: Vec<BoneKeyframe>,
    /// Byte count of the last export.
    pub total_bytes: usize,
}

/// Returns the default [`RigAnimExportConfig`].
pub fn default_rig_anim_config() -> RigAnimExportConfig {
    RigAnimExportConfig {
        frame_rate: 30.0,
        precision: 6,
        include_scale: true,
    }
}

/// Creates a new, empty [`RigAnimExportResult`].
pub fn new_rig_anim_export() -> RigAnimExportResult {
    RigAnimExportResult {
        keyframes: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a bone keyframe.
pub fn rig_anim_add_keyframe(result: &mut RigAnimExportResult, kf: BoneKeyframe) {
    result.keyframes.push(kf);
}

/// Returns the number of keyframes.
pub fn rig_anim_keyframe_count(result: &RigAnimExportResult) -> usize {
    result.keyframes.len()
}

/// Returns the number of unique bones referenced in keyframes.
pub fn rig_anim_bone_count(result: &RigAnimExportResult) -> usize {
    let mut names: Vec<&str> = result.keyframes.iter().map(|k| k.bone_name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    names.len()
}

/// Returns the animation duration in seconds (last time – first time).
pub fn rig_anim_duration(result: &RigAnimExportResult) -> f32 {
    match (result.keyframes.first(), result.keyframes.last()) {
        (Some(first), Some(last)) => (last.time_sec - first.time_sec).max(0.0),
        _ => 0.0,
    }
}

/// Returns the configured frame rate.
pub fn rig_anim_frame_rate(cfg: &RigAnimExportConfig) -> f32 {
    cfg.frame_rate
}

/// Serialises all keyframes as JSON.
pub fn rig_anim_to_json(result: &RigAnimExportResult, cfg: &RigAnimExportConfig) -> String {
    let prec = cfg.precision;
    let mut out = String::from("{\"keyframes\":[\n");
    for (i, kf) in result.keyframes.iter().enumerate() {
        let comma = if i + 1 < result.keyframes.len() { "," } else { "" };
        let t = kf.translation;
        let r = kf.rotation;
        let s = kf.scale;
        let mut entry = format!(
            "  {{\"bone\":\"{}\",\"time\":{:.prec$},\
            \"translation\":[{:.prec$},{:.prec$},{:.prec$}],\
            \"rotation\":[{:.prec$},{:.prec$},{:.prec$},{:.prec$}]",
            kf.bone_name, kf.time_sec, t[0], t[1], t[2], r[0], r[1], r[2], r[3],
        );
        if cfg.include_scale {
            entry.push_str(&format!(
                ",\"scale\":[{:.prec$},{:.prec$},{:.prec$}]",
                s[0], s[1], s[2]
            ));
        }
        entry.push('}');
        out.push_str(&entry);
        out.push_str(comma);
        out.push('\n');
    }
    out.push_str("]}");
    out
}

/// Writes JSON export to a file path (stub – returns byte count).
pub fn rig_anim_write_to_file(
    result: &mut RigAnimExportResult,
    cfg: &RigAnimExportConfig,
    _path: &str,
) -> usize {
    let json = rig_anim_to_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all keyframes and resets state.
pub fn rig_anim_clear(result: &mut RigAnimExportResult) {
    result.keyframes.clear();
    result.total_bytes = 0;
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn sample_keyframe(bone: &str, t: f32) -> BoneKeyframe {
    BoneKeyframe {
        bone_name: bone.to_string(),
        time_sec: t,
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_rig_anim_config();
        assert!((cfg.frame_rate - 30.0).abs() < 1e-5);
        assert_eq!(cfg.precision, 6);
        assert!(cfg.include_scale);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_rig_anim_export();
        assert_eq!(rig_anim_keyframe_count(&r), 0);
    }

    #[test]
    fn add_keyframe_increments_count() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 0.0));
        assert_eq!(rig_anim_keyframe_count(&r), 1);
    }

    #[test]
    fn bone_count_unique_only() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 0.0));
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 1.0));
        rig_anim_add_keyframe(&mut r, sample_keyframe("spine", 0.0));
        assert_eq!(rig_anim_bone_count(&r), 2);
    }

    #[test]
    fn duration_two_frames() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 0.0));
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 3.0));
        assert!((rig_anim_duration(&r) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn duration_empty_is_zero() {
        let r = new_rig_anim_export();
        assert!((rig_anim_duration(&r) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_bone_and_rotation() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("neck", 0.0));
        let cfg = default_rig_anim_config();
        let json = rig_anim_to_json(&r, &cfg);
        assert!(json.contains("\"bone\""));
        assert!(json.contains("\"rotation\""));
        assert!(json.contains("\"scale\""));
        assert!(json.contains("neck"));
    }

    #[test]
    fn frame_rate_accessor() {
        let cfg = default_rig_anim_config();
        assert!((rig_anim_frame_rate(&cfg) - 30.0).abs() < 1e-5);
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 0.0));
        let cfg = default_rig_anim_config();
        let n = rig_anim_write_to_file(&mut r, &cfg, "/tmp/rig.json");
        assert!(n > 0);
        assert_eq!(r.total_bytes, n);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("hips", 0.0));
        let cfg = default_rig_anim_config();
        rig_anim_write_to_file(&mut r, &cfg, "/tmp/rig.json");
        rig_anim_clear(&mut r);
        assert_eq!(rig_anim_keyframe_count(&r), 0);
        assert_eq!(r.total_bytes, 0);
    }

    #[test]
    fn no_scale_in_json_when_disabled() {
        let mut r = new_rig_anim_export();
        rig_anim_add_keyframe(&mut r, sample_keyframe("spine", 0.0));
        let mut cfg = default_rig_anim_config();
        cfg.include_scale = false;
        let json = rig_anim_to_json(&r, &cfg);
        assert!(!json.contains("\"scale\""));
    }
}
