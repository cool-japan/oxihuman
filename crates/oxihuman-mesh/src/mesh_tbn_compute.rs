// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! TBN frame computation for normal mapping.

#[allow(dead_code)]
pub struct TBNFrame {
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub normal: [f32; 3],
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 { [0.0, 1.0, 0.0] } else { [v[0] / len, v[1] / len, v[2] / len] }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
pub fn tbn_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn tbn_for_triangle(pos: [[f32; 3]; 3], uv: [[f32; 2]; 3]) -> TBNFrame {
    let e1 = [pos[1][0] - pos[0][0], pos[1][1] - pos[0][1], pos[1][2] - pos[0][2]];
    let e2 = [pos[2][0] - pos[0][0], pos[2][1] - pos[0][1], pos[2][2] - pos[0][2]];
    let du1 = uv[1][0] - uv[0][0];
    let dv1 = uv[1][1] - uv[0][1];
    let du2 = uv[2][0] - uv[0][0];
    let dv2 = uv[2][1] - uv[0][1];
    let denom = du1 * dv2 - du2 * dv1;
    let f = if denom.abs() < 1e-10 { 1.0 } else { 1.0 / denom };
    let tangent = [
        f * (dv2 * e1[0] - dv1 * e2[0]),
        f * (dv2 * e1[1] - dv1 * e2[1]),
        f * (dv2 * e1[2] - dv1 * e2[2]),
    ];
    let bitangent = [
        f * (-du2 * e1[0] + du1 * e2[0]),
        f * (-du2 * e1[1] + du1 * e2[1]),
        f * (-du2 * e1[2] + du1 * e2[2]),
    ];
    let normal = cross3(e1, e2);
    TBNFrame {
        tangent: norm3(tangent),
        bitangent: norm3(bitangent),
        normal: norm3(normal),
    }
}

#[allow(dead_code)]
pub fn tbn_vertex_average(frames: &[TBNFrame]) -> TBNFrame {
    if frames.is_empty() {
        return TBNFrame { tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0], normal: [0.0, 0.0, 1.0] };
    }
    let n = frames.len() as f32;
    let mut t = [0.0f32; 3];
    let mut b = [0.0f32; 3];
    let mut nm = [0.0f32; 3];
    for f in frames {
        for i in 0..3 {
            t[i] += f.tangent[i];
            b[i] += f.bitangent[i];
            nm[i] += f.normal[i];
        }
    }
    for i in 0..3 { t[i] /= n; b[i] /= n; nm[i] /= n; }
    TBNFrame { tangent: norm3(t), bitangent: norm3(b), normal: norm3(nm) }
}

#[allow(dead_code)]
pub fn tbn_normalize_frame(frame: &mut TBNFrame) {
    frame.tangent = norm3(frame.tangent);
    frame.bitangent = norm3(frame.bitangent);
    frame.normal = norm3(frame.normal);
}

#[allow(dead_code)]
pub fn tbn_orthogonalize(frame: &mut TBNFrame) {
    let n = norm3(frame.normal);
    let t_dot_n = tbn_dot(frame.tangent, n);
    let t = [
        frame.tangent[0] - t_dot_n * n[0],
        frame.tangent[1] - t_dot_n * n[1],
        frame.tangent[2] - t_dot_n * n[2],
    ];
    frame.tangent = norm3(t);
    frame.normal = n;
    frame.bitangent = cross3(n, frame.tangent);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> TBNFrame {
        let pos = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let uv = [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        tbn_for_triangle(pos, uv)
    }

    #[test]
    fn test_tbn_nonzero_vectors() {
        let f = flat_tri();
        let tl = (f.tangent[0] * f.tangent[0] + f.tangent[1] * f.tangent[1] + f.tangent[2] * f.tangent[2]).sqrt();
        assert!(tl > 0.5);
    }

    #[test]
    fn test_normal_upward() {
        let f = flat_tri();
        assert!(f.normal[2].abs() > 0.5);
    }

    #[test]
    fn test_normalize_frame() {
        let mut f = TBNFrame {
            tangent: [2.0, 0.0, 0.0],
            bitangent: [0.0, 3.0, 0.0],
            normal: [0.0, 0.0, 5.0],
        };
        tbn_normalize_frame(&mut f);
        assert!((tbn_dot(f.tangent, f.tangent) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_orthogonalize() {
        let mut f = flat_tri();
        f.tangent[2] = 0.1;
        tbn_orthogonalize(&mut f);
        let dot = tbn_dot(f.tangent, f.normal);
        assert!(dot.abs() < 1e-5);
    }

    #[test]
    fn test_dot() {
        assert!((tbn_dot([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vertex_average_single() {
        let f = flat_tri();
        let avg = tbn_vertex_average(&[f]);
        let nl = (avg.normal[0] * avg.normal[0] + avg.normal[1] * avg.normal[1] + avg.normal[2] * avg.normal[2]).sqrt();
        assert!((nl - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vertex_average_empty() {
        let avg = tbn_vertex_average(&[]);
        assert_eq!(avg.tangent, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_dot_perpendicular() {
        assert!(tbn_dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).abs() < 1e-10);
    }
}
