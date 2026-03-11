//! Prism / extruded polygon geometry generation.
#![allow(dead_code)]

use std::f32::consts::TAU;

/// A generated prism mesh.
#[allow(dead_code)]
pub struct PrismMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub sides: usize,
    pub height: f32,
}

/// Generate a regular prism with `sides` sides and given height.
#[allow(dead_code)]
pub fn gen_prism(sides: usize, radius: f32, height: f32) -> PrismMesh {
    let sides = sides.max(3);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    // Bottom ring
    for s in 0..sides {
        let angle = (s as f32 / sides as f32) * TAU;
        positions.push([radius * angle.cos(), radius * angle.sin(), 0.0]);
    }
    // Top ring
    for s in 0..sides {
        let angle = (s as f32 / sides as f32) * TAU;
        positions.push([radius * angle.cos(), radius * angle.sin(), height]);
    }
    // Bottom cap center
    let bc = positions.len() as u32;
    positions.push([0.0, 0.0, 0.0]);
    // Top cap center
    let tc = positions.len() as u32;
    positions.push([0.0, 0.0, height]);
    // Side faces
    for s in 0..sides {
        let next_s = (s + 1) % sides;
        let b0 = s as u32; let b1 = next_s as u32;
        let t0 = (sides + s) as u32; let t1 = (sides + next_s) as u32;
        indices.extend_from_slice(&[b0, b1, t0]);
        indices.extend_from_slice(&[b1, t1, t0]);
    }
    // Bottom cap
    for s in 0..sides {
        let next_s = (s + 1) % sides;
        indices.extend_from_slice(&[bc, next_s as u32, s as u32]);
    }
    // Top cap
    for s in 0..sides {
        let next_s = (s + 1) % sides;
        indices.extend_from_slice(&[tc, (sides + s) as u32, (sides + next_s) as u32]);
    }
    PrismMesh { positions, indices, sides, height }
}

/// Generate a prism from an arbitrary polygon (XY plane) + height.
#[allow(dead_code)]
pub fn gen_prism_from_polygon(polygon: &[[f32;2]], height: f32) -> PrismMesh {
    let sides = polygon.len().max(3);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for &[x,y] in polygon {
        positions.push([x, y, 0.0]);
    }
    for &[x,y] in polygon {
        positions.push([x, y, height]);
    }
    let bc = positions.len() as u32;
    let cx = polygon.iter().map(|p| p[0]).sum::<f32>() / sides as f32;
    let cy = polygon.iter().map(|p| p[1]).sum::<f32>() / sides as f32;
    positions.push([cx, cy, 0.0]);
    let tc = positions.len() as u32;
    positions.push([cx, cy, height]);
    for s in 0..sides {
        let next_s = (s+1)%sides;
        let b0 = s as u32; let b1 = next_s as u32;
        let t0 = (sides+s) as u32; let t1 = (sides+next_s) as u32;
        indices.extend_from_slice(&[b0,b1,t0,b1,t1,t0]);
        indices.extend_from_slice(&[bc, next_s as u32, s as u32]);
        indices.extend_from_slice(&[tc, (sides+s) as u32, (sides+next_s) as u32]);
    }
    PrismMesh { positions, indices, sides, height }
}

/// Compute the volume of a regular prism.
#[allow(dead_code)]
pub fn prism_volume(sides: usize, radius: f32, height: f32) -> f32 {
    let sides = sides as f32;
    let base_area = 0.5 * sides * radius * radius * (TAU / sides).sin();
    base_area * height
}

/// Compute the surface area of a regular prism.
#[allow(dead_code)]
pub fn prism_surface_area(sides: usize, radius: f32, height: f32) -> f32 {
    let sides_f = sides as f32;
    let side_len = 2.0 * radius * (std::f32::consts::PI / sides_f).sin();
    let base_area = 0.5 * sides_f * radius * radius * (TAU / sides_f).sin();
    2.0 * base_area + sides_f * side_len * height
}

/// Return the number of faces in a prism mesh.
#[allow(dead_code)]
pub fn prism_face_count(sides: usize) -> usize {
    sides * 2 + sides * 2 // sides*2 side tris + sides bottom + sides top
}

/// Return the number of vertices in a prism mesh.
#[allow(dead_code)]
pub fn prism_vertex_count(sides: usize) -> usize {
    sides * 2 + 2 // two rings + two cap centers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_prism_vertex_count() {
        let p = gen_prism(6, 1.0, 2.0);
        assert_eq!(p.positions.len(), prism_vertex_count(6));
    }

    #[test]
    fn test_gen_prism_face_count() {
        let p = gen_prism(6, 1.0, 2.0);
        assert_eq!(p.indices.len() / 3, prism_face_count(6));
    }

    #[test]
    fn test_prism_volume_positive() {
        let v = prism_volume(6, 1.0, 2.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_prism_surface_area_positive() {
        let a = prism_surface_area(6, 1.0, 2.0);
        assert!(a > 0.0);
    }

    #[test]
    fn test_prism_face_count_formula() {
        assert_eq!(prism_face_count(4), 16);
    }

    #[test]
    fn test_prism_vertex_count_formula() {
        assert_eq!(prism_vertex_count(4), 10);
    }

    #[test]
    fn test_gen_prism_from_polygon() {
        let poly = vec![[0.0f32,0.0],[1.0,0.0],[0.5,1.0]];
        let p = gen_prism_from_polygon(&poly, 2.0);
        assert_eq!(p.sides, 3);
        assert!(!p.positions.is_empty());
    }

    #[test]
    fn test_gen_prism_height() {
        let p = gen_prism(4, 1.0, 3.0);
        assert!((p.height - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_gen_prism_indices_multiple_of_3() {
        let p = gen_prism(5, 1.0, 1.0);
        assert_eq!(p.indices.len() % 3, 0);
    }
}
