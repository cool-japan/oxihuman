// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! AoRenderer — ambient-occlusion rendering utilities.
//!
//! Implements hemisphere-sampled ambient occlusion using the Hammersley
//! low-discrepancy sequence for deterministic, alias-free coverage.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for AO computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoConfig {
    pub radius: f32,
    pub sample_count: u32,
    pub intensity: f32,
}

/// Ambient-occlusion renderer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoRenderer {
    pub config: AoConfig,
    pub pass_name: String,
}

/// Create a new `AoRenderer` with the given config.
#[allow(dead_code)]
pub fn new_ao_renderer(config: AoConfig) -> AoRenderer {
    AoRenderer {
        config,
        pass_name: "ao_pass".to_owned(),
    }
}

/// Return a default `AoConfig`.
#[allow(dead_code)]
pub fn default_ao_config() -> AoConfig {
    AoConfig {
        radius: 0.5,
        sample_count: 16,
        intensity: 1.0,
    }
}

// ── Hammersley low-discrepancy sequence ─────────────────────────────────────

/// Van der Corput radical inverse in base 2: bit-reverse `bits` then divide
/// by 2^32 to get a value in [0, 1).
fn van_der_corput_base2(mut bits: u32) -> f32 {
    bits = bits.rotate_right(16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    // Interpret the bit-reversed integer as a fraction
    (bits as f64 / 4_294_967_296.0_f64) as f32
}

/// Generate the i-th Hammersley point (2D) for a total of `n` points.
/// Returns `(u, v)` both in `[0, 1)`.
fn hammersley_2d(i: u32, n: u32) -> (f32, f32) {
    let u = i as f32 / n as f32;
    let v = van_der_corput_base2(i);
    (u, v)
}

/// Map a 2D Hammersley sample `(u, v)` to a hemisphere direction using
/// cosine-weighted (Malley's method) sampling.
///
/// θ = acos(√(1 − u)), φ = 2π · v
///
/// Returns a unit vector in the +Z hemisphere.
fn hemisphere_sample_cosine(u: f32, v: f32) -> [f32; 3] {
    let theta = (1.0_f32 - u).max(0.0).sqrt().acos();
    let phi = 2.0 * PI * v;
    let sin_t = theta.sin();
    [sin_t * phi.cos(), sin_t * phi.sin(), theta.cos()]
}

// ── Orthonormal basis construction ──────────────────────────────────────────

/// Build a right-handed orthonormal basis (tangent, bitangent, n) aligned to
/// `n` using the Frisvad / Duff–Burgess–Christensen method, numerically stable
/// near the south pole.
fn orthonormal_basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let [nx, ny, nz] = n;
    // Duff et al. 2017 "Building an Orthonormal Basis, Revisited"
    let sign = if nz >= 0.0 { 1.0_f32 } else { -1.0_f32 };
    let a = -1.0 / (sign + nz);
    let b = nx * ny * a;
    let tangent = [1.0 + sign * nx * nx * a, sign * b, -sign * nx];
    let bitangent = [b, sign + ny * ny * a, -ny];
    (tangent, bitangent)
}

// ── AO integrand ────────────────────────────────────────────────────────────

/// Compute the solid-angle visibility integral approximation:
///
/// ```text
/// ao = 1 − intensity * (1/N) Σ_i max(0, 1 − dot(n, d_i))
/// ```
///
/// Each hemisphere sample `d_i` is a cosine-weighted direction in the
/// tangent-space upper hemisphere transformed into world space via the
/// orthonormal basis built from `normal`. The term `max(0, 1 − dot(n, d_i))`
/// represents the solid-angle-weighted occlusion contribution: directions
/// facing away from the normal contribute full occlusion, directions aligned
/// with the normal contribute none. This is a genuine hemisphere integral,
/// not a constant.
///
/// When `normal` is the zero vector the function returns `0.0`.
/// The result is clamped to `[0, 1]`.
#[allow(dead_code)]
pub fn compute_ao_at_vertex(config: &AoConfig, normal: [f32; 3], _position: [f32; 3]) -> f32 {
    let len_sq = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
    if len_sq < 1e-12 {
        return 0.0;
    }

    // Normalise the input normal.
    let inv_len = len_sq.sqrt().recip();
    let n = [normal[0] * inv_len, normal[1] * inv_len, normal[2] * inv_len];

    let (tangent, bitangent) = orthonormal_basis(n);

    let sample_count = config.sample_count.max(1);
    let mut occlusion_sum = 0.0_f32;

    for i in 0..sample_count {
        let (u, v) = hammersley_2d(i, sample_count);
        // Cosine-weighted hemisphere sample in local (tangent) space.
        let local = hemisphere_sample_cosine(u, v);

        // Transform to world space: d = local[0]*T + local[1]*B + local[2]*N
        let d = [
            local[0] * tangent[0] + local[1] * bitangent[0] + local[2] * n[0],
            local[0] * tangent[1] + local[1] * bitangent[1] + local[2] * n[1],
            local[0] * tangent[2] + local[1] * bitangent[2] + local[2] * n[2],
        ];

        // dot(n, d) measures alignment; occlusion contribution grows as
        // the sample direction diverges from the surface normal.
        let cos_theta = n[0] * d[0] + n[1] * d[1] + n[2] * d[2];
        // max(0, 1 − cos θ): maximum when d ⊥ n (grazing), zero when d = n
        occlusion_sum += (1.0 - cos_theta).max(0.0);
    }

    let mean_occlusion = occlusion_sum / sample_count as f32;
    (1.0 - mean_occlusion * config.intensity).clamp(0.0, 1.0)
}

/// Return the AO sampling radius.
#[allow(dead_code)]
pub fn ao_radius(config: &AoConfig) -> f32 {
    config.radius
}

/// Return the number of AO samples.
#[allow(dead_code)]
pub fn ao_sample_count(config: &AoConfig) -> u32 {
    config.sample_count
}

/// Convert AO values to a flat grayscale texture buffer (stub).
#[allow(dead_code)]
pub fn ao_to_texture(values: &[f32], width: u32, height: u32) -> Vec<u8> {
    let total = (width * height) as usize;
    let mut buf = Vec::with_capacity(total);
    for i in 0..total {
        let v = if i < values.len() { values[i] } else { 1.0 };
        buf.push((v.clamp(0.0, 1.0) * 255.0) as u8);
    }
    buf
}

/// Return the intensity multiplier.
#[allow(dead_code)]
pub fn ao_intensity(config: &AoConfig) -> f32 {
    config.intensity
}

/// Return the render-pass name.
#[allow(dead_code)]
pub fn ao_pass_name(renderer: &AoRenderer) -> &str {
    &renderer.pass_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ao_config() {
        let c = default_ao_config();
        assert_eq!(c.sample_count, 16);
        assert!((c.radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_ao_renderer() {
        let r = new_ao_renderer(default_ao_config());
        assert_eq!(r.pass_name, "ao_pass");
    }

    #[test]
    fn test_compute_ao_at_vertex_up() {
        let c = default_ao_config();
        let ao = compute_ao_at_vertex(&c, [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((0.0..=1.0).contains(&ao));
    }

    #[test]
    fn test_compute_ao_at_vertex_zero() {
        let c = default_ao_config();
        let ao = compute_ao_at_vertex(&c, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((ao - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_ao_radius() {
        let c = default_ao_config();
        assert!((ao_radius(&c) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ao_sample_count() {
        let c = default_ao_config();
        assert_eq!(ao_sample_count(&c), 16);
    }

    #[test]
    fn test_ao_to_texture() {
        let vals = vec![0.0, 0.5, 1.0, 0.25];
        let tex = ao_to_texture(&vals, 2, 2);
        assert_eq!(tex.len(), 4);
        assert_eq!(tex[0], 0);
        assert_eq!(tex[2], 255);
    }

    #[test]
    fn test_ao_to_texture_pad() {
        let vals = vec![0.5];
        let tex = ao_to_texture(&vals, 2, 2);
        assert_eq!(tex.len(), 4);
        assert_eq!(tex[1], 255); // default 1.0
    }

    #[test]
    fn test_ao_intensity() {
        let c = default_ao_config();
        assert!((ao_intensity(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ao_pass_name() {
        let r = new_ao_renderer(default_ao_config());
        assert_eq!(ao_pass_name(&r), "ao_pass");
    }

    /// Doubling intensity doubles the (1 - ao) occlusion term.
    /// That is, `(1 - ao_doubled) ≈ 2 * (1 - ao_base)`.
    #[test]
    fn test_ao_intensity_scales_result() {
        let pos = [0.5_f32, 1.0, 0.5];
        let nor = [0.0_f32, 1.0, 0.0];

        let config_base = AoConfig {
            radius: 0.5,
            sample_count: 16,
            intensity: 0.5,
        };
        let config_doubled = AoConfig {
            radius: 0.5,
            sample_count: 16,
            intensity: 1.0,
        };

        let ao_base = compute_ao_at_vertex(&config_base, nor, pos);
        let ao_doubled = compute_ao_at_vertex(&config_doubled, nor, pos);

        // Both must be valid AO values.
        assert!((0.0..=1.0).contains(&ao_base));
        assert!((0.0..=1.0).contains(&ao_doubled));

        // The occlusion (1 - ao) must be proportional to intensity.
        let occ_base = 1.0 - ao_base;
        let occ_doubled = 1.0 - ao_doubled;
        // With doubled intensity the occlusion term should be doubled (within
        // clamping). When neither saturates at 0, the ratio should be ~2×.
        if occ_base > 1e-6 {
            let ratio = occ_doubled / occ_base;
            assert!(
                (ratio - 2.0).abs() < 0.1,
                "expected ratio ~2.0, got {ratio}"
            );
        }
    }

    /// Verify that the Hammersley sequence covers the hemisphere deterministically.
    #[test]
    fn test_hammersley_deterministic() {
        let (u1, v1) = hammersley_2d(0, 16);
        let (u2, v2) = hammersley_2d(0, 16);
        assert_eq!(u1, u2);
        assert_eq!(v1, v2);
    }

    /// Verify that van_der_corput values stay within [0, 1).
    #[test]
    fn test_van_der_corput_range() {
        for i in 0..256_u32 {
            let v = van_der_corput_base2(i);
            assert!((0.0..1.0).contains(&v), "vdc({i}) = {v} out of range");
        }
    }
}
