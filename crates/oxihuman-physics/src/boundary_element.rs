// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct BoundaryElement {
    pub node_a: usize,
    pub node_b: usize,
    pub length: f32,
    pub normal: [f32; 2],
}

#[allow(dead_code)]
pub struct BoundaryMesh {
    pub elements: Vec<BoundaryElement>,
    pub nodes: Vec<[f32; 2]>,
}

#[allow(dead_code)]
pub fn new_boundary_mesh() -> BoundaryMesh {
    BoundaryMesh {
        elements: Vec::new(),
        nodes: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn bem_add_node(m: &mut BoundaryMesh, pos: [f32; 2]) -> usize {
    let idx = m.nodes.len();
    m.nodes.push(pos);
    idx
}

#[allow(dead_code)]
pub fn bem_add_element(m: &mut BoundaryMesh, a: usize, b: usize) {
    let pa = m.nodes[a];
    let pb = m.nodes[b];
    let dx = pb[0] - pa[0];
    let dy = pb[1] - pa[1];
    let length = (dx * dx + dy * dy).sqrt();
    let normal = if length > 1e-10 {
        [dy / length, -dx / length]
    } else {
        [0.0, 1.0]
    };
    m.elements.push(BoundaryElement {
        node_a: a,
        node_b: b,
        length,
        normal,
    });
}

#[allow(dead_code)]
pub fn bem_element_count(m: &BoundaryMesh) -> usize {
    m.elements.len()
}

#[allow(dead_code)]
pub fn bem_total_length(m: &BoundaryMesh) -> f32 {
    m.elements.iter().map(|e| e.length).sum()
}

#[allow(dead_code)]
pub fn bem_centroid(m: &BoundaryMesh) -> [f32; 2] {
    if m.nodes.is_empty() {
        return [0.0; 2];
    }
    let n = m.nodes.len() as f32;
    let sx = m.nodes.iter().map(|p| p[0]).sum::<f32>();
    let sy = m.nodes.iter().map(|p| p[1]).sum::<f32>();
    [sx / n, sy / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nodes() {
        let mut m = new_boundary_mesh();
        bem_add_node(&mut m, [0.0, 0.0]);
        bem_add_node(&mut m, [1.0, 0.0]);
        assert_eq!(m.nodes.len(), 2);
    }

    #[test]
    fn test_add_element() {
        let mut m = new_boundary_mesh();
        bem_add_node(&mut m, [0.0, 0.0]);
        bem_add_node(&mut m, [1.0, 0.0]);
        bem_add_element(&mut m, 0, 1);
        assert_eq!(bem_element_count(&m), 1);
    }

    #[test]
    fn test_element_count() {
        let mut m = new_boundary_mesh();
        bem_add_node(&mut m, [0.0, 0.0]);
        bem_add_node(&mut m, [1.0, 0.0]);
        bem_add_node(&mut m, [1.0, 1.0]);
        bem_add_element(&mut m, 0, 1);
        bem_add_element(&mut m, 1, 2);
        assert_eq!(bem_element_count(&m), 2);
    }

    #[test]
    fn test_total_length() {
        let mut m = new_boundary_mesh();
        bem_add_node(&mut m, [0.0, 0.0]);
        bem_add_node(&mut m, [1.0, 0.0]);
        bem_add_element(&mut m, 0, 1);
        assert!((bem_total_length(&m) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_centroid() {
        let mut m = new_boundary_mesh();
        bem_add_node(&mut m, [0.0, 0.0]);
        bem_add_node(&mut m, [2.0, 0.0]);
        let c = bem_centroid(&m);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1]).abs() < 1e-5);
    }

    #[test]
    fn test_centroid_empty() {
        let m = new_boundary_mesh();
        let c = bem_centroid(&m);
        assert!((c[0]).abs() < 1e-6 && (c[1]).abs() < 1e-6);
    }

    #[test]
    fn test_element_length_correct() {
        let mut m = new_boundary_mesh();
        bem_add_node(&mut m, [0.0, 0.0]);
        bem_add_node(&mut m, [3.0, 4.0]);
        bem_add_element(&mut m, 0, 1);
        assert!((m.elements[0].length - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_element_count_empty() {
        let m = new_boundary_mesh();
        assert_eq!(bem_element_count(&m), 0);
    }
}
