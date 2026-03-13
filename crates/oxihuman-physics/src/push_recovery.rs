// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Push-recovery reflex controller stub.

/// Severity of a push event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushSeverity {
    None,
    Minor,
    Moderate,
    Severe,
}

/// Push recovery response.
#[derive(Debug, Clone)]
pub struct RecoveryResponse {
    pub severity: PushSeverity,
    pub step_x: f32,
    pub step_y: f32,
    pub ankle_torque: [f32; 2],
    pub requires_step: bool,
}

/// Push recovery controller configuration.
#[derive(Debug, Clone)]
pub struct PushRecoveryConfig {
    pub minor_threshold: f32,
    pub moderate_threshold: f32,
    pub severe_threshold: f32,
    pub ankle_gain: f32,
}

impl Default for PushRecoveryConfig {
    fn default() -> Self {
        Self {
            minor_threshold: 0.05,
            moderate_threshold: 0.15,
            severe_threshold: 0.30,
            ankle_gain: 80.0,
        }
    }
}

/// Classify the severity of a disturbance from capture-point error.
pub fn classify_push(cp_error: f32, cfg: &PushRecoveryConfig) -> PushSeverity {
    let e = cp_error.abs();
    if e < cfg.minor_threshold {
        PushSeverity::None
    } else if e < cfg.moderate_threshold {
        PushSeverity::Minor
    } else if e < cfg.severe_threshold {
        PushSeverity::Moderate
    } else {
        PushSeverity::Severe
    }
}

/// Compute the recovery response for a given CoM and capture-point error.
#[allow(clippy::too_many_arguments)]
pub fn compute_recovery(
    cp_error_x: f32,
    cp_error_y: f32,
    com_vel_x: f32,
    com_vel_y: f32,
    cfg: &PushRecoveryConfig,
) -> RecoveryResponse {
    let cp_error_mag = (cp_error_x * cp_error_x + cp_error_y * cp_error_y).sqrt();
    let severity = classify_push(cp_error_mag, cfg);
    let requires_step = matches!(severity, PushSeverity::Moderate | PushSeverity::Severe);
    let ankle_torque = [
        -cfg.ankle_gain * cp_error_x - 0.5 * com_vel_x,
        -cfg.ankle_gain * cp_error_y - 0.5 * com_vel_y,
    ];
    RecoveryResponse {
        severity,
        step_x: cp_error_x * 0.5,
        step_y: cp_error_y * 0.5,
        ankle_torque,
        requires_step,
    }
}

/// Return true if the disturbance requires an emergency step.
pub fn emergency_step_required(severity: PushSeverity) -> bool {
    matches!(severity, PushSeverity::Severe)
}

/// Return the recommended step reach for the given severity.
pub fn recommended_step_reach(severity: PushSeverity) -> f32 {
    match severity {
        PushSeverity::None => 0.0,
        PushSeverity::Minor => 0.05,
        PushSeverity::Moderate => 0.15,
        PushSeverity::Severe => 0.30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_none() {
        /* tiny error → None */
        let cfg = PushRecoveryConfig::default();
        assert_eq!(classify_push(0.01, &cfg), PushSeverity::None);
    }

    #[test]
    fn test_classify_minor() {
        /* small error → Minor */
        let cfg = PushRecoveryConfig::default();
        assert_eq!(classify_push(0.10, &cfg), PushSeverity::Minor);
    }

    #[test]
    fn test_classify_moderate() {
        /* medium error → Moderate */
        let cfg = PushRecoveryConfig::default();
        assert_eq!(classify_push(0.20, &cfg), PushSeverity::Moderate);
    }

    #[test]
    fn test_classify_severe() {
        /* large error → Severe */
        let cfg = PushRecoveryConfig::default();
        assert_eq!(classify_push(0.50, &cfg), PushSeverity::Severe);
    }

    #[test]
    fn test_recovery_no_step_for_none() {
        /* no step required for zero error */
        let cfg = PushRecoveryConfig::default();
        let r = compute_recovery(0.0, 0.0, 0.0, 0.0, &cfg);
        assert!(!r.requires_step);
    }

    #[test]
    fn test_recovery_step_for_severe() {
        /* step required for large error */
        let cfg = PushRecoveryConfig::default();
        let r = compute_recovery(1.0, 0.0, 0.0, 0.0, &cfg);
        assert!(r.requires_step);
    }

    #[test]
    fn test_emergency_step() {
        /* Severe → emergency step */
        assert!(emergency_step_required(PushSeverity::Severe));
        assert!(!emergency_step_required(PushSeverity::Minor));
    }

    #[test]
    fn test_recommended_reach_zero_for_none() {
        /* no disturbance → zero reach */
        assert_eq!(recommended_step_reach(PushSeverity::None), 0.0);
    }

    #[test]
    fn test_ankle_torque_direction() {
        /* positive error → negative ankle torque */
        let cfg = PushRecoveryConfig::default();
        let r = compute_recovery(0.5, 0.0, 0.0, 0.0, &cfg);
        assert!(r.ankle_torque[0] < 0.0);
    }
}
