#![allow(dead_code)]

/// Barycentric coordinates (u, v, w) where u + v + w = 1.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BaryCoord {
    pub u: f32,
    pub v: f32,
    pub w: f32,
}

/// Compute barycentric coordinates of point p in triangle (a, b, c).
#[allow(dead_code)]
pub fn barycentric_coords(
    p: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> BaryCoord {
    let v0 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v1 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let v2 = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];

    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-10 {
        return BaryCoord { u: 1.0, v: 0.0, w: 0.0 };
    }

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    BaryCoord { u, v, w }
}

/// Interpolate a value using barycentric coordinates.
#[allow(dead_code)]
pub fn barycentric_interpolate(bary: &BaryCoord, va: f32, vb: f32, vc: f32) -> f32 {
    bary.u * va + bary.v * vb + bary.w * vc
}

/// Check if a point is inside a triangle using barycentric coords.
#[allow(dead_code)]
pub fn is_inside_triangle_bary(bary: &BaryCoord) -> bool {
    bary.u >= -1e-6 && bary.v >= -1e-6 && bary.w >= -1e-6
}

/// Convert barycentric coordinates back to Cartesian.
#[allow(dead_code)]
pub fn barycentric_to_cartesian(
    bary: &BaryCoord,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> [f32; 3] {
    [
        bary.u * a[0] + bary.v * b[0] + bary.w * c[0],
        bary.u * a[1] + bary.v * b[1] + bary.w * c[1],
        bary.u * a[2] + bary.v * b[2] + bary.w * c[2],
    ]
}

/// Compute area-weighted barycentric coordinate contribution.
#[allow(dead_code)]
pub fn barycentric_area_weight(bary: &BaryCoord) -> f32 {
    bary.u.abs() + bary.v.abs() + bary.w.abs()
}

/// Find the closest point on a triangle in barycentric coordinates.
#[allow(dead_code)]
pub fn closest_point_bary(bary: &BaryCoord) -> BaryCoord {
    let u = bary.u.max(0.0);
    let v = bary.v.max(0.0);
    let w = bary.w.max(0.0);
    let sum = u + v + w;
    if sum < 1e-10 {
        return BaryCoord { u: 1.0 / 3.0, v: 1.0 / 3.0, w: 1.0 / 3.0 };
    }
    BaryCoord {
        u: u / sum,
        v: v / sum,
        w: w / sum,
    }
}

/// Compute the minimum distance from a barycentric point to any triangle edge.
#[allow(dead_code)]
pub fn bary_edge_distance(bary: &BaryCoord) -> f32 {
    bary.u.min(bary.v).min(bary.w)
}

/// Check if barycentric coordinates are valid (sum to ~1).
#[allow(dead_code)]
pub fn bary_is_valid(bary: &BaryCoord) -> bool {
    let sum = bary.u + bary.v + bary.w;
    (sum - 1.0).abs() < 1e-4
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barycentric_at_vertex() {
        let bary = barycentric_coords(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((bary.u - 1.0).abs() < 1e-4);
        assert!(bary.v.abs() < 1e-4);
        assert!(bary.w.abs() < 1e-4);
    }

    #[test]
    fn test_barycentric_center() {
        let bary = barycentric_coords(
            [1.0 / 3.0, 1.0 / 3.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((bary.u - 1.0 / 3.0).abs() < 1e-3);
    }

    #[test]
    fn test_interpolate() {
        let bary = BaryCoord { u: 0.5, v: 0.25, w: 0.25 };
        let val = barycentric_interpolate(&bary, 1.0, 2.0, 3.0);
        assert!((val - 1.75).abs() < 1e-4);
    }

    #[test]
    fn test_inside_triangle() {
        let bary = BaryCoord { u: 0.3, v: 0.3, w: 0.4 };
        assert!(is_inside_triangle_bary(&bary));
    }

    #[test]
    fn test_outside_triangle() {
        let bary = BaryCoord { u: -0.1, v: 0.5, w: 0.6 };
        assert!(!is_inside_triangle_bary(&bary));
    }

    #[test]
    fn test_to_cartesian() {
        let bary = BaryCoord { u: 1.0, v: 0.0, w: 0.0 };
        let p = barycentric_to_cartesian(&bary, [3.0, 4.0, 5.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((p[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_closest_point_bary() {
        let bary = BaryCoord { u: -0.1, v: 0.5, w: 0.6 };
        let closest = closest_point_bary(&bary);
        assert!(closest.u >= 0.0);
        assert!(bary_is_valid(&closest));
    }

    #[test]
    fn test_bary_edge_distance() {
        let bary = BaryCoord { u: 0.2, v: 0.3, w: 0.5 };
        assert!((bary_edge_distance(&bary) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_bary_is_valid() {
        let bary = BaryCoord { u: 0.3, v: 0.3, w: 0.4 };
        assert!(bary_is_valid(&bary));
    }

    #[test]
    fn test_bary_invalid() {
        let bary = BaryCoord { u: 0.5, v: 0.5, w: 0.5 };
        assert!(!bary_is_valid(&bary));
    }
}
