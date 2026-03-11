// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Glottis opening/closing morph — controls vocal fold gap width and shape.

/// Glottis morph configuration.
#[derive(Debug, Clone)]
pub struct GlottisMorph {
    pub opening: f32,
    pub posterior_gap: f32,
    pub mucosal_wave: f32,
    pub tension: f32,
}

impl GlottisMorph {
    pub fn new() -> Self {
        Self {
            opening: 1.0,
            posterior_gap: 0.0,
            mucosal_wave: 0.0,
            tension: 0.5,
        }
    }
}

impl Default for GlottisMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new glottis morph.
pub fn new_glottis_morph() -> GlottisMorph {
    GlottisMorph::new()
}

/// Set glottal opening (0 = closed, 1 = fully open).
pub fn glottis_set_opening(m: &mut GlottisMorph, v: f32) {
    m.opening = v.clamp(0.0, 1.0);
}

/// Set posterior glottal gap (breathiness).
pub fn glottis_set_posterior_gap(m: &mut GlottisMorph, v: f32) {
    m.posterior_gap = v.clamp(0.0, 1.0);
}

/// Set mucosal wave amplitude.
pub fn glottis_set_mucosal_wave(m: &mut GlottisMorph, v: f32) {
    m.mucosal_wave = v.clamp(0.0, 1.0);
}

/// Set vocal fold tension (pitch correlate).
pub fn glottis_set_tension(m: &mut GlottisMorph, v: f32) {
    m.tension = v.clamp(0.0, 1.0);
}

/// Returns true when glottis is effectively closed.
pub fn glottis_is_closed(m: &GlottisMorph) -> bool {
    m.opening < 0.05
}

/// Serialize to JSON-like string.
pub fn glottis_morph_to_json(m: &GlottisMorph) -> String {
    format!(
        r#"{{"opening":{:.4},"posterior_gap":{:.4},"mucosal_wave":{:.4},"tension":{:.4}}}"#,
        m.opening, m.posterior_gap, m.mucosal_wave, m.tension
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_glottis_morph();
        assert_eq!(m.opening, 1.0);
        assert!(!glottis_is_closed(&m));
    }

    #[test]
    fn test_close_glottis() {
        let mut m = new_glottis_morph();
        glottis_set_opening(&mut m, 0.0);
        assert!(glottis_is_closed(&m));
    }

    #[test]
    fn test_opening_clamp() {
        let mut m = new_glottis_morph();
        glottis_set_opening(&mut m, 3.0);
        assert_eq!(m.opening, 1.0);
    }

    #[test]
    fn test_posterior_gap() {
        let mut m = new_glottis_morph();
        glottis_set_posterior_gap(&mut m, 0.4);
        assert!((m.posterior_gap - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_mucosal_wave() {
        let mut m = new_glottis_morph();
        glottis_set_mucosal_wave(&mut m, 0.6);
        assert!((m.mucosal_wave - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_tension_clamp() {
        let mut m = new_glottis_morph();
        glottis_set_tension(&mut m, -1.0);
        assert_eq!(m.tension, 0.0);
    }

    #[test]
    fn test_not_closed_near_threshold() {
        let mut m = new_glottis_morph();
        glottis_set_opening(&mut m, 0.06);
        assert!(!glottis_is_closed(&m));
    }

    #[test]
    fn test_json_keys() {
        let m = new_glottis_morph();
        let s = glottis_morph_to_json(&m);
        assert!(s.contains("mucosal_wave"));
    }

    #[test]
    fn test_clone() {
        let m = new_glottis_morph();
        let m2 = m.clone();
        assert!((m2.tension - m.tension).abs() < 1e-6);
    }
}
