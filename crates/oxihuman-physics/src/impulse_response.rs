// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Compute the relative velocity of body A w.r.t. body B at the contact point.
#[allow(dead_code)]
pub fn relative_velocity(va: [f32; 3], vb: [f32; 3]) -> [f32; 3] {
    sub3(va, vb)
}

/// Compute the impulse magnitude `j` for a collision.
///
/// `rel_vel` - relative velocity of A w.r.t. B at the contact point.
/// `normal`  - collision normal (unit vector, pointing from B to A).
/// `inv_mass_a`, `inv_mass_b` - inverse masses (0 for infinite mass).
/// `restitution` - coefficient of restitution (0 = perfectly inelastic, 1 = perfectly elastic).
#[allow(dead_code)]
pub fn impulse_magnitude(
    rel_vel: [f32; 3],
    normal: [f32; 3],
    inv_mass_a: f32,
    inv_mass_b: f32,
    restitution: f32,
) -> f32 {
    let vn = dot3(rel_vel, normal);
    if vn >= 0.0 {
        // Already separating
        return 0.0;
    }
    let denom = inv_mass_a + inv_mass_b;
    if denom < 1e-12 {
        return 0.0;
    }
    -(1.0 + restitution) * vn / denom
}

/// Apply an impulse to body A's velocity (adds to velocity).
///
/// `vel`      - velocity of body A before impulse.
/// `normal`   - collision normal (from B to A).
/// `j`        - impulse magnitude (positive = push A away from B).
/// `inv_mass` - inverse mass of A.
#[allow(dead_code)]
pub fn apply_impulse_a(vel: [f32; 3], normal: [f32; 3], j: f32, inv_mass: f32) -> [f32; 3] {
    add3(vel, scale3(normal, j * inv_mass))
}

/// Apply an impulse to body B's velocity (subtracts from velocity).
///
/// `vel`      - velocity of body B before impulse.
/// `normal`   - collision normal (from B to A).
/// `j`        - impulse magnitude.
/// `inv_mass` - inverse mass of B.
#[allow(dead_code)]
pub fn apply_impulse_b(vel: [f32; 3], normal: [f32; 3], j: f32, inv_mass: f32) -> [f32; 3] {
    sub3(vel, scale3(normal, j * inv_mass))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_on_elastic_collision_equal_masses() {
        // A moving at [-1, 0, 0] toward B (stationary). Equal masses. Normal from B to A = [1,0,0].
        let va = [-1.0f32, 0.0, 0.0];
        let vb = [0.0f32; 3];
        let n = [1.0f32, 0.0, 0.0];
        let rel = relative_velocity(va, vb);
        let j = impulse_magnitude(rel, n, 1.0, 1.0, 1.0); // perfectly elastic
                                                          // vn = dot([-1,0,0],[1,0,0]) = -1; j = -(1+1)*(-1)/2 = 1.0
        assert!((j - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_impulse_a_increases_speed() {
        let va = [0.0f32; 3];
        let n = [1.0f32, 0.0, 0.0];
        let new_va = apply_impulse_a(va, n, 2.0, 1.0);
        assert!((new_va[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn apply_impulse_b_decreases_speed() {
        let vb = [1.0f32, 0.0, 0.0];
        let n = [1.0f32, 0.0, 0.0];
        let new_vb = apply_impulse_b(vb, n, 2.0, 0.5);
        assert!((new_vb[0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn separating_bodies_zero_impulse() {
        // Relative velocity already moving apart
        let rel = [1.0f32, 0.0, 0.0];
        let n = [1.0f32, 0.0, 0.0];
        let j = impulse_magnitude(rel, n, 1.0, 1.0, 1.0);
        assert_eq!(j, 0.0);
    }

    #[test]
    fn infinite_mass_wall_full_bounce() {
        let va = [-1.0f32, 0.0, 0.0]; // A moving toward wall (in -x direction)
        let vb = [0.0f32; 3]; // Wall stationary
        let n = [1.0f32, 0.0, 0.0]; // Normal from B(wall) to A points +x
        let rel = relative_velocity(va, vb);
        // vn = dot([-1,0,0],[1,0,0]) = -1; j = -(1+1)*(-1)/1 = 2
        let j = impulse_magnitude(rel, n, 1.0, 0.0, 1.0);
        let new_va = apply_impulse_a(va, n, j, 1.0);
        // Velocity should reverse: -1 + 2*1 = 1
        assert!((new_va[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn zero_restitution_inelastic() {
        let va = [2.0f32, 0.0, 0.0];
        let vb = [0.0f32; 3];
        let n = [1.0f32, 0.0, 0.0];
        let rel = relative_velocity(va, vb);
        // vn = dot([2,0,0],[1,0,0]) = 2 (positive → separating)
        let j = impulse_magnitude(rel, n, 1.0, 1.0, 0.0);
        assert_eq!(j, 0.0);
    }

    #[test]
    fn relative_velocity_correct() {
        let rel = relative_velocity([3.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((rel[0] - 2.0).abs() < 1e-6);
        assert!((rel[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn both_zero_mass_zero_impulse() {
        let rel = [-1.0f32, 0.0, 0.0];
        let n = [1.0f32, 0.0, 0.0];
        let j = impulse_magnitude(rel, n, 0.0, 0.0, 1.0);
        assert_eq!(j, 0.0);
    }

    #[test]
    fn impulse_proportional_to_restitution() {
        let va = [-1.0f32, 0.0, 0.0];
        let n = [1.0f32, 0.0, 0.0]; // Normal from wall to A; vn=-1 (approaching)
        let rel = relative_velocity(va, [0.0; 3]);
        let j0 = impulse_magnitude(rel, n, 1.0, 0.0, 0.0); // inelastic: j=1
        let j1 = impulse_magnitude(rel, n, 1.0, 0.0, 1.0); // elastic: j=2
        assert!(j1 > j0);
    }

    #[test]
    fn apply_impulse_a_zero_inv_mass_unchanged() {
        let va = [5.0f32, 0.0, 0.0];
        let result = apply_impulse_a(va, [1.0, 0.0, 0.0], 10.0, 0.0);
        assert_eq!(result, va);
    }
}
