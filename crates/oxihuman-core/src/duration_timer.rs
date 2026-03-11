// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A simple tick-based duration timer for measuring elapsed frames or
//! accumulated time without real clocks (deterministic).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DurationTimer {
    elapsed_ns: u64,
    running: bool,
    lap_times: Vec<u64>,
}

#[allow(dead_code)]
impl DurationTimer {
    pub fn new() -> Self {
        Self { elapsed_ns: 0, running: false, lap_times: Vec::new() }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn reset(&mut self) {
        self.elapsed_ns = 0;
        self.running = false;
        self.lap_times.clear();
    }

    pub fn advance(&mut self, delta_ns: u64) {
        if self.running {
            self.elapsed_ns += delta_ns;
        }
    }

    pub fn elapsed_ns(&self) -> u64 {
        self.elapsed_ns
    }

    pub fn elapsed_us(&self) -> u64 {
        self.elapsed_ns / 1_000
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ns / 1_000_000
    }

    pub fn elapsed_secs_f64(&self) -> f64 {
        self.elapsed_ns as f64 / 1_000_000_000.0
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn lap(&mut self) {
        self.lap_times.push(self.elapsed_ns);
    }

    pub fn lap_count(&self) -> usize {
        self.lap_times.len()
    }

    pub fn lap_times(&self) -> &[u64] {
        &self.lap_times
    }

    pub fn last_lap_delta(&self) -> u64 {
        if self.lap_times.len() < 2 {
            return self.lap_times.first().copied().unwrap_or(0);
        }
        let n = self.lap_times.len();
        self.lap_times[n - 1] - self.lap_times[n - 2]
    }
}

impl Default for DurationTimer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stopped() {
        let t = DurationTimer::new();
        assert!(!t.is_running());
        assert_eq!(t.elapsed_ns(), 0);
    }

    #[test]
    fn test_advance_running() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(1_000_000);
        assert_eq!(t.elapsed_ms(), 1);
    }

    #[test]
    fn test_advance_stopped() {
        let mut t = DurationTimer::new();
        t.advance(1_000_000);
        assert_eq!(t.elapsed_ns(), 0);
    }

    #[test]
    fn test_stop() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(500);
        t.stop();
        t.advance(500);
        assert_eq!(t.elapsed_ns(), 500);
    }

    #[test]
    fn test_reset() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(1000);
        t.reset();
        assert_eq!(t.elapsed_ns(), 0);
        assert!(!t.is_running());
    }

    #[test]
    fn test_elapsed_conversions() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(2_500_000_000);
        assert_eq!(t.elapsed_ms(), 2500);
        assert_eq!(t.elapsed_us(), 2_500_000);
        assert!((t.elapsed_secs_f64() - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_lap() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(100);
        t.lap();
        t.advance(200);
        t.lap();
        assert_eq!(t.lap_count(), 2);
        assert_eq!(t.last_lap_delta(), 200);
    }

    #[test]
    fn test_lap_times() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(50);
        t.lap();
        assert_eq!(t.lap_times(), &[50]);
    }

    #[test]
    fn test_single_lap_delta() {
        let mut t = DurationTimer::new();
        t.start();
        t.advance(100);
        t.lap();
        assert_eq!(t.last_lap_delta(), 100);
    }

    #[test]
    fn test_no_lap_delta() {
        let t = DurationTimer::new();
        assert_eq!(t.last_lap_delta(), 0);
    }
}
