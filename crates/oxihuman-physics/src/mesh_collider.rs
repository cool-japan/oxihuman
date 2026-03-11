// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Triangle mesh collider (AABB-based broadphase).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Triangle {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub c: [f32; 3],
}

impl Triangle {
    pub fn normal(&self) -> [f32; 3] {
        let ab = [
            self.b[0] - self.a[0],
            self.b[1] - self.a[1],
            self.b[2] - self.a[2],
        ];
        let ac = [
            self.c[0] - self.a[0],
            self.c[1] - self.a[1],
            self.c[2] - self.a[2],
        ];
        let nx = ab[1] * ac[2] - ab[2] * ac[1];
        let ny = ab[2] * ac[0] - ab[0] * ac[2];
        let nz = ab[0] * ac[1] - ab[1] * ac[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
        [nx / len, ny / len, nz / len]
    }
    pub fn area(&self) -> f32 {
        let ab = [
            self.b[0] - self.a[0],
            self.b[1] - self.a[1],
            self.b[2] - self.a[2],
        ];
        let ac = [
            self.c[0] - self.a[0],
            self.c[1] - self.a[1],
            self.c[2] - self.a[2],
        ];
        let cx = ab[1] * ac[2] - ab[2] * ac[1];
        let cy = ab[2] * ac[0] - ab[0] * ac[2];
        let cz = ab[0] * ac[1] - ab[1] * ac[0];
        0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
    }
    pub fn centroid(&self) -> [f32; 3] {
        [
            (self.a[0] + self.b[0] + self.c[0]) / 3.0,
            (self.a[1] + self.b[1] + self.c[1]) / 3.0,
            (self.a[2] + self.b[2] + self.c[2]) / 3.0,
        ]
    }
}

#[allow(dead_code)]
pub struct MeshCollider {
    triangles: Vec<Triangle>,
    aabb_min: [f32; 3],
    aabb_max: [f32; 3],
}

#[allow(dead_code)]
impl MeshCollider {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let (mn, mx) = compute_aabb(&triangles);
        Self {
            triangles,
            aabb_min: mn,
            aabb_max: mx,
        }
    }
    pub fn tri_count(&self) -> usize {
        self.triangles.len()
    }
    pub fn aabb_min(&self) -> [f32; 3] {
        self.aabb_min
    }
    pub fn aabb_max(&self) -> [f32; 3] {
        self.aabb_max
    }
    pub fn total_area(&self) -> f32 {
        self.triangles.iter().map(|t| t.area()).sum()
    }
    pub fn point_inside_aabb(&self, p: [f32; 3]) -> bool {
        (0..3).all(|i| p[i] >= self.aabb_min[i] && p[i] <= self.aabb_max[i])
    }
    pub fn sphere_overlaps_aabb(&self, center: [f32; 3], radius: f32) -> bool {
        let mut d2 = 0.0f32;
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 {
            let e = (self.aabb_min[i] - center[i])
                .max(0.0)
                .max(center[i] - self.aabb_max[i]);
            d2 += e * e;
        }
        d2 <= radius * radius
    }
    pub fn nearest_triangle_idx(&self, p: [f32; 3]) -> Option<usize> {
        self.triangles
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let ca = a.centroid();
                let cb = b.centroid();
                let da: f32 = (0..3).map(|i| (ca[i] - p[i]).powi(2)).sum::<f32>();
                let db: f32 = (0..3).map(|i| (cb[i] - p[i]).powi(2)).sum::<f32>();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }
}

fn compute_aabb(tris: &[Triangle]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::INFINITY; 3];
    let mut mx = [f32::NEG_INFINITY; 3];
    for t in tris {
        for v in [t.a, t.b, t.c] {
            for i in 0..3 {
                mn[i] = mn[i].min(v[i]);
                mx[i] = mx[i].max(v[i]);
            }
        }
    }
    if mn[0].is_infinite() {
        return ([0.0; 3], [0.0; 3]);
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn make_unit_tri() -> Triangle {
    Triangle {
        a: [0.0, 0.0, 0.0],
        b: [1.0, 0.0, 0.0],
        c: [0.0, 1.0, 0.0],
    }
}
#[allow(dead_code)]
pub fn new_mesh_collider(tris: Vec<Triangle>) -> MeshCollider {
    MeshCollider::new(tris)
}
#[allow(dead_code)]
pub fn mc_tri_count(m: &MeshCollider) -> usize {
    m.tri_count()
}
#[allow(dead_code)]
pub fn mc_total_area(m: &MeshCollider) -> f32 {
    m.total_area()
}
#[allow(dead_code)]
pub fn mc_point_in_aabb(m: &MeshCollider, p: [f32; 3]) -> bool {
    m.point_inside_aabb(p)
}
#[allow(dead_code)]
pub fn mc_sphere_overlaps(m: &MeshCollider, c: [f32; 3], r: f32) -> bool {
    m.sphere_overlaps_aabb(c, r)
}
#[allow(dead_code)]
pub fn mc_nearest_tri(m: &MeshCollider, p: [f32; 3]) -> Option<usize> {
    m.nearest_triangle_idx(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tri_count() {
        let m = new_mesh_collider(vec![make_unit_tri()]);
        assert_eq!(mc_tri_count(&m), 1);
    }
    #[test]
    fn test_normal_upward() {
        let t = make_unit_tri();
        let n = t.normal();
        assert!((n[2] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_area() {
        let t = make_unit_tri();
        assert!((t.area() - 0.5).abs() < 1e-5);
    }
    #[test]
    fn test_total_area() {
        let m = new_mesh_collider(vec![make_unit_tri(), make_unit_tri()]);
        assert!((mc_total_area(&m) - 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_aabb_min_max() {
        let m = new_mesh_collider(vec![make_unit_tri()]);
        assert_eq!(m.aabb_min(), [0.0, 0.0, 0.0]);
        assert_eq!(m.aabb_max(), [1.0, 1.0, 0.0]);
    }
    #[test]
    fn test_point_in_aabb() {
        let m = new_mesh_collider(vec![make_unit_tri()]);
        assert!(mc_point_in_aabb(&m, [0.5, 0.5, 0.0]));
        assert!(!mc_point_in_aabb(&m, [2.0, 0.0, 0.0]));
    }
    #[test]
    fn test_sphere_overlaps() {
        let m = new_mesh_collider(vec![make_unit_tri()]);
        assert!(mc_sphere_overlaps(&m, [0.5, 0.5, 0.0], 0.1));
        assert!(!mc_sphere_overlaps(&m, [10.0, 10.0, 10.0], 0.1));
    }
    #[test]
    fn test_centroid() {
        let t = make_unit_tri();
        let c = t.centroid();
        assert!((c[0] - 1.0 / 3.0).abs() < 1e-5);
    }
    #[test]
    fn test_nearest_triangle() {
        let t1 = Triangle {
            a: [0.0, 0.0, 0.0],
            b: [1.0, 0.0, 0.0],
            c: [0.5, 1.0, 0.0],
        };
        let t2 = Triangle {
            a: [5.0, 5.0, 0.0],
            b: [6.0, 5.0, 0.0],
            c: [5.5, 6.0, 0.0],
        };
        let m = new_mesh_collider(vec![t1, t2]);
        let idx = mc_nearest_tri(&m, [0.5, 0.3, 0.0]).unwrap();
        assert_eq!(idx, 0);
    }
    #[test]
    fn test_empty_collider() {
        let m = new_mesh_collider(vec![]);
        assert_eq!(mc_tri_count(&m), 0);
        assert_eq!(mc_total_area(&m), 0.0);
    }
}
