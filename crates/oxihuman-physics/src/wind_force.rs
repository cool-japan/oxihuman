// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Turbulent wind field with Simplex-like value noise for vortex and gusting effects.

// ── Config / Field ────────────────────────────────────────────────────────────

/// Configuration for a turbulent wind field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindConfig {
    /// Normalised wind direction, e.g. `[1.0, 0.0, 0.0]`.
    pub base_direction: [f32; 3],
    /// Base wind speed in m/s, e.g. `5.0`.
    pub base_speed: f32,
    /// Turbulence intensity in 0..1, default `0.3`.
    pub turbulence: f32,
    /// Gusting frequency in Hz, default `0.5`.
    pub gust_frequency: f32,
    /// Rotational (vortex) component strength, default `0.2`.
    pub vortex_strength: f32,
    /// Seed for reproducible noise.
    pub seed: u64,
}

impl Default for WindConfig {
    fn default() -> Self {
        Self {
            base_direction: [1.0, 0.0, 0.0],
            base_speed: 5.0,
            turbulence: 0.3,
            gust_frequency: 0.5,
            vortex_strength: 0.2,
            seed: 42,
        }
    }
}

/// A turbulent wind field that can be sampled at any world position and time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindField {
    pub config: WindConfig,
}

/// A single wind measurement at a point in space and time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindSample {
    /// World-space wind velocity at this point and time (m/s).
    pub velocity: [f32; 3],
    /// Dynamic pressure `= 0.5 * rho * v²`, where `rho = 1.225 kg/m³`.
    pub pressure: f32,
}

// ── core noise ────────────────────────────────────────────────────────────────

/// LCG hash: maps an integer to a pseudo-random u64.
#[inline]
fn lcg_hash(v: u64) -> u64 {
    v.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

/// Hash three integer grid coordinates and a seed to a float in `-1..1`.
#[inline]
fn hash_corner(ix: i32, iy: i32, iz: i32, seed: u64) -> f32 {
    let h = lcg_hash(lcg_hash(lcg_hash(seed ^ (ix as u64)) ^ (iy as u64)) ^ (iz as u64));
    // map 0..u64::MAX → -1..1
    (h as f32 / u64::MAX as f32) * 2.0 - 1.0
}

/// Smooth Hermite interpolation (`3t² - 2t³`).
#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Trilinear interpolation of eight scalar values.
#[allow(clippy::too_many_arguments)]
#[inline]
fn trilerp(
    c000: f32,
    c100: f32,
    c010: f32,
    c110: f32,
    c001: f32,
    c101: f32,
    c011: f32,
    c111: f32,
    tx: f32,
    ty: f32,
    tz: f32,
) -> f32 {
    let c00 = c000 + tx * (c100 - c000);
    let c10 = c010 + tx * (c110 - c010);
    let c01 = c001 + tx * (c101 - c001);
    let c11 = c011 + tx * (c111 - c011);
    let c0 = c00 + ty * (c10 - c00);
    let c1 = c01 + ty * (c11 - c01);
    c0 + tz * (c1 - c0)
}

/// Smooth value noise in the range `-1..1` using LCG corner hashing and
/// trilinear interpolation with Hermite smoothing.
#[allow(dead_code)]
pub fn value_noise_3d(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let fx = smoothstep(x - ix as f32);
    let fy = smoothstep(y - iy as f32);
    let fz = smoothstep(z - iz as f32);

    trilerp(
        hash_corner(ix, iy, iz, seed),
        hash_corner(ix + 1, iy, iz, seed),
        hash_corner(ix, iy + 1, iz, seed),
        hash_corner(ix + 1, iy + 1, iz, seed),
        hash_corner(ix, iy, iz + 1, seed),
        hash_corner(ix + 1, iy, iz + 1, seed),
        hash_corner(ix, iy + 1, iz + 1, seed),
        hash_corner(ix + 1, iy + 1, iz + 1, seed),
        fx,
        fy,
        fz,
    )
}

// ── dynamic pressure / Beaufort ───────────────────────────────────────────────

/// Dynamic pressure `= 0.5 * 1.225 * speed²` (Pa).
#[allow(dead_code)]
pub fn dynamic_pressure(speed: f32) -> f32 {
    0.5 * 1.225 * speed * speed
}

/// Convert wind speed (m/s) to Beaufort scale number (0..=12).
#[allow(dead_code)]
pub fn wind_speed_beaufort(speed_ms: f32) -> u8 {
    // Beaufort scale thresholds (upper bound exclusive in m/s)
    const THRESHOLDS: [f32; 13] = [
        0.3,
        1.6,
        3.4,
        5.5,
        8.0,
        10.8,
        13.9,
        17.2,
        20.8,
        24.5,
        28.5,
        32.7,
        f32::INFINITY,
    ];
    for (i, &upper) in THRESHOLDS.iter().enumerate() {
        if speed_ms < upper {
            return i as u8;
        }
    }
    12
}

// ── WindField impl ────────────────────────────────────────────────────────────

impl WindField {
    /// Create a new wind field with the given configuration.
    #[allow(dead_code)]
    pub fn new(config: WindConfig) -> Self {
        Self { config }
    }

    /// Sample wind velocity and pressure at `position` and `time`.
    ///
    /// Combines a base flow with turbulence (value noise), sinusoidal gusting,
    /// and a vortex component perpendicular to the base direction.
    #[allow(dead_code)]
    pub fn sample(&self, position: [f32; 3], time: f32) -> WindSample {
        let cfg = &self.config;

        // Turbulence: one noise sample per axis, scaled and offset by time
        let scale = 0.5_f32;
        let tx = value_noise_3d(
            position[0] * scale + time * 0.1,
            position[1] * scale,
            position[2] * scale,
            cfg.seed,
        );
        let ty = value_noise_3d(
            position[0] * scale,
            position[1] * scale + time * 0.1,
            position[2] * scale,
            cfg.seed.wrapping_add(1),
        );
        let tz = value_noise_3d(
            position[0] * scale,
            position[1] * scale,
            position[2] * scale + time * 0.1,
            cfg.seed.wrapping_add(2),
        );

        // Gusting: sinusoidal modulation of base speed
        let gust =
            1.0 + cfg.turbulence * (2.0 * std::f32::consts::PI * cfg.gust_frequency * time).sin();
        let speed = cfg.base_speed * gust;

        // Vortex: a vector perpendicular to base_direction
        let bd = cfg.base_direction;
        // Simple perpendicular: if bd is not [0,1,0], cross with [0,1,0]
        let up = if bd[1].abs() < 0.9 {
            [0.0_f32, 1.0, 0.0]
        } else {
            [1.0_f32, 0.0, 0.0]
        };
        let vortex = cross3(bd, up);

        let vx = bd[0] * speed
            + cfg.turbulence * tx * cfg.base_speed
            + cfg.vortex_strength * vortex[0] * cfg.base_speed;
        let vy = bd[1] * speed
            + cfg.turbulence * ty * cfg.base_speed
            + cfg.vortex_strength * vortex[1] * cfg.base_speed;
        let vz = bd[2] * speed
            + cfg.turbulence * tz * cfg.base_speed
            + cfg.vortex_strength * vortex[2] * cfg.base_speed;

        let velocity = [vx, vy, vz];
        let spd = (vx * vx + vy * vy + vz * vz).sqrt();
        let pressure = dynamic_pressure(spd);

        WindSample { velocity, pressure }
    }
}

// ── wind force helpers ────────────────────────────────────────────────────────

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
#[allow(dead_code)]
fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Aerodynamic pressure force on a face.
///
/// `F = pressure * dot(v_norm, face_normal) * area * face_normal`
/// Only applied when the wind has a component toward the face (`dot > 0`).
#[allow(dead_code)]
pub fn wind_force_on_face(wind: &WindSample, face_normal: [f32; 3], face_area: f32) -> [f32; 3] {
    let spd =
        (wind.velocity[0].powi(2) + wind.velocity[1].powi(2) + wind.velocity[2].powi(2)).sqrt();
    if spd < 1e-9 {
        return [0.0; 3];
    }
    let v_norm = [
        wind.velocity[0] / spd,
        wind.velocity[1] / spd,
        wind.velocity[2] / spd,
    ];
    let d = dot3(v_norm, face_normal);
    if d <= 0.0 {
        return [0.0; 3];
    }
    let mag = wind.pressure * d * face_area;
    [
        mag * face_normal[0],
        mag * face_normal[1],
        mag * face_normal[2],
    ]
}

/// Apply wind forces to every non-pinned particle in a cloth simulation.
///
/// For each particle, samples the wind at its position, computes a force as
/// if the particle has a unit-area face with upward normal, then integrates:
/// `velocity += force * dt / mass`.
#[allow(dead_code)]
pub fn apply_wind_to_cloth(
    cloth: &mut super::cloth::ClothSim,
    wind: &WindField,
    time: f32,
    dt: f32,
) {
    for particle in cloth.particles.iter_mut() {
        if particle.pinned {
            continue;
        }
        let ws = wind.sample(particle.position, time);
        // Unit upward normal as a proxy for the cloth surface element
        let normal = [0.0_f32, 1.0, 0.0];
        let force = wind_force_on_face(&ws, normal, 1.0);
        let inv_m = if particle.mass > 1e-9 {
            1.0 / particle.mass
        } else {
            0.0
        };
        particle.position[0] += force[0] * dt * inv_m;
        particle.position[1] += force[1] * dt * inv_m;
        particle.position[2] += force[2] * dt * inv_m;
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloth::{ClothParticle, ClothSim, Spring};

    fn default_config() -> WindConfig {
        WindConfig::default()
    }

    // 1. value_noise_3d is deterministic
    #[test]
    fn noise_deterministic() {
        let a = value_noise_3d(1.5, 2.3, 0.7, 42);
        let b = value_noise_3d(1.5, 2.3, 0.7, 42);
        assert_eq!(a, b);
    }

    // 2. different seeds produce different results
    #[test]
    fn noise_seed_differs() {
        let a = value_noise_3d(1.0, 1.0, 1.0, 0);
        let b = value_noise_3d(1.0, 1.0, 1.0, 99);
        assert_ne!(a, b);
    }

    // 3. output is within -1..1
    #[test]
    fn noise_range() {
        for i in 0..20 {
            let v = value_noise_3d(i as f32 * 0.37, i as f32 * 0.13, i as f32 * 0.71, 7);
            assert!((-1.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    // 4. nearby samples are close (smoothness)
    #[test]
    fn noise_smooth() {
        let a = value_noise_3d(1.0, 1.0, 1.0, 1);
        let b = value_noise_3d(1.001, 1.0, 1.0, 1);
        assert!((a - b).abs() < 0.01, "not smooth: diff={}", (a - b).abs());
    }

    // 5. dynamic_pressure formula
    #[test]
    fn dynamic_pressure_formula() {
        let p = dynamic_pressure(10.0);
        let expected = 0.5 * 1.225 * 100.0;
        assert!((p - expected).abs() < 1e-4, "got {p}, expected {expected}");
    }

    // 6. dynamic_pressure zero at zero speed
    #[test]
    fn dynamic_pressure_zero() {
        assert_eq!(dynamic_pressure(0.0), 0.0);
    }

    // 7. Beaufort calm (< 0.3 m/s) → 0
    #[test]
    fn beaufort_calm() {
        assert_eq!(wind_speed_beaufort(0.0), 0);
        assert_eq!(wind_speed_beaufort(0.2), 0);
    }

    // 8. Beaufort light breeze 1.6..3.4 → 2
    #[test]
    fn beaufort_breeze() {
        assert_eq!(wind_speed_beaufort(2.5), 2);
    }

    // 9. Beaufort gale ≥ 17.2 m/s → 8
    #[test]
    fn beaufort_gale() {
        assert_eq!(wind_speed_beaufort(18.0), 8);
    }

    // 10. Beaufort max: 33 m/s → 12
    #[test]
    fn beaufort_max() {
        assert_eq!(wind_speed_beaufort(33.0), 12);
    }

    // 11. wind_force_on_face zero when backfacing
    #[test]
    fn wind_force_backfacing_zero() {
        let ws = WindSample {
            velocity: [1.0, 0.0, 0.0],
            pressure: dynamic_pressure(1.0),
        };
        // normal points away from wind
        let f = wind_force_on_face(&ws, [-1.0, 0.0, 0.0], 1.0);
        assert_eq!(f, [0.0; 3]);
    }

    // 12. wind_force_on_face magnitude proportional to area
    #[test]
    fn wind_force_proportional_to_area() {
        let ws = WindSample {
            velocity: [1.0, 0.0, 0.0],
            pressure: dynamic_pressure(1.0),
        };
        let f1 = wind_force_on_face(&ws, [1.0, 0.0, 0.0], 1.0);
        let f2 = wind_force_on_face(&ws, [1.0, 0.0, 0.0], 2.0);
        let mag1 = len3(f1);
        let mag2 = len3(f2);
        assert!((mag2 - 2.0 * mag1).abs() < 1e-5, "mag1={mag1}, mag2={mag2}");
    }

    // 13. WindSample pressure matches speed via dynamic_pressure
    #[test]
    fn wind_sample_pressure_matches_speed() {
        let cfg = default_config();
        let field = WindField::new(cfg);
        let ws = field.sample([0.0, 0.0, 0.0], 0.0);
        let spd = len3(ws.velocity);
        let expected = dynamic_pressure(spd);
        assert!((ws.pressure - expected).abs() < 1e-3, "pressure mismatch");
    }

    // 14. sample at different times gives different velocities
    #[test]
    fn sample_time_varying() {
        let field = WindField::new(WindConfig {
            turbulence: 0.5,
            gust_frequency: 1.0,
            ..WindConfig::default()
        });
        let s0 = field.sample([0.0; 3], 0.0);
        let s1 = field.sample([0.0; 3], 1.0);
        // At t=0 and t=1 the gust sinusoid differs, so velocities should differ
        let diff = len3([
            s0.velocity[0] - s1.velocity[0],
            s0.velocity[1] - s1.velocity[1],
            s0.velocity[2] - s1.velocity[2],
        ]);
        assert!(diff > 1e-4, "time-varying expected, diff={diff}");
    }

    // 15. apply_wind_to_cloth runs without panic on small cloth
    #[test]
    fn apply_wind_no_panic() {
        let mut sim = ClothSim {
            particles: vec![
                ClothParticle::new([0.0, 0.0, 0.0], 0.1),
                ClothParticle::new([0.1, 0.0, 0.0], 0.1),
            ],
            springs: vec![Spring {
                a: 0,
                b: 1,
                rest_length: 0.1,
                stiffness: 100.0,
                kind: crate::cloth::SpringKind::Structural,
            }],
            gravity: [0.0, -9.81, 0.0],
            damping: 0.99,
        };
        let field = WindField::new(WindConfig::default());
        apply_wind_to_cloth(&mut sim, &field, 0.0, 0.016);
        // Simply assert positions are finite
        for p in &sim.particles {
            assert!(p.position[0].is_finite());
        }
    }
}
