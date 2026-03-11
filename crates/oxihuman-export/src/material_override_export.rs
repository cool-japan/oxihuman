// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Material override export for scene material property overrides.

/// A material property override.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialOverride {
    pub material_name: String,
    pub property: String,
    pub value: OverrideValue,
}

/// Override value types.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum OverrideValue {
    Float(f32),
    Color([f32; 4]),
    Text(String),
}

/// Collection of overrides.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialOverrideExport {
    pub overrides: Vec<MaterialOverride>,
}

/// Create new export.
#[allow(dead_code)]
pub fn new_material_override_export() -> MaterialOverrideExport {
    MaterialOverrideExport { overrides: vec![] }
}

/// Add float override.
#[allow(dead_code)]
pub fn add_float_override(e: &mut MaterialOverrideExport, mat: &str, prop: &str, val: f32) {
    e.overrides.push(MaterialOverride {
        material_name: mat.to_string(),
        property: prop.to_string(),
        value: OverrideValue::Float(val),
    });
}

/// Add color override.
#[allow(dead_code)]
pub fn add_color_override(e: &mut MaterialOverrideExport, mat: &str, prop: &str, color: [f32; 4]) {
    e.overrides.push(MaterialOverride {
        material_name: mat.to_string(),
        property: prop.to_string(),
        value: OverrideValue::Color(color),
    });
}

/// Add text override.
#[allow(dead_code)]
pub fn add_text_override(e: &mut MaterialOverrideExport, mat: &str, prop: &str, text: &str) {
    e.overrides.push(MaterialOverride {
        material_name: mat.to_string(),
        property: prop.to_string(),
        value: OverrideValue::Text(text.to_string()),
    });
}

/// Override count.
#[allow(dead_code)]
pub fn mo_count(e: &MaterialOverrideExport) -> usize {
    e.overrides.len()
}

/// Get overrides for a material.
#[allow(dead_code)]
pub fn overrides_for_material<'a>(
    e: &'a MaterialOverrideExport,
    name: &str,
) -> Vec<&'a MaterialOverride> {
    e.overrides
        .iter()
        .filter(|o| o.material_name == name)
        .collect()
}

/// Validate.
#[allow(dead_code)]
pub fn mo_validate(e: &MaterialOverrideExport) -> bool {
    e.overrides
        .iter()
        .all(|o| !o.material_name.is_empty() && !o.property.is_empty())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn material_override_to_json(e: &MaterialOverrideExport) -> String {
    format!("{{\"override_count\":{}}}", mo_count(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_material_override_export();
        assert_eq!(mo_count(&e), 0);
    }
    #[test]
    fn test_add_float() {
        let mut e = new_material_override_export();
        add_float_override(&mut e, "skin", "roughness", 0.5);
        assert_eq!(mo_count(&e), 1);
    }
    #[test]
    fn test_add_color() {
        let mut e = new_material_override_export();
        add_color_override(&mut e, "skin", "baseColor", [1.0; 4]);
        assert_eq!(mo_count(&e), 1);
    }
    #[test]
    fn test_add_text() {
        let mut e = new_material_override_export();
        add_text_override(&mut e, "skin", "texture", "path.png");
        assert_eq!(mo_count(&e), 1);
    }
    #[test]
    fn test_filter() {
        let mut e = new_material_override_export();
        add_float_override(&mut e, "skin", "a", 1.0);
        add_float_override(&mut e, "hair", "b", 2.0);
        assert_eq!(overrides_for_material(&e, "skin").len(), 1);
    }
    #[test]
    fn test_filter_empty() {
        let e = new_material_override_export();
        assert!(overrides_for_material(&e, "x").is_empty());
    }
    #[test]
    fn test_validate() {
        let mut e = new_material_override_export();
        add_float_override(&mut e, "a", "b", 1.0);
        assert!(mo_validate(&e));
    }
    #[test]
    fn test_validate_bad() {
        let e = MaterialOverrideExport {
            overrides: vec![MaterialOverride {
                material_name: String::new(),
                property: "a".to_string(),
                value: OverrideValue::Float(0.0),
            }],
        };
        assert!(!mo_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_material_override_export();
        assert!(material_override_to_json(&e).contains("\"override_count\":0"));
    }
    #[test]
    fn test_value_types() {
        let v = OverrideValue::Float(1.0);
        assert!(matches!(v, OverrideValue::Float(_)));
    }
    #[test]
    fn test_text_value() {
        let v = OverrideValue::Text("hello".to_string());
        assert!(matches!(v, OverrideValue::Text(_)));
    }
}
