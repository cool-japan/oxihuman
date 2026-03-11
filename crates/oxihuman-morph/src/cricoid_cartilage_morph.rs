// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cricoid cartilage shape morph — controls ring dimensions and posterior plate.

/// Cricoid cartilage morph configuration.
#[derive(Debug, Clone)]
pub struct CricoidCartilageMorph {
    pub ring_radius: f32,
    pub posterior_plate_height: f32,
    pub arch_width: f32,
    pub arch_height: f32,
}

impl CricoidCartilageMorph {
    pub fn new() -> Self {
        Self {
            ring_radius: 0.5,
            posterior_plate_height: 0.5,
            arch_width: 0.5,
            arch_height: 0.3,
        }
    }
}

impl Default for CricoidCartilageMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new cricoid cartilage morph.
pub fn new_cricoid_cartilage_morph() -> CricoidCartilageMorph {
    CricoidCartilageMorph::new()
}

/// Set cricoid ring radius.
pub fn cricoid_set_ring_radius(m: &mut CricoidCartilageMorph, v: f32) {
    m.ring_radius = v.clamp(0.0, 1.0);
}

/// Set posterior plate height.
pub fn cricoid_set_posterior_plate_height(m: &mut CricoidCartilageMorph, v: f32) {
    m.posterior_plate_height = v.clamp(0.0, 1.0);
}

/// Set arch width.
pub fn cricoid_set_arch_width(m: &mut CricoidCartilageMorph, v: f32) {
    m.arch_width = v.clamp(0.0, 1.0);
}

/// Set arch height.
pub fn cricoid_set_arch_height(m: &mut CricoidCartilageMorph, v: f32) {
    m.arch_height = v.clamp(0.0, 1.0);
}

/// Approximate circumference of the ring.
pub fn cricoid_ring_circumference(m: &CricoidCartilageMorph) -> f32 {
    std::f32::consts::TAU * m.ring_radius
}

/// Serialize to JSON-like string.
pub fn cricoid_cartilage_morph_to_json(m: &CricoidCartilageMorph) -> String {
    format!(
        r#"{{"ring_radius":{:.4},"posterior_plate_height":{:.4},"arch_width":{:.4},"arch_height":{:.4}}}"#,
        m.ring_radius, m.posterior_plate_height, m.arch_width, m.arch_height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_cricoid_cartilage_morph();
        assert!((m.ring_radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ring_radius() {
        let mut m = new_cricoid_cartilage_morph();
        cricoid_set_ring_radius(&mut m, 0.7);
        assert!((m.ring_radius - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_ring_radius_clamp() {
        let mut m = new_cricoid_cartilage_morph();
        cricoid_set_ring_radius(&mut m, 3.0);
        assert_eq!(m.ring_radius, 1.0);
    }

    #[test]
    fn test_posterior_plate() {
        let mut m = new_cricoid_cartilage_morph();
        cricoid_set_posterior_plate_height(&mut m, 0.8);
        assert!((m.posterior_plate_height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_arch_width() {
        let mut m = new_cricoid_cartilage_morph();
        cricoid_set_arch_width(&mut m, 0.6);
        assert!((m.arch_width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_circumference_positive() {
        let m = new_cricoid_cartilage_morph();
        assert!(cricoid_ring_circumference(&m) > 0.0);
    }

    #[test]
    fn test_arch_height_clamp() {
        let mut m = new_cricoid_cartilage_morph();
        cricoid_set_arch_height(&mut m, -0.5);
        assert_eq!(m.arch_height, 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_cricoid_cartilage_morph();
        let s = cricoid_cartilage_morph_to_json(&m);
        assert!(s.contains("ring_radius"));
    }

    #[test]
    fn test_clone() {
        let m = new_cricoid_cartilage_morph();
        let m2 = m.clone();
        assert!((m2.ring_radius - m.ring_radius).abs() < 1e-6);
    }
}
