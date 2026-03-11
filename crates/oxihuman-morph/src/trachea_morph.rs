// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Trachea shape morph — controls calibre, length, and curvature.

/// Trachea morph configuration.
#[derive(Debug, Clone)]
pub struct TracheaMorph {
    pub calibre: f32,
    pub length: f32,
    pub curvature: f32,
    pub wall_thickness: f32,
}

impl TracheaMorph {
    pub fn new() -> Self {
        Self {
            calibre: 0.5,
            length: 0.5,
            curvature: 0.0,
            wall_thickness: 0.2,
        }
    }
}

impl Default for TracheaMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new trachea morph.
pub fn new_trachea_morph() -> TracheaMorph {
    TracheaMorph::new()
}

/// Set tracheal calibre (lumen diameter).
pub fn trachea_set_calibre(m: &mut TracheaMorph, v: f32) {
    m.calibre = v.clamp(0.0, 1.0);
}

/// Set trachea length.
pub fn trachea_set_length(m: &mut TracheaMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set curvature along the trachea axis.
pub fn trachea_set_curvature(m: &mut TracheaMorph, v: f32) {
    m.curvature = v.clamp(-1.0, 1.0);
}

/// Set wall thickness.
pub fn trachea_set_wall_thickness(m: &mut TracheaMorph, v: f32) {
    m.wall_thickness = v.clamp(0.0, 1.0);
}

/// Approximate internal volume in normalised units.
pub fn trachea_volume(m: &TracheaMorph) -> f32 {
    std::f32::consts::PI * (m.calibre * 0.5) * (m.calibre * 0.5) * m.length
}

/// Serialize to JSON-like string.
pub fn trachea_morph_to_json(m: &TracheaMorph) -> String {
    format!(
        r#"{{"calibre":{:.4},"length":{:.4},"curvature":{:.4},"wall_thickness":{:.4}}}"#,
        m.calibre, m.length, m.curvature, m.wall_thickness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_trachea_morph();
        assert!((m.calibre - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_calibre_clamp() {
        let mut m = new_trachea_morph();
        trachea_set_calibre(&mut m, 5.0);
        assert_eq!(m.calibre, 1.0);
    }

    #[test]
    fn test_set_length() {
        let mut m = new_trachea_morph();
        trachea_set_length(&mut m, 0.9);
        assert!((m.length - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_negative() {
        let mut m = new_trachea_morph();
        trachea_set_curvature(&mut m, -0.3);
        assert!((m.curvature + 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_wall_thickness() {
        let mut m = new_trachea_morph();
        trachea_set_wall_thickness(&mut m, 0.4);
        assert!((m.wall_thickness - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_volume_positive() {
        let m = new_trachea_morph();
        assert!(trachea_volume(&m) > 0.0);
    }

    #[test]
    fn test_volume_zero_length() {
        let mut m = new_trachea_morph();
        trachea_set_length(&mut m, 0.0);
        assert_eq!(trachea_volume(&m), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_trachea_morph();
        let s = trachea_morph_to_json(&m);
        assert!(s.contains("wall_thickness"));
    }

    #[test]
    fn test_clone() {
        let m = new_trachea_morph();
        let m2 = m.clone();
        assert!((m2.length - m.length).abs() < 1e-6);
    }
}
