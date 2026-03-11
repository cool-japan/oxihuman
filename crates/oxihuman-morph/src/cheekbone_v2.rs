// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Zygomatic arch and malar prominence control (v2).

#![allow(dead_code)]

/// Parameters controlling zygomatic arch shape.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekboneV2Params {
    /// Malar prominence: 0.0 = flat, 1.0 = maximum projection.
    pub prominence: f32,
    /// Zygomatic arch width: 0.0 = narrow, 1.0 = wide.
    pub arch_width: f32,
    /// Anterior–posterior depth: 0.0 = shallow, 1.0 = deep.
    pub ap_depth: f32,
    /// Superior–inferior position: -1.0 = low, 1.0 = high.
    pub vertical_pos: f32,
}

#[allow(dead_code)]
impl Default for CheekboneV2Params {
    fn default() -> Self {
        Self {
            prominence: 0.5,
            arch_width: 0.5,
            ap_depth: 0.5,
            vertical_pos: 0.0,
        }
    }
}

/// Result weights applied to the morph rig.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekboneV2Weights {
    pub malar_prominence: f32,
    pub arch_spread: f32,
    pub depth_forward: f32,
    pub vertical_shift: f32,
}

/// Create default cheekbone v2 params.
#[allow(dead_code)]
pub fn default_cheekbone_v2() -> CheekboneV2Params {
    CheekboneV2Params::default()
}

/// Evaluate morph weights from params.
#[allow(dead_code)]
pub fn evaluate_cheekbone_v2(p: &CheekboneV2Params) -> CheekboneV2Weights {
    CheekboneV2Weights {
        malar_prominence: p.prominence.clamp(0.0, 1.0),
        arch_spread: p.arch_width.clamp(0.0, 1.0),
        depth_forward: p.ap_depth.clamp(0.0, 1.0),
        vertical_shift: p.vertical_pos.clamp(-1.0, 1.0),
    }
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_cheekbone_v2(
    a: &CheekboneV2Params,
    b: &CheekboneV2Params,
    t: f32,
) -> CheekboneV2Params {
    let t = t.clamp(0.0, 1.0);
    CheekboneV2Params {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        arch_width: a.arch_width + (b.arch_width - a.arch_width) * t,
        ap_depth: a.ap_depth + (b.ap_depth - a.ap_depth) * t,
        vertical_pos: a.vertical_pos + (b.vertical_pos - a.vertical_pos) * t,
    }
}

/// Set prominence, clamping to valid range.
#[allow(dead_code)]
pub fn set_zygomatic_prominence(p: &mut CheekboneV2Params, value: f32) {
    p.prominence = value.clamp(0.0, 1.0);
}

/// Set arch width, clamping to valid range.
#[allow(dead_code)]
pub fn set_arch_width(p: &mut CheekboneV2Params, value: f32) {
    p.arch_width = value.clamp(0.0, 1.0);
}

/// Check if params are within valid ranges.
#[allow(dead_code)]
pub fn is_valid_cheekbone_v2(p: &CheekboneV2Params) -> bool {
    (0.0..=1.0).contains(&p.prominence)
        && (0.0..=1.0).contains(&p.arch_width)
        && (0.0..=1.0).contains(&p.ap_depth)
        && (-1.0..=1.0).contains(&p.vertical_pos)
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_cheekbone_v2(p: &mut CheekboneV2Params) {
    *p = CheekboneV2Params::default();
}

/// Serialize to JSON string.
#[allow(dead_code)]
pub fn cheekbone_v2_to_json(p: &CheekboneV2Params) -> String {
    format!(
        r#"{{"prominence":{:.4},"arch_width":{:.4},"ap_depth":{:.4},"vertical_pos":{:.4}}}"#,
        p.prominence, p.arch_width, p.ap_depth, p.vertical_pos
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = CheekboneV2Params::default();
        assert!((p.prominence - 0.5).abs() < 1e-6);
        assert!((p.arch_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_clamping() {
        let p = CheekboneV2Params {
            prominence: 1.5,
            arch_width: -0.5,
            ap_depth: 0.5,
            vertical_pos: 0.0,
        };
        let w = evaluate_cheekbone_v2(&p);
        assert!((w.malar_prominence - 1.0).abs() < 1e-6);
        assert!(w.arch_spread < 1e-6);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = CheekboneV2Params {
            prominence: 0.0,
            arch_width: 0.0,
            ap_depth: 0.0,
            vertical_pos: 0.0,
        };
        let b = CheekboneV2Params {
            prominence: 1.0,
            arch_width: 1.0,
            ap_depth: 1.0,
            vertical_pos: 1.0,
        };
        let m = blend_cheekbone_v2(&a, &b, 0.5);
        assert!((m.prominence - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_prominence() {
        let mut p = CheekboneV2Params::default();
        set_zygomatic_prominence(&mut p, 2.0);
        assert!((p.prominence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch_width() {
        let mut p = CheekboneV2Params::default();
        set_arch_width(&mut p, -1.0);
        assert!(p.arch_width < 1e-6);
    }

    #[test]
    fn test_is_valid_default() {
        assert!(is_valid_cheekbone_v2(&CheekboneV2Params::default()));
    }

    #[test]
    fn test_is_invalid() {
        let p = CheekboneV2Params {
            prominence: 1.5,
            arch_width: 0.5,
            ap_depth: 0.5,
            vertical_pos: 0.0,
        };
        assert!(!is_valid_cheekbone_v2(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = CheekboneV2Params {
            prominence: 0.8,
            arch_width: 0.9,
            ap_depth: 0.2,
            vertical_pos: -0.3,
        };
        reset_cheekbone_v2(&mut p);
        assert!((p.prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let p = CheekboneV2Params::default();
        let j = cheekbone_v2_to_json(&p);
        assert!(j.contains("prominence"));
        assert!(j.contains("arch_width"));
    }
}
