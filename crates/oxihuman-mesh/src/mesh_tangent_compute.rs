#![allow(dead_code)]

/// Result of tangent computation for a vertex.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TangentResult {
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub sign: f32,
}

/// Compute tangent and bitangent from triangle positions and UVs.
#[allow(dead_code)]
pub fn compute_tangents(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
) -> TangentResult {
    let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let duv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let duv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

    let denom = duv1[0] * duv2[1] - duv2[0] * duv1[1];
    let r = if denom.abs() < 1e-10 { 1.0 } else { 1.0 / denom };

    let tangent = [
        r * (duv2[1] * edge1[0] - duv1[1] * edge2[0]),
        r * (duv2[1] * edge1[1] - duv1[1] * edge2[1]),
        r * (duv2[1] * edge1[2] - duv1[1] * edge2[2]),
    ];
    let bitangent = [
        r * (-duv2[0] * edge1[0] + duv1[0] * edge2[0]),
        r * (-duv2[0] * edge1[1] + duv1[0] * edge2[1]),
        r * (-duv2[0] * edge1[2] + duv1[0] * edge2[2]),
    ];

    let sign = if denom >= 0.0 { 1.0 } else { -1.0 };

    TangentResult {
        tangent,
        bitangent,
        sign,
    }
}

/// Compute a tangent frame (tangent, bitangent, normal) from positions and UVs.
#[allow(dead_code)]
pub fn compute_tangent_frame(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let tr = compute_tangents(p0, p1, p2, uv0, uv1, uv2);
    let normal = cross(
        &[p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]],
        &[p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]],
    );
    (normalize3(tr.tangent), normalize3(tr.bitangent), normalize3(normal))
}

/// Compute tangent direction from UV difference.
#[allow(dead_code)]
pub fn tangent_from_uv(edge: [f32; 3], duv: [f32; 2]) -> [f32; 3] {
    if duv[0].abs() < 1e-10 {
        return normalize3(edge);
    }
    let scale = 1.0 / duv[0];
    normalize3([edge[0] * scale, edge[1] * scale, edge[2] * scale])
}

/// Return the sign of the tangent based on handedness.
#[allow(dead_code)]
pub fn tangent_sign(normal: [f32; 3], tangent: [f32; 3], bitangent: [f32; 3]) -> f32 {
    let c = cross(&normal, &tangent);
    if dot3(&c, &bitangent) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// Normalize a tangent vector.
#[allow(dead_code)]
pub fn normalize_tangent(t: [f32; 3]) -> [f32; 3] {
    normalize3(t)
}

/// Stub for MikkTSpace-like tangent computation; returns identity tangents.
#[allow(dead_code)]
pub fn mikktspace_stub(vertex_count: usize) -> Vec<[f32; 4]> {
    vec![[1.0, 0.0, 0.0, 1.0]; vertex_count]
}

/// Count of tangent vectors for a given vertex count.
#[allow(dead_code)]
pub fn tangent_count(vertex_count: usize) -> usize {
    vertex_count
}

/// Convert a TangentResult to a [f32; 4] array (tangent + sign).
#[allow(dead_code)]
pub fn tangent_to_array(tr: &TangentResult) -> [f32; 4] {
    [tr.tangent[0], tr.tangent[1], tr.tangent[2], tr.sign]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_tangents_basic() {
        let tr = compute_tangents(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0], [1.0, 0.0], [0.0, 1.0],
        );
        assert!((tr.tangent[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_tangent_sign_positive() {
        let s = tangent_sign([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tangent_sign_negative() {
        let s = tangent_sign([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]);
        assert!((s - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_tangent() {
        let t = normalize_tangent([3.0, 0.0, 0.0]);
        assert!((t[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mikktspace_stub() {
        let ts = mikktspace_stub(5);
        assert_eq!(ts.len(), 5);
        assert!((ts[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tangent_count() {
        assert_eq!(tangent_count(10), 10);
    }

    #[test]
    fn test_tangent_to_array() {
        let tr = TangentResult {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            sign: 1.0,
        };
        let arr = tangent_to_array(&tr);
        assert_eq!(arr, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_tangent_from_uv() {
        let t = tangent_from_uv([2.0, 0.0, 0.0], [2.0, 0.0]);
        assert!((t[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_tangent_frame() {
        let (t, _b, n) = compute_tangent_frame(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0], [1.0, 0.0], [0.0, 1.0],
        );
        assert!((t[0] - 1.0).abs() < 1e-4);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_degenerate_uv() {
        let tr = compute_tangents(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
        );
        // Should not panic
        let _ = tangent_to_array(&tr);
    }
}
