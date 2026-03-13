// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Clip-space vertex position debug overlay.

/// Clip space debug configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClipSpaceDebugConfig {
    /// Enable the debug overlay.
    pub enabled: bool,
    /// Show clip-space frustum boundaries.
    pub show_frustum: bool,
    /// Show behind-camera vertices.
    pub show_behind: bool,
    /// Color for in-frustum vertices.
    pub color_inside: [f32; 4],
    /// Color for out-of-frustum vertices.
    pub color_outside: [f32; 4],
    /// Color for behind-camera vertices.
    pub color_behind: [f32; 4],
    /// Point size for vertex display.
    pub point_size: f32,
}

impl Default for ClipSpaceDebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_frustum: true,
            show_behind: false,
            color_inside: [0.2, 0.9, 0.2, 1.0],
            color_outside: [0.9, 0.2, 0.2, 1.0],
            color_behind: [0.5, 0.5, 0.9, 1.0],
            point_size: 4.0,
        }
    }
}

/// Clip status of a vertex.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipStatus {
    Inside,
    Outside,
    Behind,
}

/// Create default config.
#[allow(dead_code)]
pub fn new_clip_space_debug_config() -> ClipSpaceDebugConfig {
    ClipSpaceDebugConfig::default()
}

/// Determine clip status of a homogeneous vertex [x, y, z, w].
#[allow(dead_code)]
pub fn classify_clip_vertex(xyzw: [f32; 4]) -> ClipStatus {
    let w = xyzw[3];
    if w <= 0.0 {
        return ClipStatus::Behind;
    }
    if xyzw[0].abs() > w || xyzw[1].abs() > w || xyzw[2].abs() > w || xyzw[2] < 0.0 {
        ClipStatus::Outside
    } else {
        ClipStatus::Inside
    }
}

/// Get color for a clip status.
#[allow(dead_code)]
pub fn clip_status_color(status: ClipStatus, cfg: &ClipSpaceDebugConfig) -> [f32; 4] {
    match status {
        ClipStatus::Inside => cfg.color_inside,
        ClipStatus::Outside => cfg.color_outside,
        ClipStatus::Behind => cfg.color_behind,
    }
}

/// Enable.
#[allow(dead_code)]
pub fn csd_enable(cfg: &mut ClipSpaceDebugConfig) {
    cfg.enabled = true;
}

/// Disable.
#[allow(dead_code)]
pub fn csd_disable(cfg: &mut ClipSpaceDebugConfig) {
    cfg.enabled = false;
}

/// Set point size.
#[allow(dead_code)]
pub fn csd_set_point_size(cfg: &mut ClipSpaceDebugConfig, size: f32) {
    cfg.point_size = size.max(1.0);
}

/// Count vertices inside frustum.
#[allow(dead_code)]
pub fn count_inside_vertices(vertices: &[[f32; 4]]) -> usize {
    vertices
        .iter()
        .filter(|&&v| classify_clip_vertex(v) == ClipStatus::Inside)
        .count()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn clip_space_debug_to_json(cfg: &ClipSpaceDebugConfig) -> String {
    format!(
        r#"{{"enabled":{},"show_frustum":{},"show_behind":{},"point_size":{:.1}}}"#,
        cfg.enabled, cfg.show_frustum, cfg.show_behind, cfg.point_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = ClipSpaceDebugConfig::default();
        assert!(!c.enabled);
        assert!(c.show_frustum);
    }

    #[test]
    fn test_classify_inside() {
        let v = [0.0f32, 0.0, 0.5, 1.0];
        assert_eq!(classify_clip_vertex(v), ClipStatus::Inside);
    }

    #[test]
    fn test_classify_outside_x() {
        let v = [2.0f32, 0.0, 0.5, 1.0];
        assert_eq!(classify_clip_vertex(v), ClipStatus::Outside);
    }

    #[test]
    fn test_classify_behind() {
        let v = [0.0f32, 0.0, 0.5, -1.0];
        assert_eq!(classify_clip_vertex(v), ClipStatus::Behind);
    }

    #[test]
    fn test_color_inside() {
        let c = ClipSpaceDebugConfig::default();
        let col = clip_status_color(ClipStatus::Inside, &c);
        assert!((col[1] - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = ClipSpaceDebugConfig::default();
        csd_enable(&mut c);
        assert!(c.enabled);
        csd_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_point_size_min() {
        let mut c = ClipSpaceDebugConfig::default();
        csd_set_point_size(&mut c, 0.0);
        assert!((c.point_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_count_inside() {
        let verts = [
            [0.0f32, 0.0, 0.5, 1.0],
            [2.0f32, 0.0, 0.5, 1.0],
            [0.0f32, 0.0, 0.5, -1.0],
        ];
        assert_eq!(count_inside_vertices(&verts), 1);
    }

    #[test]
    fn test_to_json() {
        let j = clip_space_debug_to_json(&ClipSpaceDebugConfig::default());
        assert!(j.contains("show_frustum"));
        assert!(j.contains("point_size"));
    }

    #[test]
    fn test_classify_outside_neg_z() {
        let v = [0.0f32, 0.0, -0.1, 1.0];
        assert_eq!(classify_clip_vertex(v), ClipStatus::Outside);
    }
}
