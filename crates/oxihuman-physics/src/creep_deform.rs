// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Creep deformation models for materials under sustained stress.

/// Norton power-law creep rate: epsilon_dot = A * sigma^n.
pub fn norton_creep_rate(a: f32, sigma: f32, n: f32) -> f32 {
    a * sigma.abs().powf(n)
}

/// Accumulated creep strain after time dt: d_eps = rate * dt.
pub fn creep_strain_increment(rate: f32, dt: f32) -> f32 {
    rate * dt
}

/// Larson-Miller parameter: LMP = T * (C + log10(t_r)), T in Kelvin.
pub fn larson_miller_parameter(temp_k: f32, rupture_time_h: f32, c: f32) -> f32 {
    temp_k * (c + rupture_time_h.max(1e-9).log10())
}

/// Monkman-Grant relation: t_r = M / epsilon_dot_min.
pub fn monkman_grant_rupture_time(m: f32, min_creep_rate: f32) -> f32 {
    if min_creep_rate < 1e-30 {
        return f32::MAX;
    }
    m / min_creep_rate
}

/// Creep compliance J(t) = 1/E + B * t^n (simple power-law viscoelastic).
pub fn creep_compliance(e: f32, b: f32, n: f32, t: f32) -> f32 {
    if e < 1e-12 {
        return 0.0;
    }
    1.0 / e + b * t.powf(n)
}

/// Creep strain: eps_c = J_c(t) * sigma.
pub fn creep_total_strain(e: f32, b: f32, n: f32, t: f32, sigma: f32) -> f32 {
    creep_compliance(e, b, n, t) * sigma
}

/// Remaining life fraction under creep: 1 - eps_accumulated / eps_rupture.
pub fn creep_remaining_life(eps_accumulated: f32, eps_rupture: f32) -> f32 {
    if eps_rupture < 1e-12 {
        return 0.0;
    }
    (1.0 - eps_accumulated / eps_rupture).clamp(0.0, 1.0)
}

/// Secondary creep activation energy term: exp(-Q / R*T).
pub fn creep_temperature_factor(q_over_r: f32, temp_k: f32) -> f32 {
    if temp_k < 1e-6 {
        return 0.0;
    }
    (-q_over_r / temp_k).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norton_creep_rate_positive() {
        /* rate > 0 for positive sigma */
        let r = norton_creep_rate(1e-10, 100.0, 3.0);
        assert!(r > 0.0);
    }

    #[test]
    fn test_creep_strain_increment() {
        /* eps_dot * dt */
        let inc = creep_strain_increment(0.001, 100.0);
        assert!((inc - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_larson_miller_positive() {
        /* LMP > 0 for valid inputs */
        let lmp = larson_miller_parameter(800.0, 1000.0, 20.0);
        assert!(lmp > 0.0);
    }

    #[test]
    fn test_monkman_grant() {
        /* t_r = M / rate */
        let t = monkman_grant_rupture_time(0.1, 1e-4);
        assert!((t - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_creep_compliance_elastic_limit() {
        /* at t=0 compliance = 1/E */
        let j = creep_compliance(200e9, 1e-15, 1.0, 0.0);
        assert!((j - 1.0 / 200e9).abs() < 1e-20);
    }

    #[test]
    fn test_creep_total_strain_zero_time() {
        /* at t=0 strain = sigma/E */
        let eps = creep_total_strain(200e9, 1e-15, 1.0, 0.0, 100e6);
        assert!((eps - 100e6 / 200e9).abs() < 1e-6);
    }

    #[test]
    fn test_creep_remaining_life_no_damage() {
        /* 0 accumulated -> life = 1 */
        let rl = creep_remaining_life(0.0, 0.05);
        assert!((rl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_creep_temperature_factor_range() {
        /* between 0 and 1 for positive Q/R */
        let f = creep_temperature_factor(10000.0, 1000.0);
        assert!(f > 0.0 && f <= 1.0);
    }
}
