// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tangent space (tangent + bitangent) export for normal mapping.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TangentHandedness {
    Right,
    Left,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TangentExportConfig {
    pub handedness: TangentHandedness,
    pub mikktspace: bool,
    pub normalize: bool,
}

/// Tangent and bitangent data for a mesh.
/// `tangents[i]` is `[tx, ty, tz, w]` where `w` is the handedness sign (+1 or -1).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TangentData {
    pub tangents: Vec<[f32; 4]>,
    pub bitangents: Vec<[f32; 3]>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TangentExportResult {
    pub data: TangentData,
    pub degenerate_count: usize,
    pub success: bool,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_tangent_export_config() -> TangentExportConfig {
    TangentExportConfig {
        handedness: TangentHandedness::Right,
        mikktspace: true,
        normalize: true,
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn compute_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    triangles: &[[u32; 3]],
    cfg: &TangentExportConfig,
) -> TangentExportResult {
    let vertex_count = positions.len();
    let mut tangent_accum = vec![[0.0f32; 3]; vertex_count];
    let mut bitangent_accum = vec![[0.0f32; 3]; vertex_count];
    let mut degenerate_count = 0usize;

    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= vertex_count || i1 >= vertex_count || i2 >= vertex_count {
            degenerate_count += 1;
            continue;
        }
        let p = [positions[i0], positions[i1], positions[i2]];
        let uv = [uvs[i0], uvs[i1], uvs[i2]];
        let (t, b) = tangent_for_triangle(&p, &uv);
        if is_degenerate_tangent(t) {
            degenerate_count += 1;
            continue;
        }
        for vi in [i0, i1, i2] {
            for k in 0..3 {
                tangent_accum[vi][k] += t[k];
                bitangent_accum[vi][k] += b[k];
            }
        }
    }

    let handedness_sign = match cfg.handedness {
        TangentHandedness::Right => 1.0f32,
        TangentHandedness::Left => -1.0f32,
    };

    let mut tangents = Vec::with_capacity(vertex_count);
    let mut bitangents = Vec::with_capacity(vertex_count);

    for vi in 0..vertex_count {
        let t_raw = tangent_accum[vi];
        let b_raw = bitangent_accum[vi];
        let n = normals.get(vi).copied().unwrap_or([0.0, 1.0, 0.0]);

        let t = if cfg.normalize {
            normalize_v3_tan(t_raw)
        } else {
            t_raw
        };
        let b = if cfg.normalize {
            normalize_v3_tan(b_raw)
        } else {
            b_raw
        };

        let packed = pack_tangent_w(t, b, n);
        let w = packed[3] * handedness_sign;
        tangents.push([t[0], t[1], t[2], w]);
        bitangents.push(b);
    }

    let data = TangentData {
        tangents,
        bitangents,
        vertex_count,
    };
    TangentExportResult {
        success: true,
        degenerate_count,
        data,
    }
}

/// Compute the tangent and bitangent for a single triangle.
/// `p` is 3 positions, `uv` is 3 UV coordinates.
#[allow(dead_code)]
pub fn tangent_for_triangle(p: &[[f32; 3]; 3], uv: &[[f32; 2]; 3]) -> ([f32; 3], [f32; 3]) {
    let e1 = sub3(p[1], p[0]);
    let e2 = sub3(p[2], p[0]);
    let du1 = uv[1][0] - uv[0][0];
    let dv1 = uv[1][1] - uv[0][1];
    let du2 = uv[2][0] - uv[0][0];
    let dv2 = uv[2][1] - uv[0][1];

    let denom = du1 * dv2 - du2 * dv1;
    if denom.abs() < 1e-12 {
        return ([0.0; 3], [0.0; 3]);
    }
    let r = 1.0 / denom;
    let tangent = [
        (dv2 * e1[0] - dv1 * e2[0]) * r,
        (dv2 * e1[1] - dv1 * e2[1]) * r,
        (dv2 * e1[2] - dv1 * e2[2]) * r,
    ];
    let bitangent = [
        (du1 * e2[0] - du2 * e1[0]) * r,
        (du1 * e2[1] - du2 * e1[1]) * r,
        (du1 * e2[2] - du2 * e1[2]) * r,
    ];
    (tangent, bitangent)
}

#[allow(dead_code)]
pub fn normalize_v3_tan(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
pub fn tangent_export_to_json(r: &TangentExportResult) -> String {
    format!(
        r#"{{"vertex_count":{},"degenerate_count":{},"success":{}}}"#,
        r.data.vertex_count, r.degenerate_count, r.success
    )
}

/// Returns `true` if the tangent vector has near-zero length (degenerate).
#[allow(dead_code)]
pub fn is_degenerate_tangent(t: [f32; 3]) -> bool {
    let len2 = t[0] * t[0] + t[1] * t[1] + t[2] * t[2];
    len2 < 1e-12
}

#[allow(dead_code)]
pub fn handedness_name(cfg: &TangentExportConfig) -> &'static str {
    match cfg.handedness {
        TangentHandedness::Right => "Right",
        TangentHandedness::Left => "Left",
    }
}

#[allow(dead_code)]
pub fn tangent_data_vertex_count(data: &TangentData) -> usize {
    data.vertex_count
}

/// Pack tangent+bitangent into a 4-component tangent where `w` encodes handedness.
/// `w = sign(dot(cross(N, T), B))`.
#[allow(dead_code)]
pub fn pack_tangent_w(tangent: [f32; 3], bitangent: [f32; 3], normal: [f32; 3]) -> [f32; 4] {
    let cross = cross3(normal, tangent);
    let dot = cross[0] * bitangent[0] + cross[1] * bitangent[1] + cross[2] * bitangent[2];
    let w = if dot < 0.0 { -1.0f32 } else { 1.0f32 };
    [tangent[0], tangent[1], tangent[2], w]
}

#[allow(dead_code)]
pub fn validate_tangent_data(data: &TangentData) -> bool {
    data.tangents.len() == data.vertex_count && data.bitangents.len() == data.vertex_count
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_right_handed_mikkt() {
        let cfg = default_tangent_export_config();
        assert_eq!(cfg.handedness, TangentHandedness::Right);
        assert!(cfg.mikktspace);
        assert!(cfg.normalize);
    }

    #[test]
    fn handedness_name_right() {
        let cfg = default_tangent_export_config();
        assert_eq!(handedness_name(&cfg), "Right");
    }

    #[test]
    fn handedness_name_left() {
        let cfg = TangentExportConfig {
            handedness: TangentHandedness::Left,
            mikktspace: false,
            normalize: true,
        };
        assert_eq!(handedness_name(&cfg), "Left");
    }

    #[test]
    fn is_degenerate_tangent_zero_vector() {
        assert!(is_degenerate_tangent([0.0, 0.0, 0.0]));
    }

    #[test]
    fn is_degenerate_tangent_unit_vector() {
        assert!(!is_degenerate_tangent([1.0, 0.0, 0.0]));
    }

    #[test]
    fn normalize_v3_tan_unit_x() {
        let v = normalize_v3_tan([3.0, 0.0, 0.0]);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!(v[1].abs() < 1e-6);
        assert!(v[2].abs() < 1e-6);
    }

    #[test]
    fn normalize_v3_tan_zero_returns_zero() {
        let v = normalize_v3_tan([0.0, 0.0, 0.0]);
        assert_eq!(v, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn tangent_for_triangle_xy_plane() {
        let p = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uv = [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let (t, _b) = tangent_for_triangle(&p, &uv);
        // Tangent should point along X
        assert!((t[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn compute_tangents_single_triangle() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![[0.0f32, 0.0, 1.0]; 3];
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let triangles = vec![[0u32, 1, 2]];
        let cfg = default_tangent_export_config();
        let result = compute_tangents(&positions, &normals, &uvs, &triangles, &cfg);
        assert!(result.success);
        assert_eq!(result.data.vertex_count, 3);
        assert_eq!(result.data.tangents.len(), 3);
        assert_eq!(result.data.bitangents.len(), 3);
    }

    #[test]
    fn validate_tangent_data_ok() {
        let data = TangentData {
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            bitangents: vec![[0.0, 1.0, 0.0]; 4],
            vertex_count: 4,
        };
        assert!(validate_tangent_data(&data));
    }

    #[test]
    fn validate_tangent_data_mismatch() {
        let data = TangentData {
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            bitangents: vec![[0.0, 1.0, 0.0]; 4],
            vertex_count: 4,
        };
        assert!(!validate_tangent_data(&data));
    }

    #[test]
    fn pack_tangent_w_right_handed() {
        // T = X, B = Y, N = Z => cross(N,T) = cross(Z,X) = Y, dot(Y,Y) > 0 → w = +1
        let t = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let n = [0.0f32, 0.0, 1.0];
        let packed = pack_tangent_w(t, b, n);
        assert!((packed[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn tangent_export_to_json_contains_vertex_count() {
        let result = TangentExportResult {
            data: TangentData {
                tangents: vec![],
                bitangents: vec![],
                vertex_count: 99,
            },
            degenerate_count: 0,
            success: true,
        };
        let json = tangent_export_to_json(&result);
        assert!(json.contains("99"));
    }

    #[test]
    fn tangent_data_vertex_count_accessor() {
        let data = TangentData {
            tangents: vec![],
            bitangents: vec![],
            vertex_count: 7,
        };
        assert_eq!(tangent_data_vertex_count(&data), 7);
    }
}
