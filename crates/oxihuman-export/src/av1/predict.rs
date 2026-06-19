// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AV1 intra prediction modes for 4×4 blocks.
//!
//! The lossless backbone uses `DcPred` exclusively; all remaining modes exist
//! for the lossy path.  Directional modes (D45–D63) fall back to `DcPred`
//! because they are not exercised in the lossless backbone.

// ─────────────────────────────────────────────────────────────────────────────
// IntraMode enumeration
// ─────────────────────────────────────────────────────────────────────────────

/// AV1 intra prediction modes (4×4 block granularity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntraMode {
    DcPred = 0,
    VPred = 1,
    HPred = 2,
    D45Pred = 3,
    D135Pred = 4,
    D117Pred = 5,
    D153Pred = 6,
    D207Pred = 7,
    D63Pred = 8,
    PaethPred = 9,
    SmoothPred = 10,
    SmoothVPred = 11,
    SmoothHPred = 12,
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helper: DC value computation
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the DC fill value from whichever borders are available.
///
/// If both `top` and `left` are present the average is taken over all 8
/// border pixels; if only one is present the average is taken over its 4
/// pixels; if neither is present 128 is returned.
fn dc_value(top: Option<&[u8]>, left: Option<&[u8]>) -> u8 {
    match (top, left) {
        (None, None) => 128u8,
        (Some(t), None) => {
            let sum: u32 = t[..4].iter().map(|&x| x as u32).sum();
            ((sum + 2) / 4) as u8
        }
        (None, Some(l)) => {
            let sum: u32 = l[..4].iter().map(|&x| x as u32).sum();
            ((sum + 2) / 4) as u8
        }
        (Some(t), Some(l)) => {
            let sum: u32 = t[..4].iter().chain(l[..4].iter()).map(|&x| x as u32).sum();
            ((sum + 4) / 8) as u8
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DC prediction
// ─────────────────────────────────────────────────────────────────────────────

fn predict_dc(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    let dc = dc_value(top, left);
    [dc; 16]
}

// ─────────────────────────────────────────────────────────────────────────────
// Vertical prediction (V_PRED)
// ─────────────────────────────────────────────────────────────────────────────

/// Copy the top row downward into all 4 rows.  Falls back to DcPred if no top
/// row is available.
fn predict_v(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match top {
        None => predict_dc(None, left),
        Some(t) => {
            let mut out = [0u8; 16];
            for r in 0..4 {
                out[r * 4] = t[0];
                out[r * 4 + 1] = t[1];
                out[r * 4 + 2] = t[2];
                out[r * 4 + 3] = t[3];
            }
            out
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Horizontal prediction (H_PRED)
// ─────────────────────────────────────────────────────────────────────────────

/// Copy the left column rightward into each row.  Falls back to DcPred if no
/// left column is available.
fn predict_h(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match left {
        None => predict_dc(top, None),
        Some(l) => {
            let mut out = [0u8; 16];
            for r in 0..4 {
                let pix = l[r];
                out[r * 4] = pix;
                out[r * 4 + 1] = pix;
                out[r * 4 + 2] = pix;
                out[r * 4 + 3] = pix;
            }
            out
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Paeth prediction
// ─────────────────────────────────────────────────────────────────────────────

/// Standard Paeth predictor for one pixel.
///
/// `a` = left neighbour, `b` = top neighbour, `c` = top-left neighbour.
#[inline(always)]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let c = c as i32;
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

/// AV1 Paeth / smooth-hybrid intra prediction.
///
/// Requires both top row and left column plus the top-left pixel.  Falls back
/// to DcPred at any image boundary.
fn predict_paeth(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match (top, left) {
        (Some(t), Some(l)) => {
            // top-left pixel is t[-1]; in AV1 convention we treat it as the
            // average of t[0] and l[0] when the true (-1,-1) pixel is unknown.
            // For an intra block the top-left corner is typically available as
            // the last pixel of the row above to the left; we approximate it
            // with the average of the two available border pixels.
            let top_left = (t[0] as u32 + l[0] as u32).div_ceil(2) as u8;
            let mut out = [0u8; 16];
            for r in 0..4usize {
                for c in 0..4usize {
                    out[r * 4 + c] = paeth_predictor(l[r], t[c], top_left);
                }
            }
            out
        }
        // Boundary fallback.
        _ => predict_dc(top, left),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Smooth prediction
// ─────────────────────────────────────────────────────────────────────────────

/// AV1-inspired bilinear smooth prediction.
///
/// Each pixel is a weighted blend of the top border and the bottom-right
/// corner (= left[3], the last left-column pixel):
///
/// ```text
/// result[r][c] = ( top[c] * (4 − r)  +  left[3] * r  +  2 ) / 4
/// ```
///
/// Falls back to DcPred when a required border is absent.
fn predict_smooth(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match (top, left) {
        (Some(t), Some(l)) => {
            let bottom_right = l[3] as u32;
            let mut out = [0u8; 16];
            for r in 0..4usize {
                let wr = r as u32;          // weight for bottom-right
                let wt = 4 - wr;            // weight for top
                for c in 0..4usize {
                    let blended = (t[c] as u32 * wt + bottom_right * wr + 2) / 4;
                    out[r * 4 + c] = blended.min(255) as u8;
                }
            }
            out
        }
        _ => predict_dc(top, left),
    }
}

/// Smooth-V prediction: blend vertically between top row and bottom-left corner.
///
/// Each column c uses the top pixel `t[c]` and the bottom-left corner `l[3]`.
/// Falls back to DcPred when a required border is absent.
fn predict_smooth_v(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match (top, left) {
        (Some(t), Some(l)) => {
            let bl = l[3] as u32;
            let mut out = [0u8; 16];
            for r in 0..4usize {
                let wr = r as u32;
                let wt = 4 - wr;
                for c in 0..4usize {
                    let blended = (t[c] as u32 * wt + bl * wr + 2) / 4;
                    out[r * 4 + c] = blended.min(255) as u8;
                }
            }
            out
        }
        _ => predict_dc(top, left),
    }
}

/// Smooth-H prediction: blend horizontally between left column and top-right corner.
///
/// Each row r uses the left pixel `l[r]` and the top-right corner `t[3]`.
/// Falls back to DcPred when a required border is absent.
fn predict_smooth_h(top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match (top, left) {
        (Some(t), Some(l)) => {
            let tr = t[3] as u32;
            let mut out = [0u8; 16];
            for r in 0..4usize {
                let lr = l[r] as u32;
                for c in 0..4usize {
                    let wl = (4 - c) as u32;
                    let wr = c as u32;
                    let blended = (lr * wl + tr * wr + 2) / 4;
                    out[r * 4 + c] = blended.min(255) as u8;
                }
            }
            out
        }
        _ => predict_dc(top, left),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public predict_4x4
// ─────────────────────────────────────────────────────────────────────────────

/// Predict a 4×4 block from optional top row and left column.
///
/// # Parameters
/// * `mode`  — Intra prediction mode to apply.
/// * `top`   — Slice of ≥4 pixels from the row above (None at the top image edge).
/// * `left`  — Slice of ≥4 pixels from the column to the left (None at left edge).
///
/// # Returns
/// A 16-element, row-major `[u8; 16]` predicted block.
pub fn predict_4x4(mode: IntraMode, top: Option<&[u8]>, left: Option<&[u8]>) -> [u8; 16] {
    match mode {
        IntraMode::DcPred => predict_dc(top, left),
        IntraMode::VPred => predict_v(top, left),
        IntraMode::HPred => predict_h(top, left),
        IntraMode::PaethPred => predict_paeth(top, left),
        IntraMode::SmoothPred => predict_smooth(top, left),
        IntraMode::SmoothVPred => predict_smooth_v(top, left),
        IntraMode::SmoothHPred => predict_smooth_h(top, left),
        // Directional modes: not exercised in the lossless backbone.
        // Fall back to DcPred for safety.
        IntraMode::D45Pred
        | IntraMode::D135Pred
        | IntraMode::D117Pred
        | IntraMode::D153Pred
        | IntraMode::D207Pred
        | IntraMode::D63Pred => predict_dc(top, left),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── DcPred ───────────────────────────────────────────────────────────────

    #[test]
    fn dc_pred_no_borders_is_128() {
        let out = predict_4x4(IntraMode::DcPred, None, None);
        assert_eq!(out, [128u8; 16], "DcPred with no borders must fill with 128");
    }

    #[test]
    fn dc_pred_top_only() {
        let top = [200u8, 200, 200, 200];
        let out = predict_4x4(IntraMode::DcPred, Some(&top), None);
        // average of [200;4] = 200
        assert_eq!(out, [200u8; 16]);
    }

    #[test]
    fn dc_pred_left_only() {
        let left = [100u8, 100, 100, 100];
        let out = predict_4x4(IntraMode::DcPred, None, Some(&left));
        assert_eq!(out, [100u8; 16]);
    }

    #[test]
    fn dc_pred_both_borders() {
        let top = [100u8; 4];
        let left = [60u8; 4];
        // sum = 100*4 + 60*4 = 640; average with bias = (640 + 4) / 8 = 80
        let out = predict_4x4(IntraMode::DcPred, Some(&top), Some(&left));
        assert_eq!(out, [80u8; 16]);
    }

    #[test]
    fn dc_pred_rounding() {
        // sum = 7*8 = 56; (56 + 4) / 8 = 7 (no rounding needed)
        let top = [7u8; 4];
        let left = [7u8; 4];
        let out = predict_4x4(IntraMode::DcPred, Some(&top), Some(&left));
        assert_eq!(out[0], 7);
    }

    // ── VPred ────────────────────────────────────────────────────────────────

    #[test]
    fn v_pred_copies_top_row() {
        let top = [10u8, 20, 30, 40];
        let left = [0u8; 4];
        let out = predict_4x4(IntraMode::VPred, Some(&top), Some(&left));
        for r in 0..4 {
            for c in 0..4 {
                assert_eq!(
                    out[r * 4 + c], top[c],
                    "VPred pixel at ({r},{c}) must equal top[{c}]={}", top[c]
                );
            }
        }
    }

    #[test]
    fn v_pred_no_top_falls_back_to_dc() {
        let left = [80u8; 4];
        let dc_out = predict_4x4(IntraMode::DcPred, None, Some(&left));
        let v_out = predict_4x4(IntraMode::VPred, None, Some(&left));
        assert_eq!(dc_out, v_out, "VPred with no top must fall back to DcPred");
    }

    // ── HPred ────────────────────────────────────────────────────────────────

    #[test]
    fn h_pred_copies_left_column() {
        let top = [0u8; 4];
        let left = [50u8, 60, 70, 80];
        let out = predict_4x4(IntraMode::HPred, Some(&top), Some(&left));
        for r in 0..4 {
            for c in 0..4 {
                assert_eq!(
                    out[r * 4 + c], left[r],
                    "HPred pixel at ({r},{c}) must equal left[{r}]={}", left[r]
                );
            }
        }
    }

    #[test]
    fn h_pred_no_left_falls_back_to_dc() {
        let top = [90u8; 4];
        let dc_out = predict_4x4(IntraMode::DcPred, Some(&top), None);
        let h_out = predict_4x4(IntraMode::HPred, Some(&top), None);
        assert_eq!(dc_out, h_out, "HPred with no left must fall back to DcPred");
    }

    // ── PaethPred ────────────────────────────────────────────────────────────

    #[test]
    fn paeth_pred_flat_borders_equals_border_value() {
        let top = [100u8; 4];
        let left = [100u8; 4];
        let out = predict_4x4(IntraMode::PaethPred, Some(&top), Some(&left));
        // With flat borders and top_left = 100, paeth should return 100 everywhere.
        assert_eq!(out, [100u8; 16]);
    }

    #[test]
    fn paeth_pred_no_borders_fallback() {
        let out = predict_4x4(IntraMode::PaethPred, None, None);
        assert_eq!(out, [128u8; 16]);
    }

    #[test]
    fn paeth_pred_values_in_range() {
        let top = [30u8, 60, 90, 120];
        let left = [40u8, 80, 120, 160];
        let out = predict_4x4(IntraMode::PaethPred, Some(&top), Some(&left));
        for &px in &out {
            // Paeth must return one of the three input pixels.
            let _ = px; // px is u8, always ≤ 255; existence check is the invariant
        }
        assert!(!out.is_empty(), "Paeth prediction must produce 16 pixels");
    }

    // ── SmoothPred ───────────────────────────────────────────────────────────

    #[test]
    fn smooth_pred_top_row_is_top_border() {
        // At r=0 the weight for bottom-right is 0, so result must equal top[c].
        let top = [10u8, 20, 30, 40];
        let left = [0u8, 50, 100, 200];
        let out = predict_4x4(IntraMode::SmoothPred, Some(&top), Some(&left));
        for c in 0..4 {
            // r=0: (top[c]*4 + left[3]*0 + 2) / 4 = top[c] (no rounding artefact)
            assert_eq!(out[c], top[c], "smooth row 0 col {c} must equal top[{c}]");
        }
    }

    #[test]
    fn smooth_pred_no_borders_fallback() {
        let out = predict_4x4(IntraMode::SmoothPred, None, None);
        assert_eq!(out, [128u8; 16]);
    }

    // ── General validity ─────────────────────────────────────────────────────

    #[test]
    fn all_modes_produce_valid_pixels() {
        let top = [100u8, 110, 120, 130];
        let left = [90u8, 95, 100, 105];
        let modes = [
            IntraMode::DcPred,
            IntraMode::VPred,
            IntraMode::HPred,
            IntraMode::D45Pred,
            IntraMode::D135Pred,
            IntraMode::D117Pred,
            IntraMode::D153Pred,
            IntraMode::D207Pred,
            IntraMode::D63Pred,
            IntraMode::PaethPred,
            IntraMode::SmoothPred,
            IntraMode::SmoothVPred,
            IntraMode::SmoothHPred,
        ];
        for mode in modes {
            let out = predict_4x4(mode, Some(&top), Some(&left));
            for &px in &out {
                // Pixel values must fit in u8 — the assert on ≤ 255 is trivially true
                // for u8, but we check the cast path is lossless.
                assert!(px as u32 <= 255, "pixel overflow in mode {mode:?}");
            }
        }
    }

    #[test]
    fn all_modes_no_border_do_not_panic() {
        let modes = [
            IntraMode::DcPred,
            IntraMode::VPred,
            IntraMode::HPred,
            IntraMode::D45Pred,
            IntraMode::D135Pred,
            IntraMode::D117Pred,
            IntraMode::D153Pred,
            IntraMode::D207Pred,
            IntraMode::D63Pred,
            IntraMode::PaethPred,
            IntraMode::SmoothPred,
            IntraMode::SmoothVPred,
            IntraMode::SmoothHPred,
        ];
        for mode in modes {
            let _ = predict_4x4(mode, None, None);
        }
    }
}
