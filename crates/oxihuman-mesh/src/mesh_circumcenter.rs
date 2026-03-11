// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Circumcenter computation for mesh triangles.

/// Circumcenter result for a triangle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Circumcenter {
    pub center: [f32; 3],
    pub radius: f32,
}

/// Compute the circumcenter and circumradius of a triangle.
#[allow(dead_code)]
pub fn triangle_circumcenter(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> Circumcenter {
    let a = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let b = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let d = 2.0 * dot3(cross, cross);
    if d.abs() < 1e-12 {
        let mid = [
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ];
        return Circumcenter {
            center: mid,
            radius: 0.0,
        };
    }
    let a_sq = dot3(a, a);
    let b_sq = dot3(b, b);
    let t1 = [
        b_sq * cross[1] * a[2] - b_sq * cross[2] * a[1] + a_sq * cross[2] * b[1]
            - a_sq * cross[1] * b[2],
        b_sq * cross[2] * a[0] - b_sq * cross[0] * a[2] + a_sq * cross[0] * b[2]
            - a_sq * cross[2] * b[0],
        b_sq * cross[0] * a[1] - b_sq * cross[1] * a[0] + a_sq * cross[1] * b[0]
            - a_sq * cross[0] * b[1],
    ];
    let center = [v0[0] + t1[0] / d, v0[1] + t1[1] / d, v0[2] + t1[2] / d];
    let dx = center[0] - v0[0];
    let dy = center[1] - v0[1];
    let dz = center[2] - v0[2];
    let radius = (dx * dx + dy * dy + dz * dz).sqrt();
    Circumcenter { center, radius }
}

/// Dot product.
#[allow(dead_code)]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute circumcenters for all triangles.
#[allow(dead_code)]
pub fn compute_circumcenters(positions: &[[f32; 3]], indices: &[u32]) -> Vec<Circumcenter> {
    let tri_count = indices.len() / 3;
    let mut result = Vec::with_capacity(tri_count);
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        result.push(triangle_circumcenter(
            positions[i0],
            positions[i1],
            positions[i2],
        ));
    }
    result
}

/// Average circumradius.
#[allow(dead_code)]
pub fn avg_circumradius(centers: &[Circumcenter]) -> f32 {
    if centers.is_empty() {
        return 0.0;
    }
    centers.iter().map(|c| c.radius).sum::<f32>() / centers.len() as f32
}

/// Maximum circumradius.
#[allow(dead_code)]
pub fn max_circumradius(centers: &[Circumcenter]) -> f32 {
    centers.iter().map(|c| c.radius).fold(0.0f32, f32::max)
}

/// Check if circumcenter is inside the triangle (acute triangle test).
#[allow(dead_code)]
pub fn is_acute_triangle(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> bool {
    let a2 = dist_sq(v1, v2);
    let b2 = dist_sq(v0, v2);
    let c2 = dist_sq(v0, v1);
    a2 + b2 > c2 && b2 + c2 > a2 && a2 + c2 > b2
}

/// Squared distance.
#[allow(dead_code)]
fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn circumcenter_to_json(cc: &Circumcenter) -> String {
    format!(
        "{{\"center\":[{:.6},{:.6},{:.6}],\"radius\":{:.6}}}",
        cc.center[0], cc.center[1], cc.center[2], cc.radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equilateral_circumcenter() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 0.866_025_4, 0.0];
        let cc = triangle_circumcenter(v0, v1, v2);
        assert!((cc.center[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_right_triangle_circumcenter() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [2.0, 0.0, 0.0];
        let v2 = [0.0, 2.0, 0.0];
        let cc = triangle_circumcenter(v0, v1, v2);
        // circumcenter is midpoint of hypotenuse
        assert!((cc.center[0] - 1.0).abs() < 0.1);
        assert!((cc.center[1] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_circumcenters() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let ccs = compute_circumcenters(&pos, &idx);
        assert_eq!(ccs.len(), 1);
    }

    #[test]
    fn test_avg_circumradius() {
        let ccs = vec![
            Circumcenter {
                center: [0.0; 3],
                radius: 1.0,
            },
            Circumcenter {
                center: [0.0; 3],
                radius: 3.0,
            },
        ];
        assert!((avg_circumradius(&ccs) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_circumradius() {
        let ccs = vec![
            Circumcenter {
                center: [0.0; 3],
                radius: 1.0,
            },
            Circumcenter {
                center: [0.0; 3],
                radius: 5.0,
            },
        ];
        assert!((max_circumradius(&ccs) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_acute() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 0.866_025_4, 0.0];
        assert!(is_acute_triangle(v0, v1, v2));
    }

    #[test]
    fn test_degenerate() {
        let v = [0.0, 0.0, 0.0];
        let cc = triangle_circumcenter(v, v, v);
        assert!((cc.radius).abs() < 1e-6);
    }

    #[test]
    fn test_empty() {
        let ccs = compute_circumcenters(&[], &[]);
        assert!(ccs.is_empty());
        assert!((avg_circumradius(&ccs)).abs() < 1e-9);
    }

    #[test]
    fn test_to_json() {
        let cc = Circumcenter {
            center: [1.0, 2.0, 3.0],
            radius: 4.0,
        };
        let j = circumcenter_to_json(&cc);
        assert!(j.contains("\"radius\":4.0"));
    }

    #[test]
    fn test_multiple_circumcenters() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let ccs = compute_circumcenters(&pos, &idx);
        assert_eq!(ccs.len(), 2);
    }
}
