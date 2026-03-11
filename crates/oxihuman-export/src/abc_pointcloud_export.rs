// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Alembic point cloud export stub.

/// Configuration for Alembic point cloud export.
#[derive(Debug, Clone)]
pub struct AbcPointcloudConfig {
    pub object_path: String,
    pub fps: f32,
    pub frame_start: i32,
    pub frame_end: i32,
}

impl Default for AbcPointcloudConfig {
    fn default() -> Self {
        Self {
            object_path: "/pointcloud".to_string(),
            fps: 24.0,
            frame_start: 0,
            frame_end: 1,
        }
    }
}

/// An Alembic point cloud export.
#[derive(Debug, Clone)]
pub struct AbcPointcloudExport {
    pub config: AbcPointcloudConfig,
    pub frames: Vec<AbcPointcloudFrame>,
}

/// A single frame of point cloud data.
#[derive(Debug, Clone)]
pub struct AbcPointcloudFrame {
    pub frame: i32,
    pub positions: Vec<[f32; 3]>,
    pub radii: Vec<f32>,
}

/// Create a new Alembic point cloud export.
pub fn new_abc_pointcloud(config: AbcPointcloudConfig) -> AbcPointcloudExport {
    AbcPointcloudExport {
        config,
        frames: Vec::new(),
    }
}

/// Add a frame of positions to the export.
pub fn add_abc_frame(
    export: &mut AbcPointcloudExport,
    frame: i32,
    positions: Vec<[f32; 3]>,
    radii: Vec<f32>,
) {
    export.frames.push(AbcPointcloudFrame {
        frame,
        positions,
        radii,
    });
}

/// Return the total number of frames.
pub fn frame_count(export: &AbcPointcloudExport) -> usize {
    export.frames.len()
}

/// Return the total number of points across all frames.
pub fn total_point_count(export: &AbcPointcloudExport) -> usize {
    export.frames.iter().map(|f| f.positions.len()).sum()
}

/// Estimate the export size in bytes (stub).
pub fn estimate_abc_size_bytes(export: &AbcPointcloudExport) -> usize {
    /* Rough: 12 bytes per position + 4 bytes per radius */
    export.frames.iter().map(|f| f.positions.len() * 16).sum()
}

/// Validate the export (check FPS > 0, at least one frame).
pub fn validate_abc_pointcloud(export: &AbcPointcloudExport) -> bool {
    export.config.fps > 0.0 && !export.frames.is_empty()
}

/// Serialize the config as a JSON-like string (stub).
pub fn abc_pointcloud_config_to_json(config: &AbcPointcloudConfig) -> String {
    format!(
        r#"{{"object_path":"{}","fps":{},"frame_start":{},"frame_end":{}}}"#,
        config.object_path, config.fps, config.frame_start, config.frame_end
    )
}

/// Return the point count for a specific frame index.
pub fn frame_point_count(export: &AbcPointcloudExport, frame_idx: usize) -> usize {
    export
        .frames
        .get(frame_idx)
        .map(|f| f.positions.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> AbcPointcloudExport {
        let cfg = AbcPointcloudConfig::default();
        let mut exp = new_abc_pointcloud(cfg);
        add_abc_frame(
            &mut exp,
            0,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![0.1, 0.1],
        );
        exp
    }

    #[test]
    fn test_frame_count() {
        let exp = sample_export();
        assert_eq!(frame_count(&exp), 1);
    }

    #[test]
    fn test_total_point_count() {
        let exp = sample_export();
        assert_eq!(total_point_count(&exp), 2);
    }

    #[test]
    fn test_estimate_size() {
        let exp = sample_export();
        assert!(estimate_abc_size_bytes(&exp) > 0);
    }

    #[test]
    fn test_validate_valid() {
        let exp = sample_export();
        assert!(validate_abc_pointcloud(&exp));
    }

    #[test]
    fn test_validate_empty_frames() {
        let cfg = AbcPointcloudConfig::default();
        let exp = new_abc_pointcloud(cfg);
        assert!(!validate_abc_pointcloud(&exp));
    }

    #[test]
    fn test_config_to_json() {
        let cfg = AbcPointcloudConfig::default();
        let json = abc_pointcloud_config_to_json(&cfg);
        assert!(json.contains("pointcloud"));
    }

    #[test]
    fn test_frame_point_count() {
        let exp = sample_export();
        assert_eq!(frame_point_count(&exp, 0), 2);
        assert_eq!(frame_point_count(&exp, 99), 0);
    }

    #[test]
    fn test_default_fps() {
        let cfg = AbcPointcloudConfig::default();
        assert!((cfg.fps - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_multiple_frames() {
        let cfg = AbcPointcloudConfig::default();
        let mut exp = new_abc_pointcloud(cfg);
        add_abc_frame(&mut exp, 0, vec![[0.0, 0.0, 0.0]], vec![0.1]);
        add_abc_frame(
            &mut exp,
            1,
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            vec![0.1, 0.2],
        );
        assert_eq!(total_point_count(&exp), 3);
    }
}
