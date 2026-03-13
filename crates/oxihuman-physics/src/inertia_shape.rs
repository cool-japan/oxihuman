// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Moment of inertia calculations for common geometric shapes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaShape {
    pub ixx: f32,
    pub iyy: f32,
    pub izz: f32,
}

#[allow(dead_code)]
impl InertiaShape {
    pub fn new(ixx: f32, iyy: f32, izz: f32) -> Self {
        Self { ixx, iyy, izz }
    }

    pub fn diagonal(&self) -> [f32; 3] {
        [self.ixx, self.iyy, self.izz]
    }

    /// Solid sphere: I = (2/5) * m * r^2
    pub fn solid_sphere(mass: f32, radius: f32) -> Self {
        let i = 0.4 * mass * radius * radius;
        Self::new(i, i, i)
    }

    /// Hollow sphere: I = (2/3) * m * r^2
    pub fn hollow_sphere(mass: f32, radius: f32) -> Self {
        let i = (2.0 / 3.0) * mass * radius * radius;
        Self::new(i, i, i)
    }

    /// Solid cylinder (aligned along Y): Ixx=Izz = m*(3r^2+h^2)/12, Iyy = m*r^2/2
    pub fn solid_cylinder(mass: f32, radius: f32, height: f32) -> Self {
        let iyy = 0.5 * mass * radius * radius;
        let ixx = mass * (3.0 * radius * radius + height * height) / 12.0;
        Self::new(ixx, iyy, ixx)
    }

    /// Solid box: Ixx = m*(h^2+d^2)/12, etc.
    pub fn solid_box(mass: f32, width: f32, height: f32, depth: f32) -> Self {
        let k = mass / 12.0;
        Self::new(
            k * (height * height + depth * depth),
            k * (width * width + depth * depth),
            k * (width * width + height * height),
        )
    }

    /// Thin rod along X: Iyy=Izz = m*L^2/12, Ixx ≈ 0
    pub fn thin_rod(mass: f32, length: f32) -> Self {
        let i = mass * length * length / 12.0;
        Self::new(0.0, i, i)
    }

    /// Scale all components by a factor.
    pub fn scale(&self, factor: f32) -> Self {
        Self::new(self.ixx * factor, self.iyy * factor, self.izz * factor)
    }

    /// Add two inertia tensors (for composite bodies).
    pub fn add(&self, other: &InertiaShape) -> Self {
        Self::new(
            self.ixx + other.ixx,
            self.iyy + other.iyy,
            self.izz + other.izz,
        )
    }

    /// Parallel axis theorem: I' = I + m*d^2
    pub fn parallel_axis(&self, mass: f32, offset: [f32; 3]) -> Self {
        let dx2 = offset[0] * offset[0];
        let dy2 = offset[1] * offset[1];
        let dz2 = offset[2] * offset[2];
        Self::new(
            self.ixx + mass * (dy2 + dz2),
            self.iyy + mass * (dx2 + dz2),
            self.izz + mass * (dx2 + dy2),
        )
    }

    /// Inverse inertia (for angular acceleration computation).
    pub fn inverse(&self) -> Self {
        let inv = |v: f32| if v.abs() > 1e-10 { 1.0 / v } else { 0.0 };
        Self::new(inv(self.ixx), inv(self.iyy), inv(self.izz))
    }

    /// Total (trace).
    pub fn trace(&self) -> f32 {
        self.ixx + self.iyy + self.izz
    }
}

/// Compute rotational kinetic energy: 0.5 * I * omega^2 for each axis.
#[allow(dead_code)]
pub fn rotational_kinetic_energy(inertia: &InertiaShape, omega: [f32; 3]) -> f32 {
    0.5 * (inertia.ixx * omega[0] * omega[0]
        + inertia.iyy * omega[1] * omega[1]
        + inertia.izz * omega[2] * omega[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_sphere() {
        let i = InertiaShape::solid_sphere(10.0, 1.0);
        assert!((i.ixx - 4.0).abs() < 1e-5);
        assert!((i.iyy - i.ixx).abs() < 1e-6);
    }

    #[test]
    fn test_hollow_sphere() {
        let i = InertiaShape::hollow_sphere(3.0, 1.0);
        assert!((i.ixx - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_solid_box() {
        let i = InertiaShape::solid_box(12.0, 1.0, 1.0, 1.0);
        assert!((i.ixx - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_solid_cylinder() {
        let i = InertiaShape::solid_cylinder(2.0, 1.0, 2.0);
        assert!((i.iyy - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_thin_rod() {
        let i = InertiaShape::thin_rod(12.0, 1.0);
        assert!((i.ixx).abs() < 1e-6);
        assert!((i.iyy - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_parallel_axis() {
        let i = InertiaShape::solid_sphere(1.0, 1.0);
        let shifted = i.parallel_axis(1.0, [2.0, 0.0, 0.0]);
        assert!(shifted.iyy > i.iyy);
    }

    #[test]
    fn test_add() {
        let a = InertiaShape::new(1.0, 2.0, 3.0);
        let b = InertiaShape::new(4.0, 5.0, 6.0);
        let c = a.add(&b);
        assert!((c.ixx - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_inverse() {
        let i = InertiaShape::new(2.0, 4.0, 5.0);
        let inv = i.inverse();
        assert!((inv.ixx - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_rotational_kinetic_energy() {
        let i = InertiaShape::new(1.0, 1.0, 1.0);
        let e = rotational_kinetic_energy(&i, [2.0, 0.0, 0.0]);
        assert!((e - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_trace() {
        let i = InertiaShape::new(1.0, 2.0, 3.0);
        assert!((i.trace() - 6.0).abs() < 1e-6);
    }
}
