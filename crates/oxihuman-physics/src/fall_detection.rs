// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fall detection trigger stub.

/// Fall detection state machine state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallState {
    Upright,
    Tilting,
    Falling,
    Fallen,
}

/// Fall detector configuration.
#[derive(Debug, Clone)]
pub struct FallDetectorConfig {
    pub tilt_angle_deg: f32,
    pub fall_angle_deg: f32,
    pub fallen_angle_deg: f32,
    pub angular_vel_threshold: f32,
}

impl Default for FallDetectorConfig {
    fn default() -> Self {
        Self {
            tilt_angle_deg: 10.0,
            fall_angle_deg: 30.0,
            fallen_angle_deg: 60.0,
            angular_vel_threshold: 0.5,
        }
    }
}

/// Fall detector state.
#[derive(Debug, Clone)]
pub struct FallDetector {
    pub config: FallDetectorConfig,
    pub state: FallState,
}

impl FallDetector {
    pub fn new(config: FallDetectorConfig) -> Self {
        Self {
            config,
            state: FallState::Upright,
        }
    }

    pub fn default_detector() -> Self {
        Self::new(FallDetectorConfig::default())
    }
}

/// Classify fall state from trunk tilt angle (degrees) and angular velocity.
pub fn classify_fall_state(tilt_deg: f32, angular_vel: f32, cfg: &FallDetectorConfig) -> FallState {
    let tilt = tilt_deg.abs();
    if tilt >= cfg.fallen_angle_deg {
        FallState::Fallen
    } else if tilt >= cfg.fall_angle_deg {
        FallState::Falling
    } else if tilt >= cfg.tilt_angle_deg || angular_vel.abs() >= cfg.angular_vel_threshold {
        FallState::Tilting
    } else {
        FallState::Upright
    }
}

/// Update the fall detector with new sensor data.
pub fn update_fall_detector(detector: &mut FallDetector, tilt_deg: f32, angular_vel: f32) {
    detector.state = classify_fall_state(tilt_deg, angular_vel, &detector.config);
}

/// Return true if a fall has been detected (state is Falling or Fallen).
pub fn fall_detected(state: FallState) -> bool {
    matches!(state, FallState::Falling | FallState::Fallen)
}

/// Return whether recovery is still possible (Upright or Tilting).
pub fn recovery_possible(state: FallState) -> bool {
    matches!(state, FallState::Upright | FallState::Tilting)
}

/// Return a descriptive label for the fall state.
pub fn fall_state_label(state: FallState) -> &'static str {
    match state {
        FallState::Upright => "upright",
        FallState::Tilting => "tilting",
        FallState::Falling => "falling",
        FallState::Fallen => "fallen",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upright_state() {
        /* small tilt is upright */
        let cfg = FallDetectorConfig::default();
        assert_eq!(classify_fall_state(2.0, 0.0, &cfg), FallState::Upright);
    }

    #[test]
    fn test_tilting_state() {
        /* medium tilt is tilting */
        let cfg = FallDetectorConfig::default();
        assert_eq!(classify_fall_state(15.0, 0.0, &cfg), FallState::Tilting);
    }

    #[test]
    fn test_falling_state() {
        /* large tilt is falling */
        let cfg = FallDetectorConfig::default();
        assert_eq!(classify_fall_state(45.0, 0.0, &cfg), FallState::Falling);
    }

    #[test]
    fn test_fallen_state() {
        /* very large tilt is fallen */
        let cfg = FallDetectorConfig::default();
        assert_eq!(classify_fall_state(75.0, 0.0, &cfg), FallState::Fallen);
    }

    #[test]
    fn test_tilting_from_angular_vel() {
        /* high angular velocity → tilting */
        let cfg = FallDetectorConfig::default();
        assert_eq!(classify_fall_state(5.0, 1.0, &cfg), FallState::Tilting);
    }

    #[test]
    fn test_fall_detected_true() {
        /* falling/fallen triggers detection */
        assert!(fall_detected(FallState::Falling));
        assert!(fall_detected(FallState::Fallen));
    }

    #[test]
    fn test_fall_detected_false() {
        /* upright/tilting are not falls */
        assert!(!fall_detected(FallState::Upright));
    }

    #[test]
    fn test_recovery_possible() {
        /* recovery possible only in early states */
        assert!(recovery_possible(FallState::Tilting));
        assert!(!recovery_possible(FallState::Fallen));
    }

    #[test]
    fn test_fall_state_label() {
        /* labels are correct */
        assert_eq!(fall_state_label(FallState::Upright), "upright");
        assert_eq!(fall_state_label(FallState::Fallen), "fallen");
    }
}
