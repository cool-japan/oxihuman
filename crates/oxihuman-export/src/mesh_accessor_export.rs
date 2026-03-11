#![allow(dead_code)]
//! Mesh accessor export.

/// Accessor export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshAccessorExport {
    pub component_type: u32,
    pub type_name: String,
    pub count: u32,
    pub min_values: Vec<f32>,
    pub max_values: Vec<f32>,
}

/// Export an accessor.
#[allow(dead_code)]
pub fn export_accessor(
    component_type: u32,
    type_name: &str,
    count: u32,
    min_values: Vec<f32>,
    max_values: Vec<f32>,
) -> MeshAccessorExport {
    MeshAccessorExport {
        component_type,
        type_name: type_name.to_string(),
        count,
        min_values,
        max_values,
    }
}

/// Get component type (5126 = FLOAT, 5123 = UNSIGNED_SHORT, etc).
#[allow(dead_code)]
pub fn accessor_component_type(e: &MeshAccessorExport) -> u32 {
    e.component_type
}

/// Get type name (VEC3, VEC4, SCALAR, etc).
#[allow(dead_code)]
pub fn accessor_type_name(e: &MeshAccessorExport) -> &str {
    &e.type_name
}

/// Get element count.
#[allow(dead_code)]
pub fn accessor_count_mae(e: &MeshAccessorExport) -> u32 {
    e.count
}

/// Get min values.
#[allow(dead_code)]
pub fn accessor_min(e: &MeshAccessorExport) -> &[f32] {
    &e.min_values
}

/// Get max values.
#[allow(dead_code)]
pub fn accessor_max(e: &MeshAccessorExport) -> &[f32] {
    &e.max_values
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn accessor_to_json(e: &MeshAccessorExport) -> String {
    format!(
        "{{\"componentType\":{},\"type\":\"{}\",\"count\":{}}}",
        e.component_type, e.type_name, e.count
    )
}

/// Validate accessor.
#[allow(dead_code)]
pub fn validate_accessor(e: &MeshAccessorExport) -> bool {
    e.count > 0
        && !e.type_name.is_empty()
        && e.min_values.len() == e.max_values.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_accessor() {
        let e = export_accessor(5126, "VEC3", 100, vec![0.0; 3], vec![1.0; 3]);
        assert_eq!(e.count, 100);
    }

    #[test]
    fn test_accessor_component_type() {
        let e = export_accessor(5126, "VEC3", 10, vec![], vec![]);
        assert_eq!(accessor_component_type(&e), 5126);
    }

    #[test]
    fn test_accessor_type_name() {
        let e = export_accessor(5126, "VEC4", 10, vec![], vec![]);
        assert_eq!(accessor_type_name(&e), "VEC4");
    }

    #[test]
    fn test_accessor_count() {
        let e = export_accessor(5126, "SCALAR", 50, vec![], vec![]);
        assert_eq!(accessor_count_mae(&e), 50);
    }

    #[test]
    fn test_accessor_min() {
        let e = export_accessor(5126, "VEC3", 10, vec![-1.0, -2.0, -3.0], vec![1.0; 3]);
        assert_eq!(accessor_min(&e), &[-1.0, -2.0, -3.0]);
    }

    #[test]
    fn test_accessor_max() {
        let e = export_accessor(5126, "VEC3", 10, vec![0.0; 3], vec![1.0, 2.0, 3.0]);
        assert_eq!(accessor_max(&e), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_accessor_to_json() {
        let e = export_accessor(5126, "VEC3", 10, vec![], vec![]);
        let j = accessor_to_json(&e);
        assert!(j.contains("VEC3"));
    }

    #[test]
    fn test_validate_ok() {
        let e = export_accessor(5126, "VEC3", 10, vec![0.0; 3], vec![1.0; 3]);
        assert!(validate_accessor(&e));
    }

    #[test]
    fn test_validate_zero_count() {
        let e = export_accessor(5126, "VEC3", 0, vec![], vec![]);
        assert!(!validate_accessor(&e));
    }

    #[test]
    fn test_validate_mismatch_minmax() {
        let e = export_accessor(5126, "VEC3", 10, vec![0.0; 3], vec![1.0; 2]);
        assert!(!validate_accessor(&e));
    }
}
