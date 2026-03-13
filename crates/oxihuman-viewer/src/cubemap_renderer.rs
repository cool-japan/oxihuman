// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Cubemap rendering utilities.

/// Cubemap face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CubemapFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

/// Cubemap configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CubemapConfig {
    pub face_size: u32,
    pub mip_levels: u32,
    pub format: CubemapFormat,
}

/// Cubemap format.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CubemapFormat {
    Rgba8,
    Rgba16f,
    Rgba32f,
}

/// Default cubemap config.
#[allow(dead_code)]
pub fn default_cubemap_config() -> CubemapConfig {
    CubemapConfig {
        face_size: 256,
        mip_levels: 1,
        format: CubemapFormat::Rgba8,
    }
}

/// Get the direction for a cubemap face.
#[allow(dead_code)]
pub fn face_direction(face: CubemapFace) -> [f32; 3] {
    match face {
        CubemapFace::PosX => [1.0, 0.0, 0.0],
        CubemapFace::NegX => [-1.0, 0.0, 0.0],
        CubemapFace::PosY => [0.0, 1.0, 0.0],
        CubemapFace::NegY => [0.0, -1.0, 0.0],
        CubemapFace::PosZ => [0.0, 0.0, 1.0],
        CubemapFace::NegZ => [0.0, 0.0, -1.0],
    }
}

/// Get the up vector for a cubemap face.
#[allow(dead_code)]
pub fn face_up(face: CubemapFace) -> [f32; 3] {
    match face {
        CubemapFace::PosY => [0.0, 0.0, -1.0],
        CubemapFace::NegY => [0.0, 0.0, 1.0],
        _ => [0.0, 1.0, 0.0],
    }
}

/// Total memory for cubemap in bytes.
#[allow(dead_code)]
pub fn cubemap_memory(config: &CubemapConfig) -> u64 {
    let bpp: u64 = match config.format {
        CubemapFormat::Rgba8 => 4,
        CubemapFormat::Rgba16f => 8,
        CubemapFormat::Rgba32f => 16,
    };
    let face_bytes = (config.face_size as u64) * (config.face_size as u64) * bpp;
    face_bytes * 6 * config.mip_levels.max(1) as u64
}

/// All face variants.
#[allow(dead_code)]
pub fn all_faces() -> [CubemapFace; 6] {
    [
        CubemapFace::PosX, CubemapFace::NegX,
        CubemapFace::PosY, CubemapFace::NegY,
        CubemapFace::PosZ, CubemapFace::NegZ,
    ]
}

/// Face label.
#[allow(dead_code)]
pub fn face_label(face: CubemapFace) -> &'static str {
    match face {
        CubemapFace::PosX => "+X",
        CubemapFace::NegX => "-X",
        CubemapFace::PosY => "+Y",
        CubemapFace::NegY => "-Y",
        CubemapFace::PosZ => "+Z",
        CubemapFace::NegZ => "-Z",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_cubemap_config();
        assert_eq!(c.face_size, 256);
    }

    #[test]
    fn test_face_direction() {
        let d = face_direction(CubemapFace::PosX);
        assert!((d[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_face_up() {
        let u = face_up(CubemapFace::PosY);
        assert!((u[2] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_cubemap_memory() {
        let c = default_cubemap_config();
        let mem = cubemap_memory(&c);
        assert_eq!(mem, 256 * 256 * 4 * 6);
    }

    #[test]
    fn test_all_faces() {
        let faces = all_faces();
        assert_eq!(faces.len(), 6);
    }

    #[test]
    fn test_face_label() {
        assert_eq!(face_label(CubemapFace::PosX), "+X");
        assert_eq!(face_label(CubemapFace::NegZ), "-Z");
    }

    #[test]
    fn test_rgba16_memory() {
        let mut c = default_cubemap_config();
        c.format = CubemapFormat::Rgba16f;
        let mem = cubemap_memory(&c);
        assert_eq!(mem, 256 * 256 * 8 * 6);
    }

    #[test]
    fn test_mip_levels() {
        let mut c = default_cubemap_config();
        c.mip_levels = 4;
        let mem = cubemap_memory(&c);
        assert!(mem > cubemap_memory(&default_cubemap_config()));
    }

    #[test]
    fn test_neg_face_direction() {
        let d = face_direction(CubemapFace::NegY);
        assert!((d[1] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_default_up_for_side() {
        let u = face_up(CubemapFace::PosX);
        assert!((u[1] - 1.0).abs() < 1e-6);
    }
}
