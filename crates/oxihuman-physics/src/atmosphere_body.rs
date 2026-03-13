// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Atmosphere model: density, pressure and temperature as functions of altitude.

/// ISA (International Standard Atmosphere) constants.
pub const SEA_LEVEL_DENSITY: f32 = 1.225; // kg/m³
pub const SEA_LEVEL_PRESSURE: f32 = 101_325.0; // Pa
pub const SEA_LEVEL_TEMP: f32 = 288.15; // K
pub const LAPSE_RATE: f32 = 0.0065; // K/m
pub const GAS_CONSTANT: f32 = 287.05; // J/(kg·K)
pub const GRAVITY: f32 = 9.80665; // m/s²

/// Atmosphere layer type.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AtmoLayer {
    Troposphere,
    Stratosphere,
    Mesosphere,
}

/// An atmosphere model body tracking a physical object's altitude properties.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtmosphereBody {
    pub altitude: f32,
    pub vertical_vel: f32,
    pub mass: f32,
}

#[allow(dead_code)]
impl AtmosphereBody {
    pub fn new(mass: f32) -> Self {
        Self {
            altitude: 0.0,
            vertical_vel: 0.0,
            mass,
        }
    }

    pub fn layer(&self) -> AtmoLayer {
        if self.altitude < 11_000.0 {
            AtmoLayer::Troposphere
        } else if self.altitude < 47_000.0 {
            AtmoLayer::Stratosphere
        } else {
            AtmoLayer::Mesosphere
        }
    }

    pub fn temperature(&self) -> f32 {
        temperature_at(self.altitude)
    }

    pub fn pressure(&self) -> f32 {
        pressure_at(self.altitude)
    }

    pub fn density(&self) -> f32 {
        density_at(self.altitude)
    }

    pub fn step(&mut self, dt: f32, net_force: f32) {
        let accel = net_force / self.mass - GRAVITY;
        self.vertical_vel += accel * dt;
        self.altitude = (self.altitude + self.vertical_vel * dt).max(0.0);
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.vertical_vel * self.vertical_vel
    }

    pub fn potential_energy(&self) -> f32 {
        self.mass * GRAVITY * self.altitude
    }
}

/// Temperature (K) at given altitude (m), simplified ISA troposphere.
pub fn temperature_at(altitude: f32) -> f32 {
    let alt = altitude.clamp(0.0, 11_000.0);
    SEA_LEVEL_TEMP - LAPSE_RATE * alt
}

/// Pressure (Pa) at given altitude using barometric formula.
pub fn pressure_at(altitude: f32) -> f32 {
    let alt = altitude.max(0.0);
    let temp = temperature_at(alt);
    let exponent = GRAVITY / (GAS_CONSTANT * LAPSE_RATE);
    SEA_LEVEL_PRESSURE * (temp / SEA_LEVEL_TEMP).powf(exponent)
}

/// Air density (kg/m³) at given altitude via ideal gas law.
pub fn density_at(altitude: f32) -> f32 {
    let p = pressure_at(altitude);
    let t = temperature_at(altitude);
    p / (GAS_CONSTANT * t)
}

/// Speed of sound (m/s) at given altitude.
pub fn speed_of_sound(altitude: f32) -> f32 {
    let gamma = 1.4_f32;
    let t = temperature_at(altitude);
    (gamma * GAS_CONSTANT * t).sqrt()
}

/// Mach number for a given speed at altitude.
pub fn mach_number(speed: f32, altitude: f32) -> f32 {
    let sos = speed_of_sound(altitude);
    if sos < 1e-6 {
        return 0.0;
    }
    speed / sos
}

pub fn new_atmosphere_body(mass: f32) -> AtmosphereBody {
    AtmosphereBody::new(mass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sea_level_temperature() {
        assert!((temperature_at(0.0) - SEA_LEVEL_TEMP).abs() < 1e-4);
    }

    #[test]
    fn temperature_decreases_with_altitude() {
        assert!(temperature_at(5000.0) < temperature_at(0.0));
    }

    #[test]
    fn sea_level_pressure() {
        assert!((pressure_at(0.0) - SEA_LEVEL_PRESSURE).abs() < 1.0);
    }

    #[test]
    fn pressure_decreases_with_altitude() {
        assert!(pressure_at(1000.0) < pressure_at(0.0));
    }

    #[test]
    fn density_positive() {
        assert!(density_at(0.0) > 0.0);
        assert!(density_at(5000.0) > 0.0);
    }

    #[test]
    fn speed_of_sound_positive() {
        assert!(speed_of_sound(0.0) > 300.0);
    }

    #[test]
    fn mach_one_at_sound_speed() {
        let sos = speed_of_sound(0.0);
        assert!((mach_number(sos, 0.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn body_layer_troposphere() {
        let b = new_atmosphere_body(1.0);
        assert_eq!(b.layer(), AtmoLayer::Troposphere);
    }

    #[test]
    fn body_gravity_pulls_down() {
        let mut b = new_atmosphere_body(1.0);
        b.altitude = 100.0;
        b.step(1.0, 0.0);
        assert!(b.vertical_vel < 0.0);
    }

    #[test]
    fn potential_energy_proportional_to_altitude() {
        let b1 = AtmosphereBody {
            altitude: 10.0,
            vertical_vel: 0.0,
            mass: 1.0,
        };
        let b2 = AtmosphereBody {
            altitude: 20.0,
            vertical_vel: 0.0,
            mass: 1.0,
        };
        assert!((b2.potential_energy() - 2.0 * b1.potential_energy()).abs() < 1e-4);
    }
}
