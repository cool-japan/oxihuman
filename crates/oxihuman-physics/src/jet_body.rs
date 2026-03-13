// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Jet body: a rigid body propelled by a reaction jet (rocket/thruster model).

/// Thruster configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThrusterConfig {
    pub max_thrust: f32,      // N
    pub exhaust_vel: f32,     // m/s (Isp * g)
    pub fuel_rate: f32,       // kg/s at full throttle
    pub nozzle_dir: [f32; 2], // unit direction of thrust
}

impl Default for ThrusterConfig {
    fn default() -> Self {
        Self {
            max_thrust: 1000.0,
            exhaust_vel: 2500.0,
            fuel_rate: 0.4,
            nozzle_dir: [0.0, 1.0],
        }
    }
}

/// A jet-propelled body in 2-D.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JetBody {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub dry_mass: f32,
    pub fuel_mass: f32,
    pub thruster: ThrusterConfig,
    pub throttle: f32, // 0..=1
    pub total_impulse: f32,
}

#[allow(dead_code)]
impl JetBody {
    pub fn new(dry_mass: f32, fuel_mass: f32, thruster: ThrusterConfig) -> Self {
        Self {
            pos: [0.0; 2],
            vel: [0.0; 2],
            dry_mass,
            fuel_mass,
            thruster,
            throttle: 0.0,
            total_impulse: 0.0,
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.dry_mass + self.fuel_mass
    }

    pub fn is_out_of_fuel(&self) -> bool {
        self.fuel_mass <= 0.0
    }

    /// Current thrust force.
    pub fn current_thrust(&self) -> f32 {
        if self.is_out_of_fuel() {
            return 0.0;
        }
        self.thruster.max_thrust * self.throttle.clamp(0.0, 1.0)
    }

    /// Specific impulse in seconds.
    pub fn isp(&self) -> f32 {
        self.thruster.exhaust_vel / 9.80665
    }

    /// Integrate one step under thrust and gravity (y is up).
    pub fn step(&mut self, dt: f32, gravity: f32) {
        let thrust = self.current_thrust();
        let mass = self.total_mass();
        let ax = self.thruster.nozzle_dir[0] * thrust / mass;
        let ay = self.thruster.nozzle_dir[1] * thrust / mass - gravity;
        self.vel[0] += ax * dt;
        self.vel[1] += ay * dt;
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
        // Consume fuel
        let fuel_used = self.thruster.fuel_rate * self.throttle * dt;
        self.fuel_mass = (self.fuel_mass - fuel_used).max(0.0);
        self.total_impulse += thrust * dt;
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.total_mass() * (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1])
    }

    /// Delta-v remaining (Tsiolkovsky rocket equation).
    pub fn delta_v_remaining(&self) -> f32 {
        if self.dry_mass <= 0.0 {
            return 0.0;
        }
        let mass_ratio = (self.dry_mass + self.fuel_mass) / self.dry_mass;
        self.thruster.exhaust_vel * mass_ratio.ln()
    }
}

pub fn new_jet_body(dry_mass: f32, fuel_mass: f32, cfg: ThrusterConfig) -> JetBody {
    JetBody::new(dry_mass, fuel_mass, cfg)
}

pub fn jb_set_throttle(body: &mut JetBody, throttle: f32) {
    body.throttle = throttle.clamp(0.0, 1.0);
}

pub fn jb_step(body: &mut JetBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

pub fn jb_thrust(body: &JetBody) -> f32 {
    body.current_thrust()
}

/// Tsiolkovsky delta-v from initial/final mass and exhaust velocity.
pub fn tsiolkovsky_delta_v(ve: f32, m0: f32, m1: f32) -> f32 {
    if m1 <= 0.0 || m0 <= m1 {
        return 0.0;
    }
    ve * (m0 / m1).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_throttle_no_thrust() {
        let b = new_jet_body(10.0, 5.0, ThrusterConfig::default());
        assert_eq!(jb_thrust(&b), 0.0);
    }

    #[test]
    fn full_throttle_max_thrust() {
        let mut b = new_jet_body(10.0, 5.0, ThrusterConfig::default());
        jb_set_throttle(&mut b, 1.0);
        assert!((jb_thrust(&b) - 1000.0).abs() < 1e-4);
    }

    #[test]
    fn thrust_propels_body_upward() {
        let mut b = new_jet_body(10.0, 100.0, ThrusterConfig::default());
        jb_set_throttle(&mut b, 1.0);
        jb_step(&mut b, 0.1, 0.0);
        assert!(b.vel[1] > 0.0);
    }

    #[test]
    fn fuel_consumed_during_burn() {
        let mut b = new_jet_body(10.0, 10.0, ThrusterConfig::default());
        jb_set_throttle(&mut b, 1.0);
        let old_fuel = b.fuel_mass;
        jb_step(&mut b, 0.1, 0.0);
        assert!(b.fuel_mass < old_fuel);
    }

    #[test]
    fn no_thrust_out_of_fuel() {
        let mut b = new_jet_body(10.0, 0.0, ThrusterConfig::default());
        jb_set_throttle(&mut b, 1.0);
        assert_eq!(jb_thrust(&b), 0.0);
    }

    #[test]
    fn total_impulse_accumulates() {
        let mut b = new_jet_body(10.0, 100.0, ThrusterConfig::default());
        jb_set_throttle(&mut b, 1.0);
        jb_step(&mut b, 0.1, 0.0);
        assert!(b.total_impulse > 0.0);
    }

    #[test]
    fn isp_positive() {
        let b = new_jet_body(10.0, 5.0, ThrusterConfig::default());
        assert!(b.isp() > 0.0);
    }

    #[test]
    fn delta_v_remaining_positive_with_fuel() {
        let b = new_jet_body(10.0, 5.0, ThrusterConfig::default());
        assert!(b.delta_v_remaining() > 0.0);
    }

    #[test]
    fn tsiolkovsky_formula() {
        let dv = tsiolkovsky_delta_v(2500.0, 15.0, 10.0);
        assert!(dv > 0.0);
        assert!((dv - 2500.0 * (15.0_f32 / 10.0).ln()).abs() < 0.1);
    }

    #[test]
    fn gravity_pulls_down_without_thrust() {
        let mut b = new_jet_body(10.0, 5.0, ThrusterConfig::default());
        jb_step(&mut b, 1.0, 9.81);
        assert!(b.vel[1] < 0.0);
    }
}
