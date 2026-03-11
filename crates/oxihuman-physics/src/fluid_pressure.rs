// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Static fluid pressure field (hydrostatics).

/// A static fluid pressure field.
#[derive(Debug, Clone)]
pub struct FluidPressureField {
    /// Fluid density (kg/m³).
    pub density: f64,
    /// Gravitational acceleration (m/s²).
    pub gravity: f64,
    /// Reference pressure at y=0 (Pa).
    pub p0: f64,
    /// Free surface height (y-coordinate, m).
    pub surface_y: f64,
}

impl FluidPressureField {
    /// Create a new pressure field.
    pub fn new(density: f64, gravity: f64, p0: f64, surface_y: f64) -> Self {
        FluidPressureField { density, gravity, p0, surface_y }
    }

    /// Pressure at height `y` (Pa).  Above the surface, returns `p0`.
    pub fn pressure_at(&self, y: f64) -> f64 {
        if y >= self.surface_y {
            return self.p0;
        }
        let depth = self.surface_y - y;
        self.p0 + self.density * self.gravity * depth
    }

    /// Buoyant force on a submerged volume `v` (m³).
    pub fn buoyant_force(&self, v: f64) -> f64 {
        self.density * self.gravity * v
    }

    /// Depth of point `y` below the surface (clamped to 0 if above surface).
    pub fn depth_at(&self, y: f64) -> f64 {
        (self.surface_y - y).max(0.0)
    }

    /// Pressure gradient (dp/dy = -rho * g).
    pub fn pressure_gradient(&self) -> f64 {
        -self.density * self.gravity
    }

    /// True if the point at `y` is submerged.
    pub fn is_submerged(&self, y: f64) -> bool {
        y < self.surface_y
    }
}

/// Create a standard water pressure field.
pub fn new_water_pressure_field(surface_y: f64) -> FluidPressureField {
    FluidPressureField::new(1000.0, 9.81, 101_325.0, surface_y)
}

/// Create a custom pressure field.
pub fn new_fluid_pressure_field(
    density: f64,
    gravity: f64,
    p0: f64,
    surface_y: f64,
) -> FluidPressureField {
    FluidPressureField::new(density, gravity, p0, surface_y)
}

/// Pressure at height y.
pub fn fp_pressure_at(field: &FluidPressureField, y: f64) -> f64 {
    field.pressure_at(y)
}

/// Buoyant force.
pub fn fp_buoyant_force(field: &FluidPressureField, volume: f64) -> f64 {
    field.buoyant_force(volume)
}

/// Depth at y.
pub fn fp_depth_at(field: &FluidPressureField, y: f64) -> f64 {
    field.depth_at(y)
}

/// Is submerged?
pub fn fp_is_submerged(field: &FluidPressureField, y: f64) -> bool {
    field.is_submerged(y)
}

/// Pressure gradient.
pub fn fp_pressure_gradient(field: &FluidPressureField) -> f64 {
    field.pressure_gradient()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressure_at_surface() {
        let f = new_water_pressure_field(0.0);
        assert!((fp_pressure_at(&f, 0.0) - 101_325.0).abs() < 0.1 /* at surface = p0 */);
    }

    #[test]
    fn test_pressure_increases_with_depth() {
        let f = new_water_pressure_field(0.0);
        assert!(fp_pressure_at(&f, -10.0) > fp_pressure_at(&f, -1.0) /* deeper = higher pressure */);
    }

    #[test]
    fn test_pressure_above_surface() {
        let f = new_water_pressure_field(0.0);
        assert!((fp_pressure_at(&f, 5.0) - 101_325.0).abs() < 0.1 /* above surface = p0 */);
    }

    #[test]
    fn test_buoyant_force() {
        let f = new_water_pressure_field(0.0);
        let fb = fp_buoyant_force(&f, 1.0); /* 1 m³ */
        assert!((fb - 1000.0 * 9.81).abs() < 0.1 /* rho*g*V */);
    }

    #[test]
    fn test_depth_at() {
        let f = new_water_pressure_field(10.0);
        assert!((fp_depth_at(&f, 7.0) - 3.0).abs() < 1e-9 /* 10-7=3 m */);
    }

    #[test]
    fn test_depth_above_surface_is_zero() {
        let f = new_water_pressure_field(0.0);
        assert_eq!(fp_depth_at(&f, 5.0), 0.0 /* above surface: depth=0 */);
    }

    #[test]
    fn test_is_submerged() {
        let f = new_water_pressure_field(0.0);
        assert!(fp_is_submerged(&f, -1.0) /* below surface */);
        assert!(!fp_is_submerged(&f, 1.0) /* above surface */);
    }

    #[test]
    fn test_pressure_gradient_negative() {
        let f = new_water_pressure_field(0.0);
        assert!(fp_pressure_gradient(&f) < 0.0 /* pressure decreases upward */);
    }

    #[test]
    fn test_10m_depth_pressure() {
        let f = new_water_pressure_field(0.0);
        let p = fp_pressure_at(&f, -10.0);
        /* p0 + rho*g*10 ≈ 101325 + 98100 */
        assert!((p - 199_425.0).abs() < 1.0 /* 10 m depth */);
    }

    #[test]
    fn test_custom_fluid() {
        let f = new_fluid_pressure_field(800.0, 9.81, 0.0, 0.0);
        let p = fp_pressure_at(&f, -1.0);
        assert!((p - 800.0 * 9.81).abs() < 0.1 /* oil at 1 m depth */);
    }
}
