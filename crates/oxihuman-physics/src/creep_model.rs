// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Material creep (slow time-dependent deformation) model stub.
//!
//! Implements Norton's power-law creep: ε̇ = A * σ^n * exp(-Q/RT)
//! for secondary (steady-state) creep in metals and polymers.

/// Parameters for Norton power-law creep.
#[derive(Debug, Clone)]
pub struct CreepParams {
    /// Pre-exponential constant A [1/(MPa^n · s)].
    pub a: f64,
    /// Stress exponent n.
    pub n: f64,
    /// Activation energy Q [J/mol].
    pub q_activation: f64,
    /// Universal gas constant R [J/(mol·K)].
    pub r_gas: f64,
    /// Temperature `[K]`.
    pub temperature: f64,
}

impl Default for CreepParams {
    fn default() -> Self {
        Self {
            a: 1e-10,
            n: 3.5,
            q_activation: 140_000.0,
            r_gas: 8.314,
            temperature: 700.0,
        }
    }
}

/// State of a creeping material element.
#[derive(Debug, Clone, Default)]
pub struct CreepState {
    pub elastic_strain: f64,
    pub creep_strain: f64,
    pub total_strain: f64,
    pub time_elapsed: f64,
}

impl CreepState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn creep_fraction(&self) -> f64 {
        if self.total_strain == 0.0 {
            return 0.0;
        }
        self.creep_strain / self.total_strain
    }
}

/// Compute the Norton creep rate ε̇ for a given stress [1/s].
pub fn norton_creep_rate(stress: f64, params: &CreepParams) -> f64 {
    if stress <= 0.0 {
        return 0.0;
    }
    let thermal_factor = (-params.q_activation / (params.r_gas * params.temperature)).exp();
    params.a * stress.powf(params.n) * thermal_factor
}

/// Integrate creep over a time step `dt` `[s]` at constant stress.
pub fn integrate_creep(state: &mut CreepState, stress: f64, dt: f64, params: &CreepParams) {
    let rate = norton_creep_rate(stress, params);
    let d_creep = rate * dt;
    state.creep_strain += d_creep;
    state.total_strain = state.elastic_strain + state.creep_strain;
    state.time_elapsed += dt;
}

/// Compute the Larson-Miller parameter for creep life assessment.
///
/// LMP = T * (C + log10(t_r))
///
/// `t_r` is the rupture time `[h]`, `t_kelvin` is temperature `[K]`, `c` is a material constant.
pub fn larson_miller_param(t_kelvin: f64, rupture_time_h: f64, c: f64) -> f64 {
    t_kelvin * (c + rupture_time_h.max(1e-30).log10())
}

/// Estimate rupture life from Larson-Miller parameter.
pub fn rupture_time_h(lmp: f64, t_kelvin: f64, c: f64) -> f64 {
    if t_kelvin <= 0.0 {
        return 0.0;
    }
    let log_t = lmp / t_kelvin - c;
    10.0_f64.powf(log_t)
}

/// Compute the Monkman-Grant constant: ε̇_s * t_f = C_mg.
pub fn monkman_grant_check(creep_rate: f64, rupture_time: f64) -> f64 {
    creep_rate * rupture_time
}

/// Check whether creep damage exceeds a threshold.
pub fn is_creep_damaged(state: &CreepState, threshold: f64) -> bool {
    state.creep_strain >= threshold
}

/// Compute effective creep strain rate accounting for multiaxial stress.
pub fn multiaxial_creep_rate(
    sigma_eq: f64, /* von Mises equivalent stress */
    params: &CreepParams,
) -> f64 {
    norton_creep_rate(sigma_eq, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norton_rate_zero_at_zero_stress() {
        let p = CreepParams::default();
        assert_eq!(norton_creep_rate(0.0, &p), 0.0);
    }

    #[test]
    fn test_norton_rate_positive_at_stress() {
        let p = CreepParams::default();
        assert!(norton_creep_rate(100.0, &p) > 0.0);
    }

    #[test]
    fn test_creep_increases_with_stress() {
        let p = CreepParams::default();
        let r1 = norton_creep_rate(50.0, &p);
        let r2 = norton_creep_rate(100.0, &p);
        assert!(r2 > r1);
    }

    #[test]
    fn test_integrate_creep_increases_strain() {
        let p = CreepParams::default();
        let mut s = CreepState::new();
        integrate_creep(&mut s, 100.0, 3600.0, &p);
        assert!(s.creep_strain > 0.0);
    }

    #[test]
    fn test_larson_miller_positive() {
        let lmp = larson_miller_param(900.0, 1000.0, 20.0);
        assert!(lmp > 0.0);
    }

    #[test]
    fn test_rupture_time_roundtrip() {
        let t = 900.0;
        let c = 20.0;
        let tr = 1000.0;
        let lmp = larson_miller_param(t, tr, c);
        let back = rupture_time_h(lmp, t, c);
        assert!((back - tr).abs() < 1e-3);
    }

    #[test]
    fn test_monkman_grant() {
        let mg = monkman_grant_check(1e-8, 1e8);
        assert!((mg - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_is_creep_damaged() {
        let mut s = CreepState::new();
        s.creep_strain = 0.05;
        assert!(is_creep_damaged(&s, 0.04));
        assert!(!is_creep_damaged(&s, 0.06));
    }

    #[test]
    fn test_creep_fraction() {
        let mut s = CreepState::new();
        s.elastic_strain = 0.01;
        s.creep_strain = 0.01;
        s.total_strain = 0.02;
        assert!((s.creep_fraction() - 0.5).abs() < 1e-9);
    }
}

// ── Wave 151A simple f32 creep API ─────────────────────────────────────────

/// Universal gas constant (J/mol/K).
const R_GAS: f32 = 8.314;

/// Simple creep model with f32 parameters.
#[derive(Debug, Clone)]
pub struct SimpleCreepModel {
    pub creep_rate: f32,
    pub stress_exponent: f32,
    pub activation_energy: f32,
    pub temperature: f32,
}

/// Create a new SimpleCreepModel.
pub fn new_creep_model(rate: f32, n: f32, q: f32, temp: f32) -> SimpleCreepModel {
    SimpleCreepModel {
        creep_rate: rate,
        stress_exponent: n,
        activation_energy: q,
        temperature: temp,
    }
}

/// Steady-state creep rate: eps_dot = A * sigma^n * exp(-Q/(R*T)).
pub fn steady_state_creep_rate(m: &SimpleCreepModel, stress: f32) -> f32 {
    if m.temperature < 1e-6 {
        return 0.0;
    }
    let arrhenius = (-m.activation_energy / (R_GAS * m.temperature)).exp();
    m.creep_rate * stress.powf(m.stress_exponent) * arrhenius
}

/// Total creep strain at given time: eps = eps_dot * time.
pub fn strain_at_time(m: &SimpleCreepModel, stress: f32, time: f32) -> f32 {
    steady_state_creep_rate(m, stress) * time
}

/// Returns true when creep rate under given stress exceeds 1e-6 (significant).
pub fn creep_is_significant(m: &SimpleCreepModel, stress: f32) -> bool {
    steady_state_creep_rate(m, stress) > 1e-6
}
