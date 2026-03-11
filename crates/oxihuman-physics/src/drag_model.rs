// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Models aerodynamic/hydrodynamic drag forces.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DragModel {
    drag_coefficient: f32,
    cross_section_area: f32,
    fluid_density: f32,
}

#[allow(dead_code)]
impl DragModel {
    pub fn new(drag_coefficient: f32, cross_section_area: f32, fluid_density: f32) -> Self {
        Self {
            drag_coefficient,
            cross_section_area,
            fluid_density,
        }
    }

    pub fn air_at_sea_level(drag_coefficient: f32, area: f32) -> Self {
        Self::new(drag_coefficient, area, 1.225)
    }

    pub fn water(drag_coefficient: f32, area: f32) -> Self {
        Self::new(drag_coefficient, area, 998.0)
    }

    pub fn drag_coefficient(&self) -> f32 {
        self.drag_coefficient
    }

    pub fn drag_magnitude(&self, speed: f32) -> f32 {
        0.5 * self.fluid_density * speed * speed * self.drag_coefficient * self.cross_section_area
    }

    pub fn drag_force(&self, velocity: [f32; 3]) -> [f32; 3] {
        let speed_sq: f32 = velocity.iter().map(|&v| v * v).sum();
        let speed = speed_sq.sqrt();
        if speed < 1e-9 {
            return [0.0; 3];
        }
        let mag = self.drag_magnitude(speed);
        [
            -velocity[0] / speed * mag,
            -velocity[1] / speed * mag,
            -velocity[2] / speed * mag,
        ]
    }

    pub fn terminal_velocity(&self, mass: f32, gravity: f32) -> f32 {
        let denom = self.fluid_density * self.drag_coefficient * self.cross_section_area;
        if denom.abs() < 1e-9 {
            return f32::INFINITY;
        }
        ((2.0 * mass * gravity) / denom).sqrt()
    }

    pub fn reynolds_number(&self, speed: f32, characteristic_length: f32, kinematic_viscosity: f32) -> f32 {
        if kinematic_viscosity.abs() < 1e-12 {
            return 0.0;
        }
        speed * characteristic_length / kinematic_viscosity
    }

    pub fn dynamic_pressure(&self, speed: f32) -> f32 {
        0.5 * self.fluid_density * speed * speed
    }

    pub fn power_required(&self, speed: f32) -> f32 {
        self.drag_magnitude(speed) * speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let d = DragModel::new(0.47, 1.0, 1.225);
        assert!((d.drag_coefficient() - 0.47).abs() < 1e-9);
    }

    #[test]
    fn test_air_at_sea_level() {
        let d = DragModel::air_at_sea_level(0.47, 1.0);
        assert!((d.fluid_density - 1.225).abs() < 1e-6);
    }

    #[test]
    fn test_zero_velocity_no_drag() {
        let d = DragModel::new(0.5, 1.0, 1.0);
        let force = d.drag_force([0.0; 3]);
        assert_eq!(force, [0.0; 3]);
    }

    #[test]
    fn test_drag_magnitude() {
        let d = DragModel::new(1.0, 1.0, 2.0);
        // F = 0.5 * 2.0 * 10*10 * 1.0 * 1.0 = 100
        assert!((d.drag_magnitude(10.0) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_drag_opposes_velocity() {
        let d = DragModel::new(0.5, 1.0, 1.225);
        let force = d.drag_force([10.0, 0.0, 0.0]);
        assert!(force[0] < 0.0);
    }

    #[test]
    fn test_terminal_velocity() {
        let d = DragModel::new(0.47, 0.5, 1.225);
        let vt = d.terminal_velocity(80.0, 9.81);
        assert!(vt > 0.0);
        assert!(vt < 1000.0);
    }

    #[test]
    fn test_dynamic_pressure() {
        let d = DragModel::new(1.0, 1.0, 1.0);
        // q = 0.5 * 1.0 * 5^2 = 12.5
        assert!((d.dynamic_pressure(5.0) - 12.5).abs() < 1e-5);
    }

    #[test]
    fn test_reynolds_number() {
        let d = DragModel::new(1.0, 1.0, 1.0);
        let re = d.reynolds_number(10.0, 1.0, 0.001);
        assert!((re - 10000.0).abs() < 1e-3);
    }

    #[test]
    fn test_power_required() {
        let d = DragModel::new(1.0, 1.0, 1.0);
        let power = d.power_required(10.0);
        let drag = d.drag_magnitude(10.0);
        assert!((power - drag * 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_water_density() {
        let d = DragModel::water(0.5, 1.0);
        assert!((d.fluid_density - 998.0).abs() < 1e-6);
    }
}
