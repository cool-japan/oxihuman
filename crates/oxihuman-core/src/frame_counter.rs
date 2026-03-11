// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A frame counter that tracks frame timing statistics.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrameCounter {
    frame: u64,
    total_time: f64,
    frame_times: Vec<f64>,
    max_samples: usize,
}

#[allow(dead_code)]
impl FrameCounter {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame: 0,
            total_time: 0.0,
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.frame += 1;
        self.total_time += dt;
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.remove(0);
        }
        self.frame_times.push(dt);
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn total_time(&self) -> f64 {
        self.total_time
    }

    pub fn avg_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.frame_times.iter().sum();
        sum / self.frame_times.len() as f64
    }

    pub fn fps(&self) -> f64 {
        let avg = self.avg_frame_time();
        if avg <= 0.0 {
            0.0
        } else {
            1.0 / avg
        }
    }

    pub fn min_frame_time(&self) -> f64 {
        self.frame_times
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    pub fn max_frame_time(&self) -> f64 {
        self.frame_times
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    pub fn sample_count(&self) -> usize {
        self.frame_times.len()
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.total_time = 0.0;
        self.frame_times.clear();
    }

    pub fn last_frame_time(&self) -> f64 {
        self.frame_times.last().copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fc = FrameCounter::new(60);
        assert_eq!(fc.frame(), 0);
        assert!((fc.total_time()).abs() < 1e-12);
    }

    #[test]
    fn test_tick() {
        let mut fc = FrameCounter::new(60);
        fc.tick(0.016);
        assert_eq!(fc.frame(), 1);
        assert!((fc.total_time() - 0.016).abs() < 1e-12);
    }

    #[test]
    fn test_avg_frame_time() {
        let mut fc = FrameCounter::new(60);
        fc.tick(0.010);
        fc.tick(0.020);
        assert!((fc.avg_frame_time() - 0.015).abs() < 1e-12);
    }

    #[test]
    fn test_fps() {
        let mut fc = FrameCounter::new(60);
        fc.tick(0.01);
        assert!((fc.fps() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_min_max() {
        let mut fc = FrameCounter::new(60);
        fc.tick(0.010);
        fc.tick(0.005);
        fc.tick(0.020);
        assert!((fc.min_frame_time() - 0.005).abs() < 1e-12);
        assert!((fc.max_frame_time() - 0.020).abs() < 1e-12);
    }

    #[test]
    fn test_max_samples() {
        let mut fc = FrameCounter::new(3);
        fc.tick(1.0);
        fc.tick(2.0);
        fc.tick(3.0);
        fc.tick(4.0);
        assert_eq!(fc.sample_count(), 3);
        assert!((fc.min_frame_time() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_reset() {
        let mut fc = FrameCounter::new(60);
        fc.tick(0.016);
        fc.tick(0.016);
        fc.reset();
        assert_eq!(fc.frame(), 0);
        assert_eq!(fc.sample_count(), 0);
    }

    #[test]
    fn test_last_frame_time() {
        let mut fc = FrameCounter::new(60);
        fc.tick(0.01);
        fc.tick(0.02);
        assert!((fc.last_frame_time() - 0.02).abs() < 1e-12);
    }

    #[test]
    fn test_empty_stats() {
        let fc = FrameCounter::new(60);
        assert!((fc.avg_frame_time()).abs() < 1e-12);
        assert!((fc.fps()).abs() < 1e-12);
        assert!((fc.last_frame_time()).abs() < 1e-12);
    }

    #[test]
    fn test_total_time_accumulates() {
        let mut fc = FrameCounter::new(2);
        fc.tick(1.0);
        fc.tick(2.0);
        fc.tick(3.0);
        assert!((fc.total_time() - 6.0).abs() < 1e-12);
    }
}
