// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Collision response with restitution and friction.
//! Named collision_response_solver to avoid conflict with existing collision_response module.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionResponseSolverConfig {
    pub restitution: f32,
    pub friction: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionSolverBody {
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub pos: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionSolverResult {
    pub vel_a: [f32; 3],
    pub vel_b: [f32; 3],
    pub impulse_mag: f32,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn default_collision_response_solver_config() -> CollisionResponseSolverConfig {
    CollisionResponseSolverConfig { restitution: 0.5, friction: 0.3 }
}

#[allow(dead_code)]
pub fn new_collision_solver_body(pos: [f32; 3], vel: [f32; 3], inv_mass: f32) -> CollisionSolverBody {
    CollisionSolverBody { vel, inv_mass, pos }
}

#[allow(dead_code)]
pub fn response_relative_normal_vel(a: &CollisionSolverBody, b: &CollisionSolverBody, normal: [f32; 3]) -> f32 {
    let rel_vel = [
        a.vel[0] - b.vel[0],
        a.vel[1] - b.vel[1],
        a.vel[2] - b.vel[2],
    ];
    dot3(rel_vel, normal)
}

#[allow(dead_code)]
pub fn response_is_separating(a: &CollisionSolverBody, b: &CollisionSolverBody, normal: [f32; 3]) -> bool {
    response_relative_normal_vel(a, b, normal) > 0.0
}

#[allow(dead_code)]
pub fn response_impulse_magnitude(
    a: &CollisionSolverBody,
    b: &CollisionSolverBody,
    normal: [f32; 3],
    config: &CollisionResponseSolverConfig,
) -> f32 {
    let vrel_n = response_relative_normal_vel(a, b, normal);
    if vrel_n >= 0.0 {
        return 0.0;
    }
    let denom = a.inv_mass + b.inv_mass;
    if denom < 1e-10 {
        return 0.0;
    }
    -(1.0 + config.restitution) * vrel_n / denom
}

#[allow(dead_code)]
pub fn resolve_collision_solver(
    a: &CollisionSolverBody,
    b: &CollisionSolverBody,
    normal: [f32; 3],
    config: &CollisionResponseSolverConfig,
) -> CollisionSolverResult {
    let j = response_impulse_magnitude(a, b, normal, config);
    let vel_a = [
        a.vel[0] + j * a.inv_mass * normal[0],
        a.vel[1] + j * a.inv_mass * normal[1],
        a.vel[2] + j * a.inv_mass * normal[2],
    ];
    let vel_b = [
        b.vel[0] - j * b.inv_mass * normal[0],
        b.vel[1] - j * b.inv_mass * normal[1],
        b.vel[2] - j * b.inv_mass * normal[2],
    ];
    CollisionSolverResult { vel_a, vel_b, impulse_mag: j }
}

#[allow(dead_code)]
pub fn response_to_json(result: &CollisionSolverResult) -> String {
    format!(
        "{{\"impulse_mag\":{},\"vel_a\":[{},{},{}],\"vel_b\":[{},{},{}]}}",
        result.impulse_mag,
        result.vel_a[0], result.vel_a[1], result.vel_a[2],
        result.vel_b[0], result.vel_b[1], result.vel_b[2],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_collision_response_solver_config();
        assert!((cfg.restitution - 0.5).abs() < 1e-6);
        assert!((cfg.friction - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_new_body() {
        let b = new_collision_solver_body([0.0; 3], [1.0, 0.0, 0.0], 1.0);
        assert!((b.vel[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_separating() {
        // A moving right, B moving left, n=[1,0,0] → vrel=[2,0,0], vn=2>0 → separating
        let a = new_collision_solver_body([0.0; 3], [1.0, 0.0, 0.0], 1.0);
        let b = new_collision_solver_body([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], 1.0);
        assert!(response_is_separating(&a, &b, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_approaching_not_separating() {
        // A moving toward B: a.vel=[-2,0,0], b stationary, n=[1,0,0] → vn=-2<0 → not separating
        let a = new_collision_solver_body([0.0; 3], [-2.0, 0.0, 0.0], 1.0);
        let b = new_collision_solver_body([1.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!(!response_is_separating(&a, &b, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_impulse_mag_positive() {
        // Approaching: a.vel=[-1,0,0], b.vel=[1,0,0] → vn=dot([-2,0,0],[1,0,0])=-2 → j>0
        let cfg = default_collision_response_solver_config();
        let a = new_collision_solver_body([0.0; 3], [-1.0, 0.0, 0.0], 1.0);
        let b = new_collision_solver_body([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        let j = response_impulse_magnitude(&a, &b, [1.0, 0.0, 0.0], &cfg);
        assert!(j > 0.0);
    }

    #[test]
    fn test_resolve_collision() {
        // Approaching bodies → positive impulse
        let cfg = default_collision_response_solver_config();
        let a = new_collision_solver_body([0.0; 3], [-1.0, 0.0, 0.0], 1.0);
        let b = new_collision_solver_body([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        let result = resolve_collision_solver(&a, &b, [1.0, 0.0, 0.0], &cfg);
        assert!(result.impulse_mag > 0.0);
    }

    #[test]
    fn test_resolve_separating_no_impulse() {
        // Separating bodies → zero impulse
        let cfg = default_collision_response_solver_config();
        let a = new_collision_solver_body([0.0; 3], [1.0, 0.0, 0.0], 1.0);
        let b = new_collision_solver_body([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], 1.0);
        let result = resolve_collision_solver(&a, &b, [1.0, 0.0, 0.0], &cfg);
        assert!((result.impulse_mag).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let result = CollisionSolverResult {
            vel_a: [1.0, 0.0, 0.0],
            vel_b: [-1.0, 0.0, 0.0],
            impulse_mag: 2.0,
        };
        let json = response_to_json(&result);
        assert!(json.contains("\"impulse_mag\""));
    }

    #[test]
    fn test_relative_normal_vel() {
        let a = new_collision_solver_body([0.0; 3], [2.0, 0.0, 0.0], 1.0);
        let b = new_collision_solver_body([1.0, 0.0, 0.0], [0.0; 3], 1.0);
        let vrel = response_relative_normal_vel(&a, &b, [1.0, 0.0, 0.0]);
        assert!((vrel - 2.0).abs() < 1e-6);
    }
}
