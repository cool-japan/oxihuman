// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Fluid buoyancy force computation for submerged rigid bodies.

#[allow(dead_code)]
pub struct BuoyancyBodyConfig {
    pub fluid_density: f32,
    pub gravity: f32,
    pub drag_coefficient: f32,
    pub water_level: f32,
}

#[allow(dead_code)]
pub struct BuoyancyBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub volume: f32,
    pub mass: f32,
}

#[allow(dead_code)]
pub struct BuoyancyBodyResult {
    pub buoyant_force: f32,
    pub drag_force: f32,
    pub net_force: f32,
    pub submerged_fraction: f32,
}

#[allow(dead_code)]
pub fn default_buoyancy_body_config() -> BuoyancyBodyConfig {
    BuoyancyBodyConfig {
        fluid_density: 1000.0,
        gravity: 9.81,
        drag_coefficient: 0.47,
        water_level: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_buoyancy_body(pos: [f32; 3], volume: f32, mass: f32) -> BuoyancyBody {
    BuoyancyBody {
        position: pos,
        velocity: [0.0; 3],
        volume,
        mass,
    }
}

#[allow(dead_code)]
pub fn submerged_fraction_body(body: &BuoyancyBody, water_level: f32) -> f32 {
    let radius = (body.volume * 3.0 / (4.0 * std::f32::consts::PI)).cbrt();
    let center_y = body.position[1];
    let bottom_y = center_y - radius;
    let top_y = center_y + radius;
    if top_y <= water_level {
        return 1.0;
    }
    if bottom_y >= water_level {
        return 0.0;
    }
    let depth = water_level - bottom_y;
    let total_height = top_y - bottom_y;
    (depth / total_height).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn compute_buoyancy_body(body: &BuoyancyBody, cfg: &BuoyancyBodyConfig) -> BuoyancyBodyResult {
    let frac = submerged_fraction_body(body, cfg.water_level);
    let submerged_vol = body.volume * frac;
    let buoyant_force = cfg.fluid_density * cfg.gravity * submerged_vol;
    let speed_sq = body.velocity[0] * body.velocity[0]
        + body.velocity[1] * body.velocity[1]
        + body.velocity[2] * body.velocity[2];
    let drag_force = 0.5 * cfg.fluid_density * cfg.drag_coefficient * speed_sq;
    let weight = body.mass * cfg.gravity;
    let net_force = buoyant_force - weight;
    BuoyancyBodyResult {
        buoyant_force,
        drag_force,
        net_force,
        submerged_fraction: frac,
    }
}

#[allow(dead_code)]
pub fn apply_buoyancy_body(body: &mut BuoyancyBody, cfg: &BuoyancyBodyConfig, dt: f32) {
    let result = compute_buoyancy_body(body, cfg);
    let inv_mass = if body.mass > 1e-12 { 1.0 / body.mass } else { 0.0 };
    // Net vertical force minus drag opposing velocity
    let speed_sq = body.velocity[0] * body.velocity[0]
        + body.velocity[1] * body.velocity[1]
        + body.velocity[2] * body.velocity[2];
    let speed = speed_sq.sqrt();
    let drag_decel = if speed > 1e-12 {
        result.drag_force / speed
    } else {
        0.0
    };
    body.velocity[0] -= body.velocity[0] * drag_decel * inv_mass * dt;
    body.velocity[1] += result.net_force * inv_mass * dt
        - body.velocity[1] * drag_decel * inv_mass * dt;
    body.velocity[2] -= body.velocity[2] * drag_decel * inv_mass * dt;
    body.position[0] += body.velocity[0] * dt;
    body.position[1] += body.velocity[1] * dt;
    body.position[2] += body.velocity[2] * dt;
}

#[allow(dead_code)]
pub fn is_floating_body(body: &BuoyancyBody, cfg: &BuoyancyBodyConfig) -> bool {
    cfg.fluid_density * body.volume > body.mass
}

#[allow(dead_code)]
pub fn buoyancy_body_result_to_json(r: &BuoyancyBodyResult) -> String {
    format!(
        r#"{{"buoyant_force":{},"drag_force":{},"net_force":{},"submerged_fraction":{}}}"#,
        r.buoyant_force, r.drag_force, r.net_force, r.submerged_fraction
    )
}

#[allow(dead_code)]
pub fn buoyancy_body_to_json(b: &BuoyancyBody) -> String {
    format!(
        r#"{{"position":[{},{},{}],"velocity":[{},{},{}],"volume":{},"mass":{}}}"#,
        b.position[0], b.position[1], b.position[2],
        b.velocity[0], b.velocity[1], b.velocity[2],
        b.volume, b.mass
    )
}

#[allow(dead_code)]
pub fn terminal_velocity_body(body: &BuoyancyBody, cfg: &BuoyancyBodyConfig) -> f32 {
    let net_buoy = cfg.fluid_density * cfg.gravity * body.volume - body.mass * cfg.gravity;
    let denom = (0.5 * cfg.fluid_density * cfg.drag_coefficient).max(1e-12);
    if net_buoy <= 0.0 {
        let net_g = body.mass * cfg.gravity - cfg.fluid_density * cfg.gravity * body.volume;
        (net_g / denom).max(0.0).sqrt()
    } else {
        (net_buoy / denom).max(0.0).sqrt()
    }
}

#[allow(dead_code)]
pub fn effective_weight(body: &BuoyancyBody, cfg: &BuoyancyBodyConfig) -> f32 {
    let frac = submerged_fraction_body(body, cfg.water_level);
    let buoyant = cfg.fluid_density * cfg.gravity * body.volume * frac;
    body.mass * cfg.gravity - buoyant
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water_cfg() -> BuoyancyBodyConfig {
        BuoyancyBodyConfig {
            fluid_density: 1000.0,
            gravity: 9.81,
            drag_coefficient: 0.47,
            water_level: 0.0,
        }
    }

    fn floating_body() -> BuoyancyBody {
        BuoyancyBody {
            position: [0.0, -0.1, 0.0],
            velocity: [0.0; 3],
            volume: 0.01,
            mass: 1.0, // density=100 < 1000 => floats
        }
    }

    fn sinking_body() -> BuoyancyBody {
        BuoyancyBody {
            position: [0.0, -0.5, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 5.0, // density=5000 > 1000 => sinks
        }
    }

    #[test]
    fn default_config_positive_density() {
        let cfg = default_buoyancy_body_config();
        assert!(cfg.fluid_density > 0.0);
        assert!(cfg.gravity > 0.0);
    }

    #[test]
    fn new_body_zero_velocity() {
        let b = new_buoyancy_body([1.0, 0.0, 0.0], 0.001, 1.0);
        assert_eq!(b.velocity, [0.0; 3]);
        assert!((b.position[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn submerged_fraction_fully_below() {
        let cfg = water_cfg();
        let body = BuoyancyBody {
            position: [0.0, -100.0, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 1.0,
        };
        let frac = submerged_fraction_body(&body, cfg.water_level);
        assert!((frac - 1.0).abs() < 1e-5, "frac={frac}");
    }

    #[test]
    fn submerged_fraction_fully_above() {
        let cfg = water_cfg();
        let body = BuoyancyBody {
            position: [0.0, 100.0, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 1.0,
        };
        let frac = submerged_fraction_body(&body, cfg.water_level);
        assert!(frac < 1e-5, "frac={frac}");
    }

    #[test]
    fn is_floating_body_floats() {
        let cfg = water_cfg();
        let body = floating_body();
        assert!(is_floating_body(&body, &cfg));
    }

    #[test]
    fn is_floating_body_sinks() {
        let cfg = water_cfg();
        let body = sinking_body();
        assert!(!is_floating_body(&body, &cfg));
    }

    #[test]
    fn buoyant_force_positive_when_submerged() {
        let cfg = water_cfg();
        let body = BuoyancyBody {
            position: [0.0, -10.0, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 1.0,
        };
        let r = compute_buoyancy_body(&body, &cfg);
        assert!(r.buoyant_force > 0.0);
    }

    #[test]
    fn terminal_velocity_body_positive() {
        let cfg = water_cfg();
        let body = sinking_body();
        let tv = terminal_velocity_body(&body, &cfg);
        assert!(tv >= 0.0);
    }

    #[test]
    fn effective_weight_above_water_equals_gravity_weight() {
        let cfg = water_cfg();
        let body = BuoyancyBody {
            position: [0.0, 100.0, 0.0],
            velocity: [0.0; 3],
            volume: 0.001,
            mass: 2.0,
        };
        let ew = effective_weight(&body, &cfg);
        let expected = body.mass * cfg.gravity;
        assert!((ew - expected).abs() < 1e-3, "ew={ew} expected={expected}");
    }

    #[test]
    fn apply_buoyancy_integrates_position() {
        let cfg = water_cfg();
        let mut body = floating_body();
        let initial_y = body.position[1];
        apply_buoyancy_body(&mut body, &cfg, 0.01);
        // floating body should move up
        assert!(body.position[1] > initial_y, "pos={}", body.position[1]);
    }

    #[test]
    fn json_output_contains_fields() {
        let r = BuoyancyBodyResult {
            buoyant_force: 1.0,
            drag_force: 0.5,
            net_force: 0.5,
            submerged_fraction: 0.8,
        };
        let s = buoyancy_body_result_to_json(&r);
        assert!(s.contains("buoyant_force"));
        assert!(s.contains("submerged_fraction"));
    }
}
