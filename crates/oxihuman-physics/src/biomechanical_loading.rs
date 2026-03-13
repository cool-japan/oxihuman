// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Biomechanical load model for joint force estimation.
#[derive(Debug, Clone)]
pub struct BiomechanicalLoad {
    pub body_weight: f32,
    pub activity_factor: f32,
    pub joint: String,
}

/// Create a new BiomechanicalLoad.
pub fn new_biomechanical_load(bw: f32, af: f32, joint: &str) -> BiomechanicalLoad {
    BiomechanicalLoad {
        body_weight: bw,
        activity_factor: af,
        joint: joint.to_string(),
    }
}

/// Joint reaction force: F = body_weight * activity_factor.
pub fn joint_reaction_force(b: &BiomechanicalLoad) -> f32 {
    b.body_weight * b.activity_factor
}

/// Peak load estimate during dynamic activity: F_peak = bw * af * 1.5.
pub fn peak_load_estimate(b: &BiomechanicalLoad) -> f32 {
    b.body_weight * b.activity_factor * 1.5
}

/// Returns true when reaction force exceeds 2000 N.
pub fn is_high_load(b: &BiomechanicalLoad) -> bool {
    joint_reaction_force(b) > 2000.0
}

/// Approximate daily load cycles based on activity factor.
pub fn daily_load_cycles(b: &BiomechanicalLoad) -> u32 {
    let base_steps = 10_000u32;
    let factor = (b.activity_factor * 2.0).clamp(0.5, 5.0);
    (base_steps as f32 * factor) as u32
}

/// Stress on joint cartilage (F / estimated area in cm²).
pub fn joint_cartilage_stress(b: &BiomechanicalLoad, area_cm2: f32) -> f32 {
    if area_cm2 < 1e-6 {
        return 0.0;
    }
    joint_reaction_force(b) / area_cm2
}

/// Compressive load index: (reaction force / body weight).
pub fn load_index(b: &BiomechanicalLoad) -> f32 {
    if b.body_weight < 1e-9 {
        return 0.0;
    }
    joint_reaction_force(b) / b.body_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_biomechanical_load() {
        /* constructor */
        let b = new_biomechanical_load(700.0, 3.0, "knee");
        assert!((b.body_weight - 700.0).abs() < 1e-6);
        assert_eq!(b.joint, "knee");
    }

    #[test]
    fn test_joint_reaction_force() {
        let b = new_biomechanical_load(700.0, 3.0, "knee");
        let f = joint_reaction_force(&b);
        assert!((f - 2100.0).abs() < 1e-4);
    }

    #[test]
    fn test_peak_load_estimate() {
        let b = new_biomechanical_load(700.0, 3.0, "knee");
        let f = peak_load_estimate(&b);
        assert!((f - 3150.0).abs() < 1e-3);
    }

    #[test]
    fn test_is_high_load_true() {
        /* 700 * 3 = 2100 > 2000 */
        let b = new_biomechanical_load(700.0, 3.0, "knee");
        assert!(is_high_load(&b));
    }

    #[test]
    fn test_is_high_load_false() {
        /* 500 * 3 = 1500 < 2000 */
        let b = new_biomechanical_load(500.0, 3.0, "hip");
        assert!(!is_high_load(&b));
    }

    #[test]
    fn test_daily_load_cycles_positive() {
        let b = new_biomechanical_load(700.0, 1.0, "ankle");
        assert!(daily_load_cycles(&b) > 0);
    }

    #[test]
    fn test_joint_cartilage_stress() {
        let b = new_biomechanical_load(700.0, 3.0, "knee");
        let s = joint_cartilage_stress(&b, 25.0);
        assert!((s - 2100.0 / 25.0).abs() < 1e-3);
    }

    #[test]
    fn test_load_index() {
        let b = new_biomechanical_load(700.0, 3.0, "knee");
        let idx = load_index(&b);
        assert!((idx - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_load_index_zero_weight() {
        let b = new_biomechanical_load(0.0, 3.0, "knee");
        assert!(load_index(&b).abs() < 1e-9);
    }
}
