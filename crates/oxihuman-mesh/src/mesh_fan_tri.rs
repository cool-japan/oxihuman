#![allow(dead_code)]

/// Result of a fan triangulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FanResult2 {
    pub triangles: Vec<[usize; 3]>,
    pub center_index: usize,
}

/// Triangulate a polygon using fan from vertex 0.
#[allow(dead_code)]
pub fn fan_triangulate_polygon2(polygon: &[usize]) -> Vec<[usize; 3]> {
    if polygon.len() < 3 {
        return Vec::new();
    }
    let mut tris = Vec::with_capacity(polygon.len() - 2);
    for i in 1..polygon.len() - 1 {
        tris.push([polygon[0], polygon[i], polygon[i + 1]]);
    }
    tris
}

/// Triangulate a quad using fan from vertex 0.
#[allow(dead_code)]
pub fn fan_triangulate_quad2(quad: [usize; 4]) -> [[usize; 3]; 2] {
    [
        [quad[0], quad[1], quad[2]],
        [quad[0], quad[2], quad[3]],
    ]
}

/// Create a fan from a center vertex to a ring of vertices.
#[allow(dead_code)]
pub fn fan_from_center2(center: usize, ring: &[usize]) -> Vec<[usize; 3]> {
    if ring.len() < 2 {
        return Vec::new();
    }
    let mut tris = Vec::with_capacity(ring.len());
    for i in 0..ring.len() - 1 {
        tris.push([center, ring[i], ring[i + 1]]);
    }
    // Close the fan
    tris.push([center, ring[ring.len() - 1], ring[0]]);
    tris
}

/// Count how many triangles a fan triangulation of an n-gon produces.
#[allow(dead_code)]
pub fn fan_triangle_count2(polygon_vertex_count: usize) -> usize {
    if polygon_vertex_count < 3 {
        return 0;
    }
    polygon_vertex_count - 2
}

/// Count how many indices a fan triangulation of an n-gon produces.
#[allow(dead_code)]
pub fn fan_index_count2(polygon_vertex_count: usize) -> usize {
    fan_triangle_count2(polygon_vertex_count) * 3
}

/// Validate that a fan triangulation covers all vertices.
#[allow(dead_code)]
pub fn fan_validate2(polygon: &[usize], triangles: &[[usize; 3]]) -> bool {
    if polygon.len() < 3 {
        return triangles.is_empty();
    }
    triangles.len() == polygon.len() - 2
}

/// Compute the total area of a fan triangulation.
#[allow(dead_code)]
pub fn fan_area2(vertices: &[[f32; 3]], triangles: &[[usize; 3]]) -> f32 {
    let mut area = 0.0f32;
    for tri in triangles {
        area += triangle_area(vertices, tri);
    }
    area
}

/// Compute the normal of a fan (uses first triangle's normal).
#[allow(dead_code)]
pub fn fan_normal2(vertices: &[[f32; 3]], triangles: &[[usize; 3]]) -> [f32; 3] {
    if triangles.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let t = &triangles[0];
    let a = vertices[t[0]];
    let b = vertices[t[1]];
    let c = vertices[t[2]];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = ab[1] * ac[2] - ab[2] * ac[1];
    let ny = ab[2] * ac[0] - ab[0] * ac[2];
    let nz = ab[0] * ac[1] - ab[1] * ac[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [nx / len, ny / len, nz / len]
}

fn triangle_area(vertices: &[[f32; 3]], tri: &[usize; 3]) -> f32 {
    let a = vertices[tri[0]];
    let b = vertices[tri[1]];
    let c = vertices[tri[2]];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_triangulate_polygon_triangle() {
        let tris = fan_triangulate_polygon2(&[0, 1, 2]);
        assert_eq!(tris.len(), 1);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn test_fan_triangulate_polygon_quad() {
        let tris = fan_triangulate_polygon2(&[0, 1, 2, 3]);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn test_fan_triangulate_polygon_too_small() {
        let tris = fan_triangulate_polygon2(&[0, 1]);
        assert!(tris.is_empty());
    }

    #[test]
    fn test_fan_triangulate_quad() {
        let tris = fan_triangulate_quad2([0, 1, 2, 3]);
        assert_eq!(tris[0], [0, 1, 2]);
        assert_eq!(tris[1], [0, 2, 3]);
    }

    #[test]
    fn test_fan_from_center() {
        let tris = fan_from_center2(0, &[1, 2, 3, 4]);
        assert_eq!(tris.len(), 4);
    }

    #[test]
    fn test_fan_triangle_count() {
        assert_eq!(fan_triangle_count2(3), 1);
        assert_eq!(fan_triangle_count2(5), 3);
        assert_eq!(fan_triangle_count2(2), 0);
    }

    #[test]
    fn test_fan_index_count() {
        assert_eq!(fan_index_count2(4), 6);
    }

    #[test]
    fn test_fan_validate() {
        let poly = [0, 1, 2, 3];
        let tris = fan_triangulate_polygon2(&poly);
        assert!(fan_validate2(&poly, &tris));
    }

    #[test]
    fn test_fan_area() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let tris = fan_triangulate_polygon2(&[0, 1, 2, 3]);
        let area = fan_area2(&verts, &tris);
        assert!((area - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_fan_normal() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let tris = vec![[0, 1, 2]];
        let n = fan_normal2(&verts, &tris);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }
}
