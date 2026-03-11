//! Material preview renderer stub.
//!
//! Configures PBR material parameters for preview display in the OxiHuman viewer.

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for the material preview renderer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialPreviewConfig {
    /// Preview sphere resolution (number of latitude segments).
    pub sphere_segments: u32,
    /// Whether to show an environment map in the background.
    pub show_environment: bool,
    /// Exposure value for HDR preview (EV stops).
    pub exposure: f32,
    /// Maximum number of materials in the scene.
    pub max_materials: usize,
}

// ── PBR Material ──────────────────────────────────────────────────────────────

/// A physically-based rendering material for preview.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PreviewPbrMaterial {
    /// Material name.
    pub name: String,
    /// Albedo (base colour) as linear RGB `[r, g, b]` in `[0, 1]`.
    pub albedo: [f32; 3],
    /// Metallic factor in `[0, 1]`.
    pub metallic: f32,
    /// Perceptual roughness in `[0, 1]`.
    pub roughness: f32,
    /// Emissive colour as linear RGB `[r, g, b]`.
    pub emissive: [f32; 3],
    /// Opacity / alpha in `[0, 1]`.
    pub alpha: f32,
}

// ── Scene ─────────────────────────────────────────────────────────────────────

/// A material preview scene containing one or more materials.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialPreviewScene {
    /// Scene configuration.
    pub config: MaterialPreviewConfig,
    /// Materials registered in the scene.
    pub materials: Vec<PreviewPbrMaterial>,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Returns sensible default `MaterialPreviewConfig`.
#[allow(dead_code)]
pub fn default_material_preview_config() -> MaterialPreviewConfig {
    MaterialPreviewConfig {
        sphere_segments: 32,
        show_environment: true,
        exposure: 1.0,
        max_materials: 64,
    }
}

/// Creates a new `PreviewPbrMaterial` with default values (white dielectric).
#[allow(dead_code)]
pub fn new_pbr_material(name: &str) -> PreviewPbrMaterial {
    PreviewPbrMaterial {
        name: name.to_string(),
        albedo: [1.0, 1.0, 1.0],
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
        alpha: 1.0,
    }
}

/// Sets the albedo (base colour) of a material.
#[allow(dead_code)]
pub fn pbr_set_albedo(mat: &mut PreviewPbrMaterial, r: f32, g: f32, b: f32) {
    mat.albedo = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Sets the metallic factor of a material.
#[allow(dead_code)]
pub fn pbr_set_metallic(mat: &mut PreviewPbrMaterial, metallic: f32) {
    mat.metallic = metallic.clamp(0.0, 1.0);
}

/// Sets the roughness of a material.
#[allow(dead_code)]
pub fn pbr_set_roughness(mat: &mut PreviewPbrMaterial, roughness: f32) {
    mat.roughness = roughness.clamp(0.0, 1.0);
}

/// Sets the emissive colour of a material.
#[allow(dead_code)]
pub fn pbr_set_emissive(mat: &mut PreviewPbrMaterial, r: f32, g: f32, b: f32) {
    mat.emissive = [r.max(0.0), g.max(0.0), b.max(0.0)];
}

/// Creates a new, empty `MaterialPreviewScene`.
#[allow(dead_code)]
pub fn new_material_preview_scene(cfg: &MaterialPreviewConfig) -> MaterialPreviewScene {
    MaterialPreviewScene {
        config: cfg.clone(),
        materials: Vec::new(),
    }
}

/// Adds a material to the scene.
#[allow(dead_code)]
pub fn scene_add_material(scene: &mut MaterialPreviewScene, mat: PreviewPbrMaterial) {
    scene.materials.push(mat);
}

/// Returns the number of materials in the scene.
#[allow(dead_code)]
pub fn scene_material_count(scene: &MaterialPreviewScene) -> usize {
    scene.materials.len()
}

/// Serialises a material to a human-readable string.
#[allow(dead_code)]
pub fn pbr_material_to_string(mat: &PreviewPbrMaterial) -> String {
    format!(
        "PreviewPbrMaterial {{ name: \"{}\", albedo: [{:.3},{:.3},{:.3}], \
         metallic: {:.3}, roughness: {:.3}, emissive: [{:.3},{:.3},{:.3}], alpha: {:.3} }}",
        mat.name,
        mat.albedo[0],
        mat.albedo[1],
        mat.albedo[2],
        mat.metallic,
        mat.roughness,
        mat.emissive[0],
        mat.emissive[1],
        mat.emissive[2],
        mat.alpha,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_material_preview_config();
        assert_eq!(cfg.sphere_segments, 32);
        assert!(cfg.show_environment);
        assert!((cfg.exposure - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_pbr_material_defaults() {
        let mat = new_pbr_material("TestMat");
        assert_eq!(mat.name, "TestMat");
        assert_eq!(mat.albedo, [1.0, 1.0, 1.0]);
        assert!((mat.metallic - 0.0).abs() < 1e-6);
        assert!((mat.roughness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_pbr_set_albedo() {
        let mut mat = new_pbr_material("M");
        pbr_set_albedo(&mut mat, 0.8, 0.5, 0.3);
        assert!((mat.albedo[0] - 0.8).abs() < 1e-6);
        assert!((mat.albedo[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_pbr_set_albedo_clamps() {
        let mut mat = new_pbr_material("M");
        pbr_set_albedo(&mut mat, -1.0, 2.0, 0.5);
        assert!((mat.albedo[0] - 0.0).abs() < 1e-6);
        assert!((mat.albedo[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pbr_set_metallic_roughness() {
        let mut mat = new_pbr_material("Metal");
        pbr_set_metallic(&mut mat, 1.0);
        pbr_set_roughness(&mut mat, 0.1);
        assert!((mat.metallic - 1.0).abs() < 1e-6);
        assert!((mat.roughness - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_pbr_set_emissive() {
        let mut mat = new_pbr_material("Glow");
        pbr_set_emissive(&mut mat, 2.0, 0.0, 0.5);
        assert!((mat.emissive[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_scene_add_and_count() {
        let cfg = default_material_preview_config();
        let mut scene = new_material_preview_scene(&cfg);
        scene_add_material(&mut scene, new_pbr_material("Mat1"));
        scene_add_material(&mut scene, new_pbr_material("Mat2"));
        assert_eq!(scene_material_count(&scene), 2);
    }

    #[test]
    fn test_pbr_material_to_string() {
        let mat = new_pbr_material("Preview");
        let s = pbr_material_to_string(&mat);
        assert!(s.contains("Preview"));
        assert!(s.contains("metallic"));
        assert!(s.contains("roughness"));
    }

    #[test]
    fn test_scene_starts_empty() {
        let cfg = default_material_preview_config();
        let scene = new_material_preview_scene(&cfg);
        assert_eq!(scene_material_count(&scene), 0);
    }
}
