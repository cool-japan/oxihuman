// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Larynx height position morph — controls vertical larynx position and tilt.

/// Larynx position morph configuration.
#[derive(Debug, Clone)]
pub struct LarynxPositionMorph {
    pub height: f32,
    pub tilt: f32,
    pub anterior_posterior: f32,
}

impl LarynxPositionMorph {
    pub fn new() -> Self {
        Self {
            height: 0.5,
            tilt: 0.0,
            anterior_posterior: 0.5,
        }
    }
}

impl Default for LarynxPositionMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new larynx position morph.
pub fn new_larynx_position_morph() -> LarynxPositionMorph {
    LarynxPositionMorph::new()
}

/// Set larynx height (0 = lowered, 1 = raised).
pub fn larynx_set_height(m: &mut LarynxPositionMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

/// Set larynx tilt angle.
pub fn larynx_set_tilt(m: &mut LarynxPositionMorph, v: f32) {
    m.tilt = v.clamp(-1.0, 1.0);
}

/// Set anterior-posterior position.
pub fn larynx_set_anterior_posterior(m: &mut LarynxPositionMorph, v: f32) {
    m.anterior_posterior = v.clamp(0.0, 1.0);
}

/// Effective tract lengthening from lowered larynx.
pub fn larynx_tract_lengthening(m: &LarynxPositionMorph) -> f32 {
    /* lower larynx lengthens the tract */
    (0.5 - m.height) * 0.4
}

/// Serialize to JSON-like string.
pub fn larynx_position_morph_to_json(m: &LarynxPositionMorph) -> String {
    format!(
        r#"{{"height":{:.4},"tilt":{:.4},"anterior_posterior":{:.4}}}"#,
        m.height, m.tilt, m.anterior_posterior
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_larynx_position_morph();
        assert!((m.height - 0.5).abs() < 1e-6);
        assert_eq!(m.tilt, 0.0);
    }

    #[test]
    fn test_set_height() {
        let mut m = new_larynx_position_morph();
        larynx_set_height(&mut m, 0.1);
        assert!((m.height - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_height_clamp() {
        let mut m = new_larynx_position_morph();
        larynx_set_height(&mut m, 2.0);
        assert_eq!(m.height, 1.0);
    }

    #[test]
    fn test_tilt_negative() {
        let mut m = new_larynx_position_morph();
        larynx_set_tilt(&mut m, -0.5);
        assert!((m.tilt + 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_anterior_posterior() {
        let mut m = new_larynx_position_morph();
        larynx_set_anterior_posterior(&mut m, 0.2);
        assert!((m.anterior_posterior - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_tract_lengthening_default() {
        let m = new_larynx_position_morph();
        assert!((larynx_tract_lengthening(&m)).abs() < 1e-6); /* neutral height */
    }

    #[test]
    fn test_tract_lengthening_lowered() {
        let mut m = new_larynx_position_morph();
        larynx_set_height(&mut m, 0.0);
        assert!(larynx_tract_lengthening(&m) > 0.0); /* lengthened */
    }

    #[test]
    fn test_json_keys() {
        let m = new_larynx_position_morph();
        let s = larynx_position_morph_to_json(&m);
        assert!(s.contains("anterior_posterior"));
    }

    #[test]
    fn test_clone() {
        let m = new_larynx_position_morph();
        let m2 = m.clone();
        assert!((m2.height - m.height).abs() < 1e-6);
    }
}
