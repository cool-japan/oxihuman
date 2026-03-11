// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh scaling and normalization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshScaleConfig {
    pub uniform: bool,
    pub pivot: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshScaleResult {
    pub positions: Vec<[f32; 3]>,
    pub scale_applied: [f32; 3],
}

#[allow(dead_code)]
pub fn default_mesh_scale_config() -> MeshScaleConfig {
    MeshScaleConfig { uniform: true, pivot: [0.0; 3] }
}

#[allow(dead_code)]
pub fn scale_mesh_uniform(
    positions: &[[f32; 3]],
    factor: f32,
    config: &MeshScaleConfig,
) -> MeshScaleResult {
    let piv = config.pivot;
    let out = positions
        .iter()
        .map(|p| [
            piv[0] + (p[0] - piv[0]) * factor,
            piv[1] + (p[1] - piv[1]) * factor,
            piv[2] + (p[2] - piv[2]) * factor,
        ])
        .collect();
    MeshScaleResult { positions: out, scale_applied: [factor; 3] }
}

#[allow(dead_code)]
pub fn scale_mesh_nonuniform(
    positions: &[[f32; 3]],
    scale: [f32; 3],
    config: &MeshScaleConfig,
) -> MeshScaleResult {
    let piv = config.pivot;
    let out = positions
        .iter()
        .map(|p| [
            piv[0] + (p[0] - piv[0]) * scale[0],
            piv[1] + (p[1] - piv[1]) * scale[1],
            piv[2] + (p[2] - piv[2]) * scale[2],
        ])
        .collect();
    MeshScaleResult { positions: out, scale_applied: scale }
}

#[allow(dead_code)]
pub fn normalize_mesh_to_unit(positions: &[[f32; 3]]) -> MeshScaleResult {
    if positions.is_empty() {
        return MeshScaleResult { positions: vec![], scale_applied: [1.0; 3] };
    }
    let (mn, mx) = mesh_bounding_box(positions);
    let diag = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
    let max_dim = diag[0].max(diag[1]).max(diag[2]);
    let factor = if max_dim > 1e-9 { 1.0 / max_dim } else { 1.0 };
    let cfg = MeshScaleConfig { uniform: true, pivot: mn };
    scale_mesh_uniform(positions, factor, &cfg)
}

#[allow(dead_code)]
pub fn mesh_bounding_box(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn mesh_scale_to_json(result: &MeshScaleResult) -> String {
    format!(
        r#"{{"vertex_count":{},"scale":[{:.4},{:.4},{:.4}]}}"#,
        result.positions.len(),
        result.scale_applied[0],
        result.scale_applied[1],
        result.scale_applied[2]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ]
    }

    #[test]
    fn scale_uniform_doubles_coords() {
        let pos = cube_positions();
        let cfg = default_mesh_scale_config();
        let res = scale_mesh_uniform(&pos, 2.0, &cfg);
        assert!((res.positions[1][0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn scale_nonuniform_x_axis() {
        let pos = cube_positions();
        let cfg = default_mesh_scale_config();
        let res = scale_mesh_nonuniform(&pos, [3.0, 1.0, 1.0], &cfg);
        assert!((res.positions[1][0] - 6.0).abs() < 1e-6);
        assert!((res.positions[2][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_to_unit_fits_in_one() {
        let pos = cube_positions();
        let res = normalize_mesh_to_unit(&pos);
        let (_, mx) = mesh_bounding_box(&res.positions);
        let max_dim = mx[0].max(mx[1]).max(mx[2]);
        assert!(max_dim <= 1.0 + 1e-5);
    }

    #[test]
    fn bounding_box_correct() {
        let pos = cube_positions();
        let (mn, mx) = mesh_bounding_box(&pos);
        assert!((mn[0]).abs() < 1e-6);
        assert!((mx[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn scale_applied_recorded() {
        let pos = cube_positions();
        let cfg = default_mesh_scale_config();
        let res = scale_mesh_uniform(&pos, 5.0, &cfg);
        assert!((res.scale_applied[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_empty_mesh() {
        let res = normalize_mesh_to_unit(&[]);
        assert!(res.positions.is_empty());
    }

    #[test]
    fn to_json_has_scale() {
        let pos = cube_positions();
        let cfg = default_mesh_scale_config();
        let res = scale_mesh_uniform(&pos, 1.0, &cfg);
        let json = mesh_scale_to_json(&res);
        assert!(json.contains("scale"));
        assert!(json.contains("vertex_count"));
    }

    #[test]
    fn pivot_used_correctly() {
        let pos = vec![[1.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let cfg = MeshScaleConfig { uniform: true, pivot: [1.0, 0.0, 0.0] };
        let res = scale_mesh_uniform(&pos, 2.0, &cfg);
        // p[0]=1 → pivot+(1-1)*2 = 1
        assert!((res.positions[0][0] - 1.0).abs() < 1e-6);
        // p[1]=2 → pivot+(2-1)*2 = 3
        assert!((res.positions[1][0] - 3.0).abs() < 1e-6);
    }
}
