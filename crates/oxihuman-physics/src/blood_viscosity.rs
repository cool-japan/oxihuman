// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Blood rheology using the Casson model.
pub struct Blood {
    pub hematocrit: f32,
    pub plasma_viscosity: f32,
    pub yield_stress: f32,
}

impl Blood {
    pub fn new(hematocrit: f32) -> Self {
        // yield stress scales roughly with hematocrit
        let yield_stress = 0.005 * hematocrit;
        Blood {
            hematocrit,
            plasma_viscosity: 0.0012,
            yield_stress,
        }
    }
}

pub fn new_blood(hematocrit: f32) -> Blood {
    Blood::new(hematocrit)
}

pub fn blood_yield_stress(b: &Blood) -> f32 {
    b.yield_stress
}

/// Casson model: sqrt(τ) = sqrt(τ_y) + sqrt(η_p * γ)
/// => τ = (sqrt(τ_y) + sqrt(η_p * γ))²  for γ > 0
/// => apparent viscosity = τ / γ
pub fn blood_viscosity_casson(b: &Blood, shear_rate: f32) -> f32 {
    if shear_rate <= 0.0 {
        return f32::INFINITY;
    }
    let sqrt_ty = b.yield_stress.sqrt();
    let sqrt_etap_gamma = (b.plasma_viscosity * shear_rate).sqrt();
    let tau = (sqrt_ty + sqrt_etap_gamma).powi(2);
    tau / shear_rate
}

pub fn blood_apparent_viscosity(b: &Blood, shear_rate: f32) -> f32 {
    blood_viscosity_casson(b, shear_rate)
}

/// Blood flows if driving stress exceeds yield stress.
pub fn blood_is_flowing(b: &Blood, shear_rate: f32, driving_stress: f32) -> bool {
    shear_rate > 0.0 && driving_stress > b.yield_stress
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new blood has positive yield stress */
        let b = new_blood(0.45);
        assert!(b.yield_stress > 0.0);
    }

    #[test]
    fn test_viscosity_positive() {
        /* viscosity is positive at positive shear rate */
        let b = new_blood(0.45);
        assert!(blood_viscosity_casson(&b, 10.0) > 0.0);
    }

    #[test]
    fn test_viscosity_decreases_with_rate() {
        /* viscosity decreases as shear rate increases (shear thinning) */
        let b = new_blood(0.45);
        let v_low = blood_viscosity_casson(&b, 1.0);
        let v_high = blood_viscosity_casson(&b, 100.0);
        assert!(v_high < v_low);
    }

    #[test]
    fn test_yield_stress() {
        /* yield_stress matches what was set */
        let b = new_blood(0.45);
        assert!((blood_yield_stress(&b) - b.yield_stress).abs() < 1e-9);
    }

    #[test]
    fn test_apparent_viscosity_matches() {
        /* apparent viscosity is same as Casson viscosity */
        let b = new_blood(0.45);
        let v1 = blood_viscosity_casson(&b, 50.0);
        let v2 = blood_apparent_viscosity(&b, 50.0);
        assert!((v1 - v2).abs() < 1e-9);
    }

    #[test]
    fn test_is_flowing_true() {
        /* blood flows when stress exceeds yield */
        let b = new_blood(0.45);
        assert!(blood_is_flowing(&b, 1.0, b.yield_stress + 0.01));
    }

    #[test]
    fn test_is_flowing_false() {
        /* no flow below yield stress */
        let b = new_blood(0.45);
        assert!(!blood_is_flowing(&b, 0.0, 0.001));
    }
}
