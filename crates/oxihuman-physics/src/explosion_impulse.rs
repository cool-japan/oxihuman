//! Radial impulse wave from an explosion point (force falls off with distance).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExplosionConfig {
    pub peak_force: f32,
    pub radius: f32,
    pub falloff_exponent: f32,
    pub min_distance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidBodyProxy {
    pub position: [f32; 3],
    pub mass: f32,
    pub velocity: [f32; 3],
    pub affected: bool,
}

#[allow(dead_code)]
pub struct ExplosionImpulse {
    config: ExplosionConfig,
    origin: [f32; 3],
    bodies: Vec<RigidBodyProxy>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExplosionResult {
    pub affected_count: usize,
    pub peak_impulse: f32,
    pub applied_impulses: Vec<[f32; 3]>,
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = length3(v);
    if l < 1e-10 { [0.0; 3] } else { [v[0]/l, v[1]/l, v[2]/l] }
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0]*s, v[1]*s, v[2]*s]
}

#[allow(dead_code)]
pub fn default_explosion_config() -> ExplosionConfig {
    ExplosionConfig {
        peak_force: 1000.0,
        radius: 5.0,
        falloff_exponent: 2.0,
        min_distance: 0.1,
    }
}

#[allow(dead_code)]
pub fn new_explosion_impulse(config: ExplosionConfig, origin: [f32; 3]) -> ExplosionImpulse {
    ExplosionImpulse { config, origin, bodies: Vec::new() }
}

/// Compute the impulse force vector for a body at `body_pos` given explosion origin.
#[allow(dead_code)]
pub fn explosion_compute_force(impulse: &ExplosionImpulse, body_pos: [f32; 3]) -> [f32; 3] {
    let diff = [
        body_pos[0] - impulse.origin[0],
        body_pos[1] - impulse.origin[1],
        body_pos[2] - impulse.origin[2],
    ];
    let d = length3(diff).max(impulse.config.min_distance);
    if d > impulse.config.radius {
        return [0.0; 3];
    }
    let t = 1.0 - (d / impulse.config.radius);
    let magnitude = impulse.config.peak_force * t.powf(impulse.config.falloff_exponent);
    let dir = normalize3(diff);
    scale3(dir, magnitude)
}

#[allow(dead_code)]
pub fn explosion_apply_to_bodies(impulse: &mut ExplosionImpulse, bodies: Vec<RigidBodyProxy>) -> ExplosionResult {
    impulse.bodies = bodies;

    // First pass: compute forces (needs immutable borrow of impulse config/origin)
    let forces: Vec<[f32; 3]> = {
        let origin = impulse.origin;
        let config = &impulse.config;
        impulse.bodies.iter().map(|body| {
            let diff = [
                body.position[0] - origin[0],
                body.position[1] - origin[1],
                body.position[2] - origin[2],
            ];
            let d = length3(diff).max(config.min_distance);
            if d > config.radius {
                return [0.0; 3];
            }
            let t = 1.0 - (d / config.radius);
            let magnitude = config.peak_force * t.powf(config.falloff_exponent);
            let dir = normalize3(diff);
            scale3(dir, magnitude)
        }).collect()
    };

    // Second pass: apply forces to bodies
    let mut applied = Vec::new();
    let mut affected_count = 0usize;

    for (body, f) in impulse.bodies.iter_mut().zip(forces.iter()) {
        let mag = length3(*f);
        if mag > 1e-6 {
            body.affected = true;
            affected_count += 1;
            let inv_m = if body.mass > 0.0 { 1.0 / body.mass } else { 0.0 };
            let dv = scale3(*f, inv_m);
            body.velocity[0] += dv[0];
            body.velocity[1] += dv[1];
            body.velocity[2] += dv[2];
            applied.push(*f);
        } else {
            applied.push([0.0; 3]);
        }
    }

    let peak = applied.iter().map(|f| length3(*f)).fold(0.0f32, f32::max);

    ExplosionResult { affected_count, peak_impulse: peak, applied_impulses: applied }
}

#[allow(dead_code)]
pub fn explosion_affected_count(result: &ExplosionResult) -> usize {
    result.affected_count
}

#[allow(dead_code)]
pub fn explosion_peak_force(impulse: &ExplosionImpulse) -> f32 {
    impulse.config.peak_force
}

#[allow(dead_code)]
pub fn explosion_radius(impulse: &ExplosionImpulse) -> f32 {
    impulse.config.radius
}

#[allow(dead_code)]
pub fn explosion_falloff(impulse: &ExplosionImpulse) -> f32 {
    impulse.config.falloff_exponent
}

#[allow(dead_code)]
pub fn explosion_to_json(impulse: &ExplosionImpulse) -> String {
    format!(
        "{{\"origin\":[{:.4},{:.4},{:.4}],\"peak_force\":{:.4},\"radius\":{:.4},\"falloff\":{:.4},\"body_count\":{}}}",
        impulse.origin[0], impulse.origin[1], impulse.origin[2],
        impulse.config.peak_force, impulse.config.radius, impulse.config.falloff_exponent,
        impulse.bodies.len()
    )
}

#[allow(dead_code)]
pub fn explosion_reset(impulse: &mut ExplosionImpulse) {
    impulse.bodies.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_explosion() -> ExplosionImpulse {
        new_explosion_impulse(default_explosion_config(), [0.0, 0.0, 0.0])
    }

    #[test]
    fn test_default_config() {
        let cfg = default_explosion_config();
        assert!(cfg.peak_force > 0.0);
        assert!(cfg.radius > 0.0);
    }

    #[test]
    fn test_peak_force_accessor() {
        let e = make_explosion();
        assert!((explosion_peak_force(&e) - 1000.0).abs() < 1e-4);
    }

    #[test]
    fn test_radius_accessor() {
        let e = make_explosion();
        assert!((explosion_radius(&e) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_force_at_origin_is_zero_dir() {
        let e = make_explosion();
        // At origin direction is zero, force should be zero-direction
        let f = explosion_compute_force(&e, [0.0, 0.0, 0.0]);
        // min_distance clamp keeps it non-zero magnitude but origin = [0,0,0]
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_force_decreases_with_distance() {
        let e = make_explosion();
        let f_near = explosion_compute_force(&e, [1.0, 0.0, 0.0]);
        let f_far  = explosion_compute_force(&e, [3.0, 0.0, 0.0]);
        assert!(length3(f_near) > length3(f_far));
    }

    #[test]
    fn test_force_zero_outside_radius() {
        let e = make_explosion();
        let f = explosion_compute_force(&e, [10.0, 0.0, 0.0]);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_apply_to_bodies() {
        let mut e = make_explosion();
        let bodies = vec![
            RigidBodyProxy { position: [1.0, 0.0, 0.0], mass: 1.0, velocity: [0.0; 3], affected: false },
            RigidBodyProxy { position: [10.0, 0.0, 0.0], mass: 1.0, velocity: [0.0; 3], affected: false },
        ];
        let res = explosion_apply_to_bodies(&mut e, bodies);
        assert_eq!(explosion_affected_count(&res), 1);
    }

    #[test]
    fn test_reset_clears_bodies() {
        let mut e = make_explosion();
        let bodies = vec![
            RigidBodyProxy { position: [1.0, 0.0, 0.0], mass: 1.0, velocity: [0.0; 3], affected: false },
        ];
        explosion_apply_to_bodies(&mut e, bodies);
        explosion_reset(&mut e);
        assert_eq!(e.bodies.len(), 0);
    }

    #[test]
    fn test_to_json_fields() {
        let e = make_explosion();
        let json = explosion_to_json(&e);
        assert!(json.contains("origin"));
        assert!(json.contains("peak_force"));
        assert!(json.contains("radius"));
    }

    #[test]
    fn test_falloff_accessor() {
        let e = make_explosion();
        assert!((explosion_falloff(&e) - 2.0).abs() < 1e-4);
    }
}
