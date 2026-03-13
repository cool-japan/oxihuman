#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Turbulence force field using procedural noise.

/// Parameters for turbulence generation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TurbParams {
    pub octaves: u32,
    pub frequency: f32,
    pub amplitude: f32,
    pub seed: u32,
}

/// A turbulence force field.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TurbulenceForce {
    pub params: TurbParams,
    pub time: f32,
}

/// Create default turbulence parameters.
#[allow(dead_code)]
pub fn default_turb_params() -> TurbParams {
    TurbParams { octaves: 4, frequency: 1.0, amplitude: 1.0, seed: 42 }
}

/// Create a new `TurbulenceForce`.
#[allow(dead_code)]
pub fn new_turbulence_force(params: TurbParams) -> TurbulenceForce {
    TurbulenceForce { params, time: 0.0 }
}

/// Simple deterministic noise from position + seed.
fn value_noise(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let h = (ix.wrapping_mul(374761393_i32))
        .wrapping_add(iy.wrapping_mul(668265263_i32))
        .wrapping_add(iz.wrapping_mul(1274126177_i32))
        .wrapping_add((seed as i32).wrapping_mul(1013904223_i32));
    (h as f32 / i32::MAX as f32).clamp(-1.0, 1.0)
}

/// Evaluate turbulence at a world position and return a 3-D force vector.
#[allow(dead_code)]
pub fn turbulence_at(tf: &TurbulenceForce, pos: [f32; 3]) -> [f32; 3] {
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;
    let mut fz = 0.0f32;
    let mut amp = tf.params.amplitude;
    let mut freq = tf.params.frequency;
    for _oct in 0..tf.params.octaves {
        let t = tf.time;
        fx += value_noise(pos[0] * freq + t, pos[1] * freq, pos[2] * freq, tf.params.seed) * amp;
        fy += value_noise(pos[0] * freq, pos[1] * freq + t, pos[2] * freq, tf.params.seed.wrapping_add(1)) * amp;
        fz += value_noise(pos[0] * freq, pos[1] * freq, pos[2] * freq + t, tf.params.seed.wrapping_add(2)) * amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    [fx, fy, fz]
}

/// Advance the turbulence time.
#[allow(dead_code)]
pub fn turbulence_step(tf: &mut TurbulenceForce, dt: f32) {
    tf.time += dt;
}

/// Return the number of octaves.
#[allow(dead_code)]
pub fn turb_octaves(tf: &TurbulenceForce) -> u32 {
    tf.params.octaves
}

/// Return the base frequency.
#[allow(dead_code)]
pub fn turb_frequency(tf: &TurbulenceForce) -> f32 {
    tf.params.frequency
}

/// Return the base amplitude.
#[allow(dead_code)]
pub fn turb_amplitude(tf: &TurbulenceForce) -> f32 {
    tf.params.amplitude
}

/// Return the seed value.
#[allow(dead_code)]
pub fn turb_seed_value(tf: &TurbulenceForce) -> u32 {
    tf.params.seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_turbulence_force() {
        let tf = new_turbulence_force(default_turb_params());
        assert_eq!(turb_octaves(&tf), 4);
    }

    #[test]
    fn test_turbulence_at_returns_vec3() {
        let tf = new_turbulence_force(default_turb_params());
        let f = turbulence_at(&tf, [0.0, 0.0, 0.0]);
        assert_eq!(f.len(), 3);
    }

    #[test]
    fn test_turbulence_nonzero() {
        let tf = new_turbulence_force(default_turb_params());
        let f = turbulence_at(&tf, [1.5, 2.3, 0.7]);
        assert!(f[0].abs() > 0.0 || f[1].abs() > 0.0 || f[2].abs() > 0.0);
    }

    #[test]
    fn test_turbulence_step() {
        let mut tf = new_turbulence_force(default_turb_params());
        turbulence_step(&mut tf, 0.1);
        assert!((tf.time - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_turb_octaves() {
        let tf = new_turbulence_force(TurbParams { octaves: 2, frequency: 1.0, amplitude: 1.0, seed: 0 });
        assert_eq!(turb_octaves(&tf), 2);
    }

    #[test]
    fn test_turb_frequency() {
        let tf = new_turbulence_force(TurbParams { octaves: 1, frequency: 0.5, amplitude: 1.0, seed: 0 });
        assert!((turb_frequency(&tf) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_turb_amplitude() {
        let tf = new_turbulence_force(TurbParams { octaves: 1, frequency: 1.0, amplitude: 3.0, seed: 0 });
        assert!((turb_amplitude(&tf) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_turb_seed_value() {
        let params = TurbParams { octaves: 1, frequency: 1.0, amplitude: 1.0, seed: 777 };
        let tf = new_turbulence_force(params);
        assert_eq!(turb_seed_value(&tf), 777);
    }

    #[test]
    fn test_turb_time_advances() {
        let mut tf = new_turbulence_force(default_turb_params());
        turbulence_step(&mut tf, 1.0);
        turbulence_step(&mut tf, 1.0);
        assert!((tf.time - 2.0).abs() < 1e-6);
    }
}
