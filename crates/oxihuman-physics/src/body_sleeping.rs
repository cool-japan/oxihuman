// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Tracks body sleeping state based on velocity thresholds.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BodySleeping {
    linear_threshold: f32,
    angular_threshold: f32,
    time_to_sleep: f32,
    idle_time: f32,
    sleeping: bool,
}

#[allow(dead_code)]
impl BodySleeping {
    pub fn new(linear_threshold: f32, angular_threshold: f32, time_to_sleep: f32) -> Self {
        Self {
            linear_threshold,
            angular_threshold,
            time_to_sleep,
            idle_time: 0.0,
            sleeping: false,
        }
    }

    pub fn default_config() -> Self {
        Self::new(0.01, 0.01, 0.5)
    }

    pub fn update(&mut self, linear_speed: f32, angular_speed: f32, dt: f32) {
        if linear_speed < self.linear_threshold && angular_speed < self.angular_threshold {
            self.idle_time += dt;
            if self.idle_time >= self.time_to_sleep {
                self.sleeping = true;
            }
        } else {
            self.idle_time = 0.0;
            self.sleeping = false;
        }
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    pub fn wake(&mut self) {
        self.sleeping = false;
        self.idle_time = 0.0;
    }

    pub fn force_sleep(&mut self) {
        self.sleeping = true;
    }

    pub fn idle_time(&self) -> f32 {
        self.idle_time
    }

    pub fn linear_threshold(&self) -> f32 {
        self.linear_threshold
    }

    pub fn angular_threshold(&self) -> f32 {
        self.angular_threshold
    }

    pub fn time_to_sleep(&self) -> f32 {
        self.time_to_sleep
    }

    pub fn set_thresholds(&mut self, linear: f32, angular: f32) {
        self.linear_threshold = linear;
        self.angular_threshold = angular;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bs = BodySleeping::new(0.01, 0.01, 0.5);
        assert!(!bs.is_sleeping());
        assert!((bs.idle_time()).abs() < 1e-9);
    }

    #[test]
    fn test_default_config() {
        let bs = BodySleeping::default_config();
        assert!((bs.linear_threshold() - 0.01).abs() < 1e-9);
        assert!((bs.time_to_sleep() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_goes_to_sleep() {
        let mut bs = BodySleeping::new(0.1, 0.1, 1.0);
        for _ in 0..20 {
            bs.update(0.001, 0.001, 0.1);
        }
        assert!(bs.is_sleeping());
    }

    #[test]
    fn test_stays_awake_with_motion() {
        let mut bs = BodySleeping::new(0.1, 0.1, 1.0);
        for _ in 0..20 {
            bs.update(1.0, 0.5, 0.1);
        }
        assert!(!bs.is_sleeping());
    }

    #[test]
    fn test_wake() {
        let mut bs = BodySleeping::new(0.1, 0.1, 0.1);
        bs.update(0.0, 0.0, 1.0);
        assert!(bs.is_sleeping());
        bs.wake();
        assert!(!bs.is_sleeping());
        assert!((bs.idle_time()).abs() < 1e-9);
    }

    #[test]
    fn test_force_sleep() {
        let mut bs = BodySleeping::new(0.1, 0.1, 1.0);
        bs.force_sleep();
        assert!(bs.is_sleeping());
    }

    #[test]
    fn test_motion_resets_idle() {
        let mut bs = BodySleeping::new(0.1, 0.1, 1.0);
        bs.update(0.0, 0.0, 0.5);
        assert!(bs.idle_time() > 0.0);
        bs.update(1.0, 1.0, 0.1);
        assert!((bs.idle_time()).abs() < 1e-9);
    }

    #[test]
    fn test_set_thresholds() {
        let mut bs = BodySleeping::new(0.1, 0.1, 1.0);
        bs.set_thresholds(0.5, 0.5);
        assert!((bs.linear_threshold() - 0.5).abs() < 1e-9);
        assert!((bs.angular_threshold() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_angular_prevents_sleep() {
        let mut bs = BodySleeping::new(0.1, 0.1, 0.5);
        for _ in 0..20 {
            bs.update(0.0, 1.0, 0.1);
        }
        assert!(!bs.is_sleeping());
    }

    #[test]
    fn test_exact_threshold_not_sleeping() {
        let mut bs = BodySleeping::new(0.1, 0.1, 1.0);
        for _ in 0..20 {
            bs.update(0.1, 0.1, 0.1);
        }
        assert!(!bs.is_sleeping());
    }
}
