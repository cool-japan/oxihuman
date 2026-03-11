//! Magnetic field forces for charged particle simulation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MagneticConfig {
    pub permeability: f32,
    pub field_strength: f32,
    pub field_direction: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChargedParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub charge: f32,
    pub mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MagneticForceResult {
    pub lorentz_force: [f32; 3],
    pub acceleration: [f32; 3],
    pub gyration_radius: f32,
}

#[allow(dead_code)]
pub fn default_magnetic_config() -> MagneticConfig {
    MagneticConfig {
        permeability: 1.256_637e-6,
        field_strength: 1.0,
        field_direction: [0.0, 1.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn new_charged_particle(pos: [f32; 3], charge: f32, mass: f32) -> ChargedParticle {
    ChargedParticle {
        position: pos,
        velocity: [0.0, 0.0, 0.0],
        charge,
        mass,
    }
}

#[allow(dead_code)]
pub fn cross_v3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot_v3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len_v3(a: [f32; 3]) -> f32 {
    dot_v3(a, a).sqrt()
}

#[allow(dead_code)]
pub fn normalize_field(cfg: &mut MagneticConfig) {
    let l = len_v3(cfg.field_direction);
    if l > 1e-10 {
        cfg.field_direction[0] /= l;
        cfg.field_direction[1] /= l;
        cfg.field_direction[2] /= l;
    }
}

/// Compute the Lorentz force F = q * (v × B).
#[allow(dead_code)]
pub fn compute_lorentz_force(p: &ChargedParticle, cfg: &MagneticConfig) -> MagneticForceResult {
    let b = [
        cfg.field_direction[0] * cfg.field_strength,
        cfg.field_direction[1] * cfg.field_strength,
        cfg.field_direction[2] * cfg.field_strength,
    ];
    let v_cross_b = cross_v3(p.velocity, b);
    let lorentz_force = [
        p.charge * v_cross_b[0],
        p.charge * v_cross_b[1],
        p.charge * v_cross_b[2],
    ];
    let mass = if p.mass.abs() > 1e-30 { p.mass } else { 1e-30 };
    let acceleration = [
        lorentz_force[0] / mass,
        lorentz_force[1] / mass,
        lorentz_force[2] / mass,
    ];
    let gr = gyration_radius(p, cfg);
    MagneticForceResult { lorentz_force, acceleration, gyration_radius: gr }
}

/// r = m|v_perp| / (|q| * |B|)
#[allow(dead_code)]
pub fn gyration_radius(p: &ChargedParticle, cfg: &MagneticConfig) -> f32 {
    let b_hat = cfg.field_direction;
    let v_dot_b = dot_v3(p.velocity, b_hat);
    // v_perp = v - (v·b̂)b̂
    let v_par = [v_dot_b * b_hat[0], v_dot_b * b_hat[1], v_dot_b * b_hat[2]];
    let v_perp = [
        p.velocity[0] - v_par[0],
        p.velocity[1] - v_par[1],
        p.velocity[2] - v_par[2],
    ];
    let v_perp_mag = len_v3(v_perp);
    let b_mag = cfg.field_strength.abs();
    let q_abs = p.charge.abs();
    if q_abs < 1e-30 || b_mag < 1e-30 {
        return 0.0;
    }
    p.mass * v_perp_mag / (q_abs * b_mag)
}

/// ω_c = |q| * |B| / m
#[allow(dead_code)]
pub fn cyclotron_frequency(p: &ChargedParticle, cfg: &MagneticConfig) -> f32 {
    let b_mag = cfg.field_strength.abs();
    let q_abs = p.charge.abs();
    let mass = if p.mass.abs() > 1e-30 { p.mass } else { 1e-30 };
    q_abs * b_mag / mass
}

#[allow(dead_code)]
pub fn step_charged_particle(p: &mut ChargedParticle, cfg: &MagneticConfig, dt: f32) {
    let result = compute_lorentz_force(p, cfg);
    p.velocity[0] += result.acceleration[0] * dt;
    p.velocity[1] += result.acceleration[1] * dt;
    p.velocity[2] += result.acceleration[2] * dt;
    p.position[0] += p.velocity[0] * dt;
    p.position[1] += p.velocity[1] * dt;
    p.position[2] += p.velocity[2] * dt;
}

#[allow(dead_code)]
pub fn magnetic_force_to_json(r: &MagneticForceResult) -> String {
    format!(
        "{{\"lorentz_force\":[{:.6},{:.6},{:.6}],\"acceleration\":[{:.6},{:.6},{:.6}],\"gyration_radius\":{:.6}}}",
        r.lorentz_force[0], r.lorentz_force[1], r.lorentz_force[2],
        r.acceleration[0], r.acceleration[1], r.acceleration[2],
        r.gyration_radius
    )
}

/// KE = 0.5 * m * |v|^2
#[allow(dead_code)]
pub fn particle_kinetic_energy(p: &ChargedParticle) -> f32 {
    0.5 * p.mass * dot_v3(p.velocity, p.velocity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_positive_field() {
        let cfg = default_magnetic_config();
        assert!(cfg.field_strength > 0.0);
        assert!(cfg.permeability > 0.0);
    }

    #[test]
    fn new_charged_particle_has_zero_velocity() {
        let p = new_charged_particle([1.0, 2.0, 3.0], 1.0, 1.0);
        assert_eq!(p.velocity, [0.0, 0.0, 0.0]);
        assert_eq!(p.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn cross_v3_x_cross_y_is_z() {
        let x = [1.0f32, 0.0, 0.0];
        let y = [0.0f32, 1.0, 0.0];
        let z = cross_v3(x, y);
        assert!((z[0]).abs() < 1e-6);
        assert!((z[1]).abs() < 1e-6);
        assert!((z[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lorentz_force_stationary_particle_is_zero() {
        let cfg = default_magnetic_config();
        let p = new_charged_particle([0.0, 0.0, 0.0], 1.0, 1.0);
        let result = compute_lorentz_force(&p, &cfg);
        assert!(result.lorentz_force[0].abs() < 1e-10);
        assert!(result.lorentz_force[1].abs() < 1e-10);
        assert!(result.lorentz_force[2].abs() < 1e-10);
    }

    #[test]
    fn lorentz_force_along_field_direction_is_zero() {
        // v parallel to B => v × B = 0
        let cfg = MagneticConfig {
            permeability: 1e-6,
            field_strength: 1.0,
            field_direction: [0.0, 1.0, 0.0],
        };
        let mut p = new_charged_particle([0.0, 0.0, 0.0], 1.0, 1.0);
        p.velocity = [0.0, 5.0, 0.0]; // parallel to B
        let result = compute_lorentz_force(&p, &cfg);
        assert!(result.lorentz_force[0].abs() < 1e-6);
        assert!(result.lorentz_force[1].abs() < 1e-6);
        assert!(result.lorentz_force[2].abs() < 1e-6);
    }

    #[test]
    fn cyclotron_frequency_positive() {
        let cfg = default_magnetic_config();
        let p = new_charged_particle([0.0, 0.0, 0.0], 1.0, 1.0);
        let freq = cyclotron_frequency(&p, &cfg);
        assert!(freq > 0.0);
    }

    #[test]
    fn kinetic_energy_rest_is_zero() {
        let p = new_charged_particle([0.0, 0.0, 0.0], 1.0, 1.0);
        assert!((particle_kinetic_energy(&p)).abs() < 1e-10);
    }

    #[test]
    fn kinetic_energy_moving() {
        let mut p = new_charged_particle([0.0, 0.0, 0.0], 1.0, 2.0);
        p.velocity = [3.0, 4.0, 0.0]; // speed = 5, KE = 0.5*2*25 = 25
        let ke = particle_kinetic_energy(&p);
        assert!((ke - 25.0).abs() < 1e-4);
    }

    #[test]
    fn normalize_field_produces_unit_vector() {
        let mut cfg = MagneticConfig {
            permeability: 1e-6,
            field_strength: 2.0,
            field_direction: [3.0, 4.0, 0.0],
        };
        normalize_field(&mut cfg);
        let len = (cfg.field_direction[0].powi(2)
            + cfg.field_direction[1].powi(2)
            + cfg.field_direction[2].powi(2))
        .sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn step_charged_particle_moves_position() {
        let cfg = MagneticConfig {
            permeability: 1e-6,
            field_strength: 1.0,
            field_direction: [0.0, 1.0, 0.0],
        };
        let mut p = ChargedParticle {
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            charge: 1.0,
            mass: 1.0,
        };
        step_charged_particle(&mut p, &cfg, 0.01);
        // position should have moved
        let dist = (p.position[0].powi(2) + p.position[1].powi(2) + p.position[2].powi(2)).sqrt();
        assert!(dist > 0.0);
    }

    #[test]
    fn magnetic_force_to_json_contains_fields() {
        let r = MagneticForceResult {
            lorentz_force: [1.0, 2.0, 3.0],
            acceleration: [4.0, 5.0, 6.0],
            gyration_radius: 0.5,
        };
        let json = magnetic_force_to_json(&r);
        assert!(json.contains("lorentz_force"));
        assert!(json.contains("gyration_radius"));
    }
}
