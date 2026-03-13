//! Skin material/shader parameter morphs (SSS, roughness, color tints).

/// Skin body zone classification.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SkinZone {
    Face,
    Neck,
    Arms,
    Torso,
    Legs,
}

/// Parameters controlling skin material appearance.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SkinShaderParams {
    /// Subsurface scattering strength (0..1).
    pub sss_strength: f32,
    /// Surface roughness (0 = mirror, 1 = fully rough).
    pub roughness: f32,
    /// Melanin level controlling skin darkness (0..1).
    pub melanin: f32,
    /// Hemoglobin level controlling redness (0..1).
    pub hemoglobin: f32,
    /// RGB tint applied on top of computed color.
    pub tint: [f32; 3],
}

/// A named collection of skin shader parameters for all zones.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SkinPreset {
    /// Human-readable preset name.
    pub name: String,
    /// Parameters for each of the 5 zones (Face, Neck, Arms, Torso, Legs).
    pub zones: Vec<(SkinZone, SkinShaderParams)>,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create default skin shader parameters (medium Caucasian skin).
#[allow(dead_code)]
pub fn default_skin_params() -> SkinShaderParams {
    SkinShaderParams {
        sss_strength: 0.5,
        roughness: 0.4,
        melanin: 0.3,
        hemoglobin: 0.2,
        tint: [1.0, 1.0, 1.0],
    }
}

/// Create a new skin preset with the given name and default params for all zones.
#[allow(dead_code)]
pub fn new_skin_preset(name: &str) -> SkinPreset {
    let all_zones = [
        SkinZone::Face,
        SkinZone::Neck,
        SkinZone::Arms,
        SkinZone::Torso,
        SkinZone::Legs,
    ];
    let zones = all_zones
        .iter()
        .map(|&z| (z, default_skin_params()))
        .collect();
    SkinPreset {
        name: name.to_string(),
        zones,
    }
}

// ---------------------------------------------------------------------------
// Setters
// ---------------------------------------------------------------------------

/// Set subsurface scattering strength, clamped to [0, 1].
#[allow(dead_code)]
pub fn set_sss_strength(params: &mut SkinShaderParams, strength: f32) {
    params.sss_strength = strength.clamp(0.0, 1.0);
}

/// Set surface roughness, clamped to [0, 1].
#[allow(dead_code)]
pub fn set_roughness(params: &mut SkinShaderParams, roughness: f32) {
    params.roughness = roughness.clamp(0.0, 1.0);
}

/// Set melanin level (skin darkness), clamped to [0, 1].
#[allow(dead_code)]
pub fn set_melanin(params: &mut SkinShaderParams, melanin: f32) {
    params.melanin = melanin.clamp(0.0, 1.0);
}

/// Set hemoglobin level (redness), clamped to [0, 1].
#[allow(dead_code)]
pub fn set_hemoglobin(params: &mut SkinShaderParams, hemoglobin: f32) {
    params.hemoglobin = hemoglobin.clamp(0.0, 1.0);
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/// Blend two skin parameter sets by factor `t` (0 = all `a`, 1 = all `b`).
#[allow(dead_code)]
pub fn blend_skin_params(a: &SkinShaderParams, b: &SkinShaderParams, t: f32) -> SkinShaderParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    SkinShaderParams {
        sss_strength: a.sss_strength * inv + b.sss_strength * t,
        roughness: a.roughness * inv + b.roughness * t,
        melanin: a.melanin * inv + b.melanin * t,
        hemoglobin: a.hemoglobin * inv + b.hemoglobin * t,
        tint: [
            a.tint[0] * inv + b.tint[0] * t,
            a.tint[1] * inv + b.tint[1] * t,
            a.tint[2] * inv + b.tint[2] * t,
        ],
    }
}

/// Compute an approximate RGB skin color from melanin and hemoglobin values.
///
/// This uses a simplified model:
/// - Base color starts light and darkens with melanin.
/// - Red channel is boosted by hemoglobin.
///
/// Returns `[r, g, b]` each in 0..1.
#[allow(dead_code)]
pub fn skin_color_from_params(params: &SkinShaderParams) -> [f32; 3] {
    // Base skin color (light skin).
    let base_r = 1.0;
    let base_g = 0.85;
    let base_b = 0.72;

    // Melanin darkens all channels.
    let mel = params.melanin;
    let r = base_r * (1.0 - mel * 0.7);
    let g = base_g * (1.0 - mel * 0.75);
    let b = base_b * (1.0 - mel * 0.8);

    // Hemoglobin adds redness.
    let hemo = params.hemoglobin;
    let r = (r + hemo * 0.15).min(1.0);
    let g = (g - hemo * 0.05).max(0.0);
    let b = (b - hemo * 0.08).max(0.0);

    // Apply tint.
    [
        (r * params.tint[0]).clamp(0.0, 1.0),
        (g * params.tint[1]).clamp(0.0, 1.0),
        (b * params.tint[2]).clamp(0.0, 1.0),
    ]
}

/// Apply aging effects: increases roughness, reduces SSS, slightly increases melanin.
#[allow(dead_code)]
pub fn apply_age_effect(params: &mut SkinShaderParams, age_factor: f32) {
    let factor = age_factor.clamp(0.0, 1.0);
    params.roughness = (params.roughness + factor * 0.3).min(1.0);
    params.sss_strength = (params.sss_strength - factor * 0.2).max(0.0);
    params.melanin = (params.melanin + factor * 0.05).min(1.0);
}

/// Get the parameters for a specific zone from a preset.
/// Returns `None` if the zone is not in the preset.
#[allow(dead_code)]
pub fn zone_params(preset: &SkinPreset, zone: SkinZone) -> Option<&SkinShaderParams> {
    preset
        .zones
        .iter()
        .find(|(z, _)| *z == zone)
        .map(|(_, p)| p)
}

/// Set a color tint for a specific zone in the preset.
/// Returns `true` if the zone was found and updated.
#[allow(dead_code)]
pub fn set_zone_tint(preset: &mut SkinPreset, zone: SkinZone, tint: [f32; 3]) -> bool {
    for (z, p) in &mut preset.zones {
        if *z == zone {
            p.tint = tint;
            return true;
        }
    }
    false
}

/// Serialize a skin preset to a JSON string.
#[allow(dead_code)]
pub fn skin_preset_to_json(preset: &SkinPreset) -> String {
    let mut parts = Vec::new();
    for (zone, params) in &preset.zones {
        let zone_name = match zone {
            SkinZone::Face => "Face",
            SkinZone::Neck => "Neck",
            SkinZone::Arms => "Arms",
            SkinZone::Torso => "Torso",
            SkinZone::Legs => "Legs",
        };
        let color = skin_color_from_params(params);
        parts.push(format!(
            "{{\"zone\":\"{}\",\"sss\":{:.4},\"roughness\":{:.4},\"melanin\":{:.4},\"hemoglobin\":{:.4},\"color\":[{:.4},{:.4},{:.4}]}}",
            zone_name, params.sss_strength, params.roughness, params.melanin, params.hemoglobin,
            color[0], color[1], color[2]
        ));
    }
    format!(
        "{{\"name\":\"{}\",\"zones\":[{}]}}",
        preset.name,
        parts.join(",")
    )
}

/// Return the number of zones in the preset.
#[allow(dead_code)]
pub fn preset_count(preset: &SkinPreset) -> usize {
    preset.zones.len()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_skin_params() {
        let p = default_skin_params();
        assert!(p.sss_strength > 0.0);
        assert!(p.roughness > 0.0);
        assert!(p.melanin >= 0.0 && p.melanin <= 1.0);
    }

    #[test]
    fn test_new_skin_preset_has_all_zones() {
        let preset = new_skin_preset("test");
        assert_eq!(preset.zones.len(), 5);
        assert_eq!(preset.name, "test");
    }

    #[test]
    fn test_set_sss_strength() {
        let mut p = default_skin_params();
        set_sss_strength(&mut p, 0.8);
        assert!((p.sss_strength - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_sss_strength_clamps() {
        let mut p = default_skin_params();
        set_sss_strength(&mut p, 2.0);
        assert!((p.sss_strength - 1.0).abs() < 1e-6);
        set_sss_strength(&mut p, -1.0);
        assert!((p.sss_strength - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_roughness() {
        let mut p = default_skin_params();
        set_roughness(&mut p, 0.7);
        assert!((p.roughness - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_melanin() {
        let mut p = default_skin_params();
        set_melanin(&mut p, 0.9);
        assert!((p.melanin - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_hemoglobin() {
        let mut p = default_skin_params();
        set_hemoglobin(&mut p, 0.6);
        assert!((p.hemoglobin - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_skin_params_zero() {
        let a = default_skin_params();
        let mut b = default_skin_params();
        b.sss_strength = 1.0;
        let r = blend_skin_params(&a, &b, 0.0);
        assert!((r.sss_strength - a.sss_strength).abs() < 1e-6);
    }

    #[test]
    fn test_blend_skin_params_one() {
        let a = default_skin_params();
        let mut b = default_skin_params();
        b.sss_strength = 1.0;
        let r = blend_skin_params(&a, &b, 1.0);
        assert!((r.sss_strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_skin_color_from_params_light() {
        let p = SkinShaderParams {
            sss_strength: 0.5,
            roughness: 0.4,
            melanin: 0.0,
            hemoglobin: 0.0,
            tint: [1.0, 1.0, 1.0],
        };
        let c = skin_color_from_params(&p);
        assert!(c[0] > 0.9); // light skin = bright
        assert!(c[1] > 0.8);
    }

    #[test]
    fn test_skin_color_from_params_dark() {
        let p = SkinShaderParams {
            sss_strength: 0.5,
            roughness: 0.4,
            melanin: 1.0,
            hemoglobin: 0.0,
            tint: [1.0, 1.0, 1.0],
        };
        let c = skin_color_from_params(&p);
        assert!(c[0] < 0.5); // dark skin = darker
    }

    #[test]
    fn test_apply_age_effect() {
        let mut p = default_skin_params();
        let orig_roughness = p.roughness;
        let orig_sss = p.sss_strength;
        apply_age_effect(&mut p, 0.5);
        assert!(p.roughness > orig_roughness);
        assert!(p.sss_strength < orig_sss);
    }

    #[test]
    fn test_zone_params_found() {
        let preset = new_skin_preset("test");
        let p = zone_params(&preset, SkinZone::Face);
        assert!(p.is_some());
    }

    #[test]
    fn test_set_zone_tint() {
        let mut preset = new_skin_preset("test");
        let ok = set_zone_tint(&mut preset, SkinZone::Arms, [0.5, 0.6, 0.7]);
        assert!(ok);
        let p = zone_params(&preset, SkinZone::Arms).expect("should succeed");
        assert!((p.tint[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_skin_preset_to_json() {
        let preset = new_skin_preset("demo");
        let json = skin_preset_to_json(&preset);
        assert!(json.contains("\"name\":\"demo\""));
        assert!(json.contains("\"zone\":\"Face\""));
    }

    #[test]
    fn test_preset_count() {
        let preset = new_skin_preset("test");
        assert_eq!(preset_count(&preset), 5);
    }
}
