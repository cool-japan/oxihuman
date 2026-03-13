// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physically-based skin color model using the Fitzpatrick phototype scale
//! and a simplified melanin/hemoglobin reflectance model.

// ── Fitzpatrick Phototype ────────────────────────────────────────────────────

/// Fitzpatrick phototype scale (I–VI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitzpatrickType {
    /// Very fair, always burns, never tans.
    Type1,
    /// Fair, usually burns, sometimes tans.
    Type2,
    /// Medium, sometimes burns, always tans.
    Type3,
    /// Olive, rarely burns, always tans.
    Type4,
    /// Brown, very rarely burns, tans very easily.
    Type5,
    /// Dark brown/black, never burns, tans darkly.
    Type6,
}

impl FitzpatrickType {
    /// Melanin concentration in [0.0, 1.0].
    ///
    /// Type1 = 0.0 (minimum melanin), Type6 = 1.0 (maximum melanin).
    pub fn melanin_level(&self) -> f32 {
        match self {
            Self::Type1 => 0.0,
            Self::Type2 => 0.2,
            Self::Type3 => 0.4,
            Self::Type4 => 0.6,
            Self::Type5 => 0.8,
            Self::Type6 => 1.0,
        }
    }

    /// Representative sRGB base color for this phototype.
    pub fn base_rgb(&self) -> [u8; 3] {
        match self {
            Self::Type1 => [255, 224, 196],
            Self::Type2 => [240, 200, 168],
            Self::Type3 => [210, 168, 128],
            Self::Type4 => [172, 124, 88],
            Self::Type5 => [120, 78, 48],
            Self::Type6 => [62, 36, 22],
        }
    }

    /// Human-readable name for this phototype.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Type1 => "Type I – Very Fair",
            Self::Type2 => "Type II – Fair",
            Self::Type3 => "Type III – Medium",
            Self::Type4 => "Type IV – Olive",
            Self::Type5 => "Type V – Brown",
            Self::Type6 => "Type VI – Dark Brown/Black",
        }
    }

    /// All six Fitzpatrick types in ascending order.
    pub fn all() -> [FitzpatrickType; 6] {
        [
            Self::Type1,
            Self::Type2,
            Self::Type3,
            Self::Type4,
            Self::Type5,
            Self::Type6,
        ]
    }
}

// ── SkinColor ────────────────────────────────────────────────────────────────

/// A skin color defined by biophysical parameters.
#[derive(Debug, Clone)]
pub struct SkinColor {
    /// Melanin concentration [0, 1]. Higher = darker skin.
    pub melanin: f32,
    /// Hemoglobin saturation [0, 1]. Higher = redder, more flushed skin.
    pub hemoglobin: f32,
    /// Subsurface scattering strength [0, 1].
    pub subsurface: f32,
    /// Skin oiliness / glossiness [0, 1].
    pub oiliness: f32,
}

/// Base "white" skin tone before melanin/hemoglobin contributions.
const BASE_WHITE: [f32; 3] = [255.0, 220.0, 185.0];
/// Dark-melanin target color (very dark brown).
const MELANIN_DARK: [f32; 3] = [50.0, 30.0, 20.0];
/// Hemoglobin reddish tint per unit.
const HEMOGLOBIN_TINT: [f32; 3] = [20.0, -5.0, -10.0];

impl SkinColor {
    /// Construct a `SkinColor` from a Fitzpatrick phototype using representative
    /// biophysical parameters.
    pub fn from_fitzpatrick(t: FitzpatrickType) -> Self {
        let melanin = t.melanin_level();
        // Hemoglobin is slightly elevated for lighter types (more visible vasculature)
        // and lower for darker types where melanin dominates.
        let hemoglobin = (0.35 - melanin * 0.2).clamp(0.0, 1.0);
        let subsurface = (0.5 - melanin * 0.15).clamp(0.0, 1.0);
        let oiliness = 0.3;
        Self {
            melanin,
            hemoglobin,
            subsurface,
            oiliness,
        }
    }

    /// Compute an sRGB [0..255] base color from the melanin/hemoglobin model.
    ///
    /// Algorithm:
    /// 1. Start from base skin white `[255, 220, 185]`.
    /// 2. Mix toward `[50, 30, 20]` using `melanin`.
    /// 3. Add reddish tint `[20, -5, -10]` × `hemoglobin`.
    /// 4. Clamp all channels to [0, 255].
    pub fn to_rgb(&self) -> [u8; 3] {
        let m = self.melanin.clamp(0.0, 1.0);
        let h = self.hemoglobin.clamp(0.0, 1.0);

        let channels: [u8; 3] = std::array::from_fn(|i| {
            let base = BASE_WHITE[i] + (MELANIN_DARK[i] - BASE_WHITE[i]) * m;
            let v = base + HEMOGLOBIN_TINT[i] * h;
            v.clamp(0.0, 255.0).round() as u8
        });
        channels
    }

    /// Compute RGBA where alpha is always 255 (fully opaque).
    pub fn to_rgba(&self) -> [u8; 4] {
        let [r, g, b] = self.to_rgb();
        [r, g, b, 255]
    }

    /// Linearly interpolate between `self` and `other` by factor `t` ∈ [0, 1].
    pub fn lerp(&self, other: &SkinColor, t: f32) -> SkinColor {
        let t = t.clamp(0.0, 1.0);
        SkinColor {
            melanin: self.melanin + (other.melanin - self.melanin) * t,
            hemoglobin: self.hemoglobin + (other.hemoglobin - self.hemoglobin) * t,
            subsurface: self.subsurface + (other.subsurface - self.subsurface) * t,
            oiliness: self.oiliness + (other.oiliness - self.oiliness) * t,
        }
    }

    /// Apply a suntan effect: increase melanin by `amount`, clamped to 1.0.
    pub fn apply_tan(&self, amount: f32) -> SkinColor {
        SkinColor {
            melanin: (self.melanin + amount).clamp(0.0, 1.0),
            hemoglobin: self.hemoglobin,
            subsurface: self.subsurface,
            oiliness: self.oiliness,
        }
    }

    /// Apply a blush effect: temporarily increase hemoglobin by `amount`.
    pub fn apply_blush(&self, amount: f32) -> SkinColor {
        SkinColor {
            melanin: self.melanin,
            hemoglobin: (self.hemoglobin + amount).clamp(0.0, 1.0),
            subsurface: self.subsurface,
            oiliness: self.oiliness,
        }
    }

    /// Return a slightly varied version by adding small deterministic offsets
    /// derived from the given `seed`.
    ///
    /// Uses a simple LCG so there is no dependency on external RNG crates.
    pub fn with_variation(&self, seed: u32) -> SkinColor {
        // LCG parameters (Numerical Recipes)
        let lcg = |s: u32| s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let to_f = |s: u32| (s >> 8) as f32 / 16_777_215.0; // [0, 1]

        let s0 = lcg(seed);
        let s1 = lcg(s0);
        let s2 = lcg(s1);
        let s3 = lcg(s2);

        // Vary each parameter by ±0.05
        let vary = |v: f32, r: f32| (v + (r - 0.5) * 0.10).clamp(0.0, 1.0);

        SkinColor {
            melanin: vary(self.melanin, to_f(s0)),
            hemoglobin: vary(self.hemoglobin, to_f(s1)),
            subsurface: vary(self.subsurface, to_f(s2)),
            oiliness: vary(self.oiliness, to_f(s3)),
        }
    }
}

// ── SkinColorMap ─────────────────────────────────────────────────────────────

/// A gradient of skin colors for different body regions.
#[derive(Debug, Clone)]
pub struct SkinColorMap {
    /// Default body skin color.
    pub base: SkinColor,
    /// Face skin color (may be slightly different from body).
    pub face: SkinColor,
    /// Hands skin color (often slightly darker/more worn).
    pub hands: SkinColor,
    /// Lip color (higher hemoglobin).
    pub lips: SkinColor,
    /// Nail color (slightly translucent / pink tint).
    pub nails: SkinColor,
}

impl SkinColorMap {
    /// Create a uniform map where all regions share the same color.
    pub fn uniform(color: SkinColor) -> Self {
        Self {
            base: color.clone(),
            face: color.clone(),
            hands: color.clone(),
            lips: color.clone(),
            nails: color,
        }
    }

    /// Create a map from a Fitzpatrick phototype with region-specific adjustments.
    pub fn from_fitzpatrick(t: FitzpatrickType) -> Self {
        let base = SkinColor::from_fitzpatrick(t);

        // Face: slightly lower melanin, similar hemoglobin
        let face = SkinColor {
            melanin: (base.melanin - 0.03).clamp(0.0, 1.0),
            hemoglobin: (base.hemoglobin + 0.03).clamp(0.0, 1.0),
            subsurface: (base.subsurface + 0.05).clamp(0.0, 1.0),
            oiliness: base.oiliness,
        };

        // Hands: slightly more melanin (sun exposure)
        let hands = SkinColor {
            melanin: (base.melanin + 0.05).clamp(0.0, 1.0),
            hemoglobin: base.hemoglobin,
            subsurface: base.subsurface,
            oiliness: base.oiliness,
        };

        // Lips: high hemoglobin, low melanin
        let lips = SkinColor {
            melanin: (base.melanin * 0.5).clamp(0.0, 1.0),
            hemoglobin: (base.hemoglobin + 0.40).clamp(0.0, 1.0),
            subsurface: 0.8,
            oiliness: 0.4,
        };

        // Nails: pink/translucent — low melanin, moderate hemoglobin
        let nails = SkinColor {
            melanin: (base.melanin * 0.3).clamp(0.0, 1.0),
            hemoglobin: (base.hemoglobin + 0.15).clamp(0.0, 1.0),
            subsurface: 0.6,
            oiliness: 0.6,
        };

        Self {
            base,
            face,
            hands,
            lips,
            nails,
        }
    }

    /// Apply a tan to all regions, returning a new map.
    pub fn apply_tan(&self, amount: f32) -> Self {
        Self {
            base: self.base.apply_tan(amount),
            face: self.face.apply_tan(amount),
            hands: self.hands.apply_tan(amount),
            lips: self.lips.apply_tan(amount),
            nails: self.nails.apply_tan(amount),
        }
    }
}

// ── Gamma utilities ──────────────────────────────────────────────────────────

/// Convert a linear-light value to sRGB (gamma-encoded).
///
/// Uses the IEC 61966-2-1 piecewise transfer function.
pub fn linear_to_srgb(linear: f32) -> f32 {
    let v = linear.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert an sRGB (gamma-encoded) value to linear light.
///
/// Uses the IEC 61966-2-1 piecewise transfer function.
pub fn srgb_to_linear(srgb: f32) -> f32 {
    let v = srgb.clamp(0.0, 1.0);
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitzpatrick_melanin_levels_ordered() {
        let types = FitzpatrickType::all();
        for i in 0..types.len() - 1 {
            assert!(
                types[i].melanin_level() < types[i + 1].melanin_level(),
                "melanin level not strictly ascending at index {}",
                i
            );
        }
    }

    #[test]
    fn fitzpatrick_base_rgb_type1_is_light() {
        let [r, _, _] = FitzpatrickType::Type1.base_rgb();
        assert!(r > 200, "Type1 R channel should be > 200, got {}", r);
    }

    #[test]
    fn fitzpatrick_base_rgb_type6_is_dark() {
        let [r, _, _] = FitzpatrickType::Type6.base_rgb();
        assert!(r < 100, "Type6 R channel should be < 100, got {}", r);
    }

    #[test]
    fn fitzpatrick_all_has_six_types() {
        assert_eq!(FitzpatrickType::all().len(), 6);
    }

    #[test]
    fn skin_color_from_fitzpatrick_type1() {
        let sc = SkinColor::from_fitzpatrick(FitzpatrickType::Type1);
        assert!((sc.melanin - 0.0).abs() < 1e-6, "Type1 melanin should be 0");
        assert!(sc.hemoglobin > 0.0, "hemoglobin should be positive");
    }

    #[test]
    fn skin_color_to_rgb_type1_is_light() {
        let sc = SkinColor::from_fitzpatrick(FitzpatrickType::Type1);
        let [r, _, _] = sc.to_rgb();
        assert!(r > 200, "Type1 skin R should be > 200, got {}", r);
    }

    #[test]
    fn skin_color_to_rgb_type6_is_dark() {
        let sc = SkinColor::from_fitzpatrick(FitzpatrickType::Type6);
        let [r, _, _] = sc.to_rgb();
        assert!(r < 100, "Type6 skin R should be < 100, got {}", r);
    }

    #[test]
    fn skin_color_lerp_at_zero_equals_self() {
        let a = SkinColor::from_fitzpatrick(FitzpatrickType::Type1);
        let b = SkinColor::from_fitzpatrick(FitzpatrickType::Type6);
        let result = a.lerp(&b, 0.0);
        assert!((result.melanin - a.melanin).abs() < 1e-6);
        assert!((result.hemoglobin - a.hemoglobin).abs() < 1e-6);
    }

    #[test]
    fn skin_color_lerp_at_one_equals_other() {
        let a = SkinColor::from_fitzpatrick(FitzpatrickType::Type1);
        let b = SkinColor::from_fitzpatrick(FitzpatrickType::Type6);
        let result = a.lerp(&b, 1.0);
        assert!((result.melanin - b.melanin).abs() < 1e-6);
        assert!((result.hemoglobin - b.hemoglobin).abs() < 1e-6);
    }

    #[test]
    fn skin_color_apply_tan_increases_melanin() {
        let sc = SkinColor::from_fitzpatrick(FitzpatrickType::Type2);
        let tanned = sc.apply_tan(0.1);
        assert!(tanned.melanin > sc.melanin, "tan should increase melanin");
    }

    #[test]
    fn skin_color_apply_blush_increases_hemoglobin() {
        let sc = SkinColor::from_fitzpatrick(FitzpatrickType::Type2);
        let blushed = sc.apply_blush(0.1);
        assert!(
            blushed.hemoglobin > sc.hemoglobin,
            "blush should increase hemoglobin"
        );
    }

    #[test]
    fn skin_color_to_rgba_alpha_is_255() {
        let sc = SkinColor::from_fitzpatrick(FitzpatrickType::Type3);
        let [_, _, _, a] = sc.to_rgba();
        assert_eq!(a, 255, "alpha channel should always be 255");
    }

    #[test]
    fn skin_color_map_uniform_all_same() {
        let base = SkinColor::from_fitzpatrick(FitzpatrickType::Type3);
        let map = SkinColorMap::uniform(base.clone());
        assert!((map.base.melanin - map.face.melanin).abs() < 1e-6);
        assert!((map.base.melanin - map.hands.melanin).abs() < 1e-6);
        assert!((map.base.melanin - map.lips.melanin).abs() < 1e-6);
        assert!((map.base.melanin - map.nails.melanin).abs() < 1e-6);
    }

    #[test]
    fn skin_color_map_from_fitzpatrick() {
        let map = SkinColorMap::from_fitzpatrick(FitzpatrickType::Type4);
        // Lips should have higher hemoglobin than the base
        assert!(
            map.lips.hemoglobin > map.base.hemoglobin,
            "lips should be more hemoglobin-rich than base"
        );
        // Hands should have slightly more melanin
        assert!(
            map.hands.melanin >= map.base.melanin,
            "hands melanin should be >= base"
        );
    }

    #[test]
    fn linear_to_srgb_and_back_roundtrip() {
        for &v in &[0.0_f32, 0.1, 0.5, 0.9, 1.0] {
            let encoded = linear_to_srgb(v);
            let decoded = srgb_to_linear(encoded);
            assert!(
                (decoded - v).abs() < 1e-5,
                "roundtrip failed for {}: got {}",
                v,
                decoded
            );
        }
    }

    #[test]
    fn srgb_to_linear_0_is_0() {
        assert!((srgb_to_linear(0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn srgb_to_linear_1_is_1() {
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);
    }
}
