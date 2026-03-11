// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export a camera animation path (keyframe positions + orientations).

#![allow(dead_code)]

/// Configuration for camera-path export.
#[derive(Debug, Clone)]
pub struct CameraPathExportConfig {
    /// Float precision for output.
    pub precision: usize,
    /// Whether to include field-of-view data.
    pub include_fov: bool,
}

/// A single camera keyframe.
#[derive(Debug, Clone)]
pub struct CameraKeyframe {
    /// Time in seconds.
    pub time_sec: f32,
    /// Camera position [x, y, z].
    pub position: [f64; 3],
    /// Camera look-target [x, y, z].
    pub target: [f64; 3],
    /// Up vector [x, y, z].
    pub up: [f64; 3],
    /// Vertical field of view in degrees.
    pub fov_deg: f32,
}

/// Result accumulating all camera keyframes for export.
#[derive(Debug, Clone)]
pub struct CameraPathExportResult {
    /// All keyframes in time order.
    pub keyframes: Vec<CameraKeyframe>,
    /// Byte count of the last export.
    pub total_bytes: usize,
}

/// Returns the default [`CameraPathExportConfig`].
#[allow(dead_code)]
pub fn default_camera_path_config() -> CameraPathExportConfig {
    CameraPathExportConfig {
        precision: 6,
        include_fov: true,
    }
}

/// Creates a new, empty [`CameraPathExportResult`].
#[allow(dead_code)]
pub fn new_camera_path_export() -> CameraPathExportResult {
    CameraPathExportResult {
        keyframes: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a keyframe.
#[allow(dead_code)]
pub fn camera_path_add_keyframe(result: &mut CameraPathExportResult, kf: CameraKeyframe) {
    result.keyframes.push(kf);
}

/// Serialises all keyframes as JSON.
#[allow(dead_code)]
pub fn camera_path_to_json(
    result: &CameraPathExportResult,
    cfg: &CameraPathExportConfig,
) -> String {
    let prec = cfg.precision;
    let mut out = String::from("{\"keyframes\":[\n");
    for (i, kf) in result.keyframes.iter().enumerate() {
        let comma = if i + 1 < result.keyframes.len() { "," } else { "" };
        let pos = format!(
            "[{:.prec$},{:.prec$},{:.prec$}]",
            kf.position[0], kf.position[1], kf.position[2]
        );
        let tgt = format!(
            "[{:.prec$},{:.prec$},{:.prec$}]",
            kf.target[0], kf.target[1], kf.target[2]
        );
        let up = format!(
            "[{:.prec$},{:.prec$},{:.prec$}]",
            kf.up[0], kf.up[1], kf.up[2]
        );
        let mut entry = format!(
            "  {{\"time\":{:.prec$},\"position\":{},\"target\":{},\"up\":{}",
            kf.time_sec, pos, tgt, up
        );
        if cfg.include_fov {
            entry.push_str(&format!(",\"fov\":{:.prec$}", kf.fov_deg));
        }
        entry.push('}');
        out.push_str(&entry);
        out.push_str(comma);
        out.push('\n');
    }
    out.push_str("]}");
    out
}

/// Serialises all keyframes as CSV.
#[allow(dead_code)]
pub fn camera_path_to_csv(
    result: &CameraPathExportResult,
    cfg: &CameraPathExportConfig,
) -> String {
    let prec = cfg.precision;
    let mut header = String::from("time,px,py,pz,tx,ty,tz,ux,uy,uz");
    if cfg.include_fov {
        header.push_str(",fov");
    }
    header.push('\n');
    let mut out = header;
    for kf in &result.keyframes {
        let row = format!(
            "{:.prec$},{:.prec$},{:.prec$},{:.prec$},{:.prec$},{:.prec$},{:.prec$},{:.prec$},{:.prec$},{:.prec$}",
            kf.time_sec,
            kf.position[0], kf.position[1], kf.position[2],
            kf.target[0], kf.target[1], kf.target[2],
            kf.up[0], kf.up[1], kf.up[2],
        );
        out.push_str(&row);
        if cfg.include_fov {
            out.push_str(&format!(",{:.prec$}", kf.fov_deg));
        }
        out.push('\n');
    }
    out
}

/// Returns the number of keyframes.
#[allow(dead_code)]
pub fn camera_path_keyframe_count(result: &CameraPathExportResult) -> usize {
    result.keyframes.len()
}

/// Returns the total path duration in seconds.
#[allow(dead_code)]
pub fn camera_path_duration(result: &CameraPathExportResult) -> f32 {
    match (result.keyframes.first(), result.keyframes.last()) {
        (Some(first), Some(last)) => last.time_sec - first.time_sec,
        _ => 0.0,
    }
}

/// Writes the JSON export to a file (stub – returns byte count).
#[allow(dead_code)]
pub fn camera_path_write_to_file(
    result: &mut CameraPathExportResult,
    cfg: &CameraPathExportConfig,
    _path: &str,
) -> usize {
    let json = camera_path_to_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all keyframes.
#[allow(dead_code)]
pub fn camera_path_clear(result: &mut CameraPathExportResult) {
    result.keyframes.clear();
    result.total_bytes = 0;
}

/// Returns the total byte count of the last export.
#[allow(dead_code)]
pub fn camera_path_total_bytes(result: &CameraPathExportResult) -> usize {
    result.total_bytes
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn sample_keyframe(t: f32) -> CameraKeyframe {
    CameraKeyframe {
        time_sec: t,
        position: [0.0, 2.0, -5.0],
        target: [0.0, 0.9, 0.0],
        up: [0.0, 1.0, 0.0],
        fov_deg: 60.0,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_includes_fov() {
        let cfg = default_camera_path_config();
        assert!(cfg.include_fov);
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_camera_path_export();
        assert_eq!(camera_path_keyframe_count(&r), 0);
    }

    #[test]
    fn add_keyframe_increases_count() {
        let mut r = new_camera_path_export();
        camera_path_add_keyframe(&mut r, sample_keyframe(0.0));
        assert_eq!(camera_path_keyframe_count(&r), 1);
    }

    #[test]
    fn duration_of_two_frames() {
        let mut r = new_camera_path_export();
        camera_path_add_keyframe(&mut r, sample_keyframe(0.0));
        camera_path_add_keyframe(&mut r, sample_keyframe(2.0));
        let dur = camera_path_duration(&r);
        assert!((dur - 2.0).abs() < 1e-5);
    }

    #[test]
    fn duration_empty_is_zero() {
        let r = new_camera_path_export();
        assert!((camera_path_duration(&r) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_time_and_position() {
        let mut r = new_camera_path_export();
        camera_path_add_keyframe(&mut r, sample_keyframe(1.5));
        let cfg = default_camera_path_config();
        let json = camera_path_to_json(&r, &cfg);
        assert!(json.contains("\"time\""));
        assert!(json.contains("\"position\""));
        assert!(json.contains("\"fov\""));
    }

    #[test]
    fn csv_starts_with_header() {
        let r = new_camera_path_export();
        let cfg = default_camera_path_config();
        let csv = camera_path_to_csv(&r, &cfg);
        assert!(csv.starts_with("time,px,py,pz"));
    }

    #[test]
    fn write_to_file_updates_bytes() {
        let mut r = new_camera_path_export();
        camera_path_add_keyframe(&mut r, sample_keyframe(0.0));
        let cfg = default_camera_path_config();
        let bytes = camera_path_write_to_file(&mut r, &cfg, "/tmp/cam.json");
        assert!(bytes > 0);
        assert_eq!(camera_path_total_bytes(&r), bytes);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_camera_path_export();
        camera_path_add_keyframe(&mut r, sample_keyframe(0.0));
        camera_path_clear(&mut r);
        assert_eq!(camera_path_keyframe_count(&r), 0);
        assert_eq!(camera_path_total_bytes(&r), 0);
    }

    #[test]
    fn multiple_keyframes_in_json() {
        let mut r = new_camera_path_export();
        for i in 0..4 {
            camera_path_add_keyframe(&mut r, sample_keyframe(i as f32));
        }
        assert_eq!(camera_path_keyframe_count(&r), 4);
        let cfg = default_camera_path_config();
        let json = camera_path_to_json(&r, &cfg);
        assert!(json.len() > 10);
    }
}
