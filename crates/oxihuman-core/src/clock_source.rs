// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A deterministic clock source for simulation and testing.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ClockSource {
    current_time: f64,
    delta: f64,
    frame: u64,
    paused: bool,
    speed: f64,
}

#[allow(dead_code)]
impl ClockSource {
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            delta: 1.0 / 60.0,
            frame: 0,
            paused: false,
            speed: 1.0,
        }
    }

    pub fn with_delta(delta: f64) -> Self {
        Self {
            delta,
            ..Self::new()
        }
    }

    pub fn advance(&mut self) {
        if !self.paused {
            self.current_time += self.delta * self.speed;
            self.frame += 1;
        }
    }

    pub fn advance_by(&mut self, dt: f64) {
        if !self.paused {
            self.current_time += dt * self.speed;
            self.frame += 1;
        }
    }

    pub fn time(&self) -> f64 {
        self.current_time
    }

    pub fn delta(&self) -> f64 {
        self.delta
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.frame = 0;
    }
}

impl Default for ClockSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = ClockSource::new();
        assert!((c.time()).abs() < 1e-12);
        assert_eq!(c.frame(), 0);
    }

    #[test]
    fn test_advance() {
        let mut c = ClockSource::with_delta(0.5);
        c.advance();
        assert!((c.time() - 0.5).abs() < 1e-12);
        assert_eq!(c.frame(), 1);
    }

    #[test]
    fn test_advance_by() {
        let mut c = ClockSource::new();
        c.advance_by(2.0);
        assert!((c.time() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_paused() {
        let mut c = ClockSource::with_delta(1.0);
        c.pause();
        c.advance();
        assert!((c.time()).abs() < 1e-12);
        assert_eq!(c.frame(), 0);
    }

    #[test]
    fn test_resume() {
        let mut c = ClockSource::with_delta(1.0);
        c.pause();
        c.resume();
        c.advance();
        assert!((c.time() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_speed() {
        let mut c = ClockSource::with_delta(1.0);
        c.set_speed(2.0);
        c.advance();
        assert!((c.time() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_reset() {
        let mut c = ClockSource::with_delta(1.0);
        c.advance();
        c.advance();
        c.reset();
        assert!((c.time()).abs() < 1e-12);
        assert_eq!(c.frame(), 0);
    }

    #[test]
    fn test_delta() {
        let c = ClockSource::with_delta(0.01);
        assert!((c.delta() - 0.01).abs() < 1e-12);
    }

    #[test]
    fn test_multiple_advances() {
        let mut c = ClockSource::with_delta(0.25);
        for _ in 0..4 {
            c.advance();
        }
        assert!((c.time() - 1.0).abs() < 1e-12);
        assert_eq!(c.frame(), 4);
    }

    #[test]
    fn test_default() {
        let c = ClockSource::default();
        assert!(!c.is_paused());
        assert!((c.speed() - 1.0).abs() < 1e-12);
    }
}
