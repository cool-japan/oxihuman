// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Aerodynamic drag model — drag force as a function of speed and shape.

/// Common drag coefficient presets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragShape {
    Sphere,
    Cylinder,
    FlatPlate,
    Streamlined,
    Custom(f64),
}

impl DragShape {
    /// Drag coefficient Cd for the shape.
    pub fn cd(&self) -> f64 {
        match self {
            DragShape::Sphere => 0.47,
            DragShape::Cylinder => 1.0,
            DragShape::FlatPlate => 1.28,
            DragShape::Streamlined => 0.04,
            DragShape::Custom(cd) => *cd,
        }
    }
}

/// Drag force magnitude: F_drag = 0.5 * ρ * v² * Cd * A.
pub fn drag_force(density: f64, velocity: f64, cd: f64, area: f64) -> f64 {
    0.5 * density * velocity * velocity * cd * area
}

/// Drag deceleration: a = F_drag / m.
pub fn drag_deceleration(density: f64, velocity: f64, cd: f64, area: f64, mass: f64) -> f64 {
    if mass < 1e-30 { return 0.0; }
    drag_force(density, velocity, cd, area) / mass
}

/// Terminal velocity: v_t = sqrt(2mg / (ρ * Cd * A)).
pub fn terminal_velocity(mass: f64, g: f64, density: f64, cd: f64, area: f64) -> f64 {
    let denom = density * cd * area;
    if denom < 1e-30 { return f64::INFINITY; }
    (2.0 * mass * g / denom).sqrt()
}

/// Reynolds number: Re = ρ * v * L / μ.
pub fn reynolds_number(density: f64, velocity: f64, length: f64, viscosity: f64) -> f64 {
    if viscosity < 1e-30 { return f64::INFINITY; }
    density * velocity * length / viscosity
}

/// Drag coefficient from Reynolds number (Stokes regime: Re << 1).
pub fn cd_stokes(reynolds: f64) -> f64 {
    if reynolds < 1e-12 { return f64::INFINITY; }
    24.0 / reynolds
}

/// A drag model combining shape and medium properties.
#[derive(Debug, Clone)]
pub struct DragModel {
    pub shape: DragShape,
    pub reference_area: f64,
    pub fluid_density: f64,
}

impl DragModel {
    pub fn new(shape: DragShape, area: f64, fluid_density: f64) -> Self {
        DragModel { shape, reference_area: area, fluid_density }
    }

    /// Drag force at speed `v`.
    pub fn force_at(&self, v: f64) -> f64 {
        drag_force(self.fluid_density, v, self.shape.cd(), self.reference_area)
    }

    /// Terminal velocity for mass `m` under gravity `g`.
    pub fn terminal_velocity(&self, m: f64, g: f64) -> f64 {
        terminal_velocity(m, g, self.fluid_density, self.shape.cd(), self.reference_area)
    }
}

/// Create a new drag model.
pub fn new_drag_model(shape: DragShape, area: f64, fluid_density: f64) -> DragModel {
    DragModel::new(shape, area, fluid_density)
}

/// Drag force at speed.
pub fn dm_force_at(model: &DragModel, v: f64) -> f64 {
    model.force_at(v)
}

/// Terminal velocity.
pub fn dm_terminal_velocity(model: &DragModel, mass: f64, g: f64) -> f64 {
    model.terminal_velocity(mass, g)
}

/// Cd for a shape.
pub fn dm_cd(shape: DragShape) -> f64 {
    shape.cd()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_cd() {
        assert!((dm_cd(DragShape::Sphere) - 0.47).abs() < 1e-9 /* sphere Cd = 0.47 */);
    }

    #[test]
    fn test_flat_plate_cd() {
        assert!((dm_cd(DragShape::FlatPlate) - 1.28).abs() < 1e-9 /* flat plate */);
    }

    #[test]
    fn test_custom_cd() {
        assert!((dm_cd(DragShape::Custom(0.33)) - 0.33).abs() < 1e-9 /* custom */);
    }

    #[test]
    fn test_drag_force_increases_with_speed() {
        let f1 = drag_force(1.225, 10.0, 0.47, 0.01);
        let f2 = drag_force(1.225, 20.0, 0.47, 0.01);
        assert!(f2 > f1 /* higher speed → more drag */);
    }

    #[test]
    fn test_drag_force_zero_speed() {
        let f = drag_force(1.225, 0.0, 0.47, 0.01);
        assert_eq!(f, 0.0 /* no motion → no drag */);
    }

    #[test]
    fn test_terminal_velocity_positive() {
        let v = terminal_velocity(1.0, 9.81, 1.225, 0.47, 0.01);
        assert!(v > 0.0 /* positive terminal velocity */);
    }

    #[test]
    fn test_terminal_velocity_heavier_object() {
        let v1 = terminal_velocity(1.0, 9.81, 1.225, 0.47, 0.01);
        let v2 = terminal_velocity(10.0, 9.81, 1.225, 0.47, 0.01);
        assert!(v2 > v1 /* heavier → faster terminal velocity */);
    }

    #[test]
    fn test_reynolds_number() {
        let re = reynolds_number(1.225, 10.0, 0.1, 1.81e-5);
        assert!(re > 0.0 /* positive Re */);
    }

    #[test]
    fn test_stokes_drag_high_re() {
        let cd = cd_stokes(100.0);
        assert!((cd - 0.24).abs() < 1e-9 /* 24/100 = 0.24 */);
    }

    #[test]
    fn test_drag_model_force() {
        let m = new_drag_model(DragShape::Sphere, 0.01, 1.225);
        let f = dm_force_at(&m, 10.0);
        assert!(f > 0.0 /* positive drag force */);
    }

    #[test]
    fn test_drag_model_terminal_velocity() {
        let m = new_drag_model(DragShape::Sphere, 0.01, 1.225);
        let vt = dm_terminal_velocity(&m, 0.1, 9.81);
        assert!(vt > 0.0 /* positive terminal velocity */);
    }
}
