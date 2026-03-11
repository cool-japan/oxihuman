// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Geodesic sphere via icosahedral subdivision.

/// Parameters for geodesic sphere generation.
#[derive(Debug, Clone)]
pub struct GeosphereParams {
    /// Radius of the sphere.
    pub radius: f32,
    /// Subdivision level (0 = icosahedron, 1 = 80 tris, 2 = 320 tris, …).
    pub subdivisions: u32,
}

impl Default for GeosphereParams {
    fn default() -> Self {
        Self {
            radius: 0.5,
            subdivisions: 2,
        }
    }
}

/// Generated geodesic sphere mesh.
#[derive(Debug, Clone)]
pub struct GeosphereMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl GeosphereMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Build a geodesic sphere.
pub fn build_geosphere(params: &GeosphereParams) -> GeosphereMesh {
    /* start with an icosahedron */
    let t = (1.0 + 5.0f32.sqrt()) * 0.5;
    let verts_raw = [
        [-1.0, t, 0.0],
        [1.0, t, 0.0],
        [-1.0, -t, 0.0],
        [1.0, -t, 0.0],
        [0.0, -1.0, t],
        [0.0, 1.0, t],
        [0.0, -1.0, -t],
        [0.0, 1.0, -t],
        [t, 0.0, -1.0],
        [t, 0.0, 1.0],
        [-t, 0.0, -1.0],
        [-t, 0.0, 1.0],
    ];
    let mut positions: Vec<[f32; 3]> = verts_raw.iter().map(|v| normalize3(*v)).collect();
    let mut indices: Vec<u32> = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7,
        1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9,
        8, 1,
    ];
    /* subdivide */
    use std::collections::HashMap;
    for _ in 0..params.subdivisions {
        let mut midpoints: HashMap<(u32, u32), u32> = HashMap::new();
        let mut new_indices = Vec::new();
        let tri_count = indices.len() / 3;
        for t in 0..tri_count {
            let a = indices[t * 3];
            let b = indices[t * 3 + 1];
            let c = indices[t * 3 + 2];
            let ab = get_midpoint(&mut positions, &mut midpoints, a, b);
            let bc = get_midpoint(&mut positions, &mut midpoints, b, c);
            let ca = get_midpoint(&mut positions, &mut midpoints, c, a);
            new_indices.extend_from_slice(&[a, ab, ca, ab, b, bc, ca, bc, c, ab, bc, ca]);
        }
        indices = new_indices;
    }
    /* scale to radius */
    for p in &mut positions {
        p[0] *= params.radius;
        p[1] *= params.radius;
        p[2] *= params.radius;
    }
    let normals: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| normalize3_scaled(p, params.radius))
        .collect();
    GeosphereMesh {
        positions,
        normals,
        indices,
    }
}

fn get_midpoint(
    positions: &mut Vec<[f32; 3]>,
    cache: &mut std::collections::HashMap<(u32, u32), u32>,
    a: u32,
    b: u32,
) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let pa = positions[a as usize];
    let pb = positions[b as usize];
    let mid = normalize3([
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ]);
    let idx = positions.len() as u32;
    positions.push(mid);
    cache.insert(key, idx);
    idx
}

/// Expected triangle count at a given subdivision level.
pub fn expected_triangle_count(subdivisions: u32) -> usize {
    20 * 4usize.pow(subdivisions)
}

/// Validate geosphere params.
pub fn validate_geosphere_params(p: &GeosphereParams) -> bool {
    p.radius > 0.0 && p.subdivisions <= 6
}

/// Compute the surface area of the sphere.
pub fn sphere_surface_area(radius: f32) -> f32 {
    4.0 * std::f32::consts::PI * radius * radius
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn normalize3_scaled(v: &[f32; 3], radius: f32) -> [f32; 3] {
    if radius < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / radius, v[1] / radius, v[2] / radius]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosahedron_at_level0() {
        /* level 0 = 20 triangles */
        let p = GeosphereParams {
            subdivisions: 0,
            radius: 1.0,
        };
        let m = build_geosphere(&p);
        assert_eq!(m.triangle_count(), 20);
    }

    #[test]
    fn level1_triangle_count() {
        /* level 1 = 80 triangles */
        let p = GeosphereParams {
            subdivisions: 1,
            radius: 1.0,
        };
        let m = build_geosphere(&p);
        assert_eq!(m.triangle_count(), 80);
    }

    #[test]
    fn level2_triangle_count() {
        let p = GeosphereParams {
            subdivisions: 2,
            radius: 1.0,
        };
        let m = build_geosphere(&p);
        assert_eq!(m.triangle_count(), 320);
    }

    #[test]
    fn all_vertices_on_sphere() {
        /* all verts should be at radius ≈ 1 */
        let p = GeosphereParams {
            subdivisions: 1,
            radius: 1.0,
        };
        let m = build_geosphere(&p);
        for v in &m.positions {
            let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-4, "r={r}");
        }
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_geosphere(&GeosphereParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_unit_length() {
        let m = build_geosphere(&GeosphereParams {
            subdivisions: 1,
            radius: 0.5,
        });
        for nrm in &m.normals {
            let len = (nrm[0] * nrm[0] + nrm[1] * nrm[1] + nrm[2] * nrm[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn surface_area_sphere() {
        /* 4π r² */
        let area = sphere_surface_area(1.0);
        assert!((area - 4.0 * std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn expected_triangle_count_formula() {
        assert_eq!(expected_triangle_count(0), 20);
        assert_eq!(expected_triangle_count(2), 320);
    }

    #[test]
    fn validate_ok() {
        assert!(validate_geosphere_params(&GeosphereParams::default()));
    }

    #[test]
    fn validate_bad_subdivisions() {
        let p = GeosphereParams {
            radius: 1.0,
            subdivisions: 7,
        };
        assert!(!validate_geosphere_params(&p));
    }
}
