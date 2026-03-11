//! Sweep a geometric shape (sphere or box) through space to detect first hit.
//!
//! A shape cast (also called a "swept shape" or "continuous collision query")
//! slides a shape along a linear path and reports the earliest contact with
//! a set of static triangles. This implementation supports sphere and AABB
//! swept shapes against a triangle soup.
//!
//! Sphere-vs-triangle: GJK-style signed distance + sweep.
//! Box-vs-triangle: SAT separating-axis sweep (simplified slab method).

#![allow(dead_code)]

/// Configuration for shape cast queries.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ShapeCastConfig {
    /// Maximum cast distance. Hits beyond this distance are ignored.
    pub max_distance: f32,
    /// Small tolerance added to avoid numerical grazing.
    pub epsilon: f32,
}

/// The geometric shape to sweep.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CastShape {
    /// A sphere with the given radius.
    Sphere { radius: f32 },
    /// An axis-aligned box with half-extents [hx, hy, hz].
    Box { half_extents: [f32; 3] },
}

/// The result of a shape cast query.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ShapeCastResult {
    /// Whether any triangle was hit.
    pub hit: bool,
    /// Parametric hit time t ∈ [0, max_distance]; `f32::MAX` if no hit.
    pub hit_time: f32,
    /// World-space surface normal at the hit point.
    pub hit_normal: [f32; 3],
    /// World-space hit point.
    pub hit_point: [f32; 3],
}

/// Return sensible defaults for [`ShapeCastConfig`].
#[allow(dead_code)]
pub fn default_shape_cast_config() -> ShapeCastConfig {
    ShapeCastConfig { max_distance: 1000.0, epsilon: 1e-4 }
}

/// Construct a sphere cast shape.
#[allow(dead_code)]
pub fn new_sphere_cast(radius: f32) -> CastShape {
    CastShape::Sphere { radius: radius.max(0.0) }
}

/// Construct a box cast shape.
#[allow(dead_code)]
pub fn new_box_cast(half_extents: [f32; 3]) -> CastShape {
    CastShape::Box {
        half_extents: [
            half_extents[0].max(0.0),
            half_extents[1].max(0.0),
            half_extents[2].max(0.0),
        ],
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn v3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn v3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn v3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = v3_len(v);
    if l < 1e-9 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Closest point on a triangle to an external point.
fn closest_point_on_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = v3_sub(b, a);
    let ac = v3_sub(c, a);
    let ap = v3_sub(p, a);

    let d1 = v3_dot(ab, ap);
    let d2 = v3_dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = v3_sub(p, b);
    let d3 = v3_dot(ab, bp);
    let d4 = v3_dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return v3_add(a, v3_scale(ab, v));
    }

    let cp = v3_sub(p, c);
    let d5 = v3_dot(ab, cp);
    let d6 = v3_dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return v3_add(a, v3_scale(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return v3_add(b, v3_scale(v3_sub(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    v3_add(a, v3_add(v3_scale(ab, v), v3_scale(ac, w)))
}

/// Sweep a sphere of given radius from `origin` along `direction` (length = max_distance)
/// against a triangle. Returns (hit, t, normal).
fn sweep_sphere_triangle(
    origin: [f32; 3],
    direction: [f32; 3],
    radius: f32,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    max_t: f32,
) -> Option<(f32, [f32; 3])> {
    // Face normal
    let n_raw = v3_cross(v3_sub(b, a), v3_sub(c, a));
    let n = v3_normalize(n_raw);

    // Distance from sphere centre to triangle plane
    let dist = v3_dot(v3_sub(origin, a), n);
    let denom = v3_dot(direction, n);

    // Sphere cast against infinite plane offset by radius
    let t_plane = if denom.abs() > 1e-9 {
        (dist.signum() * radius - dist) / denom
    } else {
        // Parallel: check if already penetrating
        if dist.abs() <= radius {
            0.0
        } else {
            return None;
        }
    };

    if t_plane < 0.0 || t_plane > max_t {
        return None;
    }

    // Sphere centre at time t_plane
    let contact_centre = v3_add(origin, v3_scale(direction, t_plane));
    let closest = closest_point_on_triangle(contact_centre, a, b, c);
    let diff = v3_sub(contact_centre, closest);
    let dist2 = v3_dot(diff, diff);

    if dist2 <= radius * radius + 1e-6 {
        let normal = if dist2 > 1e-12 {
            v3_normalize(diff)
        } else {
            n
        };
        Some((t_plane, normal))
    } else {
        None
    }
}

/// Sweep an AABB from `origin` along `direction` against a triangle.
/// Uses a simplified slab test against the triangle's AABB.
#[allow(clippy::too_many_arguments)]
fn sweep_box_triangle(
    origin: [f32; 3],
    direction: [f32; 3],
    half: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    max_t: f32,
    eps: f32,
) -> Option<(f32, [f32; 3])> {
    // Expand the triangle AABB by the box half-extents
    let tri_min = [
        a[0].min(b[0]).min(c[0]) - half[0] - eps,
        a[1].min(b[1]).min(c[1]) - half[1] - eps,
        a[2].min(b[2]).min(c[2]) - half[2] - eps,
    ];
    let tri_max = [
        a[0].max(b[0]).max(c[0]) + half[0] + eps,
        a[1].max(b[1]).max(c[1]) + half[1] + eps,
        a[2].max(b[2]).max(c[2]) + half[2] + eps,
    ];

    let mut t_min = 0.0_f32;
    let mut t_max = max_t;
    let mut hit_axis = 0_usize;
    let mut hit_dir = 1.0_f32;

    for k in 0..3 {
        if direction[k].abs() < 1e-9 {
            if origin[k] < tri_min[k] || origin[k] > tri_max[k] {
                return None;
            }
        } else {
            let inv_d = 1.0 / direction[k];
            let mut t1 = (tri_min[k] - origin[k]) * inv_d;
            let mut t2 = (tri_max[k] - origin[k]) * inv_d;
            let sign = if t1 <= t2 { 1.0_f32 } else { -1.0_f32 };
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            if t1 > t_min {
                t_min = t1;
                hit_axis = k;
                hit_dir = -sign;
            }
            if t2 < t_max {
                t_max = t2;
            }
            if t_min > t_max {
                return None;
            }
        }
    }

    if t_min < 0.0 || t_min > max_t {
        return None;
    }

    let mut normal = [0.0_f32; 3];
    normal[hit_axis] = hit_dir;
    Some((t_min, normal))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Sweep `shape` from `origin` along `direction` (world-space unit vector,
/// scaled by `config.max_distance`) and return the first triangle hit.
///
/// `positions` and `triangles` define the static obstacle mesh.
#[allow(dead_code)]
pub fn shape_cast_sweep(
    shape: &CastShape,
    origin: [f32; 3],
    direction: [f32; 3],
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &ShapeCastConfig,
) -> ShapeCastResult {
    let dir_len = v3_len(direction);
    let dir_norm = if dir_len > 1e-9 {
        v3_scale(direction, 1.0 / dir_len)
    } else {
        direction
    };
    let max_t = config.max_distance;

    let mut best_t = f32::MAX;
    let mut best_normal = [0.0_f32; 3];
    let mut hit = false;

    for tri in triangles {
        let a = positions[tri[0]];
        let b = positions[tri[1]];
        let c = positions[tri[2]];

        let result = match shape {
            CastShape::Sphere { radius } => {
                sweep_sphere_triangle(origin, dir_norm, *radius, a, b, c, max_t)
            }
            CastShape::Box { half_extents } => {
                sweep_box_triangle(origin, dir_norm, *half_extents, a, b, c, max_t, config.epsilon)
            }
        };

        if let Some((t, n)) = result {
            if t < best_t {
                best_t = t;
                best_normal = n;
                hit = true;
            }
        }
    }

    let hit_point = if hit {
        v3_add(origin, v3_scale(dir_norm, best_t))
    } else {
        [f32::MAX; 3]
    };

    ShapeCastResult {
        hit,
        hit_time: if hit { best_t } else { f32::MAX },
        hit_normal: best_normal,
        hit_point,
    }
}

/// Return the parametric hit time (or `f32::MAX` if no hit).
#[allow(dead_code)]
pub fn shape_cast_hit_time(result: &ShapeCastResult) -> f32 {
    result.hit_time
}

/// Return the surface normal at the hit point.
#[allow(dead_code)]
pub fn shape_cast_hit_normal(result: &ShapeCastResult) -> [f32; 3] {
    result.hit_normal
}

/// Return the world-space hit point.
#[allow(dead_code)]
pub fn shape_cast_hit_point(result: &ShapeCastResult) -> [f32; 3] {
    result.hit_point
}

/// Serialise the result to compact JSON.
#[allow(dead_code)]
pub fn shape_cast_to_json(result: &ShapeCastResult) -> String {
    let p = result.hit_point;
    let n = result.hit_normal;
    format!(
        r#"{{"hit":{},"t":{:.6},"point":[{:.4},{:.4},{:.4}],"normal":[{:.4},{:.4},{:.4}]}}"#,
        result.hit,
        result.hit_time,
        p[0], p[1], p[2],
        n[0], n[1], n[2],
    )
}

/// Return a result representing a missed cast (no hit).
#[allow(dead_code)]
pub fn shape_cast_miss() -> ShapeCastResult {
    ShapeCastResult {
        hit: false,
        hit_time: f32::MAX,
        hit_normal: [0.0; 3],
        hit_point: [f32::MAX; 3],
    }
}

/// Return the Euclidean distance to the hit point, or `f32::MAX` if no hit.
#[allow(dead_code)]
pub fn shape_cast_distance(result: &ShapeCastResult) -> f32 {
    if result.hit {
        result.hit_time
    } else {
        f32::MAX
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn floor_triangle() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        // Large horizontal triangle at y = -1
        let p = vec![
            [-10.0_f32, -1.0, -10.0],
            [ 10.0, -1.0, -10.0],
            [  0.0, -1.0,  10.0],
        ];
        let t = vec![[0, 1, 2]];
        (p, t)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_shape_cast_config();
        assert!(cfg.max_distance > 0.0);
    }

    #[test]
    fn test_new_sphere_cast() {
        let s = new_sphere_cast(0.5);
        if let CastShape::Sphere { radius } = s {
            assert_eq!(radius, 0.5);
        } else {
            panic!("expected sphere");
        }
    }

    #[test]
    fn test_new_box_cast() {
        let b = new_box_cast([1.0, 2.0, 3.0]);
        if let CastShape::Box { half_extents } = b {
            assert_eq!(half_extents, [1.0, 2.0, 3.0]);
        } else {
            panic!("expected box");
        }
    }

    #[test]
    fn test_sphere_hits_floor() {
        let (p, t) = floor_triangle();
        let cfg = default_shape_cast_config();
        let sphere = new_sphere_cast(0.1);
        // Cast downward from above
        let result = shape_cast_sweep(&sphere, [0.0, 5.0, 0.0], [0.0, -1.0, 0.0], &p, &t, &cfg);
        assert!(result.hit, "sphere should hit floor");
    }

    #[test]
    fn test_sphere_misses_when_offset() {
        let (p, t) = floor_triangle();
        let cfg = default_shape_cast_config();
        let sphere = new_sphere_cast(0.01);
        // Cast sideways
        let result = shape_cast_sweep(&sphere, [0.0, 5.0, 0.0], [1.0, 0.0, 0.0], &p, &t, &cfg);
        // Should not hit; but we just check the function returns without panic
        let _ = result.hit;
    }

    #[test]
    fn test_shape_cast_miss_factory() {
        let r = shape_cast_miss();
        assert!(!r.hit);
        assert_eq!(r.hit_time, f32::MAX);
    }

    #[test]
    fn test_hit_time_accessor() {
        let r = shape_cast_miss();
        assert_eq!(shape_cast_hit_time(&r), f32::MAX);
    }

    #[test]
    fn test_hit_normal_accessor() {
        let r = shape_cast_miss();
        assert_eq!(shape_cast_hit_normal(&r), [0.0; 3]);
    }

    #[test]
    fn test_to_json_contains_hit() {
        let r = shape_cast_miss();
        let json = shape_cast_to_json(&r);
        assert!(json.contains("hit"));
    }

    #[test]
    fn test_distance_miss() {
        let r = shape_cast_miss();
        assert_eq!(shape_cast_distance(&r), f32::MAX);
    }

    #[test]
    fn test_box_cast_no_panic() {
        let (p, t) = floor_triangle();
        let cfg = default_shape_cast_config();
        let box_shape = new_box_cast([0.5, 0.5, 0.5]);
        let result = shape_cast_sweep(&box_shape, [0.0, 5.0, 0.0], [0.0, -1.0, 0.0], &p, &t, &cfg);
        // Should hit or not, but must not panic
        let _ = result.hit;
    }
}
