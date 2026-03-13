// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ShadingMode {
    Smooth,
    Flat,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmoothShadeConfig {
    pub mode: ShadingMode,
    pub crease_threshold_deg: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmoothShadeResult {
    pub normals: Vec<[f32; 3]>,
    pub shading_mode: ShadingMode,
}

#[allow(dead_code)]
pub fn default_smooth_shade_config() -> SmoothShadeConfig {
    SmoothShadeConfig {
        mode: ShadingMode::Smooth,
        crease_threshold_deg: 30.0,
    }
}

#[allow(dead_code)]
pub fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

#[allow(dead_code)]
pub fn compute_smooth_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    let n_faces = indices.len() / 3;
    for f in 0..n_faces {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let fn_ = face_normal(positions[i0], positions[i1], positions[i2]);
        for &i in &[i0, i1, i2] {
            normals[i][0] += fn_[0];
            normals[i][1] += fn_[1];
            normals[i][2] += fn_[2];
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-12 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }
    normals
}

#[allow(dead_code)]
pub fn compute_flat_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    let n_faces = indices.len() / 3;
    for f in 0..n_faces {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let fn_ = face_normal(positions[i0], positions[i1], positions[i2]);
        normals[i0] = fn_;
        normals[i1] = fn_;
        normals[i2] = fn_;
    }
    normals
}

#[allow(dead_code)]
pub fn shade_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &SmoothShadeConfig,
) -> SmoothShadeResult {
    let normals = match config.mode {
        ShadingMode::Smooth => compute_smooth_normals(positions, indices),
        ShadingMode::Flat => compute_flat_normals(positions, indices),
    };
    SmoothShadeResult {
        normals,
        shading_mode: config.mode.clone(),
    }
}

#[allow(dead_code)]
pub fn smooth_shade_to_json(result: &SmoothShadeResult) -> String {
    let mode = match result.shading_mode {
        ShadingMode::Smooth => "smooth",
        ShadingMode::Flat => "flat",
    };
    format!(
        r#"{{"shading_mode":"{}","normal_count":{}}}"#,
        mode,
        result.normals.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }
    fn tri_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_smooth_shade_config();
        assert_eq!(cfg.mode, ShadingMode::Smooth);
        assert!((cfg.crease_threshold_deg - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_face_normal_z_up() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_face_normal_degenerate() {
        let n = face_normal([0.0; 3], [0.0; 3], [0.0; 3]);
        assert_eq!(n, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_smooth_normals_count() {
        let pos = tri_positions();
        let idx = tri_indices();
        let ns = compute_smooth_normals(&pos, &idx);
        assert_eq!(ns.len(), 3);
    }

    #[test]
    fn test_flat_normals_count() {
        let pos = tri_positions();
        let idx = tri_indices();
        let ns = compute_flat_normals(&pos, &idx);
        assert_eq!(ns.len(), 3);
    }

    #[test]
    fn test_shade_smooth() {
        let pos = tri_positions();
        let idx = tri_indices();
        let cfg = default_smooth_shade_config();
        let res = shade_mesh(&pos, &idx, &cfg);
        assert_eq!(res.shading_mode, ShadingMode::Smooth);
        assert_eq!(res.normals.len(), 3);
    }

    #[test]
    fn test_shade_flat() {
        let pos = tri_positions();
        let idx = tri_indices();
        let mut cfg = default_smooth_shade_config();
        cfg.mode = ShadingMode::Flat;
        let res = shade_mesh(&pos, &idx, &cfg);
        assert_eq!(res.shading_mode, ShadingMode::Flat);
    }

    #[test]
    fn test_to_json() {
        let pos = tri_positions();
        let idx = tri_indices();
        let cfg = default_smooth_shade_config();
        let res = shade_mesh(&pos, &idx, &cfg);
        let j = smooth_shade_to_json(&res);
        assert!(j.contains("smooth"));
        assert!(j.contains("normal_count"));
    }
}
