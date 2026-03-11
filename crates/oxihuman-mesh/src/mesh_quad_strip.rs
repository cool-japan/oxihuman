#![allow(dead_code)]

/// A strip of quads defined by two parallel paths of vertices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuadStrip {
    pub top: Vec<[f32; 3]>,
    pub bottom: Vec<[f32; 3]>,
}

/// Create a new quad strip from two parallel vertex paths.
#[allow(dead_code)]
pub fn new_quad_strip(top: Vec<[f32; 3]>, bottom: Vec<[f32; 3]>) -> QuadStrip {
    QuadStrip { top, bottom }
}

/// Create a quad strip by offsetting a path by a width vector.
#[allow(dead_code)]
pub fn quad_strip_from_path(path: &[[f32; 3]], offset: [f32; 3]) -> QuadStrip {
    let top = path.to_vec();
    let bottom: Vec<[f32; 3]> = path
        .iter()
        .map(|p| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]])
        .collect();
    QuadStrip { top, bottom }
}

/// Count the number of quads in the strip.
#[allow(dead_code)]
pub fn quad_count(strip: &QuadStrip) -> usize {
    let min_len = strip.top.len().min(strip.bottom.len());
    if min_len < 2 {
        return 0;
    }
    min_len - 1
}

/// Count the number of unique vertices in the strip.
#[allow(dead_code)]
pub fn quad_strip_vertex_count(strip: &QuadStrip) -> usize {
    strip.top.len() + strip.bottom.len()
}

/// Convert the quad strip to triangles (two triangles per quad).
#[allow(dead_code)]
pub fn quad_strip_to_triangles(strip: &QuadStrip) -> Vec<[[f32; 3]; 3]> {
    let mut tris = Vec::new();
    let count = quad_count(strip);
    for i in 0..count {
        let t0 = strip.top[i];
        let t1 = strip.top[i + 1];
        let b0 = strip.bottom[i];
        let b1 = strip.bottom[i + 1];
        tris.push([t0, t1, b0]);
        tris.push([t1, b1, b0]);
    }
    tris
}

/// Compute the total surface area of the quad strip.
#[allow(dead_code)]
pub fn quad_strip_area(strip: &QuadStrip) -> f32 {
    let tris = quad_strip_to_triangles(strip);
    let mut area = 0.0f32;
    for tri in &tris {
        area += tri_area(tri);
    }
    area
}

/// Generate UV coordinates for the quad strip (0..1 along length and width).
#[allow(dead_code)]
pub fn quad_strip_uvs(strip: &QuadStrip) -> Vec<[f32; 2]> {
    let n = strip.top.len();
    if n == 0 {
        return Vec::new();
    }
    let mut uvs = Vec::with_capacity(n * 2);
    for i in 0..n {
        let u = if n > 1 {
            i as f32 / (n - 1) as f32
        } else {
            0.0
        };
        uvs.push([u, 0.0]); // top
    }
    for i in 0..strip.bottom.len() {
        let u = if strip.bottom.len() > 1 {
            i as f32 / (strip.bottom.len() - 1) as f32
        } else {
            0.0
        };
        uvs.push([u, 1.0]); // bottom
    }
    uvs
}

/// Compute normals for each quad (uses first triangle's normal).
#[allow(dead_code)]
pub fn quad_strip_normals(strip: &QuadStrip) -> Vec<[f32; 3]> {
    let count = quad_count(strip);
    let mut normals = Vec::with_capacity(count);
    for i in 0..count {
        let a = strip.top[i];
        let b = strip.top[i + 1];
        let c = strip.bottom[i];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = cross(ab, ac);
        normals.push(normalize(n));
    }
    normals
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn tri_area(tri: &[[f32; 3]; 3]) -> f32 {
    let ab = [
        tri[1][0] - tri[0][0],
        tri[1][1] - tri[0][1],
        tri[1][2] - tri[0][2],
    ];
    let ac = [
        tri[2][0] - tri[0][0],
        tri[2][1] - tri[0][1],
        tri[2][2] - tri[0][2],
    ];
    let c = cross(ab, ac);
    0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_strip() -> QuadStrip {
        new_quad_strip(
            vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        )
    }

    #[test]
    fn test_new_quad_strip() {
        let s = sample_strip();
        assert_eq!(s.top.len(), 3);
        assert_eq!(s.bottom.len(), 3);
    }

    #[test]
    fn test_quad_count() {
        let s = sample_strip();
        assert_eq!(quad_count(&s), 2);
    }

    #[test]
    fn test_quad_strip_vertex_count() {
        let s = sample_strip();
        assert_eq!(quad_strip_vertex_count(&s), 6);
    }

    #[test]
    fn test_to_triangles() {
        let s = sample_strip();
        let tris = quad_strip_to_triangles(&s);
        assert_eq!(tris.len(), 4);
    }

    #[test]
    fn test_quad_strip_area() {
        let s = sample_strip();
        let area = quad_strip_area(&s);
        assert!((area - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_quad_strip_uvs() {
        let s = sample_strip();
        let uvs = quad_strip_uvs(&s);
        assert_eq!(uvs.len(), 6);
        assert!((uvs[0][1] - 0.0).abs() < 1e-6);
        assert!((uvs[3][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quad_strip_normals() {
        let s = sample_strip();
        let normals = quad_strip_normals(&s);
        assert_eq!(normals.len(), 2);
        assert!((normals[0][2].abs() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_quad_strip_from_path() {
        let path = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let s = quad_strip_from_path(&path, [0.0, 1.0, 0.0]);
        assert_eq!(quad_count(&s), 1);
    }

    #[test]
    fn test_empty_strip() {
        let s = new_quad_strip(vec![], vec![]);
        assert_eq!(quad_count(&s), 0);
    }

    #[test]
    fn test_single_point_strip() {
        let s = new_quad_strip(vec![[0.0, 0.0, 0.0]], vec![[0.0, 1.0, 0.0]]);
        assert_eq!(quad_count(&s), 0);
    }
}
