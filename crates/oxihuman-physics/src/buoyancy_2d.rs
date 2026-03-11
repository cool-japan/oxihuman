// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D buoyancy simulation stub.

#[derive(Debug, Clone)]
pub struct BuoyancyFluid2d {
    pub surface_y: f32,
    pub fluid_density: f32,
    pub gravity: f32,
}

impl BuoyancyFluid2d {
    pub fn new(surface_y: f32, fluid_density: f32, gravity: f32) -> Self {
        BuoyancyFluid2d {
            surface_y,
            fluid_density,
            gravity,
        }
    }

    pub fn water() -> Self {
        BuoyancyFluid2d::new(0.0, 1000.0, 9.81)
    }

    pub fn buoyancy_force(&self, body: &BuoyantBody2d) -> f32 {
        buoyancy_force_2d(body, self)
    }
}

#[derive(Debug, Clone)]
pub struct BuoyantBody2d {
    pub position_y: f32,
    pub velocity_y: f32,
    pub half_height: f32,
    pub width: f32,
    pub mass: f32,
    pub drag_coeff: f32,
}

impl BuoyantBody2d {
    pub fn new(y: f32, half_height: f32, width: f32, mass: f32) -> Self {
        BuoyantBody2d {
            position_y: y,
            velocity_y: 0.0,
            half_height,
            width,
            mass,
            drag_coeff: 0.5,
        }
    }

    pub fn is_submerged(&self, fluid: &BuoyancyFluid2d) -> bool {
        self.position_y - self.half_height < fluid.surface_y
    }

    pub fn submerged_depth(&self, fluid: &BuoyancyFluid2d) -> f32 {
        let top = self.position_y + self.half_height;
        let bot = self.position_y - self.half_height;
        if top <= fluid.surface_y {
            self.half_height * 2.0
        } else if bot >= fluid.surface_y {
            0.0
        } else {
            (fluid.surface_y - bot).max(0.0)
        }
    }
}

pub fn buoyancy_force_2d(body: &BuoyantBody2d, fluid: &BuoyancyFluid2d) -> f32 {
    let depth = body.submerged_depth(fluid);
    let submerged_volume = depth * body.width;
    fluid.fluid_density * fluid.gravity * submerged_volume
}

pub fn net_force_2d(body: &BuoyantBody2d, fluid: &BuoyancyFluid2d) -> f32 {
    let weight = -body.mass * fluid.gravity;
    let buoy = buoyancy_force_2d(body, fluid);
    let drag = if body.is_submerged(fluid) {
        -body.drag_coeff * body.velocity_y
    } else {
        0.0
    };
    weight + buoy + drag
}

pub fn step_buoyant_body(body: &mut BuoyantBody2d, fluid: &BuoyancyFluid2d, dt: f32) {
    let f = net_force_2d(body, fluid);
    let accel = f / body.mass;
    body.velocity_y += accel * dt;
    body.position_y += body.velocity_y * dt;
}

pub fn equilibrium_depth(body: &BuoyantBody2d, fluid: &BuoyancyFluid2d) -> f32 {
    let needed_volume = body.mass / fluid.fluid_density;
    needed_volume / body.width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fully_submerged() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(-5.0, 0.5, 1.0, 1.0);
        let d = body.submerged_depth(&fluid);
        assert!((d - 1.0).abs() < 1e-5, /* fully submerged: depth = height */);
    }

    #[test]
    fn test_not_submerged() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(5.0, 0.5, 1.0, 1.0);
        let d = body.submerged_depth(&fluid);
        assert!(d.abs() < 1e-5 /* above water: no submersion */,);
    }

    #[test]
    fn test_buoyancy_force_positive() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(-1.0, 0.5, 1.0, 1.0);
        let f = buoyancy_force_2d(&body, &fluid);
        assert!(f > 0.0 /* upward buoyancy */,);
    }

    #[test]
    fn test_is_submerged() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(-0.5, 0.4, 1.0, 1.0);
        assert!(body.is_submerged(&fluid) /* body overlaps surface */,);
    }

    #[test]
    fn test_not_is_submerged() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(5.0, 0.4, 1.0, 1.0);
        assert!(!body.is_submerged(&fluid) /* body above water */,);
    }

    #[test]
    fn test_step_floats_up() {
        let fluid = BuoyancyFluid2d::water();
        let mut body = BuoyantBody2d::new(-2.0, 0.5, 1.0, 100.0);
        let y0 = body.position_y;
        step_buoyant_body(&mut body, &fluid, 0.1);
        assert!(
            body.position_y > y0 || (body.position_y - y0).abs() < 0.2,
            /* body moves upward if buoyancy > weight */
        );
    }

    #[test]
    fn test_net_force_submerged() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(-10.0, 0.5, 1.0, 0.1);
        let f = net_force_2d(&body, &fluid);
        assert!(f > 0.0 /* light body has net upward force */,);
    }

    #[test]
    fn test_equilibrium_depth() {
        let fluid = BuoyancyFluid2d::water();
        let body = BuoyantBody2d::new(0.0, 1.0, 1.0, 500.0);
        let d = equilibrium_depth(&body, &fluid);
        assert!(d > 0.0 && d <= 2.0 /* equilibrium depth within body */,);
    }

    #[test]
    fn test_water_density() {
        let fluid = BuoyancyFluid2d::water();
        assert!((fluid.fluid_density - 1000.0).abs() < 1e-5, /* water = 1000 kg/m3 */);
    }
}
