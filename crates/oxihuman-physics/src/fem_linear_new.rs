// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

// Note: named fem_linear_new to avoid conflict with existing fem_linear module

#[allow(dead_code)]
pub struct FemLinearElement {
    pub nodes: [usize; 3],
    pub stiffness: f32,
    pub area: f32,
}

#[allow(dead_code)]
pub struct FemLinearMesh {
    pub elements: Vec<FemLinearElement>,
    pub positions: Vec<[f32; 2]>,
}

#[allow(dead_code)]
pub fn new_fem_linear_mesh() -> FemLinearMesh {
    FemLinearMesh { elements: Vec::new(), positions: Vec::new() }
}

#[allow(dead_code)]
pub fn fem_add_node(m: &mut FemLinearMesh, pos: [f32; 2]) -> usize {
    let idx = m.positions.len();
    m.positions.push(pos);
    idx
}

fn triangle_area(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> f32 {
    let ax = p1[0] - p0[0];
    let ay = p1[1] - p0[1];
    let bx = p2[0] - p0[0];
    let by = p2[1] - p0[1];
    (ax * by - ay * bx).abs() * 0.5
}

#[allow(dead_code)]
pub fn fem_add_element(m: &mut FemLinearMesh, i: usize, j: usize, k: usize, stiffness: f32) -> usize {
    let area = triangle_area(m.positions[i], m.positions[j], m.positions[k]);
    let idx = m.elements.len();
    m.elements.push(FemLinearElement { nodes: [i, j, k], stiffness, area });
    idx
}

#[allow(dead_code)]
pub fn fem_element_count(m: &FemLinearMesh) -> usize {
    m.elements.len()
}

#[allow(dead_code)]
pub fn fem_node_count(m: &FemLinearMesh) -> usize {
    m.positions.len()
}

#[allow(dead_code)]
pub fn fem_total_area(m: &FemLinearMesh) -> f32 {
    m.elements.iter().map(|e| e.area).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nodes() {
        let mut m = new_fem_linear_mesh();
        fem_add_node(&mut m, [0.0, 0.0]);
        fem_add_node(&mut m, [1.0, 0.0]);
        fem_add_node(&mut m, [0.0, 1.0]);
        assert_eq!(fem_node_count(&m), 3);
    }

    #[test]
    fn test_add_element() {
        let mut m = new_fem_linear_mesh();
        fem_add_node(&mut m, [0.0, 0.0]);
        fem_add_node(&mut m, [1.0, 0.0]);
        fem_add_node(&mut m, [0.0, 1.0]);
        fem_add_element(&mut m, 0, 1, 2, 1.0);
        assert_eq!(fem_element_count(&m), 1);
    }

    #[test]
    fn test_element_count() {
        let mut m = new_fem_linear_mesh();
        fem_add_node(&mut m, [0.0, 0.0]);
        fem_add_node(&mut m, [2.0, 0.0]);
        fem_add_node(&mut m, [0.0, 2.0]);
        fem_add_node(&mut m, [2.0, 2.0]);
        fem_add_element(&mut m, 0, 1, 2, 1.0);
        fem_add_element(&mut m, 1, 3, 2, 1.0);
        assert_eq!(fem_element_count(&m), 2);
    }

    #[test]
    fn test_total_area() {
        let mut m = new_fem_linear_mesh();
        fem_add_node(&mut m, [0.0, 0.0]);
        fem_add_node(&mut m, [1.0, 0.0]);
        fem_add_node(&mut m, [0.0, 1.0]);
        fem_add_element(&mut m, 0, 1, 2, 1.0);
        assert!((fem_total_area(&m) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_total_area_unit_square() {
        let mut m = new_fem_linear_mesh();
        fem_add_node(&mut m, [0.0, 0.0]);
        fem_add_node(&mut m, [1.0, 0.0]);
        fem_add_node(&mut m, [0.0, 1.0]);
        fem_add_node(&mut m, [1.0, 1.0]);
        fem_add_element(&mut m, 0, 1, 2, 1.0);
        fem_add_element(&mut m, 1, 3, 2, 1.0);
        assert!((fem_total_area(&m) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_node_count_empty() {
        let m = new_fem_linear_mesh();
        assert_eq!(fem_node_count(&m), 0);
    }

    #[test]
    fn test_element_count_empty() {
        let m = new_fem_linear_mesh();
        assert_eq!(fem_element_count(&m), 0);
    }

    #[test]
    fn test_stiffness_stored() {
        let mut m = new_fem_linear_mesh();
        fem_add_node(&mut m, [0.0, 0.0]);
        fem_add_node(&mut m, [1.0, 0.0]);
        fem_add_node(&mut m, [0.0, 1.0]);
        fem_add_element(&mut m, 0, 1, 2, 42.0);
        assert!((m.elements[0].stiffness - 42.0).abs() < 1e-5);
    }
}
