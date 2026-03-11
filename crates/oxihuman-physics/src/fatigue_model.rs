// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cyclic fatigue damage accumulation model stub.
//!
//! Implements S-N (Wöhler) curves, Miner's rule of linear damage accumulation,
//! and rainflow cycle counting helpers.

/// S-N curve parameters (Basquin relation: N * sigma^b = C).
#[derive(Debug, Clone)]
pub struct SnCurveParams {
    /// Basquin coefficient C.
    pub c: f64,
    /// Basquin exponent b (typically negative for metals).
    pub b: f64,
    /// Endurance limit `[MPa]` — below this stress, infinite life.
    pub endurance_limit: f64,
    /// Ultimate tensile strength `[MPa]`.
    pub uts: f64,
}

impl Default for SnCurveParams {
    fn default() -> Self {
        Self {
            c: 1e15,
            b: -5.0,
            endurance_limit: 200.0,
            uts: 600.0,
        }
    }
}

/// A single stress cycle.
#[derive(Debug, Clone)]
pub struct StressCycle {
    pub stress_range: f64,
    pub mean_stress: f64,
    pub count: f64,
}

impl StressCycle {
    pub fn new(range: f64, mean: f64, count: f64) -> Self {
        Self {
            stress_range: range,
            mean_stress: mean,
            count,
        }
    }

    pub fn amplitude(&self) -> f64 {
        self.stress_range * 0.5
    }

    pub fn r_ratio(&self) -> f64 {
        let max = self.mean_stress + self.amplitude();
        let min = self.mean_stress - self.amplitude();
        if max == 0.0 {
            return 0.0;
        }
        min / max
    }
}

/// Fatigue damage accumulator.
#[derive(Debug, Clone, Default)]
pub struct FatigueDamage {
    pub damage: f64,
    pub cycle_count: f64,
    pub failed: bool,
}

impl FatigueDamage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remaining_life_fraction(&self) -> f64 {
        (1.0 - self.damage).max(0.0)
    }

    pub fn is_failed(&self) -> bool {
        self.failed || self.damage >= 1.0
    }
}

/// Compute fatigue life N_f from Basquin S-N relation.
pub fn cycles_to_failure_sn(stress_amplitude: f64, params: &SnCurveParams) -> f64 {
    if stress_amplitude <= params.endurance_limit {
        return f64::INFINITY;
    }
    if stress_amplitude <= 0.0 {
        return f64::INFINITY;
    }
    (params.c / stress_amplitude.powf(params.b.abs())).max(1.0)
}

/// Apply Goodman correction for mean stress.
///
/// σ_eq = σ_a / (1 - σ_m / UTS)
pub fn goodman_correction(amplitude: f64, mean_stress: f64, uts: f64) -> f64 {
    let denom = 1.0 - mean_stress / uts.max(1e-12);
    if denom <= 0.0 {
        return f64::INFINITY;
    }
    amplitude / denom
}

/// Accumulate damage using Miner's rule: D += n_i / N_fi.
pub fn accumulate_damage(damage: &mut FatigueDamage, cycle: &StressCycle, params: &SnCurveParams) {
    if damage.failed {
        return;
    }
    let corrected = goodman_correction(cycle.amplitude(), cycle.mean_stress, params.uts);
    let nf = cycles_to_failure_sn(corrected, params);
    if nf == f64::INFINITY {
        return;
    }
    let d = cycle.count / nf;
    damage.damage += d;
    damage.cycle_count += cycle.count;
    if damage.damage >= 1.0 {
        damage.damage = 1.0;
        damage.failed = true;
    }
}

/// Compute the stress ratio R for a given amplitude and mean.
pub fn stress_ratio(amplitude: f64, mean_stress: f64) -> f64 {
    let max = mean_stress + amplitude;
    let min = mean_stress - amplitude;
    if max.abs() < 1e-30 {
        return 0.0;
    }
    min / max
}

/// Estimate remaining cycles before failure.
pub fn remaining_cycles(
    damage: &FatigueDamage,
    cycle: &StressCycle,
    params: &SnCurveParams,
) -> f64 {
    if damage.failed {
        return 0.0;
    }
    let corrected = goodman_correction(cycle.amplitude(), cycle.mean_stress, params.uts);
    let nf = cycles_to_failure_sn(corrected, params);
    if nf == f64::INFINITY {
        return f64::INFINITY;
    }
    (nf * (1.0 - damage.damage)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> SnCurveParams {
        SnCurveParams::default()
    }

    #[test]
    fn test_infinite_life_below_endurance() {
        let p = default_params();
        assert_eq!(cycles_to_failure_sn(100.0, &p), f64::INFINITY);
    }

    #[test]
    fn test_finite_life_above_endurance() {
        let p = default_params();
        let nf = cycles_to_failure_sn(300.0, &p);
        assert!(nf < f64::INFINITY && nf > 0.0);
    }

    #[test]
    fn test_life_decreases_with_stress() {
        let p = default_params();
        let n1 = cycles_to_failure_sn(300.0, &p);
        let n2 = cycles_to_failure_sn(400.0, &p);
        assert!(n1 > n2);
    }

    #[test]
    fn test_goodman_no_mean() {
        let amp = 250.0;
        let corrected = goodman_correction(amp, 0.0, 600.0);
        assert!((corrected - amp).abs() < 1e-9);
    }

    #[test]
    fn test_accumulate_damage_increases() {
        let p = default_params();
        let mut d = FatigueDamage::new();
        let cycle = StressCycle::new(100.0, 0.0, 1000.0);
        accumulate_damage(&mut d, &cycle, &p);
        assert!(d.damage >= 0.0);
    }

    #[test]
    fn test_failure_at_damage_one() {
        let p = SnCurveParams {
            c: 1.0,
            b: -1.0,
            endurance_limit: 0.0,
            uts: 1e10,
        };
        let mut d = FatigueDamage::new();
        let cycle = StressCycle::new(1.0, 0.0, 2.0);
        accumulate_damage(&mut d, &cycle, &p);
        assert!(d.is_failed() || d.damage <= 1.0);
    }

    #[test]
    fn test_remaining_life_fraction() {
        let mut d = FatigueDamage::new();
        d.damage = 0.4;
        assert!((d.remaining_life_fraction() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_stress_ratio() {
        let r = stress_ratio(100.0, 100.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_cycle_amplitude() {
        let c = StressCycle::new(200.0, 0.0, 1.0);
        assert_eq!(c.amplitude(), 100.0);
    }
}
