#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bend mesh along a curve path.

#[allow(dead_code)]
pub struct BendResult {
    pub verts: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn bend_along_curve(verts: &[[f32; 3]], curve: &[[f32; 3]], axis: u8) -> BendResult {
    if curve.is_empty() || verts.is_empty() {
        return BendResult { verts: verts.to_vec() };
    }
    let total_len = curve_length(curve);
    let mut out = Vec::with_capacity(verts.len());
    for &v in verts {
        let coord = if axis == 0 { v[0] } else if axis == 1 { v[1] } else { v[2] };
        let t = if total_len > 1e-7 { (coord / total_len).clamp(0.0, 1.0) } else { 0.0 };
        let base = curve_at_t(curve, t);
        let tang = curve_tangent_at_t(curve, t);
        let perp = perp_vec(tang);
        let secondary = if axis == 1 { v[0] } else { v[1] };
        out.push([
            base[0] + perp[0] * secondary,
            base[1] + perp[1] * secondary,
            base[2] + tang[2] * secondary,
        ]);
    }
    BendResult { verts: out }
}

#[allow(dead_code)]
pub fn curve_length(curve: &[[f32; 3]]) -> f32 {
    if curve.len() < 2 {
        return 0.0;
    }
    curve.windows(2).map(|w| dist3(w[0], w[1])).sum()
}

#[allow(dead_code)]
pub fn curve_at_t(curve: &[[f32; 3]], t: f32) -> [f32; 3] {
    if curve.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    if curve.len() == 1 {
        return curve[0];
    }
    let t = t.clamp(0.0, 1.0);
    let segs = (curve.len() - 1) as f32;
    let ft = t * segs;
    let i = (ft as usize).min(curve.len() - 2);
    let u = ft - i as f32;
    let a = curve[i];
    let b = curve[i + 1];
    [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u, a[2] + (b[2] - a[2]) * u]
}

#[allow(dead_code)]
pub fn curve_tangent_at_t(curve: &[[f32; 3]], t: f32) -> [f32; 3] {
    if curve.len() < 2 {
        return [0.0, 0.0, 1.0];
    }
    let t = t.clamp(0.0, 1.0);
    let segs = (curve.len() - 1) as f32;
    let ft = t * segs;
    let i = (ft as usize).min(curve.len() - 2);
    let a = curve[i];
    let b = curve[i + 1];
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    normalize3(d)
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-7 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn perp_vec(v: [f32; 3]) -> [f32; 3] {
    if v[0].abs() < 0.9 {
        normalize3([0.0, -v[2], v[1]])
    } else {
        normalize3([v[2], 0.0, -v[0]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_curve() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
    }

    #[test]
    fn curve_length_straight() {
        let c = straight_curve();
        let l = curve_length(&c);
        assert!((l - 2.0).abs() < 1e-5);
    }

    #[test]
    fn curve_length_empty() {
        assert!((curve_length(&[])).abs() < 1e-5);
    }

    #[test]
    fn curve_length_single_point() {
        assert!((curve_length(&[[0.0, 0.0, 0.0]])).abs() < 1e-5);
    }

    #[test]
    fn curve_at_t_start() {
        let c = straight_curve();
        let p = curve_at_t(&c, 0.0);
        assert!((p[0]).abs() < 1e-5);
    }

    #[test]
    fn curve_at_t_end() {
        let c = straight_curve();
        let p = curve_at_t(&c, 1.0);
        assert!((p[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn curve_at_t_midpoint() {
        let c = straight_curve();
        let p = curve_at_t(&c, 0.5);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn curve_tangent_at_t_horizontal() {
        let c = straight_curve();
        let t = curve_tangent_at_t(&c, 0.0);
        assert!((t[0] - 1.0).abs() < 1e-5);
        assert!(t[1].abs() < 1e-5);
    }

    #[test]
    fn bend_along_curve_empty_curve() {
        let verts = vec![[0.0, 0.0, 0.0]];
        let result = bend_along_curve(&verts, &[], 0);
        assert_eq!(result.verts.len(), 1);
    }

    #[test]
    fn bend_along_curve_preserves_count() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let c = straight_curve();
        let result = bend_along_curve(&verts, &c, 0);
        assert_eq!(result.verts.len(), 2);
    }
}
