// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A body oscillating according to a sinusoidal wave (e.g. for ocean buoys).

use std::f32::consts::PI;

/// A body driven by a sinusoidal wave forcing function.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaveBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    pub damping: f32,
    /// Wave amplitude.
    pub amplitude: f32,
    /// Wave angular frequency (rad/s).
    pub frequency: f32,
    /// Wave phase offset (rad).
    pub phase: f32,
    /// Wave propagation direction (unit vector).
    pub wave_dir: [f32; 3],
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl WaveBody {
    pub fn new(mass: f32, damping: f32, amplitude: f32, frequency_hz: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass: mass.max(1e-6),
            damping: damping.max(0.0),
            amplitude,
            frequency: 2.0 * PI * frequency_hz,
            phase: 0.0,
            wave_dir: [0.0, 1.0, 0.0],
            time: 0.0,
            steps: 0,
        }
    }

    pub fn with_phase(mut self, phase_rad: f32) -> Self {
        self.phase = phase_rad;
        self
    }

    pub fn with_wave_dir(mut self, dir: [f32; 3]) -> Self {
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2])
            .sqrt()
            .max(1e-6);
        self.wave_dir = [dir[0] / len, dir[1] / len, dir[2] / len];
        self
    }

    /// Wave height at current time.
    pub fn wave_height(&self) -> f32 {
        self.amplitude * (self.frequency * self.time + self.phase).sin()
    }

    /// Buoyancy restoring force along wave_dir.
    pub fn restoring_force(&self) -> [f32; 3] {
        let h = self.wave_height();
        let proj = self.pos[0] * self.wave_dir[0]
            + self.pos[1] * self.wave_dir[1]
            + self.pos[2] * self.wave_dir[2];
        let delta = h - proj;
        let k = 50.0; // stiffness
        let f = k * delta;
        [
            self.wave_dir[0] * f,
            self.wave_dir[1] * f,
            self.wave_dir[2] * f,
        ]
    }

    pub fn step(&mut self, dt: f32, gravity: [f32; 3]) {
        let rf = self.restoring_force();
        let inv_m = 1.0 / self.mass;
        for i in 0..3 {
            let drag = -self.damping * self.vel[i];
            let acc = (gravity[i] + rf[i] + drag) * inv_m;
            self.vel[i] += acc * dt;
            self.pos[i] += self.vel[i] * dt;
        }
        self.time += dt;
        self.steps += 1;
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed() * self.speed()
    }

    pub fn reset(&mut self) {
        self.pos = [0.0; 3];
        self.vel = [0.0; 3];
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for WaveBody {
    fn default() -> Self {
        Self::new(1.0, 0.5, 1.0, 0.25)
    }
}

pub fn new_wave_body(mass: f32, damping: f32, amplitude: f32, frequency_hz: f32) -> WaveBody {
    WaveBody::new(mass, damping, amplitude, frequency_hz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_height_oscillates() {
        let mut b = new_wave_body(1.0, 0.0, 2.0, 1.0);
        b.time = 0.0;
        let h0 = b.wave_height();
        b.time = 0.25;
        let h1 = b.wave_height();
        assert!((h1 - 2.0).abs() < 0.05); // near peak at t=0.25s for 1Hz
        let _ = h0;
    }

    #[test]
    fn restoring_force_towards_wave() {
        let b = new_wave_body(1.0, 0.0, 1.0, 1.0);
        let rf = b.restoring_force();
        // pos[1]=0, wave_height=0 at t=0 => delta=0 => force=0
        assert!(rf.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn step_advances_time() {
        let mut b = new_wave_body(1.0, 0.1, 1.0, 1.0);
        b.step(0.1, [0.0; 3]);
        assert!((b.time - 0.1).abs() < 1e-6);
    }

    #[test]
    fn steps_increment() {
        let mut b = new_wave_body(1.0, 0.1, 1.0, 1.0);
        b.step(0.01, [0.0; 3]);
        b.step(0.01, [0.0; 3]);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut b = new_wave_body(1.0, 0.5, 2.0, 1.0);
        b.step(0.5, [0.0, -9.81, 0.0]);
        assert!(b.kinetic_energy() >= 0.0);
    }

    #[test]
    fn wave_dir_normalized() {
        let b = WaveBody::new(1.0, 0.0, 1.0, 1.0).with_wave_dir([3.0, 4.0, 0.0]);
        let len = (b.wave_dir[0] * b.wave_dir[0]
            + b.wave_dir[1] * b.wave_dir[1]
            + b.wave_dir[2] * b.wave_dir[2])
            .sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_zeroes_state() {
        let mut b = new_wave_body(1.0, 0.1, 1.0, 0.5);
        b.step(1.0, [0.0, -9.81, 0.0]);
        b.reset();
        assert_eq!(b.steps, 0);
        assert!(b.speed() < 1e-5);
    }

    #[test]
    fn phase_shifts_wave() {
        let b0 = WaveBody::new(1.0, 0.0, 1.0, 1.0);
        let b1 = WaveBody::new(1.0, 0.0, 1.0, 1.0).with_phase(PI);
        assert!((b0.wave_height() + b1.wave_height()).abs() < 1e-5);
    }

    #[test]
    fn frequency_hz_converts() {
        let b = new_wave_body(1.0, 0.0, 1.0, 2.0);
        assert!((b.frequency - 2.0 * PI * 2.0).abs() < 1e-4);
    }

    #[test]
    fn default_is_valid() {
        let b = WaveBody::default();
        assert!(b.mass > 0.0);
        assert!(b.amplitude > 0.0);
    }
}
