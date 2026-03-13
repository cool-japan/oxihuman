// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Wind force field with multiple zones, turbulence, and gusts.
//!
//! Provides spatial wind zones that can be combined to model complex
//! wind environments for cloth and hair simulation.

// ── Type aliases ─────────────────────────────────────────────────────────────

/// Result type for wind zone operations: `(zone_index, success)`.
#[allow(dead_code)]
pub type WindZoneOp = (usize, bool);

// ── Structs ──────────────────────────────────────────────────────────────────

/// A spatial wind zone with position, radius, and wind parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindZone {
    /// Center of the zone in world space.
    pub center: [f32; 3],
    /// Radius of influence.
    pub radius: f32,
    /// Wind direction (normalised).
    pub direction: [f32; 3],
    /// Base wind speed in m/s.
    pub speed: f32,
    /// Turbulence intensity [0, 1].
    pub turbulence: f32,
    /// Gust amplitude multiplier.
    pub gust_amplitude: f32,
    /// Gust frequency in Hz.
    pub gust_frequency: f32,
}

/// Configuration for the wind field system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindFieldConfig {
    /// Global drag coefficient.
    pub drag_coefficient: f32,
    /// Global lift coefficient.
    pub lift_coefficient: f32,
    /// Air density in kg/m^3.
    pub air_density: f32,
    /// Time step for wind updates.
    pub dt: f32,
    /// Maximum number of zones.
    pub max_zones: usize,
}

/// A sampled wind value at a specific point and time.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct WindFieldSample {
    /// Wind velocity vector at the sample point.
    pub velocity: [f32; 3],
    /// Wind speed magnitude.
    pub speed: f32,
    /// Turbulence contribution at the sample point.
    pub turbulence: [f32; 3],
}

// ── Default config ───────────────────────────────────────────────────────────

/// Return a sensible default [`WindFieldConfig`].
#[allow(dead_code)]
pub fn default_wind_config() -> WindFieldConfig {
    WindFieldConfig {
        drag_coefficient: 0.47,
        lift_coefficient: 0.2,
        air_density: 1.225,
        dt: 1.0 / 60.0,
        max_zones: 16,
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Simple LCG-based deterministic hash for reproducible turbulence.
#[inline]
fn lcg_hash(mut s: u64) -> f32 {
    s = s
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Map to [-1, 1]
    let bits = ((s >> 33) as u32) & 0x7FFF_FFFF;
    (bits as f32 / 0x7FFF_FFFF_u32 as f32) * 2.0 - 1.0
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Create a new wind zone with the given parameters.
#[allow(dead_code)]
pub fn new_wind_zone(center: [f32; 3], radius: f32, direction: [f32; 3], speed: f32) -> WindZone {
    let dir_len = len3(direction);
    let dir = if dir_len > 1e-8 {
        scale3(direction, 1.0 / dir_len)
    } else {
        [1.0, 0.0, 0.0]
    };
    WindZone {
        center,
        radius: radius.max(0.01),
        direction: dir,
        speed,
        turbulence: 0.3,
        gust_amplitude: 0.5,
        gust_frequency: 0.5,
    }
}

/// Sample wind at a given point from a single zone.
///
/// Returns `None` if the point is outside the zone's radius.
#[allow(dead_code)]
pub fn wind_at_point(zone: &WindZone, point: [f32; 3], time: f32) -> Option<WindFieldSample> {
    let diff = sub3(point, zone.center);
    let dist = len3(diff);
    if dist > zone.radius {
        return None;
    }

    // Falloff: linear from center (full) to edge (zero)
    let falloff = 1.0 - (dist / zone.radius);

    // Gust modulation
    let gust = wind_gust_factor(zone.gust_frequency, zone.gust_amplitude, time);

    // Turbulence offset
    let turb = turbulence_offset(point, time, zone.turbulence);

    let base_vel = scale3(zone.direction, zone.speed * falloff * gust);
    let velocity = add3(base_vel, turb);
    let speed = len3(velocity);

    Some(WindFieldSample {
        velocity,
        speed,
        turbulence: turb,
    })
}

/// Compute a turbulence offset vector based on position and time.
///
/// Uses deterministic LCG hashing for reproducible results.
#[allow(dead_code)]
pub fn turbulence_offset(point: [f32; 3], time: f32, intensity: f32) -> [f32; 3] {
    let seed_x = (point[0] * 1000.0 + time * 100.0) as u64;
    let seed_y = (point[1] * 1000.0 + time * 137.0) as u64;
    let seed_z = (point[2] * 1000.0 + time * 251.0) as u64;

    [
        lcg_hash(seed_x) * intensity,
        lcg_hash(seed_y) * intensity,
        lcg_hash(seed_z) * intensity,
    ]
}

/// Apply wind forces to an array of particle positions and velocities.
///
/// `velocities` are updated in-place based on the aggregate wind from all zones.
#[allow(dead_code)]
pub fn apply_wind_to_particles(
    positions: &[[f32; 3]],
    velocities: &mut [[f32; 3]],
    zones: &[WindZone],
    time: f32,
    cfg: &WindFieldConfig,
) {
    let n = positions.len().min(velocities.len());
    for i in 0..n {
        let mut total_force = [0.0f32; 3];

        for zone in zones {
            if let Some(sample) = wind_at_point(zone, positions[i], time) {
                let drag = wind_drag_force(&sample, velocities[i], cfg);
                total_force = add3(total_force, drag);

                let lift = wind_lift_force(&sample, velocities[i], cfg);
                total_force = add3(total_force, lift);
            }
        }

        // Apply force as velocity change: dv = F * dt (unit mass)
        velocities[i] = add3(velocities[i], scale3(total_force, cfg.dt));
    }
}

/// Compute drag force from wind on a particle.
///
/// F_drag = 0.5 * rho * Cd * |v_rel|^2 * v_rel_hat
#[allow(dead_code)]
pub fn wind_drag_force(
    sample: &WindFieldSample,
    particle_vel: [f32; 3],
    cfg: &WindFieldConfig,
) -> [f32; 3] {
    let v_rel = sub3(sample.velocity, particle_vel);
    let speed = len3(v_rel);
    if speed < 1e-8 {
        return [0.0; 3];
    }
    let dir = scale3(v_rel, 1.0 / speed);
    let magnitude = 0.5 * cfg.air_density * cfg.drag_coefficient * speed * speed;
    scale3(dir, magnitude)
}

/// Compute lift force from wind on a particle.
///
/// Simplified lift perpendicular to relative velocity.
#[allow(dead_code)]
pub fn wind_lift_force(
    sample: &WindFieldSample,
    particle_vel: [f32; 3],
    cfg: &WindFieldConfig,
) -> [f32; 3] {
    let v_rel = sub3(sample.velocity, particle_vel);
    let speed = len3(v_rel);
    if speed < 1e-8 {
        return [0.0; 3];
    }
    // Lift direction: perpendicular to v_rel, biased upward
    let up = [0.0f32, 1.0, 0.0];
    let v_hat = scale3(v_rel, 1.0 / speed);
    let perp_component = dot3(up, v_hat);
    let lift_dir = sub3(up, scale3(v_hat, perp_component));
    let lift_len = len3(lift_dir);
    if lift_len < 1e-8 {
        return [0.0; 3];
    }
    let lift_hat = scale3(lift_dir, 1.0 / lift_len);
    let magnitude = 0.5 * cfg.air_density * cfg.lift_coefficient * speed * speed;
    scale3(lift_hat, magnitude)
}

/// Advance the wind time parameter. Returns the new time.
#[allow(dead_code)]
pub fn update_wind_time(current_time: f32, dt: f32) -> f32 {
    current_time + dt
}

/// Compute the gust factor at a given time.
///
/// Uses a sine-based modulation: `1.0 + amplitude * sin(2*pi*freq*t)`.
#[allow(dead_code)]
pub fn wind_gust_factor(frequency: f32, amplitude: f32, time: f32) -> f32 {
    1.0 + amplitude * (std::f32::consts::TAU * frequency * time).sin()
}

/// Convert wind direction from degrees (0 = north/+Z, 90 = east/+X) to a unit vector.
#[allow(dead_code)]
pub fn wind_direction_degrees(degrees: f32) -> [f32; 3] {
    let rad = degrees.to_radians();
    [rad.sin(), 0.0, rad.cos()]
}

/// Compute the wind speed magnitude from a velocity vector.
#[allow(dead_code)]
pub fn wind_speed(velocity: [f32; 3]) -> f32 {
    len3(velocity)
}

/// Return the number of wind zones in the list.
#[allow(dead_code)]
pub fn wind_zone_count(zones: &[WindZone]) -> usize {
    zones.len()
}

/// Add a wind zone to the list. Returns the index of the new zone.
#[allow(dead_code)]
pub fn add_wind_zone(zones: &mut Vec<WindZone>, zone: WindZone) -> usize {
    let idx = zones.len();
    zones.push(zone);
    idx
}

/// Remove a wind zone by index. Returns true if the zone was removed.
#[allow(dead_code)]
pub fn remove_wind_zone(zones: &mut Vec<WindZone>, index: usize) -> bool {
    if index < zones.len() {
        zones.remove(index);
        true
    } else {
        false
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_zone() -> WindZone {
        new_wind_zone([0.0, 0.0, 0.0], 10.0, [1.0, 0.0, 0.0], 5.0)
    }

    fn cfg() -> WindFieldConfig {
        default_wind_config()
    }

    #[test]
    fn test_default_wind_config() {
        let c = default_wind_config();
        assert!(c.drag_coefficient > 0.0);
        assert!(c.air_density > 0.0);
        assert!(c.dt > 0.0);
    }

    #[test]
    fn test_new_wind_zone_normalises_direction() {
        let z = new_wind_zone([0.0; 3], 5.0, [3.0, 0.0, 4.0], 10.0);
        let l = len3(z.direction);
        assert!((l - 1.0).abs() < 1e-5, "direction should be unit length");
    }

    #[test]
    fn test_new_wind_zone_default_direction_for_zero() {
        let z = new_wind_zone([0.0; 3], 5.0, [0.0, 0.0, 0.0], 10.0);
        assert!((z.direction[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_wind_at_point_inside_zone() {
        let z = sample_zone();
        let s = wind_at_point(&z, [1.0, 0.0, 0.0], 0.0);
        assert!(s.is_some());
        assert!(s.expect("should succeed").speed > 0.0);
    }

    #[test]
    fn test_wind_at_point_outside_zone() {
        let z = sample_zone();
        let s = wind_at_point(&z, [100.0, 0.0, 0.0], 0.0);
        assert!(s.is_none());
    }

    #[test]
    fn test_wind_at_point_center_max_speed() {
        let z = sample_zone();
        let center = wind_at_point(&z, z.center, 0.0).expect("should succeed");
        let edge = wind_at_point(&z, [9.0, 0.0, 0.0], 0.0).expect("should succeed");
        // Center should have higher speed than near edge (ignoring turbulence)
        assert!(center.speed >= edge.speed * 0.5);
    }

    #[test]
    fn test_turbulence_offset_deterministic() {
        let t1 = turbulence_offset([1.0, 2.0, 3.0], 0.5, 0.3);
        let t2 = turbulence_offset([1.0, 2.0, 3.0], 0.5, 0.3);
        assert_eq!(t1[0], t2[0]);
        assert_eq!(t1[1], t2[1]);
        assert_eq!(t1[2], t2[2]);
    }

    #[test]
    fn test_turbulence_offset_zero_intensity() {
        let t = turbulence_offset([1.0, 2.0, 3.0], 0.5, 0.0);
        assert_eq!(t, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_apply_wind_to_particles() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut velocities = vec![[0.0; 3]; 2];
        let zones = vec![sample_zone()];
        apply_wind_to_particles(&positions, &mut velocities, &zones, 0.0, &cfg());
        // Particles should have gained some velocity
        let speed0 = len3(velocities[0]);
        assert!(speed0 > 0.0, "particle should be affected by wind");
    }

    #[test]
    fn test_wind_drag_force_still_air() {
        let sample = WindFieldSample {
            velocity: [0.0; 3],
            speed: 0.0,
            turbulence: [0.0; 3],
        };
        let f = wind_drag_force(&sample, [0.0; 3], &cfg());
        assert_eq!(f, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_wind_drag_force_positive() {
        let sample = WindFieldSample {
            velocity: [5.0, 0.0, 0.0],
            speed: 5.0,
            turbulence: [0.0; 3],
        };
        let f = wind_drag_force(&sample, [0.0; 3], &cfg());
        assert!(f[0] > 0.0, "drag should push in wind direction");
    }

    #[test]
    fn test_wind_lift_force_perpendicular() {
        let sample = WindFieldSample {
            velocity: [5.0, 0.0, 0.0],
            speed: 5.0,
            turbulence: [0.0; 3],
        };
        let f = wind_lift_force(&sample, [0.0; 3], &cfg());
        // Lift should be mostly upward
        assert!(f[1] > 0.0, "lift should have upward component");
    }

    #[test]
    fn test_update_wind_time() {
        let t = update_wind_time(1.0, 0.016);
        assert!((t - 1.016).abs() < 1e-6);
    }

    #[test]
    fn test_wind_gust_factor_at_zero_time() {
        let g = wind_gust_factor(1.0, 0.5, 0.0);
        assert!((g - 1.0).abs() < 1e-5, "gust at t=0 should be 1.0");
    }

    #[test]
    fn test_wind_direction_degrees_north() {
        let d = wind_direction_degrees(0.0);
        assert!(d[0].abs() < 1e-5);
        assert!((d[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_wind_direction_degrees_east() {
        let d = wind_direction_degrees(90.0);
        assert!((d[0] - 1.0).abs() < 1e-5);
        assert!(d[2].abs() < 1e-5);
    }

    #[test]
    fn test_wind_speed_fn() {
        let s = wind_speed([3.0, 4.0, 0.0]);
        assert!((s - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_wind_zone_count_empty() {
        let zones: Vec<WindZone> = Vec::new();
        assert_eq!(wind_zone_count(&zones), 0);
    }

    #[test]
    fn test_add_and_remove_wind_zone() {
        let mut zones = Vec::new();
        let idx = add_wind_zone(&mut zones, sample_zone());
        assert_eq!(idx, 0);
        assert_eq!(wind_zone_count(&zones), 1);

        let removed = remove_wind_zone(&mut zones, 0);
        assert!(removed);
        assert_eq!(wind_zone_count(&zones), 0);
    }

    #[test]
    fn test_remove_wind_zone_out_of_bounds() {
        let mut zones = Vec::new();
        let removed = remove_wind_zone(&mut zones, 5);
        assert!(!removed);
    }
}
