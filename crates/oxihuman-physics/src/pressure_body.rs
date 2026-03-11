// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Pressure-volume body (inflatable soft body approximation).
#[allow(dead_code)]
pub struct PressureBody {
    pub volume: f32,
    pub target_volume: f32,
    pub pressure_coeff: f32,
    pub surface_area: f32,
    pub stiffness: f32,
    pub mass: f32,
}

#[allow(dead_code)]
impl PressureBody {
    pub fn new(target_volume: f32, pressure_coeff: f32, stiffness: f32, mass: f32) -> Self {
        let surface_area =
            (36.0 * std::f32::consts::PI * target_volume * target_volume).powf(1.0 / 3.0);
        Self {
            volume: target_volume,
            target_volume,
            pressure_coeff,
            surface_area,
            stiffness,
            mass,
        }
    }
    pub fn pressure(&self) -> f32 {
        self.pressure_coeff / self.volume.max(1e-10)
    }
    pub fn outward_force(&self) -> f32 {
        self.pressure() * self.surface_area
    }
    pub fn volume_error(&self) -> f32 {
        self.volume - self.target_volume
    }
    pub fn restoring_force(&self) -> f32 {
        -self.stiffness * self.volume_error()
    }
    pub fn compress(&mut self, delta: f32) {
        self.volume = (self.volume - delta).max(1e-6);
    }
    pub fn expand(&mut self, delta: f32) {
        self.volume += delta;
    }
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.max(1e-6);
    }
    pub fn is_over_pressured(&self, max_pressure: f32) -> bool {
        self.pressure() > max_pressure
    }
    pub fn density(&self) -> f32 {
        self.mass / self.volume.max(1e-10)
    }
    pub fn normalized_volume(&self) -> f32 {
        self.volume / self.target_volume
    }
    pub fn isothermal_work(&self, v_initial: f32, v_final: f32) -> f32 {
        let pv = self.pressure_coeff;
        pv * (v_final / v_initial.max(1e-10)).abs().ln()
    }
}

#[allow(dead_code)]
pub fn new_pressure_body(target_vol: f32, p_coeff: f32, stiffness: f32, mass: f32) -> PressureBody {
    PressureBody::new(target_vol, p_coeff, stiffness, mass)
}
#[allow(dead_code)]
pub fn prb_pressure(b: &PressureBody) -> f32 {
    b.pressure()
}
#[allow(dead_code)]
pub fn prb_outward_force(b: &PressureBody) -> f32 {
    b.outward_force()
}
#[allow(dead_code)]
pub fn prb_volume_error(b: &PressureBody) -> f32 {
    b.volume_error()
}
#[allow(dead_code)]
pub fn prb_restoring_force(b: &PressureBody) -> f32 {
    b.restoring_force()
}
#[allow(dead_code)]
pub fn prb_compress(b: &mut PressureBody, delta: f32) {
    b.compress(delta);
}
#[allow(dead_code)]
pub fn prb_expand(b: &mut PressureBody, delta: f32) {
    b.expand(delta);
}
#[allow(dead_code)]
pub fn prb_density(b: &PressureBody) -> f32 {
    b.density()
}
#[allow(dead_code)]
pub fn prb_normalized_volume(b: &PressureBody) -> f32 {
    b.normalized_volume()
}
#[allow(dead_code)]
pub fn prb_is_over_pressured(b: &PressureBody, max_p: f32) -> bool {
    b.is_over_pressured(max_p)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pressure_positive() {
        let b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        assert!(prb_pressure(&b) > 0.0);
    }
    #[test]
    fn test_volume_error_zero_at_rest() {
        let b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        assert!((prb_volume_error(&b)).abs() < 1e-5);
    }
    #[test]
    fn test_compress_reduces_volume() {
        let mut b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        prb_compress(&mut b, 0.1);
        assert!(b.volume < 1.0);
    }
    #[test]
    fn test_expand_increases_volume() {
        let mut b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        prb_expand(&mut b, 0.5);
        assert!(b.volume > 1.0);
    }
    #[test]
    fn test_pressure_increases_when_compressed() {
        let mut b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        let p0 = prb_pressure(&b);
        prb_compress(&mut b, 0.5);
        let p1 = prb_pressure(&b);
        assert!(p1 > p0);
    }
    #[test]
    fn test_density() {
        let b = new_pressure_body(2.0, 1.0, 1.0, 4.0);
        assert!((prb_density(&b) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_normalized_volume_one_at_rest() {
        let b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        assert!((prb_normalized_volume(&b) - 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_restoring_force_zero_at_rest() {
        let b = new_pressure_body(1.0, 10.0, 5.0, 1.0);
        assert!((prb_restoring_force(&b)).abs() < 1e-5);
    }
    #[test]
    fn test_over_pressured() {
        let mut b = new_pressure_body(1.0, 100.0, 1.0, 1.0);
        prb_compress(&mut b, 0.95);
        assert!(prb_is_over_pressured(&b, 10.0));
    }
    #[test]
    fn test_outward_force_positive() {
        let b = new_pressure_body(1.0, 10.0, 1.0, 1.0);
        assert!(prb_outward_force(&b) > 0.0);
    }
}
