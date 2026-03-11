// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Space-frame lattice mesh: beam elements rendered as cylinders.

use std::f32::consts::PI;

/// A beam in the lattice (index pair into node positions).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Beam {
    pub a: usize,
    pub b: usize,
    pub radius: f32,
}

/// A lattice frame mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticeMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub beam_count: usize,
}

/// Generate a simple cubic lattice of nodes.
#[allow(dead_code)]
pub fn cubic_lattice_nodes(nx: usize, ny: usize, nz: usize, spacing: f32) -> Vec<[f32; 3]> {
    let mut nodes = Vec::with_capacity(nx * ny * nz);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                nodes.push([
                    ix as f32 * spacing,
                    iy as f32 * spacing,
                    iz as f32 * spacing,
                ]);
            }
        }
    }
    nodes
}

/// Generate beams connecting adjacent nodes in a cubic lattice.
#[allow(dead_code)]
pub fn cubic_lattice_beams(nx: usize, ny: usize, nz: usize, radius: f32) -> Vec<Beam> {
    let mut beams = Vec::new();
    let idx = |ix: usize, iy: usize, iz: usize| iz * ny * nx + iy * nx + ix;
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                if ix + 1 < nx {
                    beams.push(Beam {
                        a: idx(ix, iy, iz),
                        b: idx(ix + 1, iy, iz),
                        radius,
                    });
                }
                if iy + 1 < ny {
                    beams.push(Beam {
                        a: idx(ix, iy, iz),
                        b: idx(ix, iy + 1, iz),
                        radius,
                    });
                }
                if iz + 1 < nz {
                    beams.push(Beam {
                        a: idx(ix, iy, iz),
                        b: idx(ix, iy, iz + 1),
                        radius,
                    });
                }
            }
        }
    }
    beams
}

/// Build the lattice frame mesh from nodes and beams.
#[allow(dead_code)]
pub fn build_lattice_mesh(nodes: &[[f32; 3]], beams: &[Beam], segments: usize) -> LatticeMesh {
    let n = segments.max(3);
    let mut all_pos: Vec<[f32; 3]> = Vec::new();
    let mut all_idx: Vec<u32> = Vec::new();

    for beam in beams {
        if beam.a >= nodes.len() || beam.b >= nodes.len() {
            continue;
        }
        let from = nodes[beam.a];
        let to = nodes[beam.b];
        let fwd = normalize3([to[0] - from[0], to[1] - from[1], to[2] - from[2]]);
        let (right, up) = frame_from_forward(fwd);
        let offset = all_pos.len() as u32;

        for &base in &[from, to] {
            for i in 0..n {
                let angle = 2.0 * PI * i as f32 / n as f32;
                let (s, c) = angle.sin_cos();
                all_pos.push([
                    base[0] + (right[0] * c + up[0] * s) * beam.radius,
                    base[1] + (right[1] * c + up[1] * s) * beam.radius,
                    base[2] + (right[2] * c + up[2] * s) * beam.radius,
                ]);
            }
        }

        for i in 0..n {
            let a = offset + i as u32;
            let b = offset + ((i + 1) % n) as u32;
            let c = offset + (n + i) as u32;
            let d = offset + (n + (i + 1) % n) as u32;
            all_idx.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    LatticeMesh {
        positions: all_pos,
        indices: all_idx,
        beam_count: beams.len(),
    }
}

/// Total triangle count.
#[allow(dead_code)]
pub fn lattice_triangle_count(m: &LatticeMesh) -> usize {
    m.indices.len() / 3
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn frame_from_forward(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let hint = if fwd[1].abs() < 0.9 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let right = normalize3(cross3(fwd, hint));
    (right, normalize3(cross3(right, fwd)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_node_count() {
        let nodes = cubic_lattice_nodes(2, 2, 2, 1.0);
        assert_eq!(nodes.len(), 8);
    }

    #[test]
    fn cubic_beam_count_2x2x2() {
        let beams = cubic_lattice_beams(2, 2, 2, 0.05);
        assert_eq!(beams.len(), 12);
    }

    #[test]
    fn lattice_mesh_nonempty() {
        let nodes = cubic_lattice_nodes(2, 2, 1, 1.0);
        let beams = cubic_lattice_beams(2, 2, 1, 0.05);
        let m = build_lattice_mesh(&nodes, &beams, 4);
        assert!(!m.positions.is_empty());
    }

    #[test]
    fn lattice_indices_multiple_of_three() {
        let nodes = cubic_lattice_nodes(2, 2, 1, 1.0);
        let beams = cubic_lattice_beams(2, 2, 1, 0.05);
        let m = build_lattice_mesh(&nodes, &beams, 4);
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn beam_count_stored() {
        let nodes = cubic_lattice_nodes(2, 2, 1, 1.0);
        let beams = cubic_lattice_beams(2, 2, 1, 0.05);
        let m = build_lattice_mesh(&nodes, &beams, 4);
        assert_eq!(m.beam_count, beams.len());
    }

    #[test]
    fn empty_beams() {
        let nodes = cubic_lattice_nodes(2, 2, 2, 1.0);
        let m = build_lattice_mesh(&nodes, &[], 4);
        assert!(m.positions.is_empty());
    }

    #[test]
    fn out_of_bounds_beam_skipped() {
        let nodes = cubic_lattice_nodes(1, 1, 1, 1.0);
        let beams = vec![Beam {
            a: 0,
            b: 99,
            radius: 0.1,
        }];
        let m = build_lattice_mesh(&nodes, &beams, 4);
        assert!(m.positions.is_empty());
    }

    #[test]
    fn lattice_triangle_count_helper() {
        let nodes = cubic_lattice_nodes(2, 2, 1, 1.0);
        let beams = cubic_lattice_beams(2, 2, 1, 0.05);
        let m = build_lattice_mesh(&nodes, &beams, 4);
        assert_eq!(lattice_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn vertex_count_per_beam() {
        let nodes = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let beams = vec![Beam {
            a: 0,
            b: 1,
            radius: 0.1,
        }];
        let m = build_lattice_mesh(&nodes, &beams, 6);
        assert_eq!(m.positions.len(), 12);
    }

    #[test]
    fn nodes_positioned_correctly() {
        let nodes = cubic_lattice_nodes(2, 1, 1, 2.0);
        assert!((nodes[0][0]).abs() < 1e-6);
        assert!((nodes[1][0] - 2.0).abs() < 1e-6);
    }
}
