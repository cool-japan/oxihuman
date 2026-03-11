// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// PBR metallic-roughness material properties.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PbrMaterial {
    pub name: String,
    /// Base color RGBA [0..1]
    pub base_color: [f32; 4],
    /// Metallic factor [0..1]
    pub metallic: f32,
    /// Roughness factor [0..1]
    pub roughness: f32,
    /// Emissive RGB [0..1]
    pub emissive: [f32; 3],
    /// Double-sided rendering
    pub double_sided: bool,
    /// Alpha mode: "OPAQUE", "MASK", or "BLEND"
    pub alpha_mode: String,
}

impl PbrMaterial {
    /// Skin-like material (pinkish, low metallic, medium roughness)
    #[allow(dead_code)]
    pub fn skin() -> Self {
        Self {
            name: "skin".to_string(),
            base_color: [0.94, 0.76, 0.69, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            double_sided: true,
            alpha_mode: "OPAQUE".to_string(),
        }
    }

    /// Clothing-like material (dark, low metallic, high roughness)
    #[allow(dead_code)]
    pub fn clothing() -> Self {
        Self {
            name: "clothing".to_string(),
            base_color: [0.15, 0.15, 0.20, 1.0],
            metallic: 0.0,
            roughness: 0.85,
            emissive: [0.0, 0.0, 0.0],
            double_sided: false,
            alpha_mode: "OPAQUE".to_string(),
        }
    }

    /// Default neutral material
    #[allow(dead_code)]
    pub fn default_material() -> Self {
        Self {
            name: "default".to_string(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            double_sided: false,
            alpha_mode: "OPAQUE".to_string(),
        }
    }

    /// Convert to GLTF JSON material node
    #[allow(dead_code)]
    pub fn to_gltf_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "pbrMetallicRoughness": {
                "baseColorFactor": [
                    self.base_color[0],
                    self.base_color[1],
                    self.base_color[2],
                    self.base_color[3]
                ],
                "metallicFactor": self.metallic,
                "roughnessFactor": self.roughness
            },
            "emissiveFactor": [
                self.emissive[0],
                self.emissive[1],
                self.emissive[2]
            ],
            "doubleSided": self.double_sided,
            "alphaMode": self.alpha_mode
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skin_material_has_correct_alpha() {
        assert_eq!(PbrMaterial::skin().base_color[3], 1.0);
    }

    #[test]
    fn to_gltf_json_has_pbr_key() {
        let skin = PbrMaterial::skin();
        let json = skin.to_gltf_json();
        assert!(
            json["pbrMetallicRoughness"].is_object(),
            "pbrMetallicRoughness should be an object"
        );
    }

    #[test]
    fn clothing_is_opaque() {
        assert_eq!(PbrMaterial::clothing().alpha_mode, "OPAQUE");
    }

    #[test]
    fn gltf_json_metallic_is_float() {
        let skin = PbrMaterial::skin();
        let json = skin.to_gltf_json();
        assert!(
            json["pbrMetallicRoughness"]["metallicFactor"]
                .as_f64()
                .is_some(),
            "metallicFactor should be a float"
        );
    }
}
