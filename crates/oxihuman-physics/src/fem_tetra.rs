// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct FemTetra {
    pub nodes: [usize; 4],
    pub volume: f32,
    pub youngs: f32,
}

#[allow(dead_code)]
pub struct FemTetraMesh {
    pub elements: Vec<FemTetra>,
    pub positions: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_fem_tetra_mesh() -> FemTetraMesh {
    FemTetraMesh { elements: Vec::new(), positions: Vec::new() }
}

#[allow(dead_code)]
pub fn ftet_add_node(m: &mut FemTetraMesh, pos: [f32; 3]) -> usize {
    let idx = m.positions.len();
    m.positions.push(pos);
    idx
}

fn det3(b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    b[0] * (c[1] * d[2] - c[2] * d[1])
        - b[1] * (c[0] * d[2] - c[2] * d[0])
        + b[2] * (c[0] * d[1] - c[1] * d[0])
}

fn tet_volume(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    let ba = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ca = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let da = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    det3(ba, ca, da).abs() / 6.0
}

#[allow(dead_code)]
pub fn ftet_add_element(m: &mut FemTetraMesh, i: usize, j: usize, k: usize, l: usize, youngs: f32) -> usize {
    let volume = tet_volume(m.positions[i], m.positions[j], m.positions[k], m.positions[l]);
    let idx = m.elements.len();
    m.elements.push(FemTetra { nodes: [i, j, k, l], volume, youngs });
    idx
}

#[allow(dead_code)]
pub fn ftet_element_count(m: &FemTetraMesh) -> usize {
    m.elements.len()
}

#[allow(dead_code)]
pub fn ftet_node_count(m: &FemTetraMesh) -> usize {
    m.positions.len()
}

#[allow(dead_code)]
pub fn ftet_total_volume(m: &FemTetraMesh) -> f32 {
    m.elements.iter().map(|e| e.volume).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nodes() {
        let mut m = new_fem_tetra_mesh();
        ftet_add_node(&mut m, [0.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 1.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 0.0, 1.0]);
        assert_eq!(ftet_node_count(&m), 4);
    }

    #[test]
    fn test_add_element() {
        let mut m = new_fem_tetra_mesh();
        ftet_add_node(&mut m, [0.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 1.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 0.0, 1.0]);
        ftet_add_element(&mut m, 0, 1, 2, 3, 1e6);
        assert_eq!(ftet_element_count(&m), 1);
    }

    #[test]
    fn test_total_volume_nonzero() {
        let mut m = new_fem_tetra_mesh();
        ftet_add_node(&mut m, [0.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 1.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 0.0, 1.0]);
        ftet_add_element(&mut m, 0, 1, 2, 3, 1e6);
        let vol = ftet_total_volume(&m);
        assert!(vol > 0.0);
    }

    #[test]
    fn test_total_volume_unit_tet() {
        let mut m = new_fem_tetra_mesh();
        ftet_add_node(&mut m, [0.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 1.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 0.0, 1.0]);
        ftet_add_element(&mut m, 0, 1, 2, 3, 1e6);
        let vol = ftet_total_volume(&m);
        assert!((vol - 1.0 / 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_element_count() {
        let mut m = new_fem_tetra_mesh();
        for _ in 0..4 {
            ftet_add_node(&mut m, [0.0; 3]);
        }
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 1.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 0.0, 1.0]);
        assert_eq!(ftet_element_count(&m), 0);
    }

    #[test]
    fn test_node_count_empty() {
        let m = new_fem_tetra_mesh();
        assert_eq!(ftet_node_count(&m), 0);
    }

    #[test]
    fn test_youngs_stored() {
        let mut m = new_fem_tetra_mesh();
        ftet_add_node(&mut m, [0.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 1.0, 0.0]);
        ftet_add_node(&mut m, [0.0, 0.0, 1.0]);
        ftet_add_element(&mut m, 0, 1, 2, 3, 2e8);
        assert!((m.elements[0].youngs - 2e8).abs() < 1.0);
    }

    #[test]
    fn test_degenerate_tet_zero_volume() {
        let mut m = new_fem_tetra_mesh();
        ftet_add_node(&mut m, [0.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [1.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [2.0, 0.0, 0.0]);
        ftet_add_node(&mut m, [3.0, 0.0, 0.0]);
        ftet_add_element(&mut m, 0, 1, 2, 3, 1e6);
        assert!((ftet_total_volume(&m)).abs() < 1e-6);
    }
}
