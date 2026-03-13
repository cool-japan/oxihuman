// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Cubemap preview: cross-shaped unfolded cubemap for debugging environment maps.

use std::f32::consts::PI;

/// Cubemap face identifiers.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CubeFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

/// Configuration for cubemap preview.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CubemapPreviewConfig {
    pub face_size: u32,
    pub gap: u32,
    pub show_labels: bool,
}

#[allow(dead_code)]
pub fn default_cubemap_preview_config() -> CubemapPreviewConfig {
    CubemapPreviewConfig {
        face_size: 128,
        gap: 2,
        show_labels: true,
    }
}

#[allow(dead_code)]
pub fn face_name(face: CubeFace) -> &'static str {
    match face {
        CubeFace::PosX => "+X",
        CubeFace::NegX => "-X",
        CubeFace::PosY => "+Y",
        CubeFace::NegY => "-Y",
        CubeFace::PosZ => "+Z",
        CubeFace::NegZ => "-Z",
    }
}

/// Returns the (col, row) position of a face in a cross layout.
#[allow(dead_code)]
pub fn face_cross_position(face: CubeFace) -> (u32, u32) {
    match face {
        CubeFace::PosX => (2, 1),
        CubeFace::NegX => (0, 1),
        CubeFace::PosY => (1, 0),
        CubeFace::NegY => (1, 2),
        CubeFace::PosZ => (1, 1),
        CubeFace::NegZ => (3, 1),
    }
}

/// Calculate the pixel offset for a face in the cross layout.
#[allow(dead_code)]
pub fn face_pixel_offset(face: CubeFace, cfg: &CubemapPreviewConfig) -> (u32, u32) {
    let (col, row) = face_cross_position(face);
    let stride = cfg.face_size + cfg.gap;
    (col * stride, row * stride)
}

/// Total width/height of the cross layout.
#[allow(dead_code)]
pub fn cross_layout_size(cfg: &CubemapPreviewConfig) -> (u32, u32) {
    let stride = cfg.face_size + cfg.gap;
    (4 * stride - cfg.gap, 3 * stride - cfg.gap)
}

/// Direction vector for a cubemap UV coordinate.
#[allow(dead_code)]
pub fn uv_to_direction(face: CubeFace, u: f32, v: f32) -> [f32; 3] {
    let s = 2.0 * u - 1.0;
    let t = 2.0 * v - 1.0;
    match face {
        CubeFace::PosX => [1.0, -t, -s],
        CubeFace::NegX => [-1.0, -t, s],
        CubeFace::PosY => [s, 1.0, t],
        CubeFace::NegY => [s, -1.0, -t],
        CubeFace::PosZ => [s, -t, 1.0],
        CubeFace::NegZ => [-s, -t, -1.0],
    }
}

/// Convert a direction to spherical (theta, phi).
#[allow(dead_code)]
pub fn direction_to_spherical(dir: [f32; 3]) -> (f32, f32) {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len < 1e-9 {
        return (0.0, 0.0);
    }
    let theta = (dir[1] / len).acos();
    let phi = dir[2].atan2(dir[0]);
    (theta, phi)
}

#[allow(dead_code)]
pub fn cubemap_preview_to_json(cfg: &CubemapPreviewConfig) -> String {
    let (w, h) = cross_layout_size(cfg);
    format!(
        r#"{{"face_size":{},"width":{},"height":{},"show_labels":{}}}"#,
        cfg.face_size, w, h, cfg.show_labels
    )
}

/// Approximate mip level from roughness.
#[allow(dead_code)]
pub fn roughness_to_mip(roughness: f32, max_mip: u32) -> u32 {
    let level = (roughness * max_mip as f32).round() as u32;
    level.min(max_mip)
}

/// Solid angle subtended by one cubemap texel.
#[allow(dead_code)]
pub fn texel_solid_angle(face_size: u32) -> f32 {
    4.0 * PI / (6.0 * (face_size as f32) * (face_size as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cubemap_preview_config();
        assert_eq!(cfg.face_size, 128);
    }

    #[test]
    fn test_face_name() {
        assert_eq!(face_name(CubeFace::PosX), "+X");
        assert_eq!(face_name(CubeFace::NegZ), "-Z");
    }

    #[test]
    fn test_face_cross_position() {
        assert_eq!(face_cross_position(CubeFace::PosZ), (1, 1));
    }

    #[test]
    fn test_face_pixel_offset() {
        let cfg = default_cubemap_preview_config();
        let (x, y) = face_pixel_offset(CubeFace::NegX, &cfg);
        assert_eq!(x, 0);
        assert_eq!(y, 130);
    }

    #[test]
    fn test_cross_layout_size() {
        let cfg = default_cubemap_preview_config();
        let (w, h) = cross_layout_size(&cfg);
        assert!(w > 0 && h > 0);
    }

    #[test]
    fn test_uv_to_direction_center() {
        let dir = uv_to_direction(CubeFace::PosZ, 0.5, 0.5);
        assert!((dir[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_direction_to_spherical() {
        let (theta, _phi) = direction_to_spherical([0.0, 1.0, 0.0]);
        assert!(theta.abs() < 1e-5);
    }

    #[test]
    fn test_roughness_to_mip() {
        assert_eq!(roughness_to_mip(0.0, 8), 0);
        assert_eq!(roughness_to_mip(1.0, 8), 8);
    }

    #[test]
    fn test_texel_solid_angle() {
        let sa = texel_solid_angle(128);
        assert!(sa > 0.0);
    }

    #[test]
    fn test_cubemap_preview_to_json() {
        let cfg = default_cubemap_preview_config();
        let j = cubemap_preview_to_json(&cfg);
        assert!(j.contains("face_size"));
    }
}
