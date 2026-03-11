// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ice body: a rigid body on an icy surface with low friction and thermal melting.

/// Thermal state of the ice surface.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum IceState {
    Solid,
    Melting,
    Puddle,
}

/// An ice body simulating sliding with low friction and temperature-dependent behavior.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IceBody {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub mass: f32,
    pub temperature: f32, // °C, 0 = melting point
    pub friction_static: f32,
    pub friction_kinetic: f32,
    pub thermal_conductivity: f32,
    pub state: IceState,
}

#[allow(dead_code)]
impl IceBody {
    pub fn new(mass: f32) -> Self {
        Self {
            pos: [0.0; 2],
            vel: [0.0; 2],
            mass,
            temperature: -5.0,
            friction_static: 0.03,
            friction_kinetic: 0.01,
            thermal_conductivity: 2.18,
            state: IceState::Solid,
        }
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt()
    }

    /// Compute friction force magnitude (coulomb friction).
    pub fn friction_force(&self, normal_force: f32) -> f32 {
        let mu = if self.speed() > 1e-4 {
            self.friction_kinetic
        } else {
            self.friction_static
        };
        // Warm ice → lower friction
        let temp_factor = if self.temperature >= 0.0 { 0.5 } else { 1.0 };
        mu * normal_force * temp_factor
    }

    /// Integrate one step on a flat surface under applied force.
    pub fn step(&mut self, dt: f32, force_x: f32, gravity: f32) {
        let normal = self.mass * gravity;
        let fric = self.friction_force(normal);
        let speed = self.speed();
        let fric_x = if speed > 1e-6 {
            -fric * self.vel[0] / speed
        } else {
            -force_x.signum() * fric
        };
        let ax = (force_x + fric_x) / self.mass;
        self.vel[0] += ax * dt;
        // Clamp to zero if sliding has stopped
        if force_x.abs() < fric && self.vel[0].abs() < 0.01 {
            self.vel[0] = 0.0;
        }
        self.pos[0] += self.vel[0] * dt;
    }

    /// Apply heat (positive = warming, negative = cooling).
    pub fn apply_heat(&mut self, dt: f32, ambient_temp: f32) {
        let diff = ambient_temp - self.temperature;
        self.temperature += self.thermal_conductivity * diff * dt * 0.01;
        self.state = if self.temperature >= 0.0 {
            IceState::Melting
        } else {
            IceState::Solid
        };
    }

    pub fn is_melting(&self) -> bool {
        self.state == IceState::Melting
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1])
    }

    /// Friction coefficient (kinetic) at current temperature.
    pub fn effective_friction(&self) -> f32 {
        if self.temperature >= 0.0 {
            self.friction_kinetic * 0.5
        } else {
            self.friction_kinetic
        }
    }
}

pub fn new_ice_body(mass: f32) -> IceBody {
    IceBody::new(mass)
}

pub fn ib_step(body: &mut IceBody, dt: f32, force_x: f32, gravity: f32) {
    body.step(dt, force_x, gravity);
}

pub fn ib_heat(body: &mut IceBody, dt: f32, ambient: f32) {
    body.apply_heat(dt, ambient);
}

/// Heat of fusion for ice (J/kg).
pub const ICE_HEAT_OF_FUSION: f32 = 334_000.0;

/// Melting point of ice (°C).
pub const ICE_MELTING_POINT: f32 = 0.0;

/// Thermal conductivity of ice (W/m·K).
pub const ICE_THERMAL_CONDUCTIVITY: f32 = 2.18;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_body_solid() {
        let b = new_ice_body(1.0);
        assert_eq!(b.state, IceState::Solid);
    }

    #[test]
    fn step_with_force_moves_body() {
        let mut b = new_ice_body(1.0);
        let old_x = b.pos[0];
        ib_step(&mut b, 0.1, 10.0, 9.81);
        assert!(b.pos[0] > old_x);
    }

    #[test]
    fn heating_changes_temperature() {
        let mut b = new_ice_body(1.0);
        let old_temp = b.temperature;
        ib_heat(&mut b, 1.0, 20.0);
        assert!(b.temperature > old_temp);
    }

    #[test]
    fn melting_at_zero_degrees() {
        let mut b = new_ice_body(1.0);
        b.temperature = -0.001;
        ib_heat(&mut b, 100.0, 20.0);
        // May have warmed past 0
        if b.temperature >= 0.0 {
            assert!(b.is_melting());
        }
    }

    #[test]
    fn friction_force_positive() {
        let b = new_ice_body(1.0);
        assert!(b.friction_force(9.81) > 0.0);
    }

    #[test]
    fn warm_ice_has_lower_friction() {
        let mut cold = new_ice_body(1.0);
        cold.temperature = -5.0;
        let mut warm = new_ice_body(1.0);
        warm.temperature = 1.0;
        let n = 9.81 * 1.0;
        assert!(warm.friction_force(n) < cold.friction_force(n));
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let b = new_ice_body(2.0);
        assert_eq!(b.kinetic_energy(), 0.0);
    }

    #[test]
    fn speed_zero_at_rest() {
        let b = new_ice_body(1.0);
        assert_eq!(b.speed(), 0.0);
    }

    #[test]
    fn heat_of_fusion_constant() {
        let val: f32 = ICE_HEAT_OF_FUSION;
        assert!(val > 0.0);
    }

    #[test]
    fn effective_friction_lower_warm() {
        let mut b = new_ice_body(1.0);
        b.temperature = 1.0;
        assert!(b.effective_friction() < b.friction_kinetic);
    }
}
