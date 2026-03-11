// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SkinMicrostructure {
    pub furrow_depth_um: f32,
    pub plateau_height_um: f32,
    pub roughness_ra: f32,
    pub scale_pattern: u8,
}

pub fn new_skin_microstructure() -> SkinMicrostructure {
    SkinMicrostructure {
        furrow_depth_um: 50.0,
        plateau_height_um: 20.0,
        roughness_ra: 3.5,
        scale_pattern: 0,
    }
}

/// Normalized roughness (Ra / 100).
pub fn micro_roughness_index(m: &SkinMicrostructure) -> f32 {
    (m.roughness_ra / 100.0).clamp(0.0, 1.0)
}

pub fn micro_to_json(m: &SkinMicrostructure) -> String {
    format!(
        "{{\"furrow_depth_um\":{:.2},\"plateau_height_um\":{:.2},\"roughness_ra\":{:.3},\"scale_pattern\":{}}}",
        m.furrow_depth_um, m.plateau_height_um, m.roughness_ra, m.scale_pattern
    )
}

pub fn micro_is_smooth(m: &SkinMicrostructure) -> bool {
    m.roughness_ra < 5.0
}

/// Age index increases with roughness and furrow depth.
pub fn micro_age_index(m: &SkinMicrostructure) -> f32 {
    (m.roughness_ra / 50.0 + m.furrow_depth_um / 500.0).clamp(0.0, 1.0)
}

/// Estimate Fitzpatrick skin type from microstructure (rough heuristic, 1-6).
pub fn micro_skin_type_estimate(m: &SkinMicrostructure) -> u8 {
    match m.scale_pattern {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 5,
        _ => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_microstructure() {
        /* default fields */
        let m = new_skin_microstructure();
        assert!((m.roughness_ra - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_smooth_default() {
        /* default Ra < 5 => smooth */
        let m = new_skin_microstructure();
        assert!(micro_is_smooth(&m));
    }

    #[test]
    fn test_is_not_smooth() {
        /* high Ra => not smooth */
        let m = SkinMicrostructure {
            roughness_ra: 10.0,
            ..new_skin_microstructure()
        };
        assert!(!micro_is_smooth(&m));
    }

    #[test]
    fn test_to_json() {
        /* JSON contains roughness_ra */
        let m = new_skin_microstructure();
        let json = micro_to_json(&m);
        assert!(json.contains("roughness_ra"));
    }

    #[test]
    fn test_age_index_range() {
        /* age index is 0-1 */
        let m = new_skin_microstructure();
        let ai = micro_age_index(&m);
        assert!((0.0..=1.0).contains(&ai));
    }
}
