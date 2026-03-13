// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Locomotion finite state machine stub.

/// Locomotion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocoMode {
    Stand,
    Walk,
    Run,
    Stop,
    Fall,
    Recover,
}

/// Locomotion FSM state.
#[derive(Debug, Clone)]
pub struct LocomotionFsm {
    pub mode: LocoMode,
    pub time_in_mode: f32,
    pub target_speed: f32,
}

impl Default for LocomotionFsm {
    fn default() -> Self {
        Self {
            mode: LocoMode::Stand,
            time_in_mode: 0.0,
            target_speed: 0.0,
        }
    }
}

impl LocomotionFsm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transition_to(&mut self, mode: LocoMode) {
        if self.mode != mode {
            self.mode = mode;
            self.time_in_mode = 0.0;
        }
    }
}

/// Update the FSM given speed command and fall flag.
pub fn update_locomotion_fsm(
    fsm: &mut LocomotionFsm,
    speed_cmd: f32,
    fall_detected: bool,
    dt: f32,
) {
    fsm.time_in_mode += dt;
    if fall_detected {
        fsm.transition_to(LocoMode::Fall);
        return;
    }
    if fsm.mode == LocoMode::Fall {
        fsm.transition_to(LocoMode::Recover);
        return;
    }
    if fsm.mode == LocoMode::Recover && fsm.time_in_mode > 1.0 {
        fsm.transition_to(LocoMode::Stand);
        return;
    }
    fsm.target_speed = speed_cmd;
    let next = if speed_cmd.abs() < 0.01 {
        LocoMode::Stand
    } else if speed_cmd.abs() < 1.5 {
        LocoMode::Walk
    } else {
        LocoMode::Run
    };
    if !matches!(fsm.mode, LocoMode::Fall | LocoMode::Recover) {
        fsm.transition_to(next);
    }
}

/// Return whether the FSM is in a mobile mode (Walk or Run).
pub fn is_moving(fsm: &LocomotionFsm) -> bool {
    matches!(fsm.mode, LocoMode::Walk | LocoMode::Run)
}

/// Return a string label for the current mode.
pub fn mode_label(mode: LocoMode) -> &'static str {
    match mode {
        LocoMode::Stand => "stand",
        LocoMode::Walk => "walk",
        LocoMode::Run => "run",
        LocoMode::Stop => "stop",
        LocoMode::Fall => "fall",
        LocoMode::Recover => "recover",
    }
}

/// Return the expected step frequency for a given locomotion mode (stub).
pub fn mode_step_frequency(mode: LocoMode) -> f32 {
    match mode {
        LocoMode::Stand | LocoMode::Stop => 0.0,
        LocoMode::Walk => 1.0,
        LocoMode::Run => 2.5,
        LocoMode::Fall | LocoMode::Recover => 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_mode_stand() {
        /* FSM starts in Stand mode */
        let fsm = LocomotionFsm::new();
        assert_eq!(fsm.mode, LocoMode::Stand);
    }

    #[test]
    fn test_transition_to_walk() {
        /* speed 1.0 → walk */
        let mut fsm = LocomotionFsm::new();
        update_locomotion_fsm(&mut fsm, 1.0, false, 0.01);
        assert_eq!(fsm.mode, LocoMode::Walk);
    }

    #[test]
    fn test_transition_to_run() {
        /* speed 2.0 → run */
        let mut fsm = LocomotionFsm::new();
        update_locomotion_fsm(&mut fsm, 2.0, false, 0.01);
        assert_eq!(fsm.mode, LocoMode::Run);
    }

    #[test]
    fn test_fall_detection() {
        /* fall flag → Fall mode */
        let mut fsm = LocomotionFsm::new();
        update_locomotion_fsm(&mut fsm, 1.0, true, 0.01);
        assert_eq!(fsm.mode, LocoMode::Fall);
    }

    #[test]
    fn test_is_moving_walk() {
        /* Walk is moving */
        let mut fsm = LocomotionFsm::new();
        fsm.mode = LocoMode::Walk;
        assert!(is_moving(&fsm));
    }

    #[test]
    fn test_is_not_moving_stand() {
        /* Stand is not moving */
        let fsm = LocomotionFsm::new();
        assert!(!is_moving(&fsm));
    }

    #[test]
    fn test_mode_label() {
        /* labels match modes */
        assert_eq!(mode_label(LocoMode::Walk), "walk");
        assert_eq!(mode_label(LocoMode::Fall), "fall");
    }

    #[test]
    fn test_step_frequency_walk() {
        /* walk has positive step frequency */
        assert!(mode_step_frequency(LocoMode::Walk) > 0.0);
    }

    #[test]
    fn test_step_frequency_stand() {
        /* stand has zero frequency */
        assert_eq!(mode_step_frequency(LocoMode::Stand), 0.0);
    }
}
