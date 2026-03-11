// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Billboard impostor generation — creates camera-facing quad impostors for distant meshes.

use std::f32::consts::TAU;

/// Configuration for impostor generation.
#[derive(Debug, Clone)]
pub struct ImpostorConfig {
    /// Number of viewing angles to capture.
    pub angle_steps: u32,
    /// Texture resolution per face (width = height).
    pub resolution: u32,
    /// Whether to include alpha channel.
    pub use_alpha: bool,
}

impl Default for ImpostorConfig {
    fn default() -> Self {
        Self {
            angle_steps: 8,
            resolution: 256,
            use_alpha: true,
        }
    }
}

/// A single impostor billboard quad (two triangles).
#[derive(Debug, Clone)]
pub struct ImpostorQuad {
    pub center: [f32; 3],
    pub half_width: f32,
    pub half_height: f32,
    pub angle_index: u32,
}

/// Collection of impostor quads covering all viewing angles.
#[derive(Debug, Default, Clone)]
pub struct ImpostorAtlas {
    pub quads: Vec<ImpostorQuad>,
    pub config: ImpostorConfig,
}

impl ImpostorAtlas {
    /// Generates impostor quads for all configured angles.
    pub fn generate(config: ImpostorConfig, center: [f32; 3], half_extents: [f32; 2]) -> Self {
        let mut quads = Vec::with_capacity(config.angle_steps as usize);
        for i in 0..config.angle_steps {
            quads.push(ImpostorQuad {
                center,
                half_width: half_extents[0],
                half_height: half_extents[1],
                angle_index: i,
            });
        }
        Self { quads, config }
    }

    /// Returns the view angle in radians for a given quad index.
    pub fn angle_for_index(&self, idx: u32) -> f32 {
        (idx as f32 / self.config.angle_steps as f32) * TAU
    }

    /// Returns the number of quads.
    pub fn quad_count(&self) -> usize {
        self.quads.len()
    }
}

/// Computes the nearest impostor angle index for a camera azimuth.
pub fn nearest_angle_index(azimuth_radians: f32, angle_steps: u32) -> u32 {
    let normalized = azimuth_radians.rem_euclid(TAU) / TAU;
    ((normalized * angle_steps as f32).round() as u32) % angle_steps
}

/// Builds vertex positions for a billboard quad facing +Z.
pub fn billboard_quad_vertices(center: [f32; 3], hw: f32, hh: f32) -> [[f32; 3]; 4] {
    [
        [center[0] - hw, center[1] - hh, center[2]],
        [center[0] + hw, center[1] - hh, center[2]],
        [center[0] + hw, center[1] + hh, center[2]],
        [center[0] - hw, center[1] + hh, center[2]],
    ]
}

/// Returns the UV coordinates for a standard billboard quad.
pub fn billboard_quad_uvs() -> [[f32; 2]; 4] {
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        /* Default config should have 8 angle steps */
        let cfg = ImpostorConfig::default();
        assert_eq!(cfg.angle_steps, 8);
    }

    #[test]
    fn test_generate_quad_count() {
        /* Atlas should have one quad per angle step */
        let cfg = ImpostorConfig {
            angle_steps: 8,
            resolution: 128,
            use_alpha: true,
        };
        let atlas = ImpostorAtlas::generate(cfg, [0.0; 3], [1.0, 2.0]);
        assert_eq!(atlas.quad_count(), 8);
    }

    #[test]
    fn test_angle_for_index_zero() {
        /* Index 0 should map to angle 0 */
        let cfg = ImpostorConfig::default();
        let atlas = ImpostorAtlas::generate(cfg, [0.0; 3], [1.0, 1.0]);
        assert_eq!(atlas.angle_for_index(0), 0.0);
    }

    #[test]
    fn test_angle_for_index_half() {
        /* Index 4 of 8 should map to PI */
        let cfg = ImpostorConfig {
            angle_steps: 8,
            resolution: 128,
            use_alpha: false,
        };
        let atlas = ImpostorAtlas::generate(cfg, [0.0; 3], [1.0, 1.0]);
        assert!((atlas.angle_for_index(4) - PI).abs() < 0.001);
    }

    #[test]
    fn test_nearest_angle_index_zero() {
        /* Azimuth 0 → index 0 */
        assert_eq!(nearest_angle_index(0.0, 8), 0);
    }

    #[test]
    fn test_nearest_angle_wraps() {
        /* Azimuth TAU should wrap to 0 */
        assert_eq!(nearest_angle_index(TAU, 8), 0);
    }

    #[test]
    fn test_billboard_quad_vertices_count() {
        /* Should return exactly 4 vertices */
        let verts = billboard_quad_vertices([0.0; 3], 1.0, 1.0);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn test_billboard_quad_uvs_count() {
        /* Should return exactly 4 UV pairs */
        assert_eq!(billboard_quad_uvs().len(), 4);
    }

    #[test]
    fn test_billboard_center() {
        /* Average of quad vertices should equal center */
        let c = [1.0f32, 2.0, 3.0];
        let verts = billboard_quad_vertices(c, 1.0, 1.0);
        let avg_x = verts.iter().map(|v| v[0]).sum::<f32>() / 4.0;
        assert!((avg_x - c[0]).abs() < 0.001);
    }

    #[test]
    fn test_empty_atlas_is_empty() {
        /* Default atlas has no quads */
        let atlas = ImpostorAtlas::default();
        assert_eq!(atlas.quad_count(), 0);
    }
}
