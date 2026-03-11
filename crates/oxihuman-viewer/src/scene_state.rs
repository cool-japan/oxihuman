// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Viewer configuration and per-frame statistics.

// ── ViewerConfig ──────────────────────────────────────────────────────────────

/// Configuration for the viewer window and rendering defaults.
#[allow(dead_code)]
pub struct ViewerConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    /// RGB background colour (linear, 0.0-1.0).
    pub background_color: [f32; 3],
    /// Ambient light intensity (0.0-1.0).
    pub ambient_light: f32,
    /// Whether shadow maps are enabled.
    pub enable_shadows: bool,
}

impl ViewerConfig {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        ViewerConfig {
            width,
            height,
            title: title.to_string(),
            background_color: [0.15, 0.15, 0.18],
            ambient_light: 0.3,
            enable_shadows: false,
        }
    }
}

impl Default for ViewerConfig {
    fn default() -> Self {
        ViewerConfig::new(1280, 720, "OxiHuman Viewer")
    }
}

// ── ViewerStats ───────────────────────────────────────────────────────────────

/// Statistics returned by a single render frame call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewerStats {
    /// Total frames rendered since the viewer was created.
    pub frame_count: u64,
    /// Number of vertices in the currently uploaded mesh.
    pub vertex_count: usize,
    /// Number of triangles in the currently uploaded mesh.
    pub triangle_count: usize,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_config_defaults() {
        let cfg = ViewerConfig::default();
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
        assert_eq!(cfg.title, "OxiHuman Viewer");
        assert!((cfg.ambient_light - 0.3).abs() < 1e-6);
        assert!(!cfg.enable_shadows);
    }

    #[test]
    fn viewer_config_custom() {
        let cfg = ViewerConfig::new(800, 600, "Test");
        assert_eq!(cfg.width, 800);
        assert_eq!(cfg.height, 600);
        assert_eq!(cfg.title, "Test");
    }
}
