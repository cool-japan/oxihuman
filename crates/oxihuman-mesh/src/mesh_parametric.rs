// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Parametric shape generation with proper UVs.

use std::f32::consts::PI;

#[allow(dead_code)]
pub struct ParametricMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
pub enum ParametricShape {
    Sphere,
    Torus,
    Cylinder,
    Cone,
    Plane,
    Capsule,
}

#[allow(dead_code)]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Generate a UV sphere with proper UVs in `[0,1]`x`[0,1]`.
#[allow(dead_code)]
pub fn make_sphere(radius: f32, rings: u32, sectors: u32) -> ParametricMesh {
    let rings = rings.max(2);
    let sectors = sectors.max(3);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for r in 0..=rings {
        let phi = PI * (r as f32) / (rings as f32);
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let v = (r as f32) / (rings as f32);

        for s in 0..=sectors {
            let theta = 2.0 * PI * (s as f32) / (sectors as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let u = (s as f32) / (sectors as f32);

            let nx = sin_phi * cos_theta;
            let ny = cos_phi;
            let nz = sin_phi * sin_theta;

            positions.push([radius * nx, radius * ny, radius * nz]);
            normals.push([nx, ny, nz]);
            uvs.push([u, v]);
        }
    }

    let row_size = sectors + 1;
    for r in 0..rings {
        for s in 0..sectors {
            let cur = r * row_size + s;
            let next_r = (r + 1) * row_size + s;
            indices.push(cur);
            indices.push(next_r);
            indices.push(next_r + 1);
            indices.push(cur);
            indices.push(next_r + 1);
            indices.push(cur + 1);
        }
    }

    ParametricMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Generate a torus mesh.
#[allow(dead_code)]
pub fn make_torus(
    major_radius: f32,
    minor_radius: f32,
    major_segs: u32,
    minor_segs: u32,
) -> ParametricMesh {
    let major_segs = major_segs.max(3);
    let minor_segs = minor_segs.max(3);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for j in 0..=major_segs {
        let u = (j as f32) / (major_segs as f32);
        let theta = 2.0 * PI * u;
        let ct = theta.cos();
        let st = theta.sin();

        for i in 0..=minor_segs {
            let v = (i as f32) / (minor_segs as f32);
            let phi = 2.0 * PI * v;
            let cp = phi.cos();
            let sp = phi.sin();

            let x = (major_radius + minor_radius * cp) * ct;
            let y = minor_radius * sp;
            let z = (major_radius + minor_radius * cp) * st;

            let nx = cp * ct;
            let ny = sp;
            let nz = cp * st;

            positions.push([x, y, z]);
            normals.push(normalize3([nx, ny, nz]));
            uvs.push([u, v]);
        }
    }

    let row_size = minor_segs + 1;
    for j in 0..major_segs {
        for i in 0..minor_segs {
            let a = j * row_size + i;
            let b = (j + 1) * row_size + i;
            let c = (j + 1) * row_size + i + 1;
            let d = j * row_size + i + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    ParametricMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Generate a cylinder mesh, optionally with caps.
#[allow(dead_code)]
pub fn make_cylinder(
    radius: f32,
    height: f32,
    radial_segs: u32,
    height_segs: u32,
    caps: bool,
) -> ParametricMesh {
    let radial_segs = radial_segs.max(3);
    let height_segs = height_segs.max(1);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Tube
    for h in 0..=height_segs {
        let v = (h as f32) / (height_segs as f32);
        let y = -height / 2.0 + v * height;
        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            positions.push([radius * ct, y, radius * st]);
            normals.push([ct, 0.0, st]);
            uvs.push([u, v]);
        }
    }

    let row_size = radial_segs + 1;
    for h in 0..height_segs {
        for s in 0..radial_segs {
            let a = h * row_size + s;
            let b = (h + 1) * row_size + s;
            let c = (h + 1) * row_size + s + 1;
            let d = h * row_size + s + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    if caps {
        // Bottom cap
        let bottom_center_idx = positions.len() as u32;
        positions.push([0.0, -height / 2.0, 0.0]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([0.5, 0.5]);

        let bottom_ring_start = positions.len() as u32;
        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            positions.push([radius * ct, -height / 2.0, radius * st]);
            normals.push([0.0, -1.0, 0.0]);
            uvs.push([ct * 0.5 + 0.5, st * 0.5 + 0.5]);
        }
        for s in 0..radial_segs {
            indices.push(bottom_center_idx);
            indices.push(bottom_ring_start + s + 1);
            indices.push(bottom_ring_start + s);
        }

        // Top cap
        let top_center_idx = positions.len() as u32;
        positions.push([0.0, height / 2.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5, 0.5]);

        let top_ring_start = positions.len() as u32;
        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            positions.push([radius * ct, height / 2.0, radius * st]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([ct * 0.5 + 0.5, st * 0.5 + 0.5]);
        }
        for s in 0..radial_segs {
            indices.push(top_center_idx);
            indices.push(top_ring_start + s);
            indices.push(top_ring_start + s + 1);
        }
    }

    ParametricMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Generate a cone mesh.
#[allow(dead_code)]
pub fn make_cone(radius: f32, height: f32, radial_segs: u32, height_segs: u32) -> ParametricMesh {
    let radial_segs = radial_segs.max(3);
    let height_segs = height_segs.max(1);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let slope = (radius / height).atan();
    let sin_slope = slope.sin();
    let cos_slope = slope.cos();

    for h in 0..=height_segs {
        let v = (h as f32) / (height_segs as f32);
        let y = v * height;
        let r = radius * (1.0 - v);
        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            positions.push([r * ct, y, r * st]);
            normals.push(normalize3([ct * cos_slope, sin_slope, st * cos_slope]));
            uvs.push([u, v]);
        }
    }

    let row_size = radial_segs + 1;
    for h in 0..height_segs {
        for s in 0..radial_segs {
            let a = h * row_size + s;
            let b = (h + 1) * row_size + s;
            let c = (h + 1) * row_size + s + 1;
            let d = h * row_size + s + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    // Bottom cap
    let base_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let base_ring_start = positions.len() as u32;
    for s in 0..=radial_segs {
        let u = (s as f32) / (radial_segs as f32);
        let theta = 2.0 * PI * u;
        let ct = theta.cos();
        let st = theta.sin();
        positions.push([radius * ct, 0.0, radius * st]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([ct * 0.5 + 0.5, st * 0.5 + 0.5]);
    }
    for s in 0..radial_segs {
        indices.push(base_center_idx);
        indices.push(base_ring_start + s + 1);
        indices.push(base_ring_start + s);
    }

    ParametricMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Generate a subdivided plane.
#[allow(dead_code)]
pub fn make_plane(width: f32, height: f32, w_segs: u32, h_segs: u32) -> ParametricMesh {
    let w_segs = w_segs.max(1);
    let h_segs = h_segs.max(1);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for h in 0..=h_segs {
        let v = (h as f32) / (h_segs as f32);
        let y = v * height - height / 2.0;
        for w in 0..=w_segs {
            let u = (w as f32) / (w_segs as f32);
            let x = u * width - width / 2.0;
            positions.push([x, 0.0, y]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, v]);
        }
    }

    let row_size = w_segs + 1;
    for h in 0..h_segs {
        for w in 0..w_segs {
            let a = h * row_size + w;
            let b = (h + 1) * row_size + w;
            let c = (h + 1) * row_size + w + 1;
            let d = h * row_size + w + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    ParametricMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Generate a capsule mesh (cylinder + two hemisphere caps).
#[allow(dead_code)]
pub fn make_capsule(radius: f32, height: f32, radial_segs: u32, cap_segs: u32) -> ParametricMesh {
    let radial_segs = radial_segs.max(3);
    let cap_segs = cap_segs.max(2);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_h = height / 2.0;

    // Bottom hemisphere (phi from PI to PI/2)
    for r in 0..=cap_segs {
        let t = (r as f32) / (cap_segs as f32);
        let phi = PI - t * PI / 2.0; // PI -> PI/2
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let v = t * 0.25; // [0, 0.25]

        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            let nx = sin_phi * ct;
            let ny = cos_phi;
            let nz = sin_phi * st;
            positions.push([radius * nx, radius * ny - half_h, radius * nz]);
            normals.push([nx, ny, nz]);
            uvs.push([u, v]);
        }
    }

    let row_size = radial_segs + 1;
    let bottom_rings = cap_segs + 1;
    for r in 0..cap_segs {
        for s in 0..radial_segs {
            let a = r * row_size + s;
            let b = (r + 1) * row_size + s;
            let c = (r + 1) * row_size + s + 1;
            let d = r * row_size + s + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    // Cylinder middle (1 ring between hemispheres)
    let mid_rings = 2u32;
    let cyl_start = positions.len() as u32;
    for h in 0..=mid_rings {
        let t = (h as f32) / (mid_rings as f32);
        let y = -half_h + t * height;
        let v = 0.25 + t * 0.5;
        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            positions.push([radius * ct, y, radius * st]);
            normals.push([ct, 0.0, st]);
            uvs.push([u, v]);
        }
    }
    for h in 0..mid_rings {
        for s in 0..radial_segs {
            let a = cyl_start + h * row_size + s;
            let b = cyl_start + (h + 1) * row_size + s;
            let c = cyl_start + (h + 1) * row_size + s + 1;
            let d = cyl_start + h * row_size + s + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    // Top hemisphere (phi from PI/2 to 0)
    let top_start = positions.len() as u32;
    for r in 0..=cap_segs {
        let t = (r as f32) / (cap_segs as f32);
        let phi = PI / 2.0 - t * PI / 2.0; // PI/2 -> 0
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let v = 0.75 + t * 0.25;

        for s in 0..=radial_segs {
            let u = (s as f32) / (radial_segs as f32);
            let theta = 2.0 * PI * u;
            let ct = theta.cos();
            let st = theta.sin();
            let nx = sin_phi * ct;
            let ny = cos_phi;
            let nz = sin_phi * st;
            positions.push([radius * nx, radius * ny + half_h, radius * nz]);
            normals.push([nx, ny, nz]);
            uvs.push([u, v]);
        }
    }

    let _ = bottom_rings;
    for r in 0..cap_segs {
        for s in 0..radial_segs {
            let a = top_start + r * row_size + s;
            let b = top_start + (r + 1) * row_size + s;
            let c = top_start + (r + 1) * row_size + s + 1;
            let d = top_start + r * row_size + s + 1;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(a);
            indices.push(c);
            indices.push(d);
        }
    }

    ParametricMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Estimate vertex count for a given shape.
#[allow(dead_code)]
pub fn parametric_vertex_count(shape: ParametricShape, p1: u32, p2: u32) -> usize {
    let p1 = p1.max(2) as usize;
    let p2 = p2.max(2) as usize;
    match shape {
        ParametricShape::Sphere => (p1 + 1) * (p2 + 1),
        ParametricShape::Torus => (p1 + 1) * (p2 + 1),
        ParametricShape::Cylinder => (p1 + 1) * (p2 + 1) + 2 * (p1 + 2),
        ParametricShape::Cone => (p1 + 1) * (p2 + 1) + p1 + 2,
        ParametricShape::Plane => (p1 + 1) * (p2 + 1),
        ParametricShape::Capsule => (p1 + 1) * (p2 * 2 + 3),
    }
}

/// Estimate index count for a given shape.
#[allow(dead_code)]
pub fn parametric_index_count(shape: ParametricShape, p1: u32, p2: u32) -> usize {
    let p1 = p1.max(2) as usize;
    let p2 = p2.max(2) as usize;
    match shape {
        ParametricShape::Sphere => p1 * p2 * 6,
        ParametricShape::Torus => p1 * p2 * 6,
        ParametricShape::Cylinder => p1 * p2 * 6 + p1 * 6,
        ParametricShape::Cone => p1 * p2 * 6 + p1 * 3,
        ParametricShape::Plane => p1 * p2 * 6,
        ParametricShape::Capsule => p1 * (p2 * 2 + 2) * 6,
    }
}

/// Validate a parametric mesh: indices in range, lengths consistent.
#[allow(dead_code)]
pub fn validate_parametric_mesh(mesh: &ParametricMesh) -> bool {
    let n = mesh.positions.len();
    if n == 0 {
        return false;
    }
    if mesh.normals.len() != n || mesh.uvs.len() != n {
        return false;
    }
    if mesh.indices.is_empty() {
        return false;
    }
    for &idx in &mesh.indices {
        if idx as usize >= n {
            return false;
        }
    }
    true
}

/// Combine two parametric meshes into one.
#[allow(dead_code)]
pub fn merge_parametric(mut a: ParametricMesh, b: ParametricMesh) -> ParametricMesh {
    let offset = a.positions.len() as u32;
    a.positions.extend(b.positions);
    a.normals.extend(b.normals);
    a.uvs.extend(b.uvs);
    for idx in b.indices {
        a.indices.push(idx + offset);
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_normalized(n: [f32; 3]) -> bool {
        let len2 = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        (len2 - 1.0).abs() < 1e-4
    }

    fn uvs_in_range(uvs: &[[f32; 2]]) -> bool {
        uvs.iter()
            .all(|uv| uv[0] >= 0.0 && uv[0] <= 1.0 && uv[1] >= 0.0 && uv[1] <= 1.0)
    }

    #[test]
    fn sphere_vertex_count() {
        let m = make_sphere(1.0, 8, 16);
        assert_eq!(m.positions.len(), 9 * 17);
    }

    #[test]
    fn sphere_uvs_in_range() {
        let m = make_sphere(1.0, 8, 16);
        assert!(uvs_in_range(&m.uvs));
    }

    #[test]
    fn sphere_normals_normalized() {
        let m = make_sphere(1.0, 8, 16);
        assert!(m.normals.iter().all(|&n| is_normalized(n)));
    }

    #[test]
    fn sphere_index_count() {
        let m = make_sphere(1.0, 8, 16);
        assert_eq!(m.indices.len(), 8 * 16 * 6);
    }

    #[test]
    fn torus_index_count() {
        let m = make_torus(2.0, 0.5, 16, 8);
        assert_eq!(m.indices.len(), 16 * 8 * 6);
    }

    #[test]
    fn torus_uvs_in_range() {
        let m = make_torus(2.0, 0.5, 16, 8);
        assert!(uvs_in_range(&m.uvs));
    }

    #[test]
    fn cylinder_no_caps_vertex_count() {
        let m = make_cylinder(1.0, 2.0, 8, 4, false);
        assert_eq!(m.positions.len(), 5 * 9);
    }

    #[test]
    fn cylinder_with_caps_has_more_vertices() {
        let m_no = make_cylinder(1.0, 2.0, 8, 4, false);
        let m_yes = make_cylinder(1.0, 2.0, 8, 4, true);
        assert!(m_yes.positions.len() > m_no.positions.len());
    }

    #[test]
    fn plane_vertex_count() {
        let m = make_plane(2.0, 2.0, 4, 4);
        assert_eq!(m.positions.len(), 5 * 5);
    }

    #[test]
    fn plane_all_normals_up() {
        let m = make_plane(2.0, 2.0, 4, 4);
        for n in &m.normals {
            assert!((n[1] - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn cone_index_count() {
        let m = make_cone(1.0, 2.0, 8, 2);
        // tube: 8*2*6=96, cap: 8*3=24 => 120
        assert_eq!(m.indices.len(), 8 * 2 * 6 + 8 * 3);
    }

    #[test]
    fn capsule_normals_normalized() {
        let m = make_capsule(0.5, 1.0, 8, 4);
        assert!(m.normals.iter().all(|&n| is_normalized(n)));
    }

    #[test]
    fn validate_sphere_passes() {
        let m = make_sphere(1.0, 8, 16);
        assert!(validate_parametric_mesh(&m));
    }

    #[test]
    fn validate_empty_fails() {
        let m = ParametricMesh {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
        };
        assert!(!validate_parametric_mesh(&m));
    }

    #[test]
    fn merge_combines_vertex_counts() {
        let a = make_sphere(1.0, 4, 8);
        let b = make_plane(1.0, 1.0, 2, 2);
        let a_len = a.positions.len();
        let b_len = b.positions.len();
        let merged = merge_parametric(a, b);
        assert_eq!(merged.positions.len(), a_len + b_len);
    }

    #[test]
    fn merge_indices_valid() {
        let a = make_sphere(1.0, 4, 8);
        let b = make_plane(1.0, 1.0, 2, 2);
        let merged = merge_parametric(a, b);
        assert!(validate_parametric_mesh(&merged));
    }

    #[test]
    fn sphere_radius_positions() {
        let r = 2.0_f32;
        let m = make_sphere(r, 6, 12);
        for p in &m.positions {
            let dist = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((dist - r).abs() < 1e-4, "dist={dist} r={r}");
        }
    }
}
