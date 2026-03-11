#![allow(dead_code)]
//! Export physics collision shapes.

/// Physics shape export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsShapeExport {
    pub shape_type: String,
    pub vertices: Vec<[f32; 3]>,
    pub is_convex: bool,
}

/// Export a physics shape.
#[allow(dead_code)]
pub fn export_physics_shape(
    shape_type: &str,
    vertices: &[[f32; 3]],
    convex: bool,
) -> PhysicsShapeExport {
    PhysicsShapeExport {
        shape_type: shape_type.to_string(),
        vertices: vertices.to_vec(),
        is_convex: convex,
    }
}

/// Return shape type.
#[allow(dead_code)]
pub fn shape_type_export(exp: &PhysicsShapeExport) -> &str {
    &exp.shape_type
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn shape_to_json(exp: &PhysicsShapeExport) -> String {
    format!(
        "{{\"type\":\"{}\",\"vertices\":{},\"convex\":{}}}",
        exp.shape_type,
        exp.vertices.len(),
        exp.is_convex
    )
}

/// Serialize to bytes.
#[allow(dead_code)]
pub fn shape_to_bytes(exp: &PhysicsShapeExport) -> Vec<u8> {
    let mut buf = Vec::new();
    for v in &exp.vertices {
        for &f in v {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

/// Return vertex count.
#[allow(dead_code)]
pub fn shape_vertex_count(exp: &PhysicsShapeExport) -> usize {
    exp.vertices.len()
}

/// Check if convex.
#[allow(dead_code)]
pub fn shape_is_convex_export(exp: &PhysicsShapeExport) -> bool {
    exp.is_convex
}

/// Compute export size.
#[allow(dead_code)]
pub fn shape_export_size(exp: &PhysicsShapeExport) -> usize {
    exp.vertices.len() * 12
}

/// Validate physics shape.
#[allow(dead_code)]
pub fn validate_physics_shape(exp: &PhysicsShapeExport) -> bool {
    !exp.shape_type.is_empty()
        && !exp.vertices.is_empty()
        && exp.vertices.iter().all(|v| v.iter().all(|f| f.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_physics_shape() {
        let e = export_physics_shape("box", &[[0.0; 3], [1.0; 3]], true);
        assert_eq!(shape_vertex_count(&e), 2);
    }

    #[test]
    fn test_shape_type() {
        let e = export_physics_shape("sphere", &[[0.0; 3]], true);
        assert_eq!(shape_type_export(&e), "sphere");
    }

    #[test]
    fn test_shape_to_json() {
        let e = export_physics_shape("mesh", &[[0.0; 3]], false);
        let j = shape_to_json(&e);
        assert!(j.contains("\"type\":\"mesh\""));
    }

    #[test]
    fn test_shape_to_bytes() {
        let e = export_physics_shape("box", &[[1.0, 2.0, 3.0]], true);
        assert_eq!(shape_to_bytes(&e).len(), 12);
    }

    #[test]
    fn test_shape_is_convex() {
        let e = export_physics_shape("hull", &[[0.0; 3]], true);
        assert!(shape_is_convex_export(&e));
    }

    #[test]
    fn test_shape_not_convex() {
        let e = export_physics_shape("mesh", &[[0.0; 3]], false);
        assert!(!shape_is_convex_export(&e));
    }

    #[test]
    fn test_shape_export_size() {
        let e = export_physics_shape("box", &[[0.0; 3]; 8], true);
        assert_eq!(shape_export_size(&e), 96);
    }

    #[test]
    fn test_validate_physics_shape() {
        let e = export_physics_shape("box", &[[0.0; 3]], true);
        assert!(validate_physics_shape(&e));
    }

    #[test]
    fn test_validate_empty_verts() {
        let e = export_physics_shape("box", &[], true);
        assert!(!validate_physics_shape(&e));
    }

    #[test]
    fn test_validate_empty_type() {
        let e = export_physics_shape("", &[[0.0; 3]], true);
        assert!(!validate_physics_shape(&e));
    }
}
