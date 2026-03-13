// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Ergonomics model (simplified RULA-like scoring).
#[derive(Debug, Clone)]
pub struct ErgonomicsModel {
    pub reach_distance: f32,
    pub force_required: f32,
    pub posture_score: f32,
    pub frequency: f32,
}

/// Create a new ErgonomicsModel.
pub fn new_ergonomics_model(reach: f32, force: f32, posture: f32, freq: f32) -> ErgonomicsModel {
    ErgonomicsModel {
        reach_distance: reach,
        force_required: force,
        posture_score: posture,
        frequency: freq,
    }
}

/// Simplified RULA-like score: weighted sum of risk factors.
pub fn rula_score(m: &ErgonomicsModel) -> f32 {
    let reach_factor = (m.reach_distance / 0.5).min(3.0);
    let force_factor = (m.force_required / 10.0).min(3.0);
    let posture_factor = m.posture_score.clamp(1.0, 7.0);
    let freq_factor = (m.frequency / 2.0).min(2.0);
    reach_factor + force_factor + posture_factor + freq_factor
}

/// Returns true when the RULA score exceeds 5.0 (high ergonomic risk).
pub fn is_high_risk(m: &ErgonomicsModel) -> bool {
    rula_score(m) > 5.0
}

/// Musculoskeletal disorder risk index (0..1 scale).
pub fn musculoskeletal_risk(m: &ErgonomicsModel) -> f32 {
    let score = rula_score(m);
    (score / 10.0).clamp(0.0, 1.0)
}

/// Force demand ratio relative to maximum voluntary contraction (assumed 150 N).
pub fn force_demand_ratio(m: &ErgonomicsModel) -> f32 {
    let mvc = 150.0_f32;
    (m.force_required / mvc).clamp(0.0, 1.0)
}

/// Reach strain (ratio of reach to safe limit of 0.45 m).
pub fn reach_strain(m: &ErgonomicsModel) -> f32 {
    (m.reach_distance / 0.45).clamp(0.0, 2.0)
}

/// Duty cycle factor (fraction of time under load): frequency / 60 (60 ops/min max).
pub fn duty_cycle(m: &ErgonomicsModel) -> f32 {
    (m.frequency / 60.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ergonomics_model() {
        /* constructor */
        let m = new_ergonomics_model(0.4, 20.0, 3.0, 5.0);
        assert!((m.reach_distance - 0.4).abs() < 1e-6);
        assert!((m.force_required - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_rula_score_positive() {
        let m = new_ergonomics_model(0.3, 10.0, 3.0, 2.0);
        assert!(rula_score(&m) > 0.0);
    }

    #[test]
    fn test_rula_score_high_risk() {
        /* high reach, high force, high posture */
        let m = new_ergonomics_model(1.0, 50.0, 7.0, 10.0);
        assert!(rula_score(&m) > 5.0);
    }

    #[test]
    fn test_is_high_risk_true() {
        let m = new_ergonomics_model(1.0, 50.0, 7.0, 10.0);
        assert!(is_high_risk(&m));
    }

    #[test]
    fn test_is_high_risk_false() {
        let m = new_ergonomics_model(0.1, 2.0, 1.0, 0.5);
        assert!(!is_high_risk(&m));
    }

    #[test]
    fn test_musculoskeletal_risk_range() {
        let m = new_ergonomics_model(0.3, 10.0, 3.0, 2.0);
        let risk = musculoskeletal_risk(&m);
        assert!((0.0..=1.0).contains(&risk));
    }

    #[test]
    fn test_force_demand_ratio_clamp() {
        /* very high force -> ratio clamped at 1.0 */
        let m = new_ergonomics_model(0.3, 1000.0, 3.0, 2.0);
        let r = force_demand_ratio(&m);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reach_strain_zero() {
        let m = new_ergonomics_model(0.0, 5.0, 2.0, 1.0);
        assert!(reach_strain(&m).abs() < 1e-6);
    }

    #[test]
    fn test_duty_cycle_range() {
        let m = new_ergonomics_model(0.3, 10.0, 3.0, 30.0);
        let d = duty_cycle(&m);
        assert!((0.0..=1.0).contains(&d));
    }
}
