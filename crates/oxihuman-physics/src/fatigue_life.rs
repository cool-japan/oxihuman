// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Fatigue life model using modified Goodman criterion.
#[derive(Debug, Clone)]
pub struct FatigueModel {
    pub ultimate_strength: f32,
    pub endurance_limit: f32,
    pub stress_concentration: f32,
}

/// Create a new FatigueModel.
pub fn new_fatigue_model(su: f32, se: f32, kt: f32) -> FatigueModel {
    FatigueModel {
        ultimate_strength: su,
        endurance_limit: se,
        stress_concentration: kt,
    }
}

/// Estimate cycles to failure using Basquin's power law: N = (Se / (Kt*Sa))^b, b≈10.
pub fn cycles_to_failure(m: &FatigueModel, stress_amp: f32) -> f32 {
    let effective_stress = m.stress_concentration * stress_amp;
    if effective_stress < 1e-9 {
        return f32::INFINITY;
    }
    (m.endurance_limit / effective_stress).powf(10.0)
}

/// Goodman ratio: (Sa/Se) + (Sm/Su). Values < 1.0 indicate safe design.
pub fn goodman_ratio(m: &FatigueModel, stress_amp: f32, mean_stress: f32) -> f32 {
    let se_eff = m.endurance_limit / m.stress_concentration.max(1e-9);
    if se_eff < 1e-9 || m.ultimate_strength < 1e-9 {
        return f32::INFINITY;
    }
    stress_amp / se_eff + mean_stress / m.ultimate_strength
}

/// Returns true if the Goodman ratio is below 1.0 (safe design).
pub fn is_safe(m: &FatigueModel, stress: f32, mean_stress: f32) -> bool {
    goodman_ratio(m, stress, mean_stress) < 1.0
}

/// Modified endurance limit accounting for stress concentration.
pub fn effective_endurance_limit(m: &FatigueModel) -> f32 {
    m.endurance_limit / m.stress_concentration.max(1.0)
}

/// Fatigue strength at given cycle count (Basquin, inverse): Sa = Se / (N^(1/b)), b=10.
pub fn fatigue_strength_at_cycles(m: &FatigueModel, cycles: f32) -> f32 {
    if cycles < 1.0 {
        return m.endurance_limit;
    }
    m.endurance_limit / cycles.powf(1.0 / 10.0)
}

/// Cumulative damage ratio (Miner's rule) for a single stress amplitude.
pub fn miners_damage(m: &FatigueModel, stress_amp: f32, applied_cycles: f32) -> f32 {
    let n_f = cycles_to_failure(m, stress_amp);
    if n_f < 1e-9 {
        return f32::INFINITY;
    }
    applied_cycles / n_f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fatigue_model() {
        /* constructor */
        let m = new_fatigue_model(600.0, 300.0, 2.0);
        assert!((m.ultimate_strength - 600.0).abs() < 1e-6);
        assert!((m.endurance_limit - 300.0).abs() < 1e-6);
        assert!((m.stress_concentration - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_cycles_to_failure_infinite_below_endurance() {
        /* stress = 0 -> infinite life */
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        let n = cycles_to_failure(&m, 0.0);
        assert!(n.is_infinite());
    }

    #[test]
    fn test_cycles_to_failure_at_endurance() {
        /* at Se -> N = 1 cycle (by Basquin with b=10) */
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        let n = cycles_to_failure(&m, 300.0);
        assert!((n - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_goodman_ratio_safe() {
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        let r = goodman_ratio(&m, 100.0, 100.0);
        assert!(r < 1.0);
    }

    #[test]
    fn test_goodman_ratio_unsafe() {
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        let r = goodman_ratio(&m, 350.0, 400.0);
        assert!(r > 1.0);
    }

    #[test]
    fn test_is_safe_true() {
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        assert!(is_safe(&m, 50.0, 50.0));
    }

    #[test]
    fn test_is_safe_false() {
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        assert!(!is_safe(&m, 400.0, 500.0));
    }

    #[test]
    fn test_effective_endurance_limit() {
        let m = new_fatigue_model(600.0, 300.0, 2.0);
        let se = effective_endurance_limit(&m);
        assert!((se - 150.0).abs() < 1e-4);
    }

    #[test]
    fn test_miners_damage_ratio() {
        /* applied cycles = cycles to failure -> damage = 1 */
        let m = new_fatigue_model(600.0, 300.0, 1.0);
        let n_f = cycles_to_failure(&m, 150.0);
        let d = miners_damage(&m, 150.0, n_f);
        assert!((d - 1.0).abs() < 1e-4);
    }
}
