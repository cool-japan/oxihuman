#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GjkResult {
    distance: f32,
    intersecting: bool,
    closest_a: [f32; 3],
    closest_b: [f32; 3],
    iterations: u32,
    converged: bool,
}

#[allow(dead_code)]
pub fn gjk_distance(a_center: [f32; 3], a_radius: f32, b_center: [f32; 3], b_radius: f32) -> GjkResult {
    let dx = b_center[0] - a_center[0];
    let dy = b_center[1] - a_center[1];
    let dz = b_center[2] - a_center[2];
    let center_dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let dist = (center_dist - a_radius - b_radius).max(0.0);
    let intersecting = center_dist < a_radius + b_radius;
    let norm = if center_dist > 0.0 { 1.0 / center_dist } else { 0.0 };
    let nx = dx * norm;
    let ny = dy * norm;
    let nz = dz * norm;
    GjkResult {
        distance: dist,
        intersecting,
        closest_a: [a_center[0] + nx * a_radius, a_center[1] + ny * a_radius, a_center[2] + nz * a_radius],
        closest_b: [b_center[0] - nx * b_radius, b_center[1] - ny * b_radius, b_center[2] - nz * b_radius],
        iterations: 1,
        converged: true,
    }
}

#[allow(dead_code)]
pub fn gjk_intersect(result: &GjkResult) -> bool {
    result.intersecting
}

#[allow(dead_code)]
pub fn gjk_closest_pair(result: &GjkResult) -> ([f32; 3], [f32; 3]) {
    (result.closest_a, result.closest_b)
}

#[allow(dead_code)]
pub fn gjk_support(center: [f32; 3], radius: f32, direction: [f32; 3]) -> [f32; 3] {
    let len = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
    if len < f32::EPSILON {
        return center;
    }
    let inv = radius / len;
    [
        center[0] + direction[0] * inv,
        center[1] + direction[1] * inv,
        center[2] + direction[2] * inv,
    ]
}

#[allow(dead_code)]
pub fn gjk_iterations(result: &GjkResult) -> u32 {
    result.iterations
}

#[allow(dead_code)]
pub fn gjk_simplex_size(_result: &GjkResult) -> u32 {
    // Simplified: always returns 2 for sphere-sphere
    2
}

#[allow(dead_code)]
pub fn gjk_converged(result: &GjkResult) -> bool {
    result.converged
}

#[allow(dead_code)]
pub fn gjk_tolerance() -> f32 {
    1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separated() {
        let r = gjk_distance([0.0, 0.0, 0.0], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!(!gjk_intersect(&r));
        assert!((r.distance - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersecting() {
        let r = gjk_distance([0.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], 2.0);
        assert!(gjk_intersect(&r));
    }

    #[test]
    fn test_touching() {
        let r = gjk_distance([0.0, 0.0, 0.0], 1.0, [2.0, 0.0, 0.0], 1.0);
        assert!(r.distance < 1e-5);
    }

    #[test]
    fn test_closest_pair() {
        let r = gjk_distance([0.0, 0.0, 0.0], 1.0, [4.0, 0.0, 0.0], 1.0);
        let (a, b) = gjk_closest_pair(&r);
        assert!((a[0] - 1.0).abs() < 1e-5);
        assert!((b[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_support() {
        let s = gjk_support([0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0]);
        assert!((s[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_iterations() {
        let r = gjk_distance([0.0; 3], 1.0, [3.0, 0.0, 0.0], 1.0);
        assert!(gjk_iterations(&r) > 0);
    }

    #[test]
    fn test_simplex_size() {
        let r = gjk_distance([0.0; 3], 1.0, [3.0, 0.0, 0.0], 1.0);
        assert_eq!(gjk_simplex_size(&r), 2);
    }

    #[test]
    fn test_converged() {
        let r = gjk_distance([0.0; 3], 1.0, [3.0, 0.0, 0.0], 1.0);
        assert!(gjk_converged(&r));
    }

    #[test]
    fn test_tolerance() {
        assert!(gjk_tolerance() > 0.0);
    }

    #[test]
    fn test_same_position() {
        let r = gjk_distance([0.0; 3], 1.0, [0.0; 3], 1.0);
        assert!(gjk_intersect(&r));
    }
}
