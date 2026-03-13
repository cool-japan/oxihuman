// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cubemap face debug rendering data (6-face cross layout).

#![allow(dead_code)]

/// Names of the 6 cubemap faces.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

/// Debug rendering config for a cubemap.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CubemapDebugConfig {
    /// Resolution of each face in pixels.
    pub face_size: u32,
    /// Show face label overlay.
    pub show_labels: bool,
    /// Brightness multiplier.
    pub exposure: f32,
    /// Which face to highlight (None = all).
    pub highlight: Option<CubeFace>,
}

#[allow(dead_code)]
impl Default for CubemapDebugConfig {
    fn default() -> Self {
        Self {
            face_size: 256,
            show_labels: true,
            exposure: 1.0,
            highlight: None,
        }
    }
}

/// UV coordinates for a pixel on a given cubemap face.
#[allow(dead_code)]
pub fn face_uv(face: CubeFace, u: f32, v: f32) -> [f32; 3] {
    let u = u * 2.0 - 1.0;
    let v = v * 2.0 - 1.0;
    match face {
        CubeFace::PosX => [1.0, -v, -u],
        CubeFace::NegX => [-1.0, -v, u],
        CubeFace::PosY => [u, 1.0, v],
        CubeFace::NegY => [u, -1.0, -v],
        CubeFace::PosZ => [u, -v, 1.0],
        CubeFace::NegZ => [-u, -v, -1.0],
    }
}

/// Normalize a direction vector.
#[allow(dead_code)]
pub fn normalize_dir(d: [f32; 3]) -> [f32; 3] {
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [d[0] / len, d[1] / len, d[2] / len]
}

/// Label string for a cubemap face.
#[allow(dead_code)]
pub fn face_label(face: CubeFace) -> &'static str {
    match face {
        CubeFace::PosX => "+X",
        CubeFace::NegX => "-X",
        CubeFace::PosY => "+Y",
        CubeFace::NegY => "-Y",
        CubeFace::PosZ => "+Z",
        CubeFace::NegZ => "-Z",
    }
}

/// Create a new default config.
#[allow(dead_code)]
pub fn new_cubemap_debug_config() -> CubemapDebugConfig {
    CubemapDebugConfig::default()
}

/// Set the exposure of the debug view.
#[allow(dead_code)]
pub fn cd_set_exposure(cfg: &mut CubemapDebugConfig, value: f32) {
    cfg.exposure = value.max(0.0);
}

/// Toggle label overlay.
#[allow(dead_code)]
pub fn cd_toggle_labels(cfg: &mut CubemapDebugConfig) {
    cfg.show_labels = !cfg.show_labels;
}

/// Set which face to highlight.
#[allow(dead_code)]
pub fn cd_highlight(cfg: &mut CubemapDebugConfig, face: Option<CubeFace>) {
    cfg.highlight = face;
}

/// Total pixel count for the cross-layout (3x4 grid of face_size).
#[allow(dead_code)]
pub fn cross_layout_pixel_count(cfg: &CubemapDebugConfig) -> u64 {
    (cfg.face_size as u64) * (cfg.face_size as u64) * 12
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn cubemap_debug_to_json(cfg: &CubemapDebugConfig) -> String {
    let hl = match cfg.highlight {
        None => "null".to_string(),
        Some(f) => format!("\"{}\"", face_label(f)),
    };
    format!(
        r#"{{"face_size":{},"show_labels":{},"exposure":{:.4},"highlight":{}}}"#,
        cfg.face_size, cfg.show_labels, cfg.exposure, hl
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = CubemapDebugConfig::default();
        assert_eq!(c.face_size, 256);
        assert!(c.show_labels);
    }

    #[test]
    fn test_face_label() {
        assert_eq!(face_label(CubeFace::PosX), "+X");
        assert_eq!(face_label(CubeFace::NegY), "-Y");
    }

    #[test]
    fn test_face_uv_posx_center() {
        let d = face_uv(CubeFace::PosX, 0.5, 0.5);
        assert!((d[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_unit() {
        let d = normalize_dir([3.0, 0.0, 0.0]);
        assert!((d[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_safe() {
        let d = normalize_dir([0.0, 0.0, 0.0]);
        assert!((d[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_exposure() {
        let mut c = CubemapDebugConfig::default();
        cd_set_exposure(&mut c, -1.0);
        assert!(c.exposure < 1e-6);
    }

    #[test]
    fn test_toggle_labels() {
        let mut c = CubemapDebugConfig::default();
        cd_toggle_labels(&mut c);
        assert!(!c.show_labels);
    }

    #[test]
    fn test_cross_layout_pixel_count() {
        let c = CubemapDebugConfig {
            face_size: 4,
            ..Default::default()
        };
        assert_eq!(cross_layout_pixel_count(&c), 4 * 4 * 12);
    }

    #[test]
    fn test_to_json() {
        let j = cubemap_debug_to_json(&CubemapDebugConfig::default());
        assert!(j.contains("face_size"));
        assert!(j.contains("exposure"));
    }
}
