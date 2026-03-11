// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Generate debug visualization meshes: normals and tangents as line segments.

/// A line segment for debug rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
}

/// Generate normal visualisation lines for each vertex.
/// Each line starts at the vertex position and extends `scale` units along the normal.
#[allow(dead_code)]
pub fn generate_normal_lines(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    scale: f32,
    color: [f32; 4],
) -> Vec<DebugLine> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(&p, &n)| DebugLine {
            start: p,
            end: [
                p[0] + n[0] * scale,
                p[1] + n[1] * scale,
                p[2] + n[2] * scale,
            ],
            color,
        })
        .collect()
}

/// Generate tangent visualisation lines for each vertex.
#[allow(dead_code)]
pub fn generate_tangent_lines(
    positions: &[[f32; 3]],
    tangents: &[[f32; 3]],
    scale: f32,
    color: [f32; 4],
) -> Vec<DebugLine> {
    generate_normal_lines(positions, tangents, scale, color)
}

/// Generate bitangent lines (cross product of normal × tangent).
#[allow(dead_code)]
pub fn generate_bitangent_lines(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    tangents: &[[f32; 3]],
    scale: f32,
    color: [f32; 4],
) -> Vec<DebugLine> {
    let bitangents: Vec<[f32; 3]> = normals
        .iter()
        .zip(tangents.iter())
        .map(|(&n, &t)| cross3(n, t))
        .collect();
    generate_normal_lines(positions, &bitangents, scale, color)
}

/// Convert debug lines to a flat position buffer (pairs of points).
#[allow(dead_code)]
pub fn lines_to_position_buffer(lines: &[DebugLine]) -> Vec<[f32; 3]> {
    let mut buf = Vec::with_capacity(lines.len() * 2);
    for l in lines {
        buf.push(l.start);
        buf.push(l.end);
    }
    buf
}

/// Convert debug lines to a flat color buffer (one color per endpoint).
#[allow(dead_code)]
pub fn lines_to_color_buffer(lines: &[DebugLine]) -> Vec<[f32; 4]> {
    let mut buf = Vec::with_capacity(lines.len() * 2);
    for l in lines {
        buf.push(l.color);
        buf.push(l.color);
    }
    buf
}

/// Compute line length.
#[allow(dead_code)]
pub fn line_length(l: &DebugLine) -> f32 {
    let d = [
        l.end[0] - l.start[0],
        l.end[1] - l.start[1],
        l.end[2] - l.start[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_data() -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let nrm = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        (pos, nrm)
    }

    #[test]
    fn normal_lines_count() {
        let (pos, nrm) = simple_data();
        let lines = generate_normal_lines(&pos, &nrm, 0.1, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(lines.len(), pos.len());
    }

    #[test]
    fn normal_line_length_approx_scale() {
        let (pos, nrm) = simple_data();
        let scale = 0.5;
        let lines = generate_normal_lines(&pos, &nrm, scale, [1.0, 1.0, 1.0, 1.0]);
        for l in &lines {
            assert!((line_length(l) - scale).abs() < 1e-5);
        }
    }

    #[test]
    fn line_start_matches_position() {
        let (pos, nrm) = simple_data();
        let lines = generate_normal_lines(&pos, &nrm, 1.0, [0.0, 1.0, 0.0, 1.0]);
        for (l, &p) in lines.iter().zip(pos.iter()) {
            assert!((l.start[0] - p[0]).abs() < 1e-6);
            assert!((l.start[1] - p[1]).abs() < 1e-6);
        }
    }

    #[test]
    fn position_buffer_doubles_count() {
        let (pos, nrm) = simple_data();
        let lines = generate_normal_lines(&pos, &nrm, 1.0, [1.0, 0.0, 0.0, 1.0]);
        let buf = lines_to_position_buffer(&lines);
        assert_eq!(buf.len(), lines.len() * 2);
    }

    #[test]
    fn color_buffer_doubles_count() {
        let (pos, nrm) = simple_data();
        let lines = generate_normal_lines(&pos, &nrm, 1.0, [0.0, 0.0, 1.0, 1.0]);
        let buf = lines_to_color_buffer(&lines);
        assert_eq!(buf.len(), lines.len() * 2);
    }

    #[test]
    fn tangent_lines_same_as_normals() {
        let (pos, nrm) = simple_data();
        let n_lines = generate_normal_lines(&pos, &nrm, 0.3, [1.0, 0.0, 0.0, 1.0]);
        let t_lines = generate_tangent_lines(&pos, &nrm, 0.3, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(n_lines.len(), t_lines.len());
    }

    #[test]
    fn bitangent_lines_count() {
        let (pos, nrm) = simple_data();
        let tan = vec![[1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let bl = generate_bitangent_lines(&pos, &nrm, &tan, 0.2, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(bl.len(), pos.len());
    }

    #[test]
    fn empty_positions() {
        let lines = generate_normal_lines(&[], &[], 1.0, [1.0, 0.0, 0.0, 1.0]);
        assert!(lines.is_empty());
    }

    #[test]
    fn line_length_zero_scale() {
        let (pos, nrm) = simple_data();
        let lines = generate_normal_lines(&pos, &nrm, 0.0, [1.0, 0.0, 0.0, 1.0]);
        for l in &lines {
            assert!(line_length(l) < 1e-6);
        }
    }

    #[test]
    fn cross3_orthogonal() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let c = cross3(a, b);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }
}
