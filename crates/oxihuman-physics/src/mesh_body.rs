// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh body: physics body represented by a triangle mesh with AABB.

/// A vertex in the mesh body.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MeshVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
}

/// A triangle (indices into vertex buffer).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshTriangle {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

/// Axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MeshAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Physics mesh body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshBody {
    pub vertices: Vec<MeshVertex>,
    pub triangles: Vec<MeshTriangle>,
    pub aabb: MeshAabb,
    pub transform: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub is_static: bool,
}

/// Create a new `MeshBody`.
#[allow(dead_code)]
pub fn new_mesh_body(
    vertices: Vec<MeshVertex>,
    triangles: Vec<MeshTriangle>,
    mass: f32,
) -> MeshBody {
    let aabb = compute_aabb(&vertices);
    MeshBody {
        vertices,
        triangles,
        aabb,
        transform: [0.0; 3],
        velocity: [0.0; 3],
        mass: mass.max(1e-9),
        is_static: false,
    }
}

fn compute_aabb(verts: &[MeshVertex]) -> MeshAabb {
    if verts.is_empty() {
        return MeshAabb::default();
    }
    let mut mn = verts[0].pos;
    let mut mx = verts[0].pos;
    #[allow(clippy::needless_range_loop)]
    for i in 1..verts.len() {
        for ax in 0..3 {
            if verts[i].pos[ax] < mn[ax] {
                mn[ax] = verts[i].pos[ax];
            }
            if verts[i].pos[ax] > mx[ax] {
                mx[ax] = verts[i].pos[ax];
            }
        }
    }
    MeshAabb { min: mn, max: mx }
}

/// Translate the mesh body.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn mbody_translate(body: &mut MeshBody, delta: [f32; 3]) {
    for ax in 0..3 {
        body.transform[ax] += delta[ax];
        body.aabb.min[ax] += delta[ax];
        body.aabb.max[ax] += delta[ax];
    }
}

/// Step the body (Euler integration).
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn mbody_step(body: &mut MeshBody, gravity: [f32; 3], dt: f32) {
    if body.is_static {
        return;
    }
    for ax in 0..3 {
        body.velocity[ax] += gravity[ax] * dt;
        body.transform[ax] += body.velocity[ax] * dt;
    }
}

/// AABB volume.
#[allow(dead_code)]
pub fn mbody_aabb_volume(body: &MeshBody) -> f32 {
    let dx = (body.aabb.max[0] - body.aabb.min[0]).max(0.0);
    let dy = (body.aabb.max[1] - body.aabb.min[1]).max(0.0);
    let dz = (body.aabb.max[2] - body.aabb.min[2]).max(0.0);
    dx * dy * dz
}

/// Triangle count.
#[allow(dead_code)]
pub fn mbody_triangle_count(body: &MeshBody) -> usize {
    body.triangles.len()
}

/// Vertex count.
#[allow(dead_code)]
pub fn mbody_vertex_count(body: &MeshBody) -> usize {
    body.vertices.len()
}

/// Recompute normals (flat per triangle).
#[allow(dead_code)]
pub fn mbody_recompute_normals(body: &mut MeshBody) {
    for tri in &body.triangles {
        let pa = body.vertices[tri.a].pos;
        let pb = body.vertices[tri.b].pos;
        let pc = body.vertices[tri.c].pos;
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        let nn = [n[0] / len, n[1] / len, n[2] / len];
        body.vertices[tri.a].normal = nn;
        body.vertices[tri.b].normal = nn;
        body.vertices[tri.c].normal = nn;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box_mesh() -> MeshBody {
        let verts = vec![
            MeshVertex {
                pos: [0.0, 0.0, 0.0],
                normal: [0.0; 3],
            },
            MeshVertex {
                pos: [1.0, 0.0, 0.0],
                normal: [0.0; 3],
            },
            MeshVertex {
                pos: [0.0, 1.0, 0.0],
                normal: [0.0; 3],
            },
            MeshVertex {
                pos: [1.0, 1.0, 0.0],
                normal: [0.0; 3],
            },
        ];
        let tris = vec![MeshTriangle { a: 0, b: 1, c: 2 }];
        new_mesh_body(verts, tris, 1.0)
    }

    #[test]
    fn test_new_mesh_body() {
        let body = make_box_mesh();
        assert_eq!(mbody_vertex_count(&body), 4);
        assert_eq!(mbody_triangle_count(&body), 1);
    }

    #[test]
    fn test_aabb_computed() {
        let body = make_box_mesh();
        assert!((body.aabb.max[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_volume() {
        let verts = vec![
            MeshVertex {
                pos: [0.0, 0.0, 0.0],
                normal: [0.0; 3],
            },
            MeshVertex {
                pos: [1.0, 0.0, 0.0],
                normal: [0.0; 3],
            },
            MeshVertex {
                pos: [0.0, 1.0, 0.0],
                normal: [0.0; 3],
            },
            MeshVertex {
                pos: [0.0, 0.0, 1.0],
                normal: [0.0; 3],
            },
        ];
        let tris = vec![MeshTriangle { a: 0, b: 1, c: 2 }];
        let body = new_mesh_body(verts, tris, 1.0);
        let vol = mbody_aabb_volume(&body);
        assert!(vol > 0.0);
    }

    #[test]
    fn test_translate() {
        let mut body = make_box_mesh();
        mbody_translate(&mut body, [1.0, 0.0, 0.0]);
        assert!((body.transform[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_moves_with_gravity() {
        let mut body = make_box_mesh();
        mbody_step(&mut body, [0.0, -9.81, 0.0], 1.0);
        assert!(body.transform[1] < 0.0);
    }

    #[test]
    fn test_static_does_not_move() {
        let mut body = make_box_mesh();
        body.is_static = true;
        mbody_step(&mut body, [0.0, -9.81, 0.0], 1.0);
        assert!((body.transform[1]).abs() < 1e-9);
    }

    #[test]
    fn test_recompute_normals() {
        let mut body = make_box_mesh();
        mbody_recompute_normals(&mut body);
        let n = body.vertices[0].normal;
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_mesh_aabb() {
        let body = new_mesh_body(vec![], vec![], 1.0);
        assert!((mbody_aabb_volume(&body)).abs() < 1e-9);
    }

    #[test]
    fn test_mass_clamped() {
        let body = new_mesh_body(vec![], vec![], -99.0);
        assert!(body.mass > 0.0);
    }
}
