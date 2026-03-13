#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Body sleep state management for physics simulation.

/// The sleep state of a body.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SleepState {
    Awake,
    Sleeping,
}

/// Configuration for sleep detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SleepConfig {
    pub threshold: f32,
    pub time_to_sleep: f32,
    pub timer: f32,
    pub state: SleepState,
}

#[allow(dead_code)]
pub fn new_sleep_config(threshold: f32, time_to_sleep: f32) -> SleepConfig {
    SleepConfig {
        threshold,
        time_to_sleep,
        timer: 0.0,
        state: SleepState::Awake,
    }
}

#[allow(dead_code)]
pub fn default_sleep_config() -> SleepConfig {
    new_sleep_config(0.01, 0.5)
}

#[allow(dead_code)]
pub fn should_sleep(config: &mut SleepConfig, velocity_sq: f32, dt: f32) -> bool {
    if velocity_sq < config.threshold * config.threshold {
        config.timer += dt;
        if config.timer >= config.time_to_sleep {
            config.state = SleepState::Sleeping;
            return true;
        }
    } else {
        config.timer = 0.0;
        config.state = SleepState::Awake;
    }
    false
}

#[allow(dead_code)]
pub fn wake_body(config: &mut SleepConfig) {
    config.state = SleepState::Awake;
    config.timer = 0.0;
}

#[allow(dead_code)]
pub fn is_sleeping(config: &SleepConfig) -> bool {
    config.state == SleepState::Sleeping
}

#[allow(dead_code)]
pub fn sleep_timer(config: &SleepConfig) -> f32 {
    config.timer
}

#[allow(dead_code)]
pub fn reset_sleep_timer(config: &mut SleepConfig) {
    config.timer = 0.0;
}

#[allow(dead_code)]
pub fn sleep_threshold(config: &SleepConfig) -> f32 {
    config.threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_sleep_config();
        assert!(!is_sleeping(&c));
    }

    #[test]
    fn test_new_config() {
        let c = new_sleep_config(0.05, 1.0);
        assert_eq!(c.threshold, 0.05);
        assert_eq!(c.time_to_sleep, 1.0);
    }

    #[test]
    fn test_should_sleep_low_velocity() {
        let mut c = new_sleep_config(1.0, 0.5);
        // velocity_sq = 0.0001, below threshold^2 = 1.0
        assert!(!should_sleep(&mut c, 0.0001, 0.3));
        assert!(should_sleep(&mut c, 0.0001, 0.3)); // timer exceeds 0.5
    }

    #[test]
    fn test_wake() {
        let mut c = default_sleep_config();
        c.state = SleepState::Sleeping;
        wake_body(&mut c);
        assert!(!is_sleeping(&c));
    }

    #[test]
    fn test_high_velocity_resets() {
        let mut c = new_sleep_config(0.01, 0.5);
        should_sleep(&mut c, 0.0, 0.3);
        should_sleep(&mut c, 100.0, 0.1); // high velocity
        assert_eq!(sleep_timer(&c), 0.0);
    }

    #[test]
    fn test_sleep_timer() {
        let mut c = new_sleep_config(1.0, 10.0);
        should_sleep(&mut c, 0.0, 0.25);
        assert!((sleep_timer(&c) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_reset_timer() {
        let mut c = default_sleep_config();
        c.timer = 5.0;
        reset_sleep_timer(&mut c);
        assert_eq!(sleep_timer(&c), 0.0);
    }

    #[test]
    fn test_threshold() {
        let c = new_sleep_config(0.02, 1.0);
        assert!((sleep_threshold(&c) - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_is_sleeping() {
        let mut c = SleepConfig {
            threshold: 1.0,
            time_to_sleep: 0.0,
            timer: 0.0,
            state: SleepState::Sleeping,
        };
        assert!(is_sleeping(&c));
        wake_body(&mut c);
        assert!(!is_sleeping(&c));
    }

    #[test]
    fn test_sleep_state_eq() {
        assert_eq!(SleepState::Awake, SleepState::Awake);
        assert_ne!(SleepState::Awake, SleepState::Sleeping);
    }
}
