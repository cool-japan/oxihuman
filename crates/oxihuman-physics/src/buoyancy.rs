// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Buoyancy force simulation for objects in fluid.

#[allow(dead_code)]
pub struct BuoyancyConfig {
    pub fluid_density: f32,
    pub gravity: f32,
    pub surface_y: f32,
    pub drag_coeff: f32,
}

#[allow(dead_code)]
pub struct SubmergedBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub volume: f32,
    pub mass: f32,
}

#[allow(dead_code)]
pub struct BuoyancyResult {
    pub force: [f32; 3],
    pub submerged_fraction: f32,
    pub torque: [f32; 3],
}

/// Fraction of the body's volume submerged below `cfg.surface_y`.
/// Assumes body is a sphere with center at `body.position[1]` and
/// radius derived from `body.volume = (4/3)π r³`.
#[allow(dead_code)]
pub fn submerged_fraction(body: &SubmergedBody, cfg: &BuoyancyConfig) -> f32 {
    let radius = (body.volume * 3.0 / (4.0 * std::f32::consts::PI)).cbrt();
    let center_y = body.position[1];
    let bottom_y = center_y - radius;
    let top_y = center_y + radius;
    if top_y <= cfg.surface_y {
        // Fully submerged
        return 1.0;
    }
    if bottom_y >= cfg.surface_y {
        // Fully above surface
        return 0.0;
    }
    // Partial submersion: fraction of height below surface
    let depth = cfg.surface_y - bottom_y;
    let total_height = top_y - bottom_y;
    (depth / total_height).clamp(0.0, 1.0)
}

/// Archimedes force: ρ * g * V (scalar, positive upward).
#[allow(dead_code)]
pub fn archimedes_force(submerged_volume: f32, cfg: &BuoyancyConfig) -> f32 {
    cfg.fluid_density * cfg.gravity * submerged_volume
}

/// Drag force vector opposing velocity.
#[allow(dead_code)]
pub fn drag_force(velocity: [f32; 3], drag_coeff: f32, fluid_density: f32) -> [f32; 3] {
    let speed_sq = velocity[0].powi(2) + velocity[1].powi(2) + velocity[2].powi(2);
    if speed_sq < 1e-12 {
        return [0.0; 3];
    }
    let speed = speed_sq.sqrt();
    let mag = -0.5 * fluid_density * drag_coeff * speed_sq;
    [
        mag * velocity[0] / speed,
        mag * velocity[1] / speed,
        mag * velocity[2] / speed,
    ]
}

/// Compute buoyancy force, fraction, and (zero) torque for a body.
#[allow(dead_code)]
pub fn compute_buoyancy_force(body: &SubmergedBody, cfg: &BuoyancyConfig) -> BuoyancyResult {
    let frac = submerged_fraction(body, cfg);
    let sub_vol = body.volume * frac;
    let buoy = archimedes_force(sub_vol, cfg);
    let drag = drag_force(body.velocity, cfg.drag_coeff, cfg.fluid_density);
    let gravity_force = -body.mass * cfg.gravity;
    BuoyancyResult {
        force: [drag[0], buoy + gravity_force + drag[1], drag[2]],
        submerged_fraction: frac,
        torque: [0.0; 3],
    }
}

/// Terminal velocity magnitude (upward) for the body.
#[allow(dead_code)]
pub fn terminal_velocity(body: &SubmergedBody, cfg: &BuoyancyConfig) -> f32 {
    // At terminal velocity, net force = 0:
    // ρ*g*V - m*g - 0.5*ρ*Cd*v² = 0  (fully submerged)
    let net_buoy = cfg.fluid_density * cfg.gravity * body.volume - body.mass * cfg.gravity;
    if net_buoy <= 0.0 {
        // Body sinks; return sinking terminal velocity
        let net_gravity = body.mass * cfg.gravity - cfg.fluid_density * cfg.gravity * body.volume;
        let v_sq = (2.0 * net_gravity) / (cfg.fluid_density * cfg.drag_coeff).max(1e-10);
        v_sq.max(0.0).sqrt()
    } else {
        let v_sq = (2.0 * net_buoy) / (cfg.fluid_density * cfg.drag_coeff).max(1e-10);
        v_sq.max(0.0).sqrt()
    }
}

/// Perform one Euler integration step with buoyancy + drag + gravity.
#[allow(dead_code)]
pub fn step_body(body: &mut SubmergedBody, cfg: &BuoyancyConfig, dt: f32) {
    let result = compute_buoyancy_force(body, cfg);
    let inv_mass = if body.mass > 1e-10 {
        1.0 / body.mass
    } else {
        0.0
    };
    body.velocity[0] += result.force[0] * inv_mass * dt;
    body.velocity[1] += result.force[1] * inv_mass * dt;
    body.velocity[2] += result.force[2] * inv_mass * dt;
    body.position[0] += body.velocity[0] * dt;
    body.position[1] += body.velocity[1] * dt;
    body.position[2] += body.velocity[2] * dt;
}

/// Returns true if the body is positively buoyant (net upward force when submerged).
#[allow(dead_code)]
pub fn is_floating(body: &SubmergedBody, cfg: &BuoyancyConfig) -> bool {
    cfg.fluid_density * body.volume > body.mass
}

/// Equilibrium depth: the y-position of center at which the body floats at rest.
/// Returns the position where buoyancy equals gravity.
#[allow(dead_code)]
pub fn equilibrium_depth(body: &SubmergedBody, cfg: &BuoyancyConfig) -> f32 {
    if !is_floating(body, cfg) {
        return f32::NEG_INFINITY; // sinks
    }
    // Required submerged volume: mass / fluid_density
    let req_vol_frac = body.mass / (cfg.fluid_density * body.volume);
    let radius = (body.volume * 3.0 / (4.0 * std::f32::consts::PI)).cbrt();
    // For sphere: submerged fraction ≈ (depth below surface) / (2*radius)
    // depth = frac * 2 * radius, center_y = surface_y - depth + radius
    let depth_frac = req_vol_frac.clamp(0.0, 1.0);
    cfg.surface_y - depth_frac * 2.0 * radius + radius
}

/// Compute wave force on a body (sinusoidal).
#[allow(dead_code)]
pub fn compute_wave_force(
    body: &SubmergedBody,
    cfg: &BuoyancyConfig,
    time: f32,
    wave_amplitude: f32,
    wave_freq: f32,
) -> [f32; 3] {
    let frac = submerged_fraction(body, cfg);
    let phase = wave_freq * time + body.position[0];
    let wave = wave_amplitude * phase.sin() * frac;
    [wave, 0.0, 0.0]
}

/// Compute buoyancy results for multiple bodies.
#[allow(dead_code)]
pub fn multi_body_buoyancy(bodies: &[SubmergedBody], cfg: &BuoyancyConfig) -> Vec<BuoyancyResult> {
    bodies
        .iter()
        .map(|b| compute_buoyancy_force(b, cfg))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water_cfg() -> BuoyancyConfig {
        BuoyancyConfig {
            fluid_density: 1000.0,
            gravity: 9.81,
            surface_y: 0.0,
            drag_coeff: 0.47,
        }
    }

    fn floating_ball() -> SubmergedBody {
        // Half the density of water, so it floats
        SubmergedBody {
            position: [0.0, -0.1, 0.0], // slightly below surface
            velocity: [0.0; 3],
            volume: 0.001, // 1 liter = 0.001 m^3
            mass: 0.5,     // 0.5 kg => density 500 kg/m^3 < water
        }
    }

    fn sinking_ball() -> SubmergedBody {
        SubmergedBody {
            position: [0.0, -0.5, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 5.0, // density >> water
        }
    }

    #[test]
    fn archimedes_force_positive() {
        let cfg = water_cfg();
        let f = archimedes_force(0.001, &cfg);
        assert!(f > 0.0);
    }

    #[test]
    fn archimedes_force_proportional_to_volume() {
        let cfg = water_cfg();
        let f1 = archimedes_force(0.001, &cfg);
        let f2 = archimedes_force(0.002, &cfg);
        assert!((f2 / f1 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn fully_submerged_fraction_is_one() {
        let cfg = water_cfg();
        let body = SubmergedBody {
            position: [0.0, -10.0, 0.0], // far below surface
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 1.0,
        };
        let frac = submerged_fraction(&body, &cfg);
        assert!((frac - 1.0).abs() < 1e-5, "frac={frac}");
    }

    #[test]
    fn above_surface_fraction_is_zero() {
        let cfg = water_cfg();
        let body = SubmergedBody {
            position: [0.0, 10.0, 0.0], // far above surface
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 1.0,
        };
        let frac = submerged_fraction(&body, &cfg);
        assert!(frac < 1e-5, "frac={frac}");
    }

    #[test]
    fn partial_submersion_fraction_between_zero_and_one() {
        let cfg = water_cfg();
        let body = SubmergedBody {
            position: [0.0, 0.0, 0.0], // center at surface
            velocity: [0.0; 3],
            volume: 0.524, // sphere of radius ~0.5m
            mass: 1.0,
        };
        let frac = submerged_fraction(&body, &cfg);
        assert!(frac > 0.0 && frac < 1.0, "frac={frac}");
    }

    #[test]
    fn buoyant_body_floats() {
        let cfg = water_cfg();
        let body = floating_ball();
        assert!(is_floating(&body, &cfg));
    }

    #[test]
    fn sinking_body_does_not_float() {
        let cfg = water_cfg();
        let body = sinking_ball();
        assert!(!is_floating(&body, &cfg));
    }

    #[test]
    fn drag_force_opposite_velocity() {
        let vel = [1.0_f32, 0.0, 0.0];
        let f = drag_force(vel, 1.0, 1000.0);
        assert!(f[0] < 0.0, "drag should oppose positive x velocity");
    }

    #[test]
    fn drag_force_zero_for_zero_velocity() {
        let f = drag_force([0.0; 3], 1.0, 1000.0);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn step_body_integrates_position() {
        let cfg = water_cfg();
        let mut body = SubmergedBody {
            position: [0.0, -5.0, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 5.0, // sinks
        };
        let initial_y = body.position[1];
        step_body(&mut body, &cfg, 0.01);
        // Gravity dominates, body sinks
        assert!(
            body.position[1] < initial_y,
            "sinking body should move down"
        );
    }

    #[test]
    fn step_body_buoyant_rises() {
        let cfg = water_cfg();
        let mut body = SubmergedBody {
            position: [0.0, -1.0, 0.0],
            velocity: [0.0; 3],
            volume: 1.0, // large volume => very buoyant
            mass: 0.01,
        };
        let initial_y = body.position[1];
        step_body(&mut body, &cfg, 0.01);
        assert!(body.position[1] > initial_y, "buoyant body should rise");
    }

    #[test]
    fn multi_body_buoyancy_returns_correct_count() {
        let cfg = water_cfg();
        let bodies = vec![floating_ball(), sinking_ball()];
        let results = multi_body_buoyancy(&bodies, &cfg);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn wave_force_zero_when_above_surface() {
        let cfg = water_cfg();
        let body = SubmergedBody {
            position: [0.0, 100.0, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 1.0,
        };
        let f = compute_wave_force(&body, &cfg, 0.0, 1.0, 1.0);
        assert!(f[0].abs() < 1e-5);
    }
}
