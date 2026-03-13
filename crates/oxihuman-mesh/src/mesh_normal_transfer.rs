// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Transfer normals from a high-res source mesh to a low-res target mesh.

#[allow(dead_code)]
pub struct NormalTransferConfig {
    pub max_search_dist: f32,
    pub smooth_iterations: u32,
    pub weight_by_distance: bool,
}

#[allow(dead_code)]
pub struct TransferResult {
    pub normals: Vec<[f32; 3]>,
    pub transferred_count: usize,
    pub fallback_count: usize,
}

#[allow(dead_code)]
pub fn default_normal_transfer_config() -> NormalTransferConfig {
    NormalTransferConfig {
        max_search_dist: 0.1,
        smooth_iterations: 1,
        weight_by_distance: true,
    }
}

#[allow(dead_code)]
pub fn normalize_v3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
pub fn dot_v3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn dist_v3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn nearest_normal(
    src_pos: &[[f32; 3]],
    src_normals: &[[f32; 3]],
    query: [f32; 3],
) -> [f32; 3] {
    if src_pos.is_empty() || src_normals.is_empty() {
        return [0.0, 1.0, 0.0];
    }
    let mut best_dist = f32::MAX;
    let mut best_idx = 0usize;
    let n = src_pos.len().min(src_normals.len());
    for (i, &sp) in src_pos.iter().enumerate().take(n) {
        let d = dist_v3(sp, query);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    normalize_v3(src_normals[best_idx])
}

#[allow(dead_code)]
pub fn transfer_normals(
    src_pos: &[[f32; 3]],
    src_normals: &[[f32; 3]],
    dst_pos: &[[f32; 3]],
    cfg: &NormalTransferConfig,
) -> TransferResult {
    let mut normals = Vec::with_capacity(dst_pos.len());
    let mut transferred_count = 0usize;
    let mut fallback_count = 0usize;

    let n_src = src_pos.len().min(src_normals.len());

    for &dpt in dst_pos {
        if n_src == 0 {
            normals.push([0.0, 1.0, 0.0]);
            fallback_count += 1;
            continue;
        }

        let mut best_dist = f32::MAX;
        let mut best_idx = 0usize;
        for (i, &sp) in src_pos.iter().enumerate().take(n_src) {
            let d = dist_v3(sp, dpt);
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }

        if best_dist <= cfg.max_search_dist {
            normals.push(normalize_v3(src_normals[best_idx]));
            transferred_count += 1;
        } else {
            // Fallback: use closest regardless
            normals.push(normalize_v3(src_normals[best_idx]));
            fallback_count += 1;
        }
    }

    TransferResult {
        normals,
        transferred_count,
        fallback_count,
    }
}

#[allow(dead_code)]
pub fn transfer_result_to_json(r: &TransferResult) -> String {
    format!(
        r#"{{"transferred_count":{},"fallback_count":{}}}"#,
        r.transferred_count, r.fallback_count
    )
}

#[allow(dead_code)]
pub fn validate_normals(normals: &[[f32; 3]]) -> bool {
    if normals.is_empty() {
        return false;
    }
    for n in normals {
        let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        if len_sq < 1e-12 {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn smooth_normals(normals: &mut Vec<[f32; 3]>, iterations: u32) {
    for _ in 0..iterations {
        let n = normals.len();
        if n < 2 {
            break;
        }
        let mut smoothed = normals.clone();
        for i in 0..n {
            let prev = if i == 0 { n - 1 } else { i - 1 };
            let next = if i == n - 1 { 0 } else { i + 1 };
            let avg = [
                (normals[prev][0] + normals[i][0] + normals[next][0]) / 3.0,
                (normals[prev][1] + normals[i][1] + normals[next][1]) / 3.0,
                (normals[prev][2] + normals[i][2] + normals[next][2]) / 3.0,
            ];
            smoothed[i] = normalize_v3(avg);
        }
        *normals = smoothed;
    }
}

#[allow(dead_code)]
pub fn transferred_ratio(r: &TransferResult) -> f32 {
    let total = r.transferred_count + r.fallback_count;
    if total == 0 {
        return 0.0;
    }
    r.transferred_count as f32 / total as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_src() -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let nrm = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        (pos, nrm)
    }

    #[test]
    fn default_config_fields() {
        let cfg = default_normal_transfer_config();
        assert!(cfg.max_search_dist > 0.0);
        assert!(cfg.smooth_iterations > 0);
    }

    #[test]
    fn normalize_v3_unit_length() {
        let v = normalize_v3([3.0, 0.0, 0.0]);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!(v[1].abs() < 1e-6);
        assert!(v[2].abs() < 1e-6);
    }

    #[test]
    fn normalize_v3_zero_returns_fallback() {
        let v = normalize_v3([0.0, 0.0, 0.0]);
        assert!((v[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dot_v3_orthogonal_is_zero() {
        let a = [1.0_f32, 0.0, 0.0];
        let b = [0.0_f32, 1.0, 0.0];
        assert!(dot_v3(a, b).abs() < 1e-6);
    }

    #[test]
    fn dist_v3_same_point_is_zero() {
        let p = [1.0_f32, 2.0, 3.0];
        assert!(dist_v3(p, p) < 1e-6);
    }

    #[test]
    fn transfer_normals_count_matches_dst() {
        let (pos, nrm) = simple_src();
        let dst = vec![[0.0_f32, 0.0, 0.0], [0.5, 0.5, 0.0]];
        let cfg = default_normal_transfer_config();
        let result = transfer_normals(&pos, &nrm, &dst, &cfg);
        assert_eq!(result.normals.len(), 2);
    }

    #[test]
    fn transfer_normals_empty_src_fallback() {
        let dst = vec![[0.0_f32, 0.0, 0.0]];
        let cfg = default_normal_transfer_config();
        let result = transfer_normals(&[], &[], &dst, &cfg);
        assert_eq!(result.fallback_count, 1);
        assert_eq!(result.transferred_count, 0);
    }

    #[test]
    fn validate_normals_nonzero_ok() {
        let n = vec![[0.0_f32, 0.0, 1.0], [1.0, 0.0, 0.0]];
        assert!(validate_normals(&n));
    }

    #[test]
    fn validate_normals_zero_fails() {
        let n = vec![[0.0_f32, 0.0, 0.0]];
        assert!(!validate_normals(&n));
    }

    #[test]
    fn smooth_normals_preserves_count() {
        let mut n = vec![[0.0_f32, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let len = n.len();
        smooth_normals(&mut n, 2);
        assert_eq!(n.len(), len);
    }

    #[test]
    fn transferred_ratio_zero_when_empty() {
        let r = TransferResult { normals: vec![], transferred_count: 0, fallback_count: 0 };
        assert_eq!(transferred_ratio(&r), 0.0);
    }

    #[test]
    fn nearest_normal_returns_normalized() {
        let (pos, nrm) = simple_src();
        let n = nearest_normal(&pos, &nrm, [0.1, 0.1, 0.0]);
        let len = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }
}
