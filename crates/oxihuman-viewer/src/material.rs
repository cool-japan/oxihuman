// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PBR material definitions — metallic/roughness workflow.

// ── Color4 ────────────────────────────────────────────────────────────────────

/// RGBA colour with linear floating-point components.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    /// Create a colour from explicit RGBA components.
    #[allow(dead_code)]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Opaque white.
    #[allow(dead_code)]
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    /// Opaque black.
    #[allow(dead_code)]
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    /// Convert to a `[r, g, b, a]` array.
    #[allow(dead_code)]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

// ── PbrMaterial ───────────────────────────────────────────────────────────────

/// PBR material with metallic/roughness workflow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbrMaterial {
    pub name: String,
    /// Base/albedo colour.
    pub base_color: Color4,
    /// Metallic factor in [0, 1].
    pub metallic: f32,
    /// Roughness factor in [0, 1].
    pub roughness: f32,
    /// HDR emissive colour (RGB, can exceed 1.0).
    pub emissive: [f32; 3],
    /// `None` = fully opaque; `Some(t)` = alpha-test with threshold `t`.
    pub alpha_cutoff: Option<f32>,
    /// Whether both faces should be rendered.
    pub double_sided: bool,
}

impl PbrMaterial {
    /// Realistic human skin (pinkish, low metallic, medium roughness).
    #[allow(dead_code)]
    pub fn default_skin() -> Self {
        PbrMaterial {
            name: "skin".to_string(),
            base_color: Color4::new(0.85, 0.65, 0.55, 1.0),
            metallic: 0.0,
            roughness: 0.6,
            emissive: [0.0, 0.0, 0.0],
            alpha_cutoff: None,
            double_sided: false,
        }
    }

    /// Generic fabric (mid-grey, non-metallic, rough).
    #[allow(dead_code)]
    pub fn default_cloth() -> Self {
        PbrMaterial {
            name: "cloth".to_string(),
            base_color: Color4::new(0.5, 0.5, 0.5, 1.0),
            metallic: 0.0,
            roughness: 0.9,
            emissive: [0.0, 0.0, 0.0],
            alpha_cutoff: None,
            double_sided: false,
        }
    }

    /// Metal surface (metallic=1, low roughness).
    #[allow(dead_code)]
    pub fn default_metal() -> Self {
        PbrMaterial {
            name: "metal".to_string(),
            base_color: Color4::new(0.8, 0.8, 0.8, 1.0),
            metallic: 1.0,
            roughness: 0.2,
            emissive: [0.0, 0.0, 0.0],
            alpha_cutoff: None,
            double_sided: false,
        }
    }

    /// Glass (transparent, very smooth).
    #[allow(dead_code)]
    pub fn default_glass() -> Self {
        PbrMaterial {
            name: "glass".to_string(),
            base_color: Color4::new(0.9, 0.95, 1.0, 0.15),
            metallic: 0.0,
            roughness: 0.05,
            emissive: [0.0, 0.0, 0.0],
            alpha_cutoff: Some(0.01),
            double_sided: true,
        }
    }
}

// ── MaterialLibrary ───────────────────────────────────────────────────────────

/// A flat list of [`PbrMaterial`] entries indexed by position.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MaterialLibrary {
    pub materials: Vec<PbrMaterial>,
}

impl MaterialLibrary {
    /// Create an empty library.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a material and return its index.
    #[allow(dead_code)]
    pub fn add(&mut self, mat: PbrMaterial) -> usize {
        let idx = self.materials.len();
        self.materials.push(mat);
        idx
    }

    /// Retrieve a material by index.
    #[allow(dead_code)]
    pub fn get(&self, idx: usize) -> Option<&PbrMaterial> {
        self.materials.get(idx)
    }

    /// Find the first material whose name matches `name`.
    #[allow(dead_code)]
    pub fn by_name(&self, name: &str) -> Option<&PbrMaterial> {
        self.materials.iter().find(|m| m.name == name)
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Serialize a material to a minimal GLTF `pbrMetallicRoughness` JSON snippet.
#[allow(dead_code)]
pub fn material_to_gltf_json(mat: &PbrMaterial) -> String {
    let c = &mat.base_color;
    let alpha_mode = if mat.alpha_cutoff.is_some() {
        "\"MASK\""
    } else if c.a < 1.0 {
        "\"BLEND\""
    } else {
        "\"OPAQUE\""
    };
    let cutoff_fragment = mat
        .alpha_cutoff
        .map(|t| format!(", \"alphaCutoff\": {:.4}", t))
        .unwrap_or_default();
    format!(
        r#"{{"name": "{name}", "pbrMetallicRoughness": {{"baseColorFactor": [{r:.4}, {g:.4}, {b:.4}, {a:.4}], "metallicFactor": {m:.4}, "roughnessFactor": {ro:.4}}}, "emissiveFactor": [{er:.4}, {eg:.4}, {eb:.4}], "alphaMode": {am}, "doubleSided": {ds}{cutoff}}}"#,
        name = mat.name,
        r = c.r,
        g = c.g,
        b = c.b,
        a = c.a,
        m = mat.metallic,
        ro = mat.roughness,
        er = mat.emissive[0],
        eg = mat.emissive[1],
        eb = mat.emissive[2],
        am = alpha_mode,
        ds = mat.double_sided,
        cutoff = cutoff_fragment,
    )
}

/// Convert a [`Color4`] to an `"#RRGGBBAA"` hex string.
#[allow(dead_code)]
pub fn color_to_hex(c: &Color4) -> String {
    let r = (c.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (c.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (c.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (c.a.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
}

/// Linearly interpolate between two materials component by component.
///
/// String fields (`name`, `double_sided`, `alpha_cutoff`) are taken from `a`.
#[allow(dead_code)]
pub fn lerp_material(a: &PbrMaterial, b: &PbrMaterial, t: f32) -> PbrMaterial {
    let lerp_f32 = |x: f32, y: f32| x + (y - x) * t;
    let lerp_color = |ca: &Color4, cb: &Color4| Color4 {
        r: lerp_f32(ca.r, cb.r),
        g: lerp_f32(ca.g, cb.g),
        b: lerp_f32(ca.b, cb.b),
        a: lerp_f32(ca.a, cb.a),
    };
    PbrMaterial {
        name: a.name.clone(),
        base_color: lerp_color(&a.base_color, &b.base_color),
        metallic: lerp_f32(a.metallic, b.metallic),
        roughness: lerp_f32(a.roughness, b.roughness),
        emissive: [
            lerp_f32(a.emissive[0], b.emissive[0]),
            lerp_f32(a.emissive[1], b.emissive[1]),
            lerp_f32(a.emissive[2], b.emissive[2]),
        ],
        alpha_cutoff: a.alpha_cutoff,
        double_sided: a.double_sided,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color4_white() {
        let c = Color4::white();
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!((c.g - 1.0).abs() < 1e-6);
        assert!((c.b - 1.0).abs() < 1e-6);
        assert!((c.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color4_to_array() {
        let c = Color4::new(0.1, 0.2, 0.3, 0.4);
        let arr = c.to_array();
        assert_eq!(arr, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn color_to_hex_white() {
        assert_eq!(color_to_hex(&Color4::white()), "#FFFFFFFF");
    }

    #[test]
    fn color_to_hex_black() {
        assert_eq!(color_to_hex(&Color4::black()), "#000000FF");
    }

    #[test]
    fn default_skin_low_metallic() {
        let skin = PbrMaterial::default_skin();
        assert!(skin.metallic < 0.1);
    }

    #[test]
    fn default_cloth_not_metallic() {
        let cloth = PbrMaterial::default_cloth();
        assert!((cloth.metallic - 0.0).abs() < 1e-6);
    }

    #[test]
    fn default_metal_metallic_one() {
        let metal = PbrMaterial::default_metal();
        assert!((metal.metallic - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_glass_has_alpha() {
        let glass = PbrMaterial::default_glass();
        assert!(glass.base_color.a < 1.0, "glass should be transparent");
    }

    #[test]
    fn default_glass_has_alpha_cutoff() {
        let glass = PbrMaterial::default_glass();
        assert!(glass.alpha_cutoff.is_some());
    }

    #[test]
    fn material_to_gltf_json_contains_pbr_key() {
        let mat = PbrMaterial::default_skin();
        let json = material_to_gltf_json(&mat);
        assert!(json.contains("pbrMetallicRoughness"));
    }

    #[test]
    fn library_add_and_get() {
        let mut lib = MaterialLibrary::new();
        let idx = lib.add(PbrMaterial::default_skin());
        assert_eq!(idx, 0);
        assert!(lib.get(0).is_some());
    }

    #[test]
    fn library_by_name() {
        let mut lib = MaterialLibrary::new();
        lib.add(PbrMaterial::default_skin());
        assert!(lib.by_name("skin").is_some());
        assert!(lib.by_name("nonexistent").is_none());
    }

    #[test]
    fn library_get_out_of_bounds() {
        let lib = MaterialLibrary::new();
        assert!(lib.get(99).is_none());
    }

    #[test]
    fn lerp_material_midpoint() {
        let a = PbrMaterial::default_skin(); // metallic=0.0
        let b = PbrMaterial::default_metal(); // metallic=1.0
        let mid = lerp_material(&a, &b, 0.5);
        assert!((mid.metallic - 0.5).abs() < 1e-5);
    }

    #[test]
    fn lerp_material_at_t0_matches_a() {
        let a = PbrMaterial::default_cloth();
        let b = PbrMaterial::default_metal();
        let result = lerp_material(&a, &b, 0.0);
        assert!((result.metallic - a.metallic).abs() < 1e-6);
        assert!((result.roughness - a.roughness).abs() < 1e-6);
    }

    #[test]
    fn lerp_material_at_t1_matches_b() {
        let a = PbrMaterial::default_cloth();
        let b = PbrMaterial::default_metal();
        let result = lerp_material(&a, &b, 1.0);
        assert!((result.metallic - b.metallic).abs() < 1e-6);
        assert!((result.roughness - b.roughness).abs() < 1e-6);
    }
}
