// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Position Verlet integrator for particle and cloth simulation.

#[allow(dead_code)]
pub struct VerletConfig {
    pub damping: f32,
    pub gravity: [f32; 3],
    pub sub_steps: u32,
}

#[allow(dead_code)]
pub struct VerletParticle {
    pub position: [f32; 3],
    pub prev_position: [f32; 3],
    pub acceleration: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

#[allow(dead_code)]
pub struct VerletSystem {
    pub particles: Vec<VerletParticle>,
    pub config: VerletConfig,
    pub time: f32,
}

#[allow(dead_code)]
pub fn default_verlet_config() -> VerletConfig {
    VerletConfig {
        damping: 0.98,
        gravity: [0.0, -9.81, 0.0],
        sub_steps: 4,
    }
}

#[allow(dead_code)]
pub fn new_verlet_particle(pos: [f32; 3], mass: f32) -> VerletParticle {
    VerletParticle {
        position: pos,
        prev_position: pos,
        acceleration: [0.0; 3],
        mass,
        pinned: false,
    }
}

#[allow(dead_code)]
pub fn new_verlet_system(cfg: VerletConfig) -> VerletSystem {
    VerletSystem {
        particles: Vec::new(),
        config: cfg,
        time: 0.0,
    }
}

#[allow(dead_code)]
pub fn integrate_particle(p: &mut VerletParticle, dt: f32, cfg: &VerletConfig) {
    if p.pinned {
        return;
    }
    // Verlet: x_new = 2*x - x_prev + a*dt^2
    let ax = p.acceleration[0] + cfg.gravity[0];
    let ay = p.acceleration[1] + cfg.gravity[1];
    let az = p.acceleration[2] + cfg.gravity[2];

    let new_x = 2.0 * p.position[0] - p.prev_position[0] + ax * dt * dt;
    let new_y = 2.0 * p.position[1] - p.prev_position[1] + ay * dt * dt;
    let new_z = 2.0 * p.position[2] - p.prev_position[2] + az * dt * dt;

    // Apply damping to velocity component
    let vx = (p.position[0] - p.prev_position[0]) * cfg.damping;
    let vy = (p.position[1] - p.prev_position[1]) * cfg.damping;
    let vz = (p.position[2] - p.prev_position[2]) * cfg.damping;

    p.prev_position = p.position;
    p.position = [
        p.prev_position[0] + (new_x - p.prev_position[0]) * cfg.damping + vx * 0.0,
        p.prev_position[1] + (new_y - p.prev_position[1]) * cfg.damping + vy * 0.0,
        p.prev_position[2] + (new_z - p.prev_position[2]) * cfg.damping + vz * 0.0,
    ];

    // Reset acceleration (forces are re-applied each step)
    p.acceleration = [0.0; 3];
    let _ = (vx, vy, vz);
}

#[allow(dead_code)]
pub fn step_system(sys: &mut VerletSystem, dt: f32) {
    let sub_dt = dt / sys.config.sub_steps.max(1) as f32;
    for _ in 0..sys.config.sub_steps.max(1) {
        for p in sys.particles.iter_mut() {
            // Apply gravity into acceleration before integrating
            let gx = sys.config.gravity[0];
            let gy = sys.config.gravity[1];
            let gz = sys.config.gravity[2];
            let ax = p.acceleration[0] + gx;
            let ay = p.acceleration[1] + gy;
            let az = p.acceleration[2] + gz;
            if p.pinned {
                p.acceleration = [0.0; 3];
                continue;
            }
            let new_x = 2.0 * p.position[0] - p.prev_position[0] + ax * sub_dt * sub_dt;
            let new_y = 2.0 * p.position[1] - p.prev_position[1] + ay * sub_dt * sub_dt;
            let new_z = 2.0 * p.position[2] - p.prev_position[2] + az * sub_dt * sub_dt;
            let damp = sys.config.damping;
            let prev = p.position;
            p.position = [
                p.position[0] + (new_x - p.position[0]) * damp,
                p.position[1] + (new_y - p.position[1]) * damp,
                p.position[2] + (new_z - p.position[2]) * damp,
            ];
            p.prev_position = prev;
            p.acceleration = [0.0; 3];
        }
    }
    sys.time += dt;
}

#[allow(dead_code)]
pub fn pin_particle(p: &mut VerletParticle) {
    p.pinned = true;
}

#[allow(dead_code)]
pub fn add_force(p: &mut VerletParticle, force: [f32; 3]) {
    if p.mass < 1e-12 || p.pinned {
        return;
    }
    let inv_mass = 1.0 / p.mass;
    p.acceleration[0] += force[0] * inv_mass;
    p.acceleration[1] += force[1] * inv_mass;
    p.acceleration[2] += force[2] * inv_mass;
}

#[allow(dead_code)]
pub fn particle_velocity(p: &VerletParticle, dt: f32) -> [f32; 3] {
    if dt < 1e-12 {
        return [0.0; 3];
    }
    [
        (p.position[0] - p.prev_position[0]) / dt,
        (p.position[1] - p.prev_position[1]) / dt,
        (p.position[2] - p.prev_position[2]) / dt,
    ]
}

#[allow(dead_code)]
pub fn particle_count(sys: &VerletSystem) -> usize {
    sys.particles.len()
}

#[allow(dead_code)]
pub fn verlet_system_to_json(sys: &VerletSystem) -> String {
    format!(
        r#"{{"particle_count":{},"time":{}}}"#,
        sys.particles.len(),
        sys.time
    )
}

#[allow(dead_code)]
pub fn kinetic_energy(sys: &VerletSystem, dt: f32) -> f32 {
    if dt < 1e-12 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for p in &sys.particles {
        if p.pinned {
            continue;
        }
        let v = particle_velocity(p, dt);
        let v_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        total += 0.5 * p.mass * v_sq;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sub_steps() {
        let cfg = default_verlet_config();
        assert!(cfg.sub_steps > 0);
        assert!(cfg.damping > 0.0 && cfg.damping <= 1.0);
    }

    #[test]
    fn new_particle_at_position() {
        let p = new_verlet_particle([1.0, 2.0, 3.0], 1.0);
        assert!((p.position[0] - 1.0).abs() < 1e-6);
        assert!(!p.pinned);
        assert_eq!(p.prev_position, p.position);
    }

    #[test]
    fn pin_particle_flag() {
        let mut p = new_verlet_particle([0.0; 3], 1.0);
        assert!(!p.pinned);
        pin_particle(&mut p);
        assert!(p.pinned);
    }

    #[test]
    fn pinned_particle_does_not_move() {
        let cfg = default_verlet_config();
        let mut sys = new_verlet_system(cfg);
        let mut p = new_verlet_particle([0.0, 5.0, 0.0], 1.0);
        pin_particle(&mut p);
        let init_y = p.position[1];
        sys.particles.push(p);
        step_system(&mut sys, 0.01);
        assert!((sys.particles[0].position[1] - init_y).abs() < 1e-6);
    }

    #[test]
    fn unpinned_particle_falls_under_gravity() {
        let cfg = default_verlet_config();
        let mut sys = new_verlet_system(cfg);
        let p = new_verlet_particle([0.0, 10.0, 0.0], 1.0);
        let init_y = p.position[1];
        sys.particles.push(p);
        step_system(&mut sys, 0.1);
        assert!(sys.particles[0].position[1] < init_y, "y={}", sys.particles[0].position[1]);
    }

    #[test]
    fn add_force_changes_acceleration() {
        let mut p = new_verlet_particle([0.0; 3], 2.0);
        add_force(&mut p, [4.0, 0.0, 0.0]);
        assert!((p.acceleration[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn add_force_no_effect_on_pinned() {
        let mut p = new_verlet_particle([0.0; 3], 1.0);
        pin_particle(&mut p);
        add_force(&mut p, [100.0, 0.0, 0.0]);
        assert_eq!(p.acceleration[0], 0.0);
    }

    #[test]
    fn particle_count_correct() {
        let cfg = default_verlet_config();
        let mut sys = new_verlet_system(cfg);
        sys.particles.push(new_verlet_particle([0.0; 3], 1.0));
        sys.particles.push(new_verlet_particle([1.0, 0.0, 0.0], 1.0));
        assert_eq!(particle_count(&sys), 2);
    }

    #[test]
    fn kinetic_energy_zero_at_start() {
        let cfg = default_verlet_config();
        let mut sys = new_verlet_system(cfg);
        sys.particles.push(new_verlet_particle([0.0; 3], 1.0));
        let ke = kinetic_energy(&sys, 0.01);
        assert!(ke >= 0.0);
    }

    #[test]
    fn verlet_system_to_json_contains_count() {
        let cfg = default_verlet_config();
        let sys = new_verlet_system(cfg);
        let s = verlet_system_to_json(&sys);
        assert!(s.contains("particle_count"));
        assert!(s.contains("time"));
    }

    #[test]
    fn time_advances_after_step() {
        let cfg = default_verlet_config();
        let mut sys = new_verlet_system(cfg);
        assert_eq!(sys.time, 0.0);
        step_system(&mut sys, 0.016);
        assert!((sys.time - 0.016).abs() < 1e-5);
    }
}
