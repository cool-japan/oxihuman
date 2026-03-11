// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gait phase scheduler stub.

/// Gait phase for a single leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegPhase {
    Stance,
    Swing,
    DoubleSupport,
}

/// Gait type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaitType {
    Walk,
    Trot,
    Run,
    Stand,
}

/// Gait scheduler state.
#[derive(Debug, Clone)]
pub struct GaitScheduler {
    pub gait: GaitType,
    pub cycle_period: f32,
    pub phase_offset_left: f32,
    pub phase_offset_right: f32,
    pub current_time: f32,
}

impl GaitScheduler {
    pub fn new(gait: GaitType, cycle_period: f32) -> Self {
        let phase_offset_right = match gait {
            GaitType::Walk => 0.5,
            GaitType::Trot => 0.5,
            GaitType::Run => 0.5,
            GaitType::Stand => 0.0,
        };
        Self {
            gait,
            cycle_period,
            current_time: 0.0,
            phase_offset_left: 0.0,
            phase_offset_right,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.current_time += dt;
    }
}

/// Compute the gait phase in [0, 1) for the given time and offset.
pub fn gait_phase(time: f32, period: f32, offset: f32) -> f32 {
    let p = period.max(1e-6);
    ((time / p + offset) % 1.0 + 1.0) % 1.0
}

/// Return the leg phase for a given gait phase value.
pub fn phase_to_leg_phase(phase: f32, swing_ratio: f32) -> LegPhase {
    if (0.0..swing_ratio).contains(&phase) {
        LegPhase::Swing
    } else {
        LegPhase::Stance
    }
}

/// Return the current phase for left and right legs.
pub fn scheduler_phases(sched: &GaitScheduler) -> (f32, f32) {
    let left = gait_phase(
        sched.current_time,
        sched.cycle_period,
        sched.phase_offset_left,
    );
    let right = gait_phase(
        sched.current_time,
        sched.cycle_period,
        sched.phase_offset_right,
    );
    (left, right)
}

/// Return true if both legs are in stance (double support).
pub fn is_double_support(sched: &GaitScheduler, swing_ratio: f32) -> bool {
    let (lp, rp) = scheduler_phases(sched);
    phase_to_leg_phase(lp, swing_ratio) == LegPhase::Stance
        && phase_to_leg_phase(rp, swing_ratio) == LegPhase::Stance
}

/// Return the remaining time until the next phase transition.
pub fn time_to_next_transition(phase: f32, period: f32, swing_ratio: f32) -> f32 {
    if (0.0..swing_ratio).contains(&phase) {
        (swing_ratio - phase) * period
    } else {
        (1.0 - phase) * period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gait_phase_zero() {
        /* phase starts at 0 */
        assert!((gait_phase(0.0, 1.0, 0.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_gait_phase_wraps() {
        /* phase wraps at 1.0 */
        let p = gait_phase(1.0, 1.0, 0.0);
        assert!(p < 1e-5);
    }

    #[test]
    fn test_phase_swing() {
        /* early phase is swing */
        assert_eq!(phase_to_leg_phase(0.1, 0.4), LegPhase::Swing);
    }

    #[test]
    fn test_phase_stance() {
        /* late phase is stance */
        assert_eq!(phase_to_leg_phase(0.6, 0.4), LegPhase::Stance);
    }

    #[test]
    fn test_scheduler_new() {
        /* walk gait has 0.5 right offset */
        let s = GaitScheduler::new(GaitType::Walk, 1.0);
        assert!((s.phase_offset_right - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_scheduler_step() {
        /* time advances after step */
        let mut s = GaitScheduler::new(GaitType::Walk, 1.0);
        s.step(0.1);
        assert!((s.current_time - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_scheduler_phases_len() {
        /* phases are two values */
        let s = GaitScheduler::new(GaitType::Walk, 1.0);
        let (l, r) = scheduler_phases(&s);
        assert!((0.0..=1.0).contains(&l) && (0.0..=1.0).contains(&r));
    }

    #[test]
    fn test_time_to_next_transition_positive() {
        /* remaining time is positive */
        let t = time_to_next_transition(0.1, 1.0, 0.4);
        assert!(t > 0.0);
    }

    #[test]
    fn test_stand_gait_no_swing() {
        /* stand gait — swing_ratio=0 means no swing phase ever */
        let s = GaitScheduler::new(GaitType::Stand, 1.0);
        assert!(is_double_support(&s, 0.0));
    }
}
