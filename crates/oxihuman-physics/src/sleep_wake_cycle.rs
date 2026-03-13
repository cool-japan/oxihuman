// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Two-process sleep-wake model (Borbely).

pub struct SleepWakeModel {
    pub homeostatic_pressure: f32,
    pub circadian_drive: f32,
    pub wake_rate: f32,
    pub sleep_rate: f32,
    pub is_asleep: bool,
}

pub fn new_sleep_wake_model() -> SleepWakeModel {
    SleepWakeModel {
        homeostatic_pressure: 0.0,
        circadian_drive: 1.0,
        wake_rate: 0.05,
        sleep_rate: 0.1,
        is_asleep: false,
    }
}

pub fn sleep_step(s: &mut SleepWakeModel, dt_hours: f32) {
    if s.is_asleep {
        /* pressure dissipates during sleep */
        s.homeostatic_pressure = (s.homeostatic_pressure - s.sleep_rate * dt_hours).max(0.0);
    } else {
        /* pressure builds while awake */
        s.homeostatic_pressure = (s.homeostatic_pressure + s.wake_rate * dt_hours).min(2.0);
    }
    /* circadian drive oscillates (simple stub: constant) */
}

pub fn sleep_alertness(s: &SleepWakeModel) -> f32 {
    (s.circadian_drive - s.homeostatic_pressure).max(0.0)
}

pub fn sleep_should_sleep(s: &SleepWakeModel, threshold: f32) -> bool {
    s.homeostatic_pressure > threshold
}

pub fn sleep_set_asleep(s: &mut SleepWakeModel, asleep: bool) {
    s.is_asleep = asleep;
}

pub fn sleep_hours_since_wake(s: &SleepWakeModel) -> f32 {
    /* estimate from homeostatic pressure */
    if s.wake_rate < 1e-9 {
        return 0.0;
    }
    s.homeostatic_pressure / s.wake_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sleep_wake_model() {
        /* new model starts awake with zero pressure */
        let s = new_sleep_wake_model();
        assert!(!s.is_asleep);
        assert_eq!(s.homeostatic_pressure, 0.0);
    }

    #[test]
    fn test_sleep_step_builds_pressure_awake() {
        /* pressure builds while awake */
        let mut s = new_sleep_wake_model();
        sleep_step(&mut s, 1.0);
        assert!(s.homeostatic_pressure > 0.0);
    }

    #[test]
    fn test_sleep_step_dissipates_pressure_asleep() {
        /* pressure dissipates during sleep */
        let mut s = new_sleep_wake_model();
        s.homeostatic_pressure = 1.0;
        sleep_set_asleep(&mut s, true);
        sleep_step(&mut s, 1.0);
        assert!(s.homeostatic_pressure < 1.0);
    }

    #[test]
    fn test_sleep_alertness() {
        /* alertness decreases as pressure builds */
        let mut s = new_sleep_wake_model();
        let a0 = sleep_alertness(&s);
        sleep_step(&mut s, 8.0);
        let a1 = sleep_alertness(&s);
        assert!(a1 <= a0);
    }

    #[test]
    fn test_sleep_should_sleep() {
        /* should sleep when pressure exceeds threshold */
        let mut s = new_sleep_wake_model();
        sleep_step(&mut s, 20.0); /* simulate long wake time */
        assert!(sleep_should_sleep(&s, 0.5));
    }

    #[test]
    fn test_sleep_set_asleep() {
        /* set_asleep changes sleep state */
        let mut s = new_sleep_wake_model();
        sleep_set_asleep(&mut s, true);
        assert!(s.is_asleep);
    }

    #[test]
    fn test_sleep_hours_since_wake() {
        /* hours estimate is non-negative */
        let s = new_sleep_wake_model();
        assert!(sleep_hours_since_wake(&s) >= 0.0);
    }
}
