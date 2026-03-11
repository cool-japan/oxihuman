// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Material/shader property export to JSON/glTF-compatible format.

/// A single material property.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialProperty {
    Float(f32),
    Color([f32; 4]),
    TexturePath(String),
}

/// A material that can be exported.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ExportMaterial {
    pub name: String,
    pub properties: Vec<(String, MaterialProperty)>,
}

/// Configuration for material export.
#[allow(dead_code)]
pub struct MaterialExportConfig {
    pub pretty_print: bool,
    pub include_defaults: bool,
    pub gltf_compatible: bool,
}

/// A bundle of materials for batch export.
#[allow(dead_code)]
pub struct MaterialExportBundle {
    pub materials: Vec<ExportMaterial>,
}

/// Type alias for property lookup result.
#[allow(dead_code)]
pub type PropertyLookup<'a> = Option<&'a MaterialProperty>;

/// Type alias for validation result.
#[allow(dead_code)]
pub type ValidationResult = Vec<String>;

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a default material export configuration.
#[allow(dead_code)]
pub fn default_material_export_config() -> MaterialExportConfig {
    MaterialExportConfig {
        pretty_print: true,
        include_defaults: false,
        gltf_compatible: true,
    }
}

/// Create a new empty export material.
#[allow(dead_code)]
pub fn new_export_material(name: &str) -> ExportMaterial {
    ExportMaterial {
        name: name.to_string(),
        properties: Vec::new(),
    }
}

/// Set a float property on a material.
#[allow(dead_code)]
pub fn set_property_float(mat: &mut ExportMaterial, key: &str, value: f32) {
    remove_property(mat, key);
    mat.properties
        .push((key.to_string(), MaterialProperty::Float(value)));
}

/// Set a color property (RGBA) on a material.
#[allow(dead_code)]
pub fn set_property_color(mat: &mut ExportMaterial, key: &str, rgba: [f32; 4]) {
    remove_property(mat, key);
    mat.properties
        .push((key.to_string(), MaterialProperty::Color(rgba)));
}

/// Set a texture path property on a material.
#[allow(dead_code)]
pub fn set_property_texture_path(mat: &mut ExportMaterial, key: &str, path: &str) {
    remove_property(mat, key);
    mat.properties.push((
        key.to_string(),
        MaterialProperty::TexturePath(path.to_string()),
    ));
}

/// Get a property by name.
#[allow(dead_code)]
pub fn get_property<'a>(mat: &'a ExportMaterial, key: &str) -> PropertyLookup<'a> {
    mat.properties
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

/// Serialize a material to a JSON string.
#[allow(dead_code)]
pub fn material_to_json(mat: &ExportMaterial) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"name\": \"{}\",\n", mat.name));
    out.push_str("  \"properties\": {\n");
    for (i, (k, v)) in mat.properties.iter().enumerate() {
        let comma = if i + 1 < mat.properties.len() {
            ","
        } else {
            ""
        };
        match v {
            MaterialProperty::Float(f) => {
                out.push_str(&format!("    \"{k}\": {f:.6}{comma}\n"));
            }
            MaterialProperty::Color(c) => {
                out.push_str(&format!(
                    "    \"{k}\": [{:.4}, {:.4}, {:.4}, {:.4}]{comma}\n",
                    c[0], c[1], c[2], c[3]
                ));
            }
            MaterialProperty::TexturePath(p) => {
                out.push_str(&format!("    \"{k}\": \"{p}\"{comma}\n"));
            }
        }
    }
    out.push_str("  }\n}");
    out
}

/// Serialize a material to glTF-compatible PBR JSON.
#[allow(dead_code)]
pub fn material_to_gltf_json(mat: &ExportMaterial) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"name\": \"{}\",\n", mat.name));
    out.push_str("  \"pbrMetallicRoughness\": {\n");

    let base_color = mat
        .properties
        .iter()
        .find(|(k, _)| k == "baseColor")
        .and_then(|(_, v)| match v {
            MaterialProperty::Color(c) => Some(*c),
            _ => None,
        })
        .unwrap_or([1.0, 1.0, 1.0, 1.0]);

    let metallic = mat
        .properties
        .iter()
        .find(|(k, _)| k == "metallic")
        .and_then(|(_, v)| match v {
            MaterialProperty::Float(f) => Some(*f),
            _ => None,
        })
        .unwrap_or(0.0);

    let roughness = mat
        .properties
        .iter()
        .find(|(k, _)| k == "roughness")
        .and_then(|(_, v)| match v {
            MaterialProperty::Float(f) => Some(*f),
            _ => None,
        })
        .unwrap_or(0.5);

    out.push_str(&format!(
        "    \"baseColorFactor\": [{:.4}, {:.4}, {:.4}, {:.4}],\n",
        base_color[0], base_color[1], base_color[2], base_color[3]
    ));
    out.push_str(&format!("    \"metallicFactor\": {metallic:.4},\n"));
    out.push_str(&format!("    \"roughnessFactor\": {roughness:.4}\n"));
    out.push_str("  }\n}");
    out
}

/// Return the number of materials in a bundle.
#[allow(dead_code)]
pub fn material_count(bundle: &MaterialExportBundle) -> usize {
    bundle.materials.len()
}

/// Add a material to an export bundle.
#[allow(dead_code)]
pub fn add_material_to_bundle(bundle: &mut MaterialExportBundle, mat: ExportMaterial) {
    bundle.materials.push(mat);
}

/// Return the number of properties on a material.
#[allow(dead_code)]
pub fn material_property_count(mat: &ExportMaterial) -> usize {
    mat.properties.len()
}

/// Validate a material. Returns a list of warnings (empty means valid).
#[allow(dead_code)]
pub fn validate_material(mat: &ExportMaterial) -> ValidationResult {
    let mut warnings = Vec::new();
    if mat.name.is_empty() {
        warnings.push("Material name is empty".to_string());
    }
    for (k, v) in &mat.properties {
        if k.is_empty() {
            warnings.push("Empty property key".to_string());
        }
        if let MaterialProperty::Float(f) = v {
            if f.is_nan() || f.is_infinite() {
                warnings.push(format!("Property '{k}' has non-finite value"));
            }
        }
        if let MaterialProperty::Color(c) = v {
            for (ci, &ch) in c.iter().enumerate() {
                if ch.is_nan() || ch.is_infinite() {
                    warnings.push(format!("Property '{k}' color channel {ci} is non-finite"));
                }
            }
        }
    }
    warnings
}

/// Create a default PBR material with standard properties.
#[allow(dead_code)]
pub fn default_pbr_material(name: &str) -> ExportMaterial {
    let mut mat = new_export_material(name);
    set_property_color(&mut mat, "baseColor", [0.8, 0.8, 0.8, 1.0]);
    set_property_float(&mut mat, "metallic", 0.0);
    set_property_float(&mut mat, "roughness", 0.5);
    set_property_float(&mut mat, "emissive", 0.0);
    mat
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn remove_property(mat: &mut ExportMaterial, key: &str) {
    mat.properties.retain(|(k, _)| k != key);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_material_export_config();
        assert!(cfg.pretty_print);
        assert!(!cfg.include_defaults);
        assert!(cfg.gltf_compatible);
    }

    #[test]
    fn new_material_empty() {
        let mat = new_export_material("Skin");
        assert_eq!(mat.name, "Skin");
        assert!(mat.properties.is_empty());
    }

    #[test]
    fn set_get_float() {
        let mut mat = new_export_material("M");
        set_property_float(&mut mat, "roughness", 0.7);
        match get_property(&mat, "roughness") {
            Some(MaterialProperty::Float(f)) => assert!((*f - 0.7).abs() < 1e-6),
            _ => panic!("expected float property"),
        }
    }

    #[test]
    fn set_get_color() {
        let mut mat = new_export_material("M");
        set_property_color(&mut mat, "baseColor", [1.0, 0.5, 0.0, 1.0]);
        match get_property(&mat, "baseColor") {
            Some(MaterialProperty::Color(c)) => {
                assert!((c[0] - 1.0).abs() < 1e-6);
                assert!((c[1] - 0.5).abs() < 1e-6);
            }
            _ => panic!("expected color property"),
        }
    }

    #[test]
    fn set_get_texture() {
        let mut mat = new_export_material("M");
        set_property_texture_path(&mut mat, "diffuseMap", "/tex/diffuse.png");
        match get_property(&mat, "diffuseMap") {
            Some(MaterialProperty::TexturePath(p)) => assert_eq!(p, "/tex/diffuse.png"),
            _ => panic!("expected texture property"),
        }
    }

    #[test]
    fn get_missing_property() {
        let mat = new_export_material("M");
        assert!(get_property(&mat, "nonexistent").is_none());
    }

    #[test]
    fn overwrite_property() {
        let mut mat = new_export_material("M");
        set_property_float(&mut mat, "roughness", 0.5);
        set_property_float(&mut mat, "roughness", 0.9);
        assert_eq!(material_property_count(&mat), 1);
        match get_property(&mat, "roughness") {
            Some(MaterialProperty::Float(f)) => assert!((*f - 0.9).abs() < 1e-6),
            _ => panic!("expected float"),
        }
    }

    #[test]
    fn material_to_json_contains_name() {
        let mat = new_export_material("Skin");
        let json = material_to_json(&mat);
        assert!(json.contains("\"name\": \"Skin\""));
    }

    #[test]
    fn material_to_gltf_json_contains_pbr() {
        let mat = default_pbr_material("Default");
        let json = material_to_gltf_json(&mat);
        assert!(json.contains("pbrMetallicRoughness"));
        assert!(json.contains("baseColorFactor"));
        assert!(json.contains("metallicFactor"));
    }

    #[test]
    fn bundle_count() {
        let mut bundle = MaterialExportBundle {
            materials: Vec::new(),
        };
        assert_eq!(material_count(&bundle), 0);
        add_material_to_bundle(&mut bundle, new_export_material("A"));
        add_material_to_bundle(&mut bundle, new_export_material("B"));
        assert_eq!(material_count(&bundle), 2);
    }

    #[test]
    fn property_count() {
        let mat = default_pbr_material("P");
        assert_eq!(material_property_count(&mat), 4);
    }

    #[test]
    fn validate_valid_material() {
        let mat = default_pbr_material("Valid");
        let warnings = validate_material(&mat);
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_empty_name() {
        let mat = new_export_material("");
        let warnings = validate_material(&mat);
        assert!(warnings.iter().any(|w| w.contains("name is empty")));
    }

    #[test]
    fn validate_nan_float() {
        let mut mat = new_export_material("Bad");
        set_property_float(&mut mat, "roughness", f32::NAN);
        let warnings = validate_material(&mat);
        assert!(warnings.iter().any(|w| w.contains("non-finite")));
    }

    #[test]
    fn default_pbr_has_expected_properties() {
        let mat = default_pbr_material("Std");
        assert!(get_property(&mat, "baseColor").is_some());
        assert!(get_property(&mat, "metallic").is_some());
        assert!(get_property(&mat, "roughness").is_some());
        assert!(get_property(&mat, "emissive").is_some());
    }

    #[test]
    fn gltf_json_default_metallic_zero() {
        let mut mat = new_export_material("M");
        set_property_color(&mut mat, "baseColor", [1.0, 1.0, 1.0, 1.0]);
        let json = material_to_gltf_json(&mat);
        assert!(json.contains("\"metallicFactor\": 0.0000"));
    }
}
