// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single bone element following Wolff's law remodeling.
pub struct BoneElement {
    pub density: f32,
    pub stress: f32,
    pub remodeling_rate: f32,
}

impl BoneElement {
    pub fn new(density: f32) -> Self {
        BoneElement {
            density,
            stress: 0.0,
            remodeling_rate: 0.01,
        }
    }
}

/// Equilibrium density scales linearly with applied stress (simplified model).
pub fn bone_equilibrium_density(stress: f32) -> f32 {
    // Simple linear: 1.0 kg/m³ per unit stress, clamped to [0.1, 2.5]
    (stress * 1.0).clamp(0.1, 2.5)
}

/// Step remodeling: density moves toward equilibrium for applied stress.
pub fn bone_step(b: &mut BoneElement, applied_stress: f32, dt: f32) {
    b.stress = applied_stress;
    let target = bone_equilibrium_density(applied_stress);
    b.density += b.remodeling_rate * (target - b.density) * dt;
}

pub fn bone_is_osteoporotic(b: &BoneElement) -> bool {
    b.density < 0.5
}

pub fn bone_is_dense(b: &BoneElement) -> bool {
    b.density > 1.5
}

pub fn bone_mineral_content(b: &BoneElement, volume: f32) -> f32 {
    b.density * volume
}

pub fn new_bone_element(density: f32) -> BoneElement {
    BoneElement::new(density)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new element has specified density */
        let b = new_bone_element(1.0);
        assert!((b.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_equilibrium_density_clamped() {
        /* equilibrium density is clamped */
        assert!((bone_equilibrium_density(0.0) - 0.1).abs() < 1e-4);
        assert!((bone_equilibrium_density(10.0) - 2.5).abs() < 1e-4);
    }

    #[test]
    fn test_step_toward_target() {
        /* density approaches target over time */
        let mut b = new_bone_element(0.5);
        bone_step(&mut b, 1.5, 10.0);
        assert!(b.density > 0.5);
    }

    #[test]
    fn test_osteoporotic() {
        /* density below 0.5 is osteoporotic */
        let b = new_bone_element(0.3);
        assert!(bone_is_osteoporotic(&b));
    }

    #[test]
    fn test_not_osteoporotic() {
        /* density above 0.5 is not osteoporotic */
        let b = new_bone_element(0.8);
        assert!(!bone_is_osteoporotic(&b));
    }

    #[test]
    fn test_dense() {
        /* density above 1.5 is dense */
        let b = new_bone_element(2.0);
        assert!(bone_is_dense(&b));
    }

    #[test]
    fn test_mineral_content() {
        /* mineral content = density * volume */
        let b = new_bone_element(1.2);
        assert!((bone_mineral_content(&b, 5.0) - 6.0).abs() < 1e-5);
    }
}
