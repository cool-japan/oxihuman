//! Turbulent wind field using layered deterministic LCG noise.
//!
//! Three octaves of smooth noise are combined to produce a spatially and
//! temporally varying wind velocity.  No external RNG is used; all noise is
//! derived from integer arithmetic (LCG-based value noise).

#![allow(dead_code)]

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for the wind turbulence field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TurbulenceConfig {
    /// Base (mean) wind velocity vector (m/s).
    pub base_velocity: [f32; 3],
    /// Peak amplitude of turbulent fluctuations (m/s).
    pub amplitude: f32,
    /// Spatial frequency of the turbulence (higher → smaller eddies).
    pub frequency: f32,
    /// Time scale for turbulence evolution.
    pub time_scale: f32,
    /// Number of noise octaves to layer.
    pub octaves: u32,
}

/// Turbulent wind field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindTurbulence {
    /// Configuration.
    pub config: TurbulenceConfig,
    /// Current simulation time.
    pub time: f32,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default `TurbulenceConfig`.
#[allow(dead_code)]
pub fn default_turbulence_config() -> TurbulenceConfig {
    TurbulenceConfig {
        base_velocity: [2.0, 0.0, 0.0],
        amplitude: 0.5,
        frequency: 1.0,
        time_scale: 1.0,
        octaves: 3,
    }
}

/// Create a new `WindTurbulence` from the given config.
#[allow(dead_code)]
pub fn new_wind_turbulence(config: TurbulenceConfig) -> WindTurbulence {
    WindTurbulence { config, time: 0.0 }
}

/// Sample the turbulent wind velocity at world-space `point` and current time.
///
/// Returns the base velocity plus layered turbulent perturbation.
#[allow(dead_code)]
pub fn turbulence_sample(wt: &WindTurbulence, point: [f32; 3]) -> [f32; 3] {
    let f = wt.config.frequency;
    let t = wt.time * wt.config.time_scale;
    let amp = wt.config.amplitude;
    let octaves = wt.config.octaves.min(8) as i32;

    let mut wx = 0.0f32;
    let mut wy = 0.0f32;
    let mut wz = 0.0f32;

    let mut freq = f;
    let mut a = 1.0f32;
    let mut a_sum = 0.0f32;

    for oct in 0..octaves {
        let seed_x = (oct * 7919) as u64;
        let seed_y = (oct * 6271 + 1) as u64;
        let seed_z = (oct * 5381 + 2) as u64;

        wx += a * value_noise4d(point[0] * freq, point[1] * freq, point[2] * freq, t, seed_x);
        wy += a * value_noise4d(point[0] * freq, point[1] * freq, point[2] * freq, t, seed_y);
        wz += a * value_noise4d(point[0] * freq, point[1] * freq, point[2] * freq, t, seed_z);

        a_sum += a;
        a *= 0.5;
        freq *= 2.0;
    }

    // Normalise octave sum and scale to amplitude
    let inv = if a_sum > 1e-10 { amp / a_sum } else { 0.0 };

    [
        wt.config.base_velocity[0] + wx * inv,
        wt.config.base_velocity[1] + wy * inv,
        wt.config.base_velocity[2] + wz * inv,
    ]
}

/// Set the internal simulation time.
#[allow(dead_code)]
pub fn turbulence_set_time(wt: &mut WindTurbulence, time: f32) {
    wt.time = time;
}

/// Advance the simulation time by `dt` seconds.
#[allow(dead_code)]
pub fn turbulence_advance_time(wt: &mut WindTurbulence, dt: f32) {
    wt.time += dt;
}

/// Return the base velocity.
#[allow(dead_code)]
pub fn turbulence_base_velocity(wt: &WindTurbulence) -> [f32; 3] {
    wt.config.base_velocity
}

/// Return the configured amplitude.
#[allow(dead_code)]
pub fn turbulence_amplitude(wt: &WindTurbulence) -> f32 {
    wt.config.amplitude
}

/// Return the configured frequency.
#[allow(dead_code)]
pub fn turbulence_frequency(wt: &WindTurbulence) -> f32 {
    wt.config.frequency
}

/// Serialise the config to a JSON string.
#[allow(dead_code)]
pub fn turbulence_to_json(wt: &WindTurbulence) -> String {
    format!(
        "{{\"amplitude\":{},\"frequency\":{},\"time_scale\":{},\"octaves\":{}}}",
        wt.config.amplitude,
        wt.config.frequency,
        wt.config.time_scale,
        wt.config.octaves,
    )
}

/// Reset the simulation time to zero.
#[allow(dead_code)]
pub fn turbulence_reset(wt: &mut WindTurbulence) {
    wt.time = 0.0;
}

// ── Deterministic LCG value noise ─────────────────────────────────────────────

/// Quantise a float to an integer cell coordinate.
#[inline]
fn qfloor(x: f32) -> i64 {
    x.floor() as i64
}

/// LCG hash of a 64-bit integer seed.
#[inline]
fn lcg_hash(mut x: u64) -> u64 {
    x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    x ^= x >> 32;
    x
}

/// Combine up to four integer coordinates into a single hash.
#[inline]
fn hash4(ix: i64, iy: i64, iz: i64, it: i64, seed: u64) -> f32 {
    let h = lcg_hash(
        lcg_hash(
            lcg_hash(
                lcg_hash(seed ^ (ix as u64).wrapping_mul(2654435761))
                    ^ (iy as u64).wrapping_mul(805459861)
            ) ^ (iz as u64).wrapping_mul(3674653429)
        ) ^ (it as u64).wrapping_mul(1234567891),
    );
    // Map to [-1, 1]
    (h as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
}

/// Smooth Hermite fade curve.
#[inline]
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// 4-D value noise (trilinear interpolation over 3-D lattice cells, time axis added as a 4th coord).
fn value_noise4d(x: f32, y: f32, z: f32, t: f32, seed: u64) -> f32 {
    let ix = qfloor(x);
    let iy = qfloor(y);
    let iz = qfloor(z);
    let it = qfloor(t);

    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let fz = z - iz as f32;
    let ft = t - it as f32;

    let ux = fade(fx);
    let uy = fade(fy);
    let uz = fade(fz);
    let ut = fade(ft);

    // Interpolate over time axis first (2 temporal layers)
    let lerp = |a: f32, b: f32, u: f32| a + (b - a) * u;

    let v = |dx: i64, dy: i64, dz: i64, dt: i64| {
        hash4(ix + dx, iy + dy, iz + dz, it + dt, seed)
    };

    // 8 corners at t0
    let c000t0 = lerp(v(0,0,0,0), v(0,0,0,1), ut);
    let c100t  = lerp(v(1,0,0,0), v(1,0,0,1), ut);
    let c010t  = lerp(v(0,1,0,0), v(0,1,0,1), ut);
    let c110t  = lerp(v(1,1,0,0), v(1,1,0,1), ut);
    let c001t  = lerp(v(0,0,1,0), v(0,0,1,1), ut);
    let c101t  = lerp(v(1,0,1,0), v(1,0,1,1), ut);
    let c011t  = lerp(v(0,1,1,0), v(0,1,1,1), ut);
    let c111t  = lerp(v(1,1,1,0), v(1,1,1,1), ut);

    // Trilinear interpolation over space
    let c00 = lerp(c000t0, c100t, ux);
    let c10 = lerp(c010t,  c110t, ux);
    let c01 = lerp(c001t,  c101t, ux);
    let c11 = lerp(c011t,  c111t, ux);
    let c0  = lerp(c00, c10, uy);
    let c1  = lerp(c01, c11, uy);
    lerp(c0, c1, uz)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_amplitude_positive() {
        let cfg = default_turbulence_config();
        assert!(cfg.amplitude > 0.0);
    }

    #[test]
    fn test_new_wind_turbulence_time_zero() {
        let cfg = default_turbulence_config();
        let wt = new_wind_turbulence(cfg);
        assert!((wt.time - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_turbulence_sample_finite() {
        let cfg = default_turbulence_config();
        let wt = new_wind_turbulence(cfg);
        let v = turbulence_sample(&wt, [1.0, 2.0, 3.0]);
        assert!(v[0].is_finite() && v[1].is_finite() && v[2].is_finite());
    }

    #[test]
    fn test_sample_varies_with_position() {
        let cfg = default_turbulence_config();
        let wt = new_wind_turbulence(cfg);
        let v0 = turbulence_sample(&wt, [0.0, 0.0, 0.0]);
        let v1 = turbulence_sample(&wt, [10.0, 0.0, 0.0]);
        // Should differ (almost certainly)
        let diff = (v0[0] - v1[0]).abs() + (v0[1] - v1[1]).abs() + (v0[2] - v1[2]).abs();
        assert!(diff > 0.0);
    }

    #[test]
    fn test_sample_varies_with_time() {
        let cfg = default_turbulence_config();
        let mut wt = new_wind_turbulence(cfg);
        let v0 = turbulence_sample(&wt, [0.0, 0.0, 0.0]);
        turbulence_advance_time(&mut wt, 5.0);
        let v1 = turbulence_sample(&wt, [0.0, 0.0, 0.0]);
        let diff = (v0[0] - v1[0]).abs() + (v0[1] - v1[1]).abs() + (v0[2] - v1[2]).abs();
        assert!(diff > 0.0);
    }

    #[test]
    fn test_set_time() {
        let cfg = default_turbulence_config();
        let mut wt = new_wind_turbulence(cfg);
        turbulence_set_time(&mut wt, 3.5);
        assert!((wt.time - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_advance_time() {
        let cfg = default_turbulence_config();
        let mut wt = new_wind_turbulence(cfg);
        turbulence_advance_time(&mut wt, 0.1);
        assert!((wt.time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_base_velocity_accessor() {
        let cfg = TurbulenceConfig { base_velocity: [3.0, 1.0, 0.5], ..default_turbulence_config() };
        let wt = new_wind_turbulence(cfg);
        let bv = turbulence_base_velocity(&wt);
        assert!((bv[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_amplitude_accessor() {
        let cfg = TurbulenceConfig { amplitude: 2.5, ..default_turbulence_config() };
        let wt = new_wind_turbulence(cfg);
        assert!((turbulence_amplitude(&wt) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_frequency_accessor() {
        let cfg = TurbulenceConfig { frequency: 4.0, ..default_turbulence_config() };
        let wt = new_wind_turbulence(cfg);
        assert!((turbulence_frequency(&wt) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_contains_amplitude() {
        let cfg = default_turbulence_config();
        let wt = new_wind_turbulence(cfg);
        let json = turbulence_to_json(&wt);
        assert!(json.contains("amplitude"));
        assert!(json.contains("frequency"));
    }

    #[test]
    fn test_reset_zeros_time() {
        let cfg = default_turbulence_config();
        let mut wt = new_wind_turbulence(cfg);
        turbulence_advance_time(&mut wt, 100.0);
        turbulence_reset(&mut wt);
        assert!((wt.time - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_determinism() {
        let cfg = default_turbulence_config();
        let wt = new_wind_turbulence(cfg);
        let v0 = turbulence_sample(&wt, [1.23, 4.56, 7.89]);
        let v1 = turbulence_sample(&wt, [1.23, 4.56, 7.89]);
        assert_eq!(v0, v1, "noise must be deterministic");
    }
}
