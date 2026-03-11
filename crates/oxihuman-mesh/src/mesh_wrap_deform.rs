// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wrap deformation: transfers motion from a proxy mesh to a detail mesh.

/// Config for wrap deformation.
#[allow(dead_code)]
pub struct WrapDeformV2Config {
    pub max_search_dist: f32,
    pub falloff_exponent: f32,
}

#[allow(dead_code)]
impl Default for WrapDeformV2Config {
    fn default() -> Self {
        Self { max_search_dist: 1.0, falloff_exponent: 2.0 }
    }
}

/// Result of wrap deformation.
#[allow(dead_code)]
pub struct WrapDeformV2Result {
    pub positions: Vec<[f32; 3]>,
    pub deformed_count: usize,
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0]*s, v[1]*s, v[2]*s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v,v).sqrt()
}

/// Find the closest proxy vertex to a detail vertex.
#[allow(dead_code)]
pub fn find_closest_proxy(detail_v: [f32; 3], proxy: &[[f32; 3]]) -> Option<usize> {
    proxy.iter().enumerate().min_by(|(_, a), (_, b)| {
        let da = len3(sub3(detail_v, **a));
        let db = len3(sub3(detail_v, **b));
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    }).map(|(i, _)| i)
}

/// Compute wrap weight based on distance and falloff.
#[allow(dead_code)]
pub fn wrap_weight(dist: f32, max_dist: f32, exponent: f32) -> f32 {
    if dist >= max_dist { return 0.0; }
    (1.0 - dist / max_dist).powf(exponent)
}

/// Transfer displacement from proxy_rest -> proxy_deformed to detail mesh.
#[allow(dead_code)]
pub fn wrap_deform(
    detail_positions: &[[f32; 3]],
    proxy_rest: &[[f32; 3]],
    proxy_deformed: &[[f32; 3]],
    config: &WrapDeformV2Config,
) -> WrapDeformV2Result {
    let mut result = Vec::with_capacity(detail_positions.len());
    let mut deformed_count = 0;

    for &dp in detail_positions {
        if let Some(ci) = find_closest_proxy(dp, proxy_rest) {
            let dist = len3(sub3(dp, proxy_rest[ci]));
            let w = wrap_weight(dist, config.max_search_dist, config.falloff_exponent);
            let delta = sub3(proxy_deformed[ci], proxy_rest[ci]);
            let new_p = add3(dp, scale3(delta, w));
            result.push(new_p);
            if w > 0.0 { deformed_count += 1; }
        } else {
            result.push(dp);
        }
    }

    WrapDeformV2Result { positions: result, deformed_count }
}

/// Compute average displacement magnitude.
#[allow(dead_code)]
pub fn avg_displacement_magnitude(
    original: &[[f32; 3]],
    deformed: &[[f32; 3]],
) -> f32 {
    if original.is_empty() { return 0.0; }
    let sum: f32 = original.iter().zip(deformed.iter())
        .map(|(&o, &d)| len3(sub3(d, o)))
        .sum();
    sum / original.len() as f32
}

/// Check if proxy sizes match.
#[allow(dead_code)]
pub fn proxy_sizes_match(rest: &[[f32; 3]], deformed: &[[f32; 3]]) -> bool {
    rest.len() == deformed.len()
}

/// Compute the bounding box diagonal of the proxy.
#[allow(dead_code)]
pub fn proxy_bbox_diagonal(proxy: &[[f32; 3]]) -> f32 {
    if proxy.is_empty() { return 0.0; }
    let mut mn = proxy[0];
    let mut mx = proxy[0];
    for &p in proxy.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    len3(sub3(mx, mn))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy_rest() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]]
    }

    fn proxy_deformed() -> Vec<[f32; 3]> {
        vec![[0.0,0.1,0.0],[1.0,0.1,0.0],[0.5,1.1,0.0]]
    }

    fn detail() -> Vec<[f32; 3]> {
        vec![[0.1,0.0,0.0],[0.9,0.0,0.0],[0.5,0.9,0.0]]
    }

    #[test]
    fn wrap_deform_preserves_count() {
        let cfg = WrapDeformV2Config::default();
        let r = wrap_deform(&detail(), &proxy_rest(), &proxy_deformed(), &cfg);
        assert_eq!(r.positions.len(), detail().len());
    }

    #[test]
    fn wrap_deform_moves_points() {
        let cfg = WrapDeformV2Config { max_search_dist: 2.0, falloff_exponent: 1.0 };
        let orig = detail();
        let r = wrap_deform(&orig, &proxy_rest(), &proxy_deformed(), &cfg);
        let avg = avg_displacement_magnitude(&orig, &r.positions);
        assert!(avg > 0.0);
    }

    #[test]
    fn find_closest_proxy_correct() {
        let proxy = proxy_rest();
        let ci = find_closest_proxy([0.1,0.0,0.0], &proxy);
        assert_eq!(ci, Some(0));
    }

    #[test]
    fn wrap_weight_at_zero() {
        let w = wrap_weight(0.0, 1.0, 2.0);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn wrap_weight_at_max() {
        let w = wrap_weight(1.0, 1.0, 2.0);
        assert!(w.abs() < 1e-6);
    }

    #[test]
    fn proxy_sizes_match_check() {
        let rest = proxy_rest();
        let deformed = proxy_deformed();
        assert!(proxy_sizes_match(&rest, &deformed));
    }

    #[test]
    fn proxy_bbox_diagonal_positive() {
        let proxy = proxy_rest();
        let d = proxy_bbox_diagonal(&proxy);
        assert!(d > 0.0);
    }

    #[test]
    fn avg_displacement_empty() {
        let avg = avg_displacement_magnitude(&[], &[]);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn default_config() {
        let c = WrapDeformV2Config::default();
        assert_eq!(c.falloff_exponent, 2.0);
    }
}
