// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex index/ID overlay for debugging.

/// Vertex ID overlay configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexIdOverlayConfig {
    /// Enable the overlay.
    pub enabled: bool,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color [R, G, B, A].
    pub text_color: [f32; 4],
    /// Background color [R, G, B, A].
    pub bg_color: [f32; 4],
    /// Maximum vertex ID to show (none = show all).
    pub max_id: Option<u32>,
    /// Show only every Nth vertex.
    pub stride: u32,
    /// Screen-space distance threshold for culling.
    pub screen_cull_dist: f32,
}

impl Default for VertexIdOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            font_size: 10.0,
            text_color: [1.0, 1.0, 0.0, 1.0],
            bg_color: [0.0, 0.0, 0.0, 0.5],
            max_id: Some(10_000),
            stride: 1,
            screen_cull_dist: 0.02,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_vertex_id_overlay_config() -> VertexIdOverlayConfig {
    VertexIdOverlayConfig::default()
}

/// Check if a vertex ID should be displayed.
#[allow(dead_code)]
pub fn should_display_vertex(id: u32, cfg: &VertexIdOverlayConfig) -> bool {
    if let Some(max) = cfg.max_id {
        if id > max {
            return false;
        }
    }
    let stride = cfg.stride.max(1);
    id.is_multiple_of(stride)
}

/// Format vertex ID as label.
#[allow(dead_code)]
pub fn format_vertex_id(id: u32) -> String {
    format!("v{}", id)
}

/// Enable.
#[allow(dead_code)]
pub fn vid_enable(cfg: &mut VertexIdOverlayConfig) {
    cfg.enabled = true;
}

/// Disable.
#[allow(dead_code)]
pub fn vid_disable(cfg: &mut VertexIdOverlayConfig) {
    cfg.enabled = false;
}

/// Set stride.
#[allow(dead_code)]
pub fn vid_set_stride(cfg: &mut VertexIdOverlayConfig, stride: u32) {
    cfg.stride = stride.max(1);
}

/// Set max ID filter.
#[allow(dead_code)]
pub fn vid_set_max_id(cfg: &mut VertexIdOverlayConfig, max: Option<u32>) {
    cfg.max_id = max;
}

/// Count displayable vertices from a range.
#[allow(dead_code)]
pub fn count_displayable(count: u32, cfg: &VertexIdOverlayConfig) -> u32 {
    let stride = cfg.stride.max(1);
    let effective_max = if let Some(m) = cfg.max_id {
        count.min(m + 1)
    } else {
        count
    };
    effective_max.div_ceil(stride)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn vertex_id_overlay_to_json(cfg: &VertexIdOverlayConfig) -> String {
    let max_str = match cfg.max_id {
        Some(m) => m.to_string(),
        None => "null".to_string(),
    };
    format!(
        r#"{{"enabled":{},"stride":{},"max_id":{},"font_size":{:.1}}}"#,
        cfg.enabled, cfg.stride, max_str, cfg.font_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = VertexIdOverlayConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.stride, 1);
    }

    #[test]
    fn test_should_display_stride1() {
        let c = VertexIdOverlayConfig::default();
        assert!(should_display_vertex(0, &c));
        assert!(should_display_vertex(1, &c));
    }

    #[test]
    fn test_should_display_stride2() {
        let mut c = VertexIdOverlayConfig::default();
        vid_set_stride(&mut c, 2);
        assert!(should_display_vertex(0, &c));
        assert!(!should_display_vertex(1, &c));
        assert!(should_display_vertex(2, &c));
    }

    #[test]
    fn test_should_display_max_id() {
        let c = VertexIdOverlayConfig {
            max_id: Some(100),
            ..Default::default()
        };
        assert!(!should_display_vertex(101, &c));
        assert!(should_display_vertex(100, &c));
    }

    #[test]
    fn test_format_vertex_id() {
        assert_eq!(format_vertex_id(42), "v42");
    }

    #[test]
    fn test_enable_disable() {
        let mut c = VertexIdOverlayConfig::default();
        vid_enable(&mut c);
        assert!(c.enabled);
        vid_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_stride_min() {
        let mut c = VertexIdOverlayConfig::default();
        vid_set_stride(&mut c, 0);
        assert_eq!(c.stride, 1);
    }

    #[test]
    fn test_set_max_id_none() {
        let mut c = VertexIdOverlayConfig::default();
        vid_set_max_id(&mut c, None);
        assert!(c.max_id.is_none());
    }

    #[test]
    fn test_count_displayable() {
        let c = VertexIdOverlayConfig {
            stride: 2,
            max_id: None,
            ..Default::default()
        };
        let n = count_displayable(10, &c);
        assert_eq!(n, 5);
    }

    #[test]
    fn test_to_json() {
        let j = vertex_id_overlay_to_json(&VertexIdOverlayConfig::default());
        assert!(j.contains("stride"));
        assert!(j.contains("font_size"));
    }
}
