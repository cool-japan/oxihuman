// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bone under load deformation stub (elastic beam model).

/// A simplified bone segment (Euler-Bernoulli beam).
#[derive(Debug, Clone)]
pub struct Bone {
    /// Length (m).
    pub length: f32,
    /// Elastic modulus (Pa) — cortical bone ~17 GPa.
    pub elastic_modulus: f32,
    /// Second moment of area (m^4).
    pub moment_of_area: f32,
    /// Cross-sectional area (m²).
    pub area: f32,
    /// Applied axial force (N, positive = compression).
    pub axial_force: f32,
    /// Applied bending moment (N·m).
    pub bending_moment: f32,
    /// Fracture threshold stress (Pa).
    pub fracture_stress: f32,
}

impl Bone {
    pub fn new(length: f32, elastic_modulus: f32, radius: f32) -> Self {
        let area = std::f32::consts::PI * radius * radius;
        let moment_of_area = std::f32::consts::PI * radius.powi(4) / 4.0;
        Bone {
            length,
            elastic_modulus,
            moment_of_area,
            area,
            axial_force: 0.0,
            bending_moment: 0.0,
            fracture_stress: 170_000_000.0, /* 170 MPa */
        }
    }
}

/// Create a new cylindrical bone segment.
pub fn new_bone(length: f32, elastic_modulus: f32, radius: f32) -> Bone {
    Bone::new(length, elastic_modulus, radius)
}

/// Axial stress (Pa). Positive = compressive.
pub fn bone_axial_stress(b: &Bone) -> f32 {
    b.axial_force / b.area.max(1e-12)
}

/// Maximum bending stress at surface (Pa).
pub fn bone_bending_stress(b: &Bone, outer_radius: f32) -> f32 {
    b.bending_moment * outer_radius / b.moment_of_area.max(1e-20)
}

/// Axial deformation (shortening) under load (m).
pub fn bone_axial_deformation(b: &Bone) -> f32 {
    b.axial_force * b.length / (b.elastic_modulus * b.area).max(1e-20)
}

/// Mid-span deflection under a central point load (4-point bending stub).
pub fn bone_bending_deflection(b: &Bone) -> f32 {
    /* δ = ML² / (8EI) for uniform moment */
    b.bending_moment * b.length * b.length / (8.0 * b.elastic_modulus * b.moment_of_area).max(1e-20)
}

/// Return `true` if combined stress exceeds fracture threshold.
pub fn bone_is_fractured(b: &Bone, outer_radius: f32) -> bool {
    let sigma = bone_axial_stress(b) + bone_bending_stress(b, outer_radius);
    sigma.abs() > b.fracture_stress
}

/// Set the axial load (N).
pub fn bone_set_axial(b: &mut Bone, force: f32) {
    b.axial_force = force;
}

/// Set the bending moment (N·m).
pub fn bone_set_bending(b: &mut Bone, moment: f32) {
    b.bending_moment = moment;
}

/// Return the stiffness in axial compression (N/m).
pub fn bone_axial_stiffness(b: &Bone) -> f32 {
    b.elastic_modulus * b.area / b.length.max(1e-10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bone_zero_load() {
        let b = new_bone(0.3, 17e9, 0.01);
        assert_eq!(b.axial_force, 0.0);
        assert_eq!(b.bending_moment, 0.0);
    }

    #[test]
    fn test_axial_stress() {
        let mut b = new_bone(0.3, 17e9, 0.01);
        bone_set_axial(&mut b, 1000.0);
        let s = bone_axial_stress(&b);
        assert!(s > 0.0);
    }

    #[test]
    fn test_bending_stress_zero_when_no_moment() {
        let b = new_bone(0.3, 17e9, 0.01);
        assert!(bone_bending_stress(&b, 0.01).abs() < 1e-5);
    }

    #[test]
    fn test_axial_deformation_positive_under_load() {
        let mut b = new_bone(0.3, 17e9, 0.01);
        bone_set_axial(&mut b, 5000.0);
        assert!(bone_axial_deformation(&b) > 0.0);
    }

    #[test]
    fn test_not_fractured_under_normal_load() {
        let mut b = new_bone(0.3, 17e9, 0.01);
        bone_set_axial(&mut b, 1000.0);
        assert!(!bone_is_fractured(&b, 0.01));
    }

    #[test]
    fn test_fractured_under_extreme_load() {
        let mut b = new_bone(0.3, 17e9, 0.001); /* very thin */
        bone_set_axial(&mut b, 1e8);
        assert!(bone_is_fractured(&b, 0.001));
    }

    #[test]
    fn test_axial_stiffness_positive() {
        let b = new_bone(0.3, 17e9, 0.01);
        assert!(bone_axial_stiffness(&b) > 0.0);
    }

    #[test]
    fn test_bending_deflection_under_moment() {
        let mut b = new_bone(0.3, 17e9, 0.01);
        bone_set_bending(&mut b, 100.0);
        assert!(bone_bending_deflection(&b) > 0.0);
    }

    #[test]
    fn test_area_computed_correctly() {
        let b = new_bone(0.3, 17e9, 0.01);
        let expected = std::f32::consts::PI * 0.01 * 0.01;
        assert!((b.area - expected).abs() < 1e-8);
    }
}
