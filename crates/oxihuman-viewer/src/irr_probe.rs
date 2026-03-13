// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Irradiance probe: captures and stores SH-based diffuse irradiance.

use std::f32::consts::PI;

/// Second-order spherical harmonics coefficient set (L0 + L1 + L2 = 9 × RGB).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IrrSh9 {
    /// SH coefficients: 9 entries, each an RGB triple.
    pub coeff: [[f32; 3]; 9],
}

impl Default for IrrSh9 {
    fn default() -> Self {
        Self {
            coeff: [[0.0; 3]; 9],
        }
    }
}

/// A placed irradiance probe in world space.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IrrProbe {
    pub position: [f32; 3],
    pub radius: f32,
    pub sh: IrrSh9,
    pub weight: f32,
}

impl IrrProbe {
    /// Create a new probe at `position` with given `radius`.
    #[allow(dead_code)]
    pub fn new(position: [f32; 3], radius: f32) -> Self {
        Self {
            position,
            radius,
            sh: IrrSh9::default(),
            weight: 1.0,
        }
    }
}

/// A collection of irradiance probes.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct IrrProbeSet {
    probes: Vec<IrrProbe>,
}

impl IrrProbeSet {
    /// Add a probe to the set.
    #[allow(dead_code)]
    pub fn add(&mut self, probe: IrrProbe) {
        self.probes.push(probe);
    }

    /// Number of probes.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.probes.len()
    }

    /// Return true when there are no probes.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.probes.is_empty()
    }
}

/// Evaluate L0 (ambient) irradiance from SH coefficients.
#[allow(dead_code)]
pub fn sh_l0(sh: &IrrSh9) -> [f32; 3] {
    let scale = 2.0 * (PI).sqrt();
    [
        sh.coeff[0][0] * scale,
        sh.coeff[0][1] * scale,
        sh.coeff[0][2] * scale,
    ]
}

/// Find the index of the probe closest to `point`.
#[allow(dead_code)]
pub fn nearest_probe(set: &IrrProbeSet, point: [f32; 3]) -> Option<usize> {
    if set.is_empty() {
        return None;
    }
    let mut best = 0;
    let mut best_dist2 = f32::MAX;
    for (i, p) in set.probes.iter().enumerate() {
        let dx = p.position[0] - point[0];
        let dy = p.position[1] - point[1];
        let dz = p.position[2] - point[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        if d2 < best_dist2 {
            best_dist2 = d2;
            best = i;
        }
    }
    Some(best)
}

/// Blend two SH sets by `t ∈ [0, 1]`.
#[allow(dead_code)]
pub fn blend_sh(a: &IrrSh9, b: &IrrSh9, t: f32) -> IrrSh9 {
    let t = t.clamp(0.0, 1.0);
    let mut out = IrrSh9::default();
    for i in 0..9 {
        for c in 0..3 {
            out.coeff[i][c] = a.coeff[i][c] + (b.coeff[i][c] - a.coeff[i][c]) * t;
        }
    }
    out
}

/// Scale all SH coefficients by `s`.
#[allow(dead_code)]
pub fn scale_sh(sh: &mut IrrSh9, s: f32) {
    for coeff in sh.coeff.iter_mut() {
        for c in coeff.iter_mut() {
            *c *= s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sh_zero() {
        let sh = IrrSh9::default();
        assert_eq!(sh.coeff[0], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn sh_l0_scales_by_2_sqrt_pi() {
        let mut sh = IrrSh9::default();
        sh.coeff[0] = [1.0, 1.0, 1.0];
        let l0 = sh_l0(&sh);
        let scale = 2.0 * PI.sqrt();
        assert!((l0[0] - scale).abs() < 1e-5);
    }

    #[test]
    fn nearest_probe_empty_returns_none() {
        let set = IrrProbeSet::default();
        assert!(nearest_probe(&set, [0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn nearest_probe_single() {
        let mut set = IrrProbeSet::default();
        set.add(IrrProbe::new([0.0, 0.0, 0.0], 1.0));
        assert_eq!(nearest_probe(&set, [0.5, 0.0, 0.0]), Some(0));
    }

    #[test]
    fn nearest_probe_picks_closer() {
        let mut set = IrrProbeSet::default();
        set.add(IrrProbe::new([0.0, 0.0, 0.0], 1.0));
        set.add(IrrProbe::new([10.0, 0.0, 0.0], 1.0));
        assert_eq!(nearest_probe(&set, [1.0, 0.0, 0.0]), Some(0));
    }

    #[test]
    fn blend_sh_midpoint() {
        let a = IrrSh9::default();
        let mut b = IrrSh9::default();
        b.coeff[0] = [2.0, 2.0, 2.0];
        let mid = blend_sh(&a, &b, 0.5);
        assert!((mid.coeff[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn scale_sh_doubles() {
        let mut sh = IrrSh9::default();
        sh.coeff[0] = [1.0, 1.0, 1.0];
        scale_sh(&mut sh, 2.0);
        assert!((sh.coeff[0][0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn probe_set_len() {
        let mut set = IrrProbeSet::default();
        assert!(set.is_empty());
        set.add(IrrProbe::new([0.0; 3], 1.0));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn probe_new_default_weight() {
        let p = IrrProbe::new([1.0, 2.0, 3.0], 5.0);
        assert_eq!(p.weight, 1.0);
    }
}
