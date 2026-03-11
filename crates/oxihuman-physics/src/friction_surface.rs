// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Surface friction model: static/dynamic friction with direction.

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FrictionSurface {
    pub static_coeff: f32,
    pub dynamic_coeff: f32,
}

#[allow(dead_code)]
impl FrictionSurface {
    pub fn new(static_coeff: f32, dynamic_coeff: f32) -> Self {
        Self {
            static_coeff: static_coeff.max(0.0),
            dynamic_coeff: dynamic_coeff.max(0.0).min(static_coeff),
        }
    }

    pub fn ice() -> Self { Self::new(0.05, 0.03) }
    pub fn wood() -> Self { Self::new(0.5, 0.3) }
    pub fn rubber() -> Self { Self::new(1.0, 0.7) }
    pub fn steel() -> Self { Self::new(0.6, 0.4) }

    pub fn max_static_force(&self, normal_force: f32) -> f32 {
        self.static_coeff * normal_force.abs()
    }

    pub fn dynamic_force(&self, normal_force: f32) -> f32 {
        self.dynamic_coeff * normal_force.abs()
    }

    pub fn compute_friction(
        &self,
        tangent_velocity: [f32; 3],
        normal_force: f32,
    ) -> [f32; 3] {
        let speed = vec3_len(tangent_velocity);
        if speed < 1e-8 {
            return [0.0; 3];
        }
        let direction = vec3_scale(tangent_velocity, -1.0 / speed);
        let force_mag = if speed < 0.01 {
            // static regime
            self.max_static_force(normal_force).min(speed * 1000.0)
        } else {
            self.dynamic_force(normal_force)
        };
        vec3_scale(direction, force_mag)
    }

    pub fn combine(a: &FrictionSurface, b: &FrictionSurface) -> FrictionSurface {
        FrictionSurface {
            static_coeff: (a.static_coeff * b.static_coeff).sqrt(),
            dynamic_coeff: (a.dynamic_coeff * b.dynamic_coeff).sqrt(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let f = FrictionSurface::new(0.5, 0.3);
        assert!((f.static_coeff - 0.5).abs() < 1e-5);
        assert!((f.dynamic_coeff - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_presets() {
        let ice = FrictionSurface::ice();
        let rubber = FrictionSurface::rubber();
        assert!(ice.static_coeff < rubber.static_coeff);
    }

    #[test]
    fn test_max_static_force() {
        let f = FrictionSurface::new(0.5, 0.3);
        assert!((f.max_static_force(10.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_dynamic_force() {
        let f = FrictionSurface::new(0.5, 0.3);
        assert!((f.dynamic_force(10.0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_friction_zero_velocity() {
        let f = FrictionSurface::new(0.5, 0.3);
        let result = f.compute_friction([0.0, 0.0, 0.0], 10.0);
        assert!(vec3_len(result) < 1e-5);
    }

    #[test]
    fn test_friction_direction() {
        let f = FrictionSurface::new(0.5, 0.3);
        let result = f.compute_friction([1.0, 0.0, 0.0], 10.0);
        assert!(result[0] < 0.0); // opposing direction
    }

    #[test]
    fn test_combine() {
        let a = FrictionSurface::new(0.5, 0.3);
        let b = FrictionSurface::new(0.5, 0.3);
        let c = FrictionSurface::combine(&a, &b);
        assert!((c.static_coeff - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_negative_coeff_clamped() {
        let f = FrictionSurface::new(-1.0, -0.5);
        assert!((f.static_coeff).abs() < 1e-10);
    }

    #[test]
    fn test_dynamic_clamped_to_static() {
        let f = FrictionSurface::new(0.3, 0.5); // dynamic > static
        assert!(f.dynamic_coeff <= f.static_coeff);
    }

    #[test]
    fn test_steel() {
        let s = FrictionSurface::steel();
        assert!(s.static_coeff > s.dynamic_coeff);
    }
}
