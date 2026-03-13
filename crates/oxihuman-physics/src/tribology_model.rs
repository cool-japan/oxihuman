// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Friction/wear tribology model.

/// Friction regime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrictionRegime {
    Static,
    Kinetic,
    Rolling,
}

/// Tribology parameters.
#[derive(Debug, Clone)]
pub struct TribologyParams {
    pub static_friction: f32,
    pub kinetic_friction: f32,
    pub rolling_friction: f32,
    pub wear_coefficient: f32,
    pub hardness: f32,
}

impl TribologyParams {
    pub fn new(
        static_friction: f32,
        kinetic_friction: f32,
        rolling_friction: f32,
        wear_coefficient: f32,
        hardness: f32,
    ) -> Self {
        TribologyParams {
            static_friction,
            kinetic_friction,
            rolling_friction,
            wear_coefficient,
            hardness,
        }
    }

    pub fn steel_on_steel() -> Self {
        TribologyParams::new(0.78, 0.42, 0.001, 1e-4, 7e9)
    }

    pub fn rubber_on_concrete() -> Self {
        TribologyParams::new(0.85, 0.65, 0.02, 5e-3, 5e8)
    }
}

impl Default for TribologyParams {
    fn default() -> Self {
        TribologyParams::new(0.5, 0.35, 0.005, 1e-4, 1e9)
    }
}

/// Compute friction force given normal force.
pub fn friction_force(params: &TribologyParams, normal: f32, regime: FrictionRegime) -> f32 {
    let mu = match regime {
        FrictionRegime::Static => params.static_friction,
        FrictionRegime::Kinetic => params.kinetic_friction,
        FrictionRegime::Rolling => params.rolling_friction,
    };
    mu * normal.max(0.0)
}

/// Archard wear volume: V = k * N * s / H
pub fn archard_wear_volume(params: &TribologyParams, normal: f32, sliding_dist: f32) -> f32 {
    if params.hardness <= 0.0 {
        return 0.0;
    }
    params.wear_coefficient * normal.max(0.0) * sliding_dist.max(0.0) / params.hardness
}

/// Stribeck curve: returns effective friction coefficient for a given speed.
pub fn stribeck_friction(params: &TribologyParams, speed: f32, viscosity: f32, load: f32) -> f32 {
    if load <= 0.0 || viscosity <= 0.0 {
        return params.static_friction;
    }
    let hersey = viscosity * speed.max(0.0) / load;
    /* Simplified Stribeck: boundary → mixed → hydrodynamic */
    if hersey < 1e-7 {
        params.static_friction
    } else if hersey < 1e-5 {
        params.kinetic_friction
    } else {
        /* hydrodynamic: friction rises again */
        params.kinetic_friction * (1.0 + hersey * 1e5 * 0.1).min(params.static_friction)
    }
}

/// Energy dissipated by friction over sliding distance.
pub fn friction_energy(params: &TribologyParams, normal: f32, sliding_dist: f32) -> f32 {
    friction_force(params, normal, FrictionRegime::Kinetic) * sliding_dist.max(0.0)
}

/// Return true if motion onset: applied force exceeds static friction.
pub fn is_sliding_onset(params: &TribologyParams, applied: f32, normal: f32) -> bool {
    applied.abs() > params.static_friction * normal.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friction_force_static() {
        let p = TribologyParams::default();
        let f = friction_force(&p, 10.0, FrictionRegime::Static);
        assert!((f - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_friction_force_kinetic_less_than_static() {
        let p = TribologyParams::default();
        let fs = friction_force(&p, 10.0, FrictionRegime::Static);
        let fk = friction_force(&p, 10.0, FrictionRegime::Kinetic);
        assert!(fk < fs);
    }

    #[test]
    fn test_friction_force_zero_normal() {
        let p = TribologyParams::default();
        assert_eq!(friction_force(&p, 0.0, FrictionRegime::Kinetic), 0.0);
    }

    #[test]
    fn test_archard_wear_volume_positive() {
        let p = TribologyParams::default();
        let v = archard_wear_volume(&p, 100.0, 1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_archard_wear_zero_distance() {
        let p = TribologyParams::default();
        assert_eq!(archard_wear_volume(&p, 100.0, 0.0), 0.0);
    }

    #[test]
    fn test_friction_energy_positive() {
        let p = TribologyParams::default();
        assert!(friction_energy(&p, 10.0, 2.0) > 0.0);
    }

    #[test]
    fn test_is_sliding_onset_true() {
        let p = TribologyParams::default();
        assert!(is_sliding_onset(&p, 100.0, 10.0));
    }

    #[test]
    fn test_is_sliding_onset_false() {
        let p = TribologyParams::default();
        assert!(!is_sliding_onset(&p, 1.0, 100.0));
    }

    #[test]
    fn test_steel_on_steel() {
        let p = TribologyParams::steel_on_steel();
        assert!(p.kinetic_friction < p.static_friction);
    }

    #[test]
    fn test_stribeck_low_speed() {
        let p = TribologyParams::default();
        let mu = stribeck_friction(&p, 0.0, 0.01, 100.0);
        assert!((mu - p.static_friction).abs() < 0.001);
    }
}
