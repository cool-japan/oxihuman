// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Light probe v2 — SH9 irradiance probes with blending and spatial lookup.

use std::f32::consts::PI;

/// Number of SH L2 coefficients.
pub const SH9_COUNT: usize = 9;

/// SH L2 coefficient set (9 × RGB).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Sh9 {
    pub coeffs: [[f32; 3]; SH9_COUNT],
}

impl Default for Sh9 {
    fn default() -> Self {
        Self {
            coeffs: [[0.0; 3]; SH9_COUNT],
        }
    }
}

/// A single irradiance probe v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightProbeV2 {
    pub id: u32,
    pub position: [f32; 3],
    pub radius: f32,
    pub sh: Sh9,
    pub enabled: bool,
}

/// Probe set.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LightProbeSetV2 {
    probes: Vec<LightProbeV2>,
}

#[allow(dead_code)]
pub fn new_light_probe_set_v2() -> LightProbeSetV2 {
    LightProbeSetV2::default()
}

#[allow(dead_code)]
pub fn lp2_add(set: &mut LightProbeSetV2, id: u32, pos: [f32; 3], radius: f32) {
    set.probes.push(LightProbeV2 {
        id,
        position: pos,
        radius: radius.max(0.01),
        sh: Sh9::default(),
        enabled: true,
    });
}

#[allow(dead_code)]
pub fn lp2_remove(set: &mut LightProbeSetV2, id: u32) {
    set.probes.retain(|p| p.id != id);
}

#[allow(dead_code)]
pub fn lp2_set_sh(set: &mut LightProbeSetV2, id: u32, sh: Sh9) {
    for p in set.probes.iter_mut() {
        if p.id == id {
            p.sh = sh;
        }
    }
}

#[allow(dead_code)]
pub fn lp2_set_enabled(set: &mut LightProbeSetV2, id: u32, en: bool) {
    for p in set.probes.iter_mut() {
        if p.id == id {
            p.enabled = en;
        }
    }
}

#[allow(dead_code)]
pub fn lp2_count(set: &LightProbeSetV2) -> usize {
    set.probes.len()
}

#[allow(dead_code)]
pub fn lp2_enabled_count(set: &LightProbeSetV2) -> usize {
    set.probes.iter().filter(|p| p.enabled).count()
}

#[allow(dead_code)]
pub fn lp2_clear(set: &mut LightProbeSetV2) {
    set.probes.clear();
}

/// Influence weight of probe at `pos` from a query point.
#[allow(dead_code)]
pub fn lp2_influence(probe: &LightProbeV2, query: [f32; 3]) -> f32 {
    let dx = probe.position[0] - query[0];
    let dy = probe.position[1] - query[1];
    let dz = probe.position[2] - query[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist >= probe.radius {
        0.0
    } else {
        1.0 - dist / probe.radius
    }
}

/// Find nearest probe to a query point (by centre distance).
#[allow(dead_code)]
pub fn lp2_nearest(set: &LightProbeSetV2, query: [f32; 3]) -> Option<&LightProbeV2> {
    set.probes.iter().filter(|p| p.enabled).min_by(|a, b| {
        let da = dist3(a.position, query);
        let db = dist3(b.position, query);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Blend two SH9 sets with weight t (0..=1 towards b).
#[allow(dead_code)]
pub fn lp2_blend_sh(a: &Sh9, b: &Sh9, t: f32) -> Sh9 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let mut out = Sh9::default();
    #[allow(clippy::needless_range_loop)]
    for i in 0..SH9_COUNT {
        out.coeffs[i][0] = a.coeffs[i][0] * inv + b.coeffs[i][0] * t;
        out.coeffs[i][1] = a.coeffs[i][1] * inv + b.coeffs[i][1] * t;
        out.coeffs[i][2] = a.coeffs[i][2] * inv + b.coeffs[i][2] * t;
    }
    out
}

/// Evaluate SH L2 irradiance for a normal direction.
#[allow(dead_code)]
pub fn lp2_eval_sh(sh: &Sh9, normal: [f32; 3]) -> [f32; 3] {
    let nx = normal[0];
    let ny = normal[1];
    let nz = normal[2];
    let basis = [
        0.282_095,
        0.488_603 * ny,
        0.488_603 * nz,
        0.488_603 * nx,
        1.092_548 * nx * ny,
        1.092_548 * ny * nz,
        0.315_392 * (3.0 * nz * nz - 1.0),
        1.092_548 * nx * nz,
        0.546_274 * (nx * nx - ny * ny),
    ];
    let mut out = [0.0f32; 3];
    #[allow(clippy::needless_range_loop)]
    for i in 0..SH9_COUNT {
        out[0] += sh.coeffs[i][0] * basis[i];
        out[1] += sh.coeffs[i][1] * basis[i];
        out[2] += sh.coeffs[i][2] * basis[i];
    }
    out
}

/// Ambient irradiance magnitude (for sky-blue constant probe).
#[allow(dead_code)]
pub fn lp2_ambient_magnitude(sh: &Sh9) -> f32 {
    let lum = 0.2126 * sh.coeffs[0][0] + 0.7152 * sh.coeffs[0][1] + 0.0722 * sh.coeffs[0][2];
    lum * 2.0 * PI
}

#[allow(dead_code)]
pub fn lp2_to_json(set: &LightProbeSetV2) -> String {
    format!(
        "{{\"count\":{},\"enabled\":{}}}",
        lp2_count(set),
        lp2_enabled_count(set)
    )
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set() {
        assert_eq!(lp2_count(&new_light_probe_set_v2()), 0);
    }

    #[test]
    fn add_and_count() {
        let mut s = new_light_probe_set_v2();
        lp2_add(&mut s, 1, [0.0, 0.0, 0.0], 5.0);
        assert_eq!(lp2_count(&s), 1);
    }

    #[test]
    fn remove_by_id() {
        let mut s = new_light_probe_set_v2();
        lp2_add(&mut s, 1, [0.0; 3], 5.0);
        lp2_remove(&mut s, 1);
        assert_eq!(lp2_count(&s), 0);
    }

    #[test]
    fn influence_at_center_is_one() {
        let mut s = new_light_probe_set_v2();
        lp2_add(&mut s, 1, [0.0; 3], 5.0);
        let w = lp2_influence(&s.probes[0], [0.0; 3]);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn influence_outside_radius_is_zero() {
        let mut s = new_light_probe_set_v2();
        lp2_add(&mut s, 1, [0.0; 3], 1.0);
        let w = lp2_influence(&s.probes[0], [10.0, 0.0, 0.0]);
        assert!(w < 1e-6);
    }

    #[test]
    fn nearest_returns_closest() {
        let mut s = new_light_probe_set_v2();
        lp2_add(&mut s, 1, [0.0, 0.0, 0.0], 10.0);
        lp2_add(&mut s, 2, [5.0, 0.0, 0.0], 10.0);
        let p = lp2_nearest(&s, [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(p.id, 1);
    }

    #[test]
    fn blend_sh_identity() {
        let sh = Sh9::default();
        let blended = lp2_blend_sh(&sh, &sh, 0.5);
        assert!((blended.coeffs[0][0]).abs() < 1e-6);
    }

    #[test]
    fn eval_sh_zero_for_default() {
        let sh = Sh9::default();
        let out = lp2_eval_sh(&sh, [0.0, 1.0, 0.0]);
        assert!(out[0].abs() < 1e-5);
    }

    #[test]
    fn clear_empties() {
        let mut s = new_light_probe_set_v2();
        lp2_add(&mut s, 1, [0.0; 3], 5.0);
        lp2_clear(&mut s);
        assert_eq!(lp2_count(&s), 0);
    }

    #[test]
    fn json_has_count() {
        let j = lp2_to_json(&new_light_probe_set_v2());
        assert!(j.contains("count"));
    }
}
