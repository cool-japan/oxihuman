// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GLTF 2.0 material extension support (KHR extensions as JSON).
//!
//! Produces `serde_json::Value` objects that can be embedded in GLTF material
//! `extensions` objects.  All KHR extension names follow the GLTF 2.0 spec.

#![allow(dead_code)]

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// KHR_materials_unlit
// ---------------------------------------------------------------------------

/// Return the JSON value for the `KHR_materials_unlit` extension object.
///
/// The extension object is empty (`{}`); its presence alone signals unlit
/// rendering.
pub fn khr_materials_unlit() -> Value {
    json!({})
}

// ---------------------------------------------------------------------------
// KHR_materials_emissive_strength
// ---------------------------------------------------------------------------

/// Return the JSON value for the `KHR_materials_emissive_strength` extension.
///
/// `emissive_strength` multiplies the emissive colour; values > 1.0 allow
/// HDR emissive outputs.
pub fn khr_materials_emissive_strength(emissive_strength: f32) -> Value {
    json!({ "emissiveStrength": emissive_strength })
}

// ---------------------------------------------------------------------------
// KHR_materials_clearcoat
// ---------------------------------------------------------------------------

/// Parameters for the `KHR_materials_clearcoat` extension.
#[derive(Debug, Clone, PartialEq)]
pub struct ClearcoatExt {
    /// Clear-coat layer intensity (default `0.0`).
    pub clearcoat_factor: f32,
    /// Clear-coat layer roughness (default `0.0`).
    pub clearcoat_roughness_factor: f32,
}

impl Default for ClearcoatExt {
    fn default() -> Self {
        Self {
            clearcoat_factor: 0.0,
            clearcoat_roughness_factor: 0.0,
        }
    }
}

/// Return the JSON value for the `KHR_materials_clearcoat` extension.
pub fn khr_materials_clearcoat(params: &ClearcoatExt) -> Value {
    json!({
        "clearcoatFactor":          params.clearcoat_factor,
        "clearcoatRoughnessFactor": params.clearcoat_roughness_factor,
    })
}

// ---------------------------------------------------------------------------
// KHR_materials_sheen
// ---------------------------------------------------------------------------

/// Parameters for the `KHR_materials_sheen` extension.
#[derive(Debug, Clone, PartialEq)]
pub struct SheenExt {
    /// RGB sheen colour factor (default `[0, 0, 0]`).
    pub sheen_color_factor: [f32; 3],
    /// Sheen roughness (default `0.0`).
    pub sheen_roughness_factor: f32,
}

impl Default for SheenExt {
    fn default() -> Self {
        Self {
            sheen_color_factor: [0.0, 0.0, 0.0],
            sheen_roughness_factor: 0.0,
        }
    }
}

/// Return the JSON value for the `KHR_materials_sheen` extension.
pub fn khr_materials_sheen(params: &SheenExt) -> Value {
    let [r, g, b] = params.sheen_color_factor;
    json!({
        "sheenColorFactor":     [r, g, b],
        "sheenRoughnessFactor": params.sheen_roughness_factor,
    })
}

// ---------------------------------------------------------------------------
// KHR_materials_transmission
// ---------------------------------------------------------------------------

/// Return the JSON value for the `KHR_materials_transmission` extension.
pub fn khr_materials_transmission(transmission_factor: f32) -> Value {
    json!({ "transmissionFactor": transmission_factor })
}

// ---------------------------------------------------------------------------
// KHR_materials_volume
// ---------------------------------------------------------------------------

/// Parameters for the `KHR_materials_volume` extension.
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeExt {
    /// Thickness of the volume in meters (default `0.0`).
    pub thickness_factor: f32,
    /// Distance at which the attenuation colour becomes dominant
    /// (default `f32::INFINITY`).
    pub attenuation_distance: f32,
    /// Colour of the medium when `attenuation_distance` is reached
    /// (default `[1, 1, 1]`).
    pub attenuation_color: [f32; 3],
}

impl Default for VolumeExt {
    fn default() -> Self {
        Self {
            thickness_factor: 0.0,
            attenuation_distance: f32::INFINITY,
            attenuation_color: [1.0, 1.0, 1.0],
        }
    }
}

/// Return the JSON value for the `KHR_materials_volume` extension.
///
/// `f32::INFINITY` is serialised as the JSON number that best approximates it.
/// Most runtimes accept `1.7976931348623157e308` (f64::MAX) as a stand-in for
/// infinity when the spec says "a very large number".  We use `f64::MAX` here
/// to remain well-formed JSON.
pub fn khr_materials_volume(params: &VolumeExt) -> Value {
    let attn_dist = if params.attenuation_distance.is_infinite() {
        f64::MAX
    } else {
        f64::from(params.attenuation_distance)
    };
    let [r, g, b] = params.attenuation_color;
    json!({
        "thicknessFactor":      params.thickness_factor,
        "attenuationDistance":  attn_dist,
        "attenuationColor":     [r, g, b],
    })
}

// ---------------------------------------------------------------------------
// KHR_materials_ior
// ---------------------------------------------------------------------------

/// Return the JSON value for the `KHR_materials_ior` extension.
///
/// `ior` is the index of refraction (default `1.5`).
pub fn khr_materials_ior(ior: f32) -> Value {
    json!({ "ior": ior })
}

// ---------------------------------------------------------------------------
// KHR_materials_specular
// ---------------------------------------------------------------------------

/// Parameters for the `KHR_materials_specular` extension.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecularExt {
    /// Specular intensity (default `1.0`).
    pub specular_factor: f32,
    /// Specular tint colour (default `[1, 1, 1]`).
    pub specular_color_factor: [f32; 3],
}

impl Default for SpecularExt {
    fn default() -> Self {
        Self {
            specular_factor: 1.0,
            specular_color_factor: [1.0, 1.0, 1.0],
        }
    }
}

/// Return the JSON value for the `KHR_materials_specular` extension.
pub fn khr_materials_specular(params: &SpecularExt) -> Value {
    let [r, g, b] = params.specular_color_factor;
    json!({
        "specularFactor":      params.specular_factor,
        "specularColorFactor": [r, g, b],
    })
}

// ---------------------------------------------------------------------------
// AlphaMode
// ---------------------------------------------------------------------------

/// GLTF alpha-blending mode for a material.
#[derive(Debug, Clone, PartialEq)]
pub enum AlphaMode {
    /// Fully opaque (default).
    Opaque,
    /// Alpha-test with the given cutoff value.
    Mask(f32),
    /// Alpha blending.
    Blend,
}

impl AlphaMode {
    fn as_str(&self) -> &'static str {
        match self {
            AlphaMode::Opaque => "OPAQUE",
            AlphaMode::Mask(_) => "MASK",
            AlphaMode::Blend => "BLEND",
        }
    }
}

// ---------------------------------------------------------------------------
// GltfMaterialDef
// ---------------------------------------------------------------------------

/// A complete PBR material definition with optional KHR extensions.
#[derive(Debug, Clone)]
pub struct GltfMaterialDef {
    /// Human-readable name.
    pub name: String,
    /// Base colour RGBA in linear space.
    pub base_color: [f32; 4],
    /// Metallic factor `[0, 1]`.
    pub metallic_factor: f32,
    /// Roughness factor `[0, 1]`.
    pub roughness_factor: f32,
    /// Emissive RGB factor.
    pub emissive_factor: [f32; 3],
    /// Alpha blending mode.
    pub alpha_mode: AlphaMode,
    /// Whether the material is double-sided.
    pub double_sided: bool,
    /// Ordered list of `(extensionName, extensionJSON)` pairs.
    pub extensions: Vec<(String, Value)>,
}

impl Default for GltfMaterialDef {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            extensions: Vec::new(),
        }
    }
}

impl GltfMaterialDef {
    // ------------------------------------------------------------------
    // Presets
    // ------------------------------------------------------------------

    /// Skin material preset — warm pinkish, non-metallic, medium roughness,
    /// double-sided.
    pub fn skin() -> Self {
        Self {
            name: "skin".to_string(),
            base_color: [0.94, 0.76, 0.69, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            double_sided: true,
            extensions: Vec::new(),
        }
    }

    /// Cloth material preset — uses `KHR_materials_sheen` for fabric look.
    pub fn cloth() -> Self {
        let sheen = khr_materials_sheen(&SheenExt {
            sheen_color_factor: [0.8, 0.6, 0.4],
            sheen_roughness_factor: 0.7,
        });
        Self {
            name: "cloth".to_string(),
            base_color: [0.15, 0.15, 0.20, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.85,
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            extensions: vec![("KHR_materials_sheen".to_string(), sheen)],
        }
    }

    /// Glass / transparent preset — uses `KHR_materials_transmission` +
    /// `KHR_materials_volume` + `KHR_materials_ior`.
    pub fn glass() -> Self {
        let transmission = khr_materials_transmission(1.0);
        let volume = khr_materials_volume(&VolumeExt {
            thickness_factor: 0.5,
            attenuation_distance: 5.0,
            attenuation_color: [0.95, 0.97, 1.0],
        });
        let ior = khr_materials_ior(1.5);
        Self {
            name: "glass".to_string(),
            base_color: [1.0, 1.0, 1.0, 0.0],
            metallic_factor: 0.0,
            roughness_factor: 0.05,
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            extensions: vec![
                ("KHR_materials_transmission".to_string(), transmission),
                ("KHR_materials_volume".to_string(), volume),
                ("KHR_materials_ior".to_string(), ior),
            ],
        }
    }

    /// Metallic preset — uses `KHR_materials_specular` for tinted reflections.
    pub fn metallic() -> Self {
        let specular = khr_materials_specular(&SpecularExt {
            specular_factor: 1.0,
            specular_color_factor: [0.9, 0.85, 0.8],
        });
        Self {
            name: "metallic".to_string(),
            base_color: [0.7, 0.7, 0.7, 1.0],
            metallic_factor: 1.0,
            roughness_factor: 0.1,
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            extensions: vec![("KHR_materials_specular".to_string(), specular)],
        }
    }

    // ------------------------------------------------------------------
    // Builder
    // ------------------------------------------------------------------

    /// Add (or replace) an extension by name.
    pub fn with_extension(mut self, name: &str, value: Value) -> Self {
        // Replace an existing entry with the same name if present.
        if let Some(pos) = self.extensions.iter().position(|(n, _)| n == name) {
            self.extensions[pos].1 = value;
        } else {
            self.extensions.push((name.to_string(), value));
        }
        self
    }

    // ------------------------------------------------------------------
    // Serialisation
    // ------------------------------------------------------------------

    /// Serialise to a GLTF 2.0 JSON material object.
    pub fn to_json(&self) -> Value {
        let [r, g, b, a] = self.base_color;
        let [er, eg, eb] = self.emissive_factor;

        let mut mat = json!({
            "name": self.name,
            "pbrMetallicRoughness": {
                "baseColorFactor": [r, g, b, a],
                "metallicFactor":  self.metallic_factor,
                "roughnessFactor": self.roughness_factor,
            },
            "emissiveFactor": [er, eg, eb],
            "alphaMode":      self.alpha_mode.as_str(),
            "doubleSided":    self.double_sided,
        });

        // alphaCutoff is only relevant for MASK mode.
        if let AlphaMode::Mask(cutoff) = self.alpha_mode {
            mat["alphaCutoff"] = json!(cutoff);
        }

        // Embed extensions object if any.
        if !self.extensions.is_empty() {
            let mut ext_obj = serde_json::Map::new();
            for (name, val) in &self.extensions {
                ext_obj.insert(name.clone(), val.clone());
            }
            mat["extensions"] = Value::Object(ext_obj);
        }

        mat
    }

    /// Return the names of all extensions attached to this material.
    pub fn extension_names(&self) -> Vec<&str> {
        self.extensions.iter().map(|(n, _)| n.as_str()).collect()
    }
}

// ---------------------------------------------------------------------------
// Build a GLTF materials array
// ---------------------------------------------------------------------------

/// Build a GLTF-style `materials` JSON array from a slice of
/// [`GltfMaterialDef`] values.
pub fn build_materials_json(materials: &[GltfMaterialDef]) -> Value {
    let arr: Vec<Value> = materials.iter().map(|m| m.to_json()).collect();
    Value::Array(arr)
}

// ---------------------------------------------------------------------------
// Validate a material JSON object
// ---------------------------------------------------------------------------

/// Validate that a JSON value is a well-formed GLTF material object.
///
/// Checks for the required `pbrMetallicRoughness` sub-object and that scalar
/// factors are in the expected ranges.  Returns `Err` with a description on
/// the first failure.
pub fn validate_material_json(mat: &Value) -> Result<(), String> {
    let pbr = mat
        .get("pbrMetallicRoughness")
        .ok_or_else(|| "missing 'pbrMetallicRoughness'".to_string())?;

    if !pbr.is_object() {
        return Err("'pbrMetallicRoughness' must be an object".to_string());
    }

    // Validate metallic factor.
    if let Some(mf) = pbr.get("metallicFactor") {
        let v = mf
            .as_f64()
            .ok_or_else(|| "'metallicFactor' must be a number".to_string())?;
        if !(0.0..=1.0).contains(&v) {
            return Err(format!("'metallicFactor' out of range [0,1]: {v}"));
        }
    }

    // Validate roughness factor.
    if let Some(rf) = pbr.get("roughnessFactor") {
        let v = rf
            .as_f64()
            .ok_or_else(|| "'roughnessFactor' must be a number".to_string())?;
        if !(0.0..=1.0).contains(&v) {
            return Err(format!("'roughnessFactor' out of range [0,1]: {v}"));
        }
    }

    // Validate baseColorFactor length if present.
    if let Some(bcf) = pbr.get("baseColorFactor") {
        let arr = bcf
            .as_array()
            .ok_or_else(|| "'baseColorFactor' must be an array".to_string())?;
        if arr.len() != 4 {
            return Err(format!(
                "'baseColorFactor' must have 4 elements, got {}",
                arr.len()
            ));
        }
    }

    // Validate alphaMode string if present.
    if let Some(am) = mat.get("alphaMode") {
        let s = am
            .as_str()
            .ok_or_else(|| "'alphaMode' must be a string".to_string())?;
        if !matches!(s, "OPAQUE" | "MASK" | "BLEND") {
            return Err(format!("unknown 'alphaMode': '{s}'"));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Extract extensionsUsed list
// ---------------------------------------------------------------------------

/// Extract the list of extension name strings from the top-level
/// `extensionsUsed` array of a GLTF JSON document.
///
/// Returns an empty `Vec` when the key is absent or not an array.
pub fn extract_extensions_used(gltf_json: &Value) -> Vec<String> {
    gltf_json
        .get("extensionsUsed")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // 1. khr_materials_unlit returns empty object
    // -----------------------------------------------------------------------
    #[test]
    fn test_unlit_is_empty_object() {
        let v = khr_materials_unlit();
        assert!(v.is_object());
        assert_eq!(v.as_object().unwrap().len(), 0);
    }

    // -----------------------------------------------------------------------
    // 2. khr_materials_emissive_strength carries correct value
    // -----------------------------------------------------------------------
    #[test]
    fn test_emissive_strength_value() {
        let v = khr_materials_emissive_strength(3.5);
        assert!((v["emissiveStrength"].as_f64().unwrap() - 3.5).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 3. khr_materials_clearcoat fields present
    // -----------------------------------------------------------------------
    #[test]
    fn test_clearcoat_fields() {
        let p = ClearcoatExt {
            clearcoat_factor: 0.8,
            clearcoat_roughness_factor: 0.3,
        };
        let v = khr_materials_clearcoat(&p);
        assert!((v["clearcoatFactor"].as_f64().unwrap() - 0.8).abs() < 1e-6);
        assert!((v["clearcoatRoughnessFactor"].as_f64().unwrap() - 0.3).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 4. khr_materials_sheen colour array length
    // -----------------------------------------------------------------------
    #[test]
    fn test_sheen_color_array_length() {
        let p = SheenExt {
            sheen_color_factor: [0.5, 0.2, 0.8],
            sheen_roughness_factor: 0.6,
        };
        let v = khr_materials_sheen(&p);
        assert_eq!(v["sheenColorFactor"].as_array().unwrap().len(), 3);
    }

    // -----------------------------------------------------------------------
    // 5. khr_materials_transmission value round-trip
    // -----------------------------------------------------------------------
    #[test]
    fn test_transmission_round_trip() {
        let v = khr_materials_transmission(0.75);
        assert!((v["transmissionFactor"].as_f64().unwrap() - 0.75).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 6. khr_materials_volume with infinity → large JSON number
    // -----------------------------------------------------------------------
    #[test]
    fn test_volume_infinity_becomes_large_number() {
        let p = VolumeExt::default(); // attenuation_distance = INFINITY
        let v = khr_materials_volume(&p);
        let dist = v["attenuationDistance"].as_f64().unwrap();
        assert!(dist > 1e300, "expected very large number, got {dist}");
    }

    // -----------------------------------------------------------------------
    // 7. khr_materials_ior default value
    // -----------------------------------------------------------------------
    #[test]
    fn test_ior_default() {
        let v = khr_materials_ior(1.5);
        assert!((v["ior"].as_f64().unwrap() - 1.5).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 8. khr_materials_specular colour has three components
    // -----------------------------------------------------------------------
    #[test]
    fn test_specular_color_components() {
        let p = SpecularExt::default();
        let v = khr_materials_specular(&p);
        let arr = v["specularColorFactor"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!((arr[0].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 9. GltfMaterialDef::skin preset serialises correctly
    // -----------------------------------------------------------------------
    #[test]
    fn test_skin_preset_to_json() {
        let mat = GltfMaterialDef::skin();
        let j = mat.to_json();
        assert_eq!(j["name"].as_str().unwrap(), "skin");
        assert!(j["pbrMetallicRoughness"].is_object());
        assert!(j["doubleSided"].as_bool().unwrap());
        assert_eq!(j["alphaMode"].as_str().unwrap(), "OPAQUE");
    }

    // -----------------------------------------------------------------------
    // 10. GltfMaterialDef::glass has extensions
    // -----------------------------------------------------------------------
    #[test]
    fn test_glass_preset_has_extensions() {
        let mat = GltfMaterialDef::glass();
        let names = mat.extension_names();
        assert!(names.contains(&"KHR_materials_transmission"));
        assert!(names.contains(&"KHR_materials_volume"));
        assert!(names.contains(&"KHR_materials_ior"));
        let j = mat.to_json();
        assert!(j["extensions"].is_object());
    }

    // -----------------------------------------------------------------------
    // 11. AlphaMode::Mask serialises alphaCutoff
    // -----------------------------------------------------------------------
    #[test]
    fn test_alpha_mask_cutoff_in_json() {
        let mat = GltfMaterialDef {
            alpha_mode: AlphaMode::Mask(0.5),
            ..Default::default()
        };
        let j = mat.to_json();
        assert_eq!(j["alphaMode"].as_str().unwrap(), "MASK");
        assert!((j["alphaCutoff"].as_f64().unwrap() - 0.5).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 12. build_materials_json produces correct length array
    // -----------------------------------------------------------------------
    #[test]
    fn test_build_materials_json_length() {
        let mats = vec![
            GltfMaterialDef::skin(),
            GltfMaterialDef::cloth(),
            GltfMaterialDef::glass(),
            GltfMaterialDef::metallic(),
        ];
        let j = build_materials_json(&mats);
        assert_eq!(j.as_array().unwrap().len(), 4);
    }

    // -----------------------------------------------------------------------
    // 13. validate_material_json accepts valid and rejects invalid
    // -----------------------------------------------------------------------
    #[test]
    fn test_validate_material_json() {
        let good = GltfMaterialDef::skin().to_json();
        assert!(validate_material_json(&good).is_ok());

        let bad = json!({ "name": "no_pbr" });
        assert!(validate_material_json(&bad).is_err());

        let out_of_range = json!({
            "name": "bad",
            "pbrMetallicRoughness": {
                "metallicFactor": 2.5
            }
        });
        assert!(validate_material_json(&out_of_range).is_err());
    }

    // -----------------------------------------------------------------------
    // 14. extract_extensions_used happy path
    // -----------------------------------------------------------------------
    #[test]
    fn test_extract_extensions_used() {
        let gltf = json!({
            "extensionsUsed": [
                "KHR_materials_unlit",
                "KHR_materials_transmission"
            ]
        });
        let list = extract_extensions_used(&gltf);
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"KHR_materials_unlit".to_string()));
    }

    // -----------------------------------------------------------------------
    // 15. with_extension replaces duplicate and appends new
    // -----------------------------------------------------------------------
    #[test]
    fn test_with_extension_dedup() {
        let mat = GltfMaterialDef::default()
            .with_extension("KHR_materials_unlit", khr_materials_unlit())
            .with_extension("KHR_materials_ior", khr_materials_ior(1.5))
            // Replace the ior value.
            .with_extension("KHR_materials_ior", khr_materials_ior(1.8));

        assert_eq!(
            mat.extensions.len(),
            2,
            "duplicate extension should be replaced"
        );
        let ior_val = mat
            .extensions
            .iter()
            .find(|(n, _)| n == "KHR_materials_ior")
            .map(|(_, v)| v["ior"].as_f64().unwrap())
            .unwrap();
        assert!((ior_val - 1.8).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 16. Write materials JSON to /tmp/ and read it back
    // -----------------------------------------------------------------------
    #[test]
    fn test_write_materials_to_tmp() {
        let mats = vec![GltfMaterialDef::skin(), GltfMaterialDef::glass()];
        let j = build_materials_json(&mats);
        let path = "/tmp/oxihuman_gltf_ext_test_materials.json";
        let s = serde_json::to_string_pretty(&j).unwrap();
        fs::write(path, &s).unwrap();
        let raw = fs::read_to_string(path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    // -----------------------------------------------------------------------
    // 17. cloth preset extension names
    // -----------------------------------------------------------------------
    #[test]
    fn test_cloth_preset_sheen_extension() {
        let mat = GltfMaterialDef::cloth();
        assert!(mat.extension_names().contains(&"KHR_materials_sheen"));
    }

    // -----------------------------------------------------------------------
    // 18. extract_extensions_used returns empty for missing key
    // -----------------------------------------------------------------------
    #[test]
    fn test_extract_extensions_used_missing() {
        let gltf = json!({ "asset": { "version": "2.0" } });
        let list = extract_extensions_used(&gltf);
        assert!(list.is_empty());
    }
}
