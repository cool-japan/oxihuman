// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Self-righting torque planner stub.

/// Phase of the self-righting maneuver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightingPhase {
    Idle,
    DetectDown,
    PushUp,
    Stabilize,
    Done,
}

/// Self-righting planner configuration.
#[derive(Debug, Clone)]
pub struct SelfRightingConfig {
    pub push_torque: f32,
    pub stabilize_torque: f32,
    pub righting_duration: f32,
}

impl Default for SelfRightingConfig {
    fn default() -> Self {
        Self {
            push_torque: 200.0,
            stabilize_torque: 50.0,
            righting_duration: 2.0,
        }
    }
}

/// Self-righting planner state.
#[derive(Debug, Clone)]
pub struct SelfRightingPlanner {
    pub config: SelfRightingConfig,
    pub phase: RightingPhase,
    pub elapsed: f32,
}

impl SelfRightingPlanner {
    pub fn new(config: SelfRightingConfig) -> Self {
        Self {
            config,
            phase: RightingPhase::Idle,
            elapsed: 0.0,
        }
    }

    pub fn default_planner() -> Self {
        Self::new(SelfRightingConfig::default())
    }

    pub fn trigger(&mut self) {
        self.phase = RightingPhase::DetectDown;
        self.elapsed = 0.0;
    }
}

/// Compute the self-righting torque for the current phase.
pub fn righting_torque(planner: &SelfRightingPlanner, tilt_deg: f32) -> [f32; 3] {
    let sign = if tilt_deg >= 0.0 { -1.0f32 } else { 1.0 };
    match planner.phase {
        RightingPhase::Idle | RightingPhase::Done => [0.0; 3],
        RightingPhase::DetectDown => [0.0; 3],
        RightingPhase::PushUp => [0.0, 0.0, sign * planner.config.push_torque],
        RightingPhase::Stabilize => [0.0, 0.0, sign * planner.config.stabilize_torque],
    }
}

/// Advance the planner by one time step.
pub fn advance_planner(planner: &mut SelfRightingPlanner, tilt_deg: f32, dt: f32) {
    planner.elapsed += dt;
    match planner.phase {
        RightingPhase::Idle => {}
        RightingPhase::DetectDown => {
            /* transition to push-up once we know orientation */
            planner.phase = RightingPhase::PushUp;
        }
        RightingPhase::PushUp => {
            if tilt_deg.abs() < 20.0 || planner.elapsed > planner.config.righting_duration {
                planner.phase = RightingPhase::Stabilize;
            }
        }
        RightingPhase::Stabilize => {
            if tilt_deg.abs() < 5.0 {
                planner.phase = RightingPhase::Done;
            }
        }
        RightingPhase::Done => {}
    }
}

/// Return whether the robot has successfully righted itself.
pub fn is_righted(planner: &SelfRightingPlanner) -> bool {
    planner.phase == RightingPhase::Done
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_phase_idle() {
        /* planner starts idle */
        let p = SelfRightingPlanner::default_planner();
        assert_eq!(p.phase, RightingPhase::Idle);
    }

    #[test]
    fn test_trigger_starts_detect() {
        /* trigger changes phase to DetectDown */
        let mut p = SelfRightingPlanner::default_planner();
        p.trigger();
        assert_eq!(p.phase, RightingPhase::DetectDown);
    }

    #[test]
    fn test_idle_zero_torque() {
        /* idle phase produces zero torque */
        let p = SelfRightingPlanner::default_planner();
        assert_eq!(righting_torque(&p, 45.0), [0.0; 3]);
    }

    #[test]
    fn test_push_up_nonzero_torque() {
        /* PushUp phase produces nonzero torque */
        let mut p = SelfRightingPlanner::default_planner();
        p.phase = RightingPhase::PushUp;
        let t = righting_torque(&p, 45.0);
        assert!(t[2].abs() > 0.0);
    }

    #[test]
    fn test_advance_detect_to_push() {
        /* advancing from DetectDown → PushUp */
        let mut p = SelfRightingPlanner::default_planner();
        p.trigger();
        advance_planner(&mut p, 90.0, 0.01);
        assert_eq!(p.phase, RightingPhase::PushUp);
    }

    #[test]
    fn test_advance_push_to_stabilize() {
        /* small tilt → PushUp transitions to Stabilize */
        let mut p = SelfRightingPlanner::default_planner();
        p.phase = RightingPhase::PushUp;
        advance_planner(&mut p, 10.0, 0.01);
        assert_eq!(p.phase, RightingPhase::Stabilize);
    }

    #[test]
    fn test_advance_stabilize_to_done() {
        /* near-zero tilt → Stabilize → Done */
        let mut p = SelfRightingPlanner::default_planner();
        p.phase = RightingPhase::Stabilize;
        advance_planner(&mut p, 2.0, 0.01);
        assert_eq!(p.phase, RightingPhase::Done);
    }

    #[test]
    fn test_is_righted() {
        /* Done phase → righted */
        let mut p = SelfRightingPlanner::default_planner();
        p.phase = RightingPhase::Done;
        assert!(is_righted(&p));
    }

    #[test]
    fn test_is_not_righted_when_idle() {
        /* Idle → not righted */
        let p = SelfRightingPlanner::default_planner();
        assert!(!is_righted(&p));
    }
}
