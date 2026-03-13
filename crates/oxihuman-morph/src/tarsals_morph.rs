// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tarsal bone arrangement morph — midfoot arch and calcaneal geometry.

/// Tarsals morph configuration.
#[derive(Debug, Clone)]
pub struct TarsalsMorph {
    pub arch_height: f32,
    pub calcaneus_length: f32,
    pub talus_tilt: f32,
    pub midfoot_width: f32,
}

impl TarsalsMorph {
    pub fn new() -> Self {
        Self {
            arch_height: 0.5,
            calcaneus_length: 0.5,
            talus_tilt: 0.0,
            midfoot_width: 0.5,
        }
    }
}

impl Default for TarsalsMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new tarsals morph.
pub fn new_tarsals_morph() -> TarsalsMorph {
    TarsalsMorph::new()
}

/// Set medial longitudinal arch height (0 = flat, 1 = high arch).
pub fn tars_set_arch_height(m: &mut TarsalsMorph, v: f32) {
    m.arch_height = v.clamp(0.0, 1.0);
}

/// Set calcaneus length.
pub fn tars_set_calcaneus_length(m: &mut TarsalsMorph, v: f32) {
    m.calcaneus_length = v.clamp(0.0, 1.0);
}

/// Set talar tilt (-1 = valgus/pronation, 0 = neutral, 1 = varus/supination).
pub fn tars_set_talus_tilt(m: &mut TarsalsMorph, v: f32) {
    m.talus_tilt = v.clamp(-1.0, 1.0);
}

/// Set midfoot width.
pub fn tars_set_midfoot_width(m: &mut TarsalsMorph, v: f32) {
    m.midfoot_width = v.clamp(0.0, 1.0);
}

/// Foot contact area heuristic (flat foot = more contact).
pub fn tars_contact_area(m: &TarsalsMorph) -> f32 {
    m.midfoot_width * (1.0 - m.arch_height * 0.5)
}

/// Serialize to JSON-like string.
pub fn tarsals_morph_to_json(m: &TarsalsMorph) -> String {
    format!(
        r#"{{"arch_height":{:.4},"calcaneus_length":{:.4},"talus_tilt":{:.4},"midfoot_width":{:.4}}}"#,
        m.arch_height, m.calcaneus_length, m.talus_tilt, m.midfoot_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_tarsals_morph();
        assert!((m.arch_height - 0.5).abs() < 1e-6);
        assert_eq!(m.talus_tilt, 0.0);
    }

    #[test]
    fn test_arch_height_clamp() {
        let mut m = new_tarsals_morph();
        tars_set_arch_height(&mut m, 5.0);
        assert_eq!(m.arch_height, 1.0);
    }

    #[test]
    fn test_calcaneus_set() {
        let mut m = new_tarsals_morph();
        tars_set_calcaneus_length(&mut m, 0.8);
        assert!((m.calcaneus_length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_tilt_clamp() {
        let mut m = new_tarsals_morph();
        tars_set_talus_tilt(&mut m, 2.0);
        assert_eq!(m.talus_tilt, 1.0);
    }

    #[test]
    fn test_midfoot_width_set() {
        let mut m = new_tarsals_morph();
        tars_set_midfoot_width(&mut m, 0.7);
        assert!((m.midfoot_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_contact_area_positive() {
        let m = new_tarsals_morph();
        assert!(tars_contact_area(&m) > 0.0);
    }

    #[test]
    fn test_contact_area_flat_foot() {
        let mut m = new_tarsals_morph();
        tars_set_arch_height(&mut m, 0.0);
        tars_set_midfoot_width(&mut m, 1.0);
        assert!((tars_contact_area(&m) - 1.0).abs() < 1e-5); /* max contact */
    }

    #[test]
    fn test_json_keys() {
        let m = new_tarsals_morph();
        let s = tarsals_morph_to_json(&m);
        assert!(s.contains("calcaneus_length"));
    }

    #[test]
    fn test_clone() {
        let m = new_tarsals_morph();
        let m2 = m.clone();
        assert!((m2.midfoot_width - m.midfoot_width).abs() < 1e-6);
    }
}
