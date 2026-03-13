// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Enhanced normal transfer between source and target meshes (v2).
#[allow(dead_code)]
pub struct NormalTransferV2Config {
    pub max_search_distance: f32,
    pub blend_factor: f32,
    pub use_smooth_blend: bool,
}

#[allow(dead_code)]
pub struct NormalTransferV2Result {
    pub normals: Vec<[f32; 3]>,
    pub transfer_count: usize,
    pub miss_count: usize,
}

#[allow(dead_code)]
pub fn default_normal_transfer_v2_config() -> NormalTransferV2Config {
    NormalTransferV2Config {
        max_search_distance: 0.1,
        blend_factor: 1.0,
        use_smooth_blend: true,
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn tri_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = cross3(ab, ac);
    let len = vec_len(n).max(1e-10);
    [n[0] / len, n[1] / len, n[2] / len]
}

fn lerp_normal(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let r = [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ];
    let len = vec_len(r).max(1e-10);
    [r[0] / len, r[1] / len, r[2] / len]
}

fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Transfer normals from source mesh to target positions via nearest-face lookup.
#[allow(dead_code)]
pub fn transfer_normals_v2(
    target_positions: &[[f32; 3]],
    target_normals: &[[f32; 3]],
    source_positions: &[[f32; 3]],
    source_indices: &[u32],
    cfg: &NormalTransferV2Config,
) -> NormalTransferV2Result {
    let n = target_positions.len();
    let mut result_normals = target_normals[..n.min(target_normals.len())].to_vec();
    while result_normals.len() < n {
        result_normals.push([0.0, 1.0, 0.0]);
    }

    let tri_count = source_indices.len() / 3;
    if tri_count == 0 {
        return NormalTransferV2Result {
            normals: result_normals,
            transfer_count: 0,
            miss_count: n,
        };
    }

    // Precompute source face normals and centroids
    let face_normals: Vec<[f32; 3]> = (0..tri_count)
        .filter_map(|t| {
            let ia = source_indices[t * 3] as usize;
            let ib = source_indices[t * 3 + 1] as usize;
            let ic = source_indices[t * 3 + 2] as usize;
            if ia < source_positions.len()
                && ib < source_positions.len()
                && ic < source_positions.len()
            {
                Some(tri_normal(
                    source_positions[ia],
                    source_positions[ib],
                    source_positions[ic],
                ))
            } else {
                None
            }
        })
        .collect();

    let centroids: Vec<[f32; 3]> = (0..tri_count.min(face_normals.len()))
        .map(|t| {
            let ia = source_indices[t * 3] as usize;
            let ib = source_indices[t * 3 + 1] as usize;
            let ic = source_indices[t * 3 + 2] as usize;
            let p = source_positions;
            [
                (p[ia][0] + p[ib][0] + p[ic][0]) / 3.0,
                (p[ia][1] + p[ib][1] + p[ic][1]) / 3.0,
                (p[ia][2] + p[ib][2] + p[ic][2]) / 3.0,
            ]
        })
        .collect();

    let mut transfer_count = 0;
    let mut miss_count = 0;

    for i in 0..n {
        let tp = target_positions[i];
        let mut best_dist = f32::MAX;
        let mut best_t = None;

        for (t, &cen) in centroids.iter().enumerate() {
            let d = dist3(tp, cen);
            if d < best_dist {
                best_dist = d;
                best_t = Some(t);
            }
        }

        if let Some(t) = best_t {
            if best_dist <= cfg.max_search_distance {
                let src_n = face_normals[t];
                let tgt_n = result_normals[i];
                let blend = if cfg.use_smooth_blend {
                    smooth_step(cfg.blend_factor)
                } else {
                    cfg.blend_factor.clamp(0.0, 1.0)
                };
                result_normals[i] = lerp_normal(tgt_n, src_n, blend);
                transfer_count += 1;
            } else {
                miss_count += 1;
            }
        } else {
            miss_count += 1;
        }
    }

    NormalTransferV2Result {
        normals: result_normals,
        transfer_count,
        miss_count,
    }
}

#[allow(dead_code)]
pub fn transfer_v2_success_rate(r: &NormalTransferV2Result) -> f32 {
    let total = r.transfer_count + r.miss_count;
    if total == 0 {
        return 1.0;
    }
    r.transfer_count as f32 / total as f32
}

#[allow(dead_code)]
pub fn normals_are_unit(normals: &[[f32; 3]]) -> bool {
    normals.iter().all(|&n| {
        let l = vec_len(n);
        (l - 1.0).abs() < 0.01
    })
}

#[allow(dead_code)]
pub fn transfer_v2_result_to_json(r: &NormalTransferV2Result) -> String {
    format!(
        "{{\"transfer_count\":{},\"miss_count\":{},\"success_rate\":{}}}",
        r.transfer_count,
        r.miss_count,
        transfer_v2_success_rate(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_source() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    #[test]
    fn test_transfer_same_mesh() {
        let (sp, si) = simple_source();
        let cfg = NormalTransferV2Config {
            max_search_distance: 10.0,
            blend_factor: 1.0,
            use_smooth_blend: false,
        };
        let result = transfer_normals_v2(&sp, &[[0.0, 0.0, 1.0]; 3], &sp, &si, &cfg);
        assert_eq!(result.normals.len(), 3);
    }

    #[test]
    fn test_transfer_empty_target() {
        let (sp, si) = simple_source();
        let cfg = default_normal_transfer_v2_config();
        let result = transfer_normals_v2(&[], &[], &sp, &si, &cfg);
        assert_eq!(result.normals.len(), 0);
    }

    #[test]
    fn test_transfer_no_source() {
        let (sp, _) = simple_source();
        let cfg = default_normal_transfer_v2_config();
        let result = transfer_normals_v2(&sp, &[[0.0, 1.0, 0.0]; 3], &[], &[], &cfg);
        assert_eq!(result.miss_count, 3);
    }

    #[test]
    fn test_success_rate_full_transfer() {
        let (sp, si) = simple_source();
        let cfg = NormalTransferV2Config {
            max_search_distance: 100.0,
            blend_factor: 1.0,
            use_smooth_blend: false,
        };
        let result = transfer_normals_v2(&sp, &[[0.0, 0.0, 1.0]; 3], &sp, &si, &cfg);
        assert!(transfer_v2_success_rate(&result) > 0.0);
    }

    #[test]
    fn test_result_normals_unit() {
        let (sp, si) = simple_source();
        let cfg = NormalTransferV2Config {
            max_search_distance: 100.0,
            blend_factor: 1.0,
            use_smooth_blend: false,
        };
        let result = transfer_normals_v2(&sp, &[[0.0, 0.0, 1.0]; 3], &sp, &si, &cfg);
        assert!(normals_are_unit(&result.normals));
    }

    #[test]
    fn test_default_config() {
        let cfg = default_normal_transfer_v2_config();
        assert!(cfg.max_search_distance > 0.0);
        assert!((0.0..=1.0).contains(&cfg.blend_factor));
    }

    #[test]
    fn test_to_json() {
        let (sp, si) = simple_source();
        let cfg = NormalTransferV2Config {
            max_search_distance: 100.0,
            blend_factor: 0.5,
            use_smooth_blend: true,
        };
        let result = transfer_normals_v2(&sp, &[[0.0, 0.0, 1.0]; 3], &sp, &si, &cfg);
        let j = transfer_v2_result_to_json(&result);
        assert!(j.contains("transfer_count"));
    }

    #[test]
    fn test_miss_when_too_far() {
        let target = vec![[100.0, 0.0, 0.0]];
        let (sp, si) = simple_source();
        let cfg = NormalTransferV2Config {
            max_search_distance: 0.001,
            blend_factor: 1.0,
            use_smooth_blend: false,
        };
        let result = transfer_normals_v2(&target, &[[0.0, 1.0, 0.0]], &sp, &si, &cfg);
        assert_eq!(result.miss_count, 1);
    }

    #[test]
    fn test_smooth_blend_range() {
        let t = 0.5f32;
        let s = smooth_step(t);
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn test_normals_are_unit_true() {
        assert!(normals_are_unit(&[[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]]));
    }
}
