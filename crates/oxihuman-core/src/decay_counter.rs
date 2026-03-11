// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::E;

/// A counter that decays exponentially over time.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DecayCounter {
    value: f32,
    half_life: f32,
    last_update: f32,
}

#[allow(dead_code)]
impl DecayCounter {
    pub fn new(half_life: f32) -> Self {
        Self {
            value: 0.0,
            half_life: half_life.max(f32::EPSILON),
            last_update: 0.0,
        }
    }

    pub fn with_initial(value: f32, half_life: f32) -> Self {
        Self {
            value,
            half_life: half_life.max(f32::EPSILON),
            last_update: 0.0,
        }
    }

    pub fn increment(&mut self, amount: f32) {
        self.value += amount;
    }

    pub fn update(&mut self, current_time: f32) {
        let dt = current_time - self.last_update;
        if dt > 0.0 {
            let decay_rate = (2.0_f32).ln() / self.half_life;
            self.value *= (-decay_rate * dt).exp();
            self.last_update = current_time;
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn value_at(&self, time: f32) -> f32 {
        let dt = time - self.last_update;
        if dt <= 0.0 {
            return self.value;
        }
        let decay_rate = (2.0_f32).ln() / self.half_life;
        self.value * (-decay_rate * dt).exp()
    }

    pub fn half_life(&self) -> f32 {
        self.half_life
    }

    pub fn set_half_life(&mut self, hl: f32) {
        self.half_life = hl.max(f32::EPSILON);
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
        self.last_update = 0.0;
    }

    pub fn is_negligible(&self, threshold: f32) -> bool {
        self.value.abs() < threshold
    }

    pub fn time_to_reach(&self, target: f32) -> Option<f32> {
        if self.value <= 0.0 || target <= 0.0 || target >= self.value {
            return None;
        }
        let decay_rate = (2.0_f32).ln() / self.half_life;
        Some((self.value / target).ln() / decay_rate)
    }

    /// Return the decay constant (lambda = ln(2)/half_life).
    pub fn decay_constant(&self) -> f32 {
        let _ = E; // reference E from consts
        (2.0_f32).ln() / self.half_life
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dc = DecayCounter::new(1.0);
        assert!((dc.value() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_increment() {
        let mut dc = DecayCounter::new(1.0);
        dc.increment(10.0);
        assert!((dc.value() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_one_half_life() {
        let mut dc = DecayCounter::with_initial(100.0, 1.0);
        dc.update(1.0);
        assert!((dc.value() - 50.0).abs() < 0.5);
    }

    #[test]
    fn test_value_at() {
        let dc = DecayCounter::with_initial(100.0, 1.0);
        let v = dc.value_at(1.0);
        assert!((v - 50.0).abs() < 0.5);
    }

    #[test]
    fn test_half_life_getter() {
        let dc = DecayCounter::new(2.5);
        assert!((dc.half_life() - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_half_life() {
        let mut dc = DecayCounter::new(1.0);
        dc.set_half_life(5.0);
        assert!((dc.half_life() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset() {
        let mut dc = DecayCounter::with_initial(50.0, 1.0);
        dc.reset();
        assert!((dc.value() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_negligible() {
        let dc = DecayCounter::with_initial(0.001, 1.0);
        assert!(dc.is_negligible(0.01));
        assert!(!dc.is_negligible(0.0001));
    }

    #[test]
    fn test_time_to_reach() {
        let dc = DecayCounter::with_initial(100.0, 1.0);
        let t = dc.time_to_reach(50.0).unwrap();
        assert!((t - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_decay_constant() {
        let dc = DecayCounter::new(1.0);
        let lambda = dc.decay_constant();
        assert!((lambda - (2.0_f32).ln()).abs() < 1e-5);
    }
}
