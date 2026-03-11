#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EpaResultDef {
    penetration_depth: f32,
    normal: [f32; 3],
    contact_point: [f32; 3],
    iterations: u32,
    converged: bool,
}

#[allow(dead_code)]
pub fn epa_penetration(a_center: [f32; 3], a_radius: f32, b_center: [f32; 3], b_radius: f32) -> EpaResultDef {
    let dx = b_center[0] - a_center[0];
    let dy = b_center[1] - a_center[1];
    let dz = b_center[2] - a_center[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let sum_r = a_radius + b_radius;
    let depth = (sum_r - dist).max(0.0);
    let inv = if dist > f32::EPSILON { 1.0 / dist } else { 0.0 };
    let normal = [dx * inv, dy * inv, dz * inv];
    let contact = [
        a_center[0] + normal[0] * a_radius,
        a_center[1] + normal[1] * a_radius,
        a_center[2] + normal[2] * a_radius,
    ];
    EpaResultDef {
        penetration_depth: depth,
        normal,
        contact_point: contact,
        iterations: 1,
        converged: true,
    }
}

#[allow(dead_code)]
pub fn epa_normal(result: &EpaResultDef) -> [f32; 3] {
    result.normal
}

#[allow(dead_code)]
pub fn epa_depth(result: &EpaResultDef) -> f32 {
    result.penetration_depth
}

#[allow(dead_code)]
pub fn epa_contact_point(result: &EpaResultDef) -> [f32; 3] {
    result.contact_point
}

#[allow(dead_code)]
pub fn epa_iterations_def(result: &EpaResultDef) -> u32 {
    result.iterations
}

#[allow(dead_code)]
pub fn epa_converged_def(result: &EpaResultDef) -> bool {
    result.converged
}

#[allow(dead_code)]
pub fn epa_tolerance_def() -> f32 {
    1e-6
}

#[allow(dead_code)]
pub fn epa_from_gjk(intersecting: bool, a_center: [f32; 3], a_radius: f32, b_center: [f32; 3], b_radius: f32) -> Option<EpaResultDef> {
    if intersecting {
        Some(epa_penetration(a_center, a_radius, b_center, b_radius))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_penetration() {
        let r = epa_penetration([0.0; 3], 2.0, [1.0, 0.0, 0.0], 2.0);
        assert!((epa_depth(&r) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_normal() {
        let r = epa_penetration([0.0; 3], 1.0, [1.0, 0.0, 0.0], 1.0);
        let n = epa_normal(&r);
        assert!((n[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_contact_point() {
        let r = epa_penetration([0.0; 3], 1.0, [3.0, 0.0, 0.0], 1.0);
        let cp = epa_contact_point(&r);
        assert!((cp[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_no_overlap() {
        let r = epa_penetration([0.0; 3], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert_eq!(epa_depth(&r), 0.0);
    }

    #[test]
    fn test_iterations() {
        let r = epa_penetration([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(epa_iterations_def(&r) > 0);
    }

    #[test]
    fn test_converged() {
        let r = epa_penetration([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(epa_converged_def(&r));
    }

    #[test]
    fn test_tolerance() {
        assert!(epa_tolerance_def() > 0.0);
    }

    #[test]
    fn test_from_gjk_intersecting() {
        let r = epa_from_gjk(true, [0.0; 3], 2.0, [1.0, 0.0, 0.0], 2.0);
        assert!(r.is_some());
    }

    #[test]
    fn test_from_gjk_not_intersecting() {
        let r = epa_from_gjk(false, [0.0; 3], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!(r.is_none());
    }

    #[test]
    fn test_coincident() {
        let r = epa_penetration([0.0; 3], 1.0, [0.0; 3], 1.0);
        assert!((epa_depth(&r) - 2.0).abs() < 1e-5);
    }
}
