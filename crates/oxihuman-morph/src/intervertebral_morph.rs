// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Intervertebral disc height morph — adjusts disc spacing along the spine.

/// Intervertebral disc morph configuration.
#[derive(Debug, Clone)]
pub struct IntervertebralMorph {
    pub cervical_height: f32,
    pub thoracic_height: f32,
    pub lumbar_height: f32,
    pub degeneration: f32,
}

impl IntervertebralMorph {
    pub fn new() -> Self {
        Self {
            cervical_height: 0.5,
            thoracic_height: 0.5,
            lumbar_height: 0.5,
            degeneration: 0.0,
        }
    }
}

impl Default for IntervertebralMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new intervertebral disc morph.
pub fn new_intervertebral_morph() -> IntervertebralMorph {
    IntervertebralMorph::new()
}

/// Set cervical disc height (0 = collapsed, 1 = maximal).
pub fn ivm_set_cervical_height(m: &mut IntervertebralMorph, v: f32) {
    m.cervical_height = v.clamp(0.0, 1.0);
}

/// Set thoracic disc height.
pub fn ivm_set_thoracic_height(m: &mut IntervertebralMorph, v: f32) {
    m.thoracic_height = v.clamp(0.0, 1.0);
}

/// Set lumbar disc height.
pub fn ivm_set_lumbar_height(m: &mut IntervertebralMorph, v: f32) {
    m.lumbar_height = v.clamp(0.0, 1.0);
}

/// Set degeneration factor (0 = healthy, 1 = fully degenerated).
pub fn ivm_set_degeneration(m: &mut IntervertebralMorph, v: f32) {
    m.degeneration = v.clamp(0.0, 1.0);
}

/// Effective total disc height contribution (degeneration reduces all heights).
pub fn ivm_effective_height(m: &IntervertebralMorph) -> f32 {
    let avg = (m.cervical_height + m.thoracic_height + m.lumbar_height) / 3.0;
    avg * (1.0 - m.degeneration * 0.5)
}

/// Serialize to JSON-like string.
pub fn intervertebral_morph_to_json(m: &IntervertebralMorph) -> String {
    format!(
        r#"{{"cervical_height":{:.4},"thoracic_height":{:.4},"lumbar_height":{:.4},"degeneration":{:.4}}}"#,
        m.cervical_height, m.thoracic_height, m.lumbar_height, m.degeneration
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_intervertebral_morph();
        assert!((m.cervical_height - 0.5).abs() < 1e-6);
        assert_eq!(m.degeneration, 0.0);
    }

    #[test]
    fn test_cervical_clamp_high() {
        let mut m = new_intervertebral_morph();
        ivm_set_cervical_height(&mut m, 5.0);
        assert_eq!(m.cervical_height, 1.0);
    }

    #[test]
    fn test_thoracic_clamp_low() {
        let mut m = new_intervertebral_morph();
        ivm_set_thoracic_height(&mut m, -1.0);
        assert_eq!(m.thoracic_height, 0.0);
    }

    #[test]
    fn test_lumbar_set() {
        let mut m = new_intervertebral_morph();
        ivm_set_lumbar_height(&mut m, 0.9);
        assert!((m.lumbar_height - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_degeneration_reduces_height() {
        let mut m = new_intervertebral_morph();
        let h_healthy = ivm_effective_height(&m);
        ivm_set_degeneration(&mut m, 1.0);
        let h_degen = ivm_effective_height(&m);
        assert!(h_degen < h_healthy); /* degeneration shrinks height */
    }

    #[test]
    fn test_effective_height_range() {
        let m = new_intervertebral_morph();
        let h = ivm_effective_height(&m);
        assert!((0.0..=1.0).contains(&h));
    }

    #[test]
    fn test_json_keys() {
        let m = new_intervertebral_morph();
        let s = intervertebral_morph_to_json(&m);
        assert!(s.contains("degeneration"));
    }

    #[test]
    fn test_clone() {
        let m = new_intervertebral_morph();
        let m2 = m.clone();
        assert!((m2.lumbar_height - m.lumbar_height).abs() < 1e-6);
    }

    #[test]
    fn test_degeneration_clamp() {
        let mut m = new_intervertebral_morph();
        ivm_set_degeneration(&mut m, 2.0);
        assert_eq!(m.degeneration, 1.0);
    }
}
