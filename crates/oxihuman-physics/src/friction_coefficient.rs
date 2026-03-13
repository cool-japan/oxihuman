// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Type of friction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrictionType {
    Static,
    Kinetic,
    Rolling,
}

/// Friction model with static, kinetic, and rolling coefficients.
#[derive(Debug, Clone)]
pub struct FrictionModel {
    pub static_mu: f32,
    pub kinetic_mu: f32,
    pub rolling_mu: f32,
}

/// Create a new FrictionModel.
pub fn new_friction_model(s: f32, k: f32, r: f32) -> FrictionModel {
    FrictionModel {
        static_mu: s,
        kinetic_mu: k,
        rolling_mu: r,
    }
}

/// Compute friction force given type and normal force.
pub fn friction_force(m: &FrictionModel, ft: FrictionType, normal: f32) -> f32 {
    friction_coefficient_for(m, ft) * normal.abs()
}

/// Check whether applied tangential force exceeds static friction threshold.
pub fn is_sliding(m: &FrictionModel, applied: f32, normal: f32) -> bool {
    applied.abs() > m.static_mu * normal.abs()
}

/// Return the friction coefficient for the given friction type.
pub fn friction_coefficient_for(m: &FrictionModel, ft: FrictionType) -> f32 {
    match ft {
        FrictionType::Static => m.static_mu,
        FrictionType::Kinetic => m.kinetic_mu,
        FrictionType::Rolling => m.rolling_mu,
    }
}

/// Effective friction coefficient using Coulomb model (kinetic if sliding).
pub fn effective_friction(m: &FrictionModel, applied: f32, normal: f32) -> f32 {
    if is_sliding(m, applied, normal) {
        m.kinetic_mu
    } else {
        m.static_mu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_friction_model() {
        /* constructor */
        let m = new_friction_model(0.6, 0.4, 0.02);
        assert!((m.static_mu - 0.6).abs() < 1e-9);
        assert!((m.kinetic_mu - 0.4).abs() < 1e-9);
        assert!((m.rolling_mu - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_friction_force_static() {
        let m = new_friction_model(0.5, 0.3, 0.01);
        let f = friction_force(&m, FrictionType::Static, 10.0);
        assert!((f - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_friction_force_kinetic() {
        let m = new_friction_model(0.5, 0.3, 0.01);
        let f = friction_force(&m, FrictionType::Kinetic, 10.0);
        assert!((f - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_friction_force_rolling() {
        let m = new_friction_model(0.5, 0.3, 0.01);
        let f = friction_force(&m, FrictionType::Rolling, 100.0);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_sliding_true() {
        let m = new_friction_model(0.5, 0.3, 0.01);
        /* 6 > 0.5*10 = 5 */
        assert!(is_sliding(&m, 6.0, 10.0));
    }

    #[test]
    fn test_is_sliding_false() {
        let m = new_friction_model(0.5, 0.3, 0.01);
        /* 4 < 0.5*10 = 5 */
        assert!(!is_sliding(&m, 4.0, 10.0));
    }

    #[test]
    fn test_friction_coefficient_for_static() {
        let m = new_friction_model(0.7, 0.4, 0.02);
        assert!((friction_coefficient_for(&m, FrictionType::Static) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_friction_coefficient_for_rolling() {
        let m = new_friction_model(0.7, 0.4, 0.02);
        assert!((friction_coefficient_for(&m, FrictionType::Rolling) - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_effective_friction_sliding() {
        /* when sliding, returns kinetic mu */
        let m = new_friction_model(0.5, 0.3, 0.01);
        let mu = effective_friction(&m, 10.0, 10.0);
        assert!((mu - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_effective_friction_static() {
        /* when not sliding, returns static mu */
        let m = new_friction_model(0.5, 0.3, 0.01);
        let mu = effective_friction(&m, 1.0, 10.0);
        assert!((mu - 0.5).abs() < 1e-9);
    }
}
