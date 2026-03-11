// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Tracks kinetic, potential, and total energy of a physics simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnergyTracker {
    samples: Vec<EnergySample>,
    max_samples: usize,
}

/// A single energy measurement at a point in time.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergySample {
    pub time: f32,
    pub kinetic: f32,
    pub potential: f32,
}

#[allow(dead_code)]
impl EnergySample {
    pub fn new(time: f32, kinetic: f32, potential: f32) -> Self {
        Self {
            time,
            kinetic,
            potential,
        }
    }

    pub fn total(&self) -> f32 {
        self.kinetic + self.potential
    }
}

#[allow(dead_code)]
impl EnergyTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::new(),
            max_samples: max_samples.max(1),
        }
    }

    pub fn record(&mut self, time: f32, kinetic: f32, potential: f32) {
        if self.samples.len() >= self.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(EnergySample::new(time, kinetic, potential));
    }

    pub fn latest(&self) -> Option<&EnergySample> {
        self.samples.last()
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn total_energy(&self) -> f32 {
        self.latest().map(|s| s.total()).unwrap_or(0.0)
    }

    pub fn average_kinetic(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|s| s.kinetic).sum();
        sum / self.samples.len() as f32
    }

    pub fn average_potential(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|s| s.potential).sum();
        sum / self.samples.len() as f32
    }

    pub fn max_kinetic(&self) -> f32 {
        self.samples
            .iter()
            .map(|s| s.kinetic)
            .fold(0.0_f32, f32::max)
    }

    pub fn energy_drift(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let first = self.samples[0].total();
        let last = self.samples[self.samples.len() - 1].total();
        last - first
    }

    pub fn is_stable(&self, threshold: f32) -> bool {
        self.energy_drift().abs() <= threshold
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn samples(&self) -> &[EnergySample] {
        &self.samples
    }
}

/// Compute kinetic energy: 0.5 * m * |v|^2
#[allow(dead_code)]
pub fn kinetic_energy_3d(mass: f32, velocity: [f32; 3]) -> f32 {
    let v2 = velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2];
    0.5 * mass * v2
}

/// Compute gravitational potential energy: m * g * h
#[allow(dead_code)]
pub fn gravitational_potential(mass: f32, gravity: f32, height: f32) -> f32 {
    mass * gravity * height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_sample_total() {
        let s = EnergySample::new(0.0, 10.0, 5.0);
        assert!((s.total() - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_record_and_latest() {
        let mut t = EnergyTracker::new(10);
        t.record(0.0, 10.0, 5.0);
        assert_eq!(t.sample_count(), 1);
        assert!((t.total_energy() - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_samples() {
        let mut t = EnergyTracker::new(2);
        t.record(0.0, 1.0, 0.0);
        t.record(1.0, 2.0, 0.0);
        t.record(2.0, 3.0, 0.0);
        assert_eq!(t.sample_count(), 2);
    }

    #[test]
    fn test_average_kinetic() {
        let mut t = EnergyTracker::new(10);
        t.record(0.0, 2.0, 0.0);
        t.record(1.0, 4.0, 0.0);
        assert!((t.average_kinetic() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_energy_drift() {
        let mut t = EnergyTracker::new(10);
        t.record(0.0, 10.0, 0.0);
        t.record(1.0, 9.0, 0.0);
        assert!((t.energy_drift() - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_stable() {
        let mut t = EnergyTracker::new(10);
        t.record(0.0, 10.0, 0.0);
        t.record(1.0, 10.0, 0.0);
        assert!(t.is_stable(0.01));
    }

    #[test]
    fn test_clear() {
        let mut t = EnergyTracker::new(10);
        t.record(0.0, 1.0, 1.0);
        t.clear();
        assert_eq!(t.sample_count(), 0);
    }

    #[test]
    fn test_kinetic_energy_3d() {
        let ke = kinetic_energy_3d(2.0, [3.0, 0.0, 0.0]);
        assert!((ke - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_gravitational_potential() {
        let pe = gravitational_potential(1.0, 9.81, 10.0);
        assert!((pe - 98.1).abs() < 1e-4);
    }

    #[test]
    fn test_max_kinetic() {
        let mut t = EnergyTracker::new(10);
        t.record(0.0, 5.0, 0.0);
        t.record(1.0, 15.0, 0.0);
        t.record(2.0, 10.0, 0.0);
        assert!((t.max_kinetic() - 15.0).abs() < f32::EPSILON);
    }
}
