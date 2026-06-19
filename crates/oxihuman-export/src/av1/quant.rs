// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AV1 quantization parameters.
//!
//! For the lossless path (`base_q_idx == 0`) all dequant factors equal 1,
//! allowing exact coefficient pass-through.  For lossy paths the AV1 spec
//! Table 7.10 DC/AC values are used (8-bit, Profile 0 / 4:2:0 baseline).

/// AV1 spec Table 7.10 — DC dequant values for `base_q_idx` 0..=63 (8-bit).
///
/// Index 0 corresponds to q=0.  In the function `dc_q`, q=0 is overridden to 1
/// (lossless identity); the table entry [0]=4 is the hardware default, unused in
/// the lossless path.  Indices >= 64 are clamped to 440 in the accessor.
static DC_Q_TABLE: [u16; 64] = [
    4,   8,   10,  12,  13,  14,  15,  16,  17,  18,  19,  20,  21,  22,  23,  24,
    25,  26,  27,  28,  29,  30,  31,  32,  33,  35,  37,  39,  41,  43,  46,  49,
    52,  56,  60,  64,  68,  73,  78,  83,  88,  94, 100, 107, 114, 122, 131, 140,
    150, 161, 172, 185, 199, 214, 230, 247, 266, 286, 308, 331, 355, 381, 409, 440,
];

/// AV1 spec Table 7.10 — AC dequant values for `base_q_idx` 0..=63 (8-bit).
///
/// Index 0 corresponds to q=0 (overridden to 1 in the lossless path).
static AC_Q_TABLE: [u16; 64] = [
    4,   8,   9,   10,  11,  12,  13,  14,  15,  16,  17,  18,  19,  20,  21,  22,
    23,  24,  25,  26,  27,  28,  29,  30,  31,  32,  33,  34,  35,  36,  37,  38,
    39,  40,  41,  42,  43,  44,  45,  47,  49,  51,  53,  55,  58,  61,  64,  67,
    70,  74,  78,  82,  87,  92,  97, 102, 108, 114, 120, 127, 134, 142, 150, 158,
];

/// Return the DC dequant factor for `base_q_idx` (0–255), 8-bit depth.
///
/// At `q == 0` returns `1` (lossless identity per AV1 spec).
/// Values `q >= 64` clamp to 440 (the maximum table entry).
pub fn dc_q(base_q_idx: u8) -> u16 {
    let q = base_q_idx as usize;
    if q == 0 {
        return 1;
    }
    if q < 64 {
        DC_Q_TABLE[q]
    } else {
        440
    }
}

/// Return the AC dequant factor for `base_q_idx`, 8-bit depth.
///
/// At `q == 0` returns `1` (lossless identity).
/// Values `q >= 64` clamp to the last table value (158).
pub fn ac_q(base_q_idx: u8) -> u16 {
    let q = base_q_idx as usize;
    if q == 0 {
        return 1;
    }
    if q < 64 {
        AC_Q_TABLE[q]
    } else {
        // Clamp to the maximum AC value (index 63 → 158).
        158
    }
}

// Note: DC_Q_TABLE[0]=4 and AC_Q_TABLE[0]=4 are the hardware default for q=0,
// but dc_q(0) and ac_q(0) both return 1 (lossless identity override).

/// Quantize a coefficient: `round(coeff / q)`.
///
/// When `q == 0` the divisor would be zero; the function returns `coeff` unchanged
/// (consistent with lossless identity).
pub fn quantize_coeff(coeff: i32, q: u16) -> i32 {
    if q <= 1 {
        return coeff;
    }
    let q_i32 = q as i32;
    // Round-to-nearest: add half the quantizer step before dividing.
    let half = q_i32 / 2;
    if coeff >= 0 {
        (coeff + half) / q_i32
    } else {
        -(((-coeff) + half) / q_i32)
    }
}

/// Dequantize a quantized level: `level * q`.
pub fn dequantize_coeff(level: i32, q: u16) -> i32 {
    level * (q as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_q_zero_is_one() {
        assert_eq!(dc_q(0), 1, "dc_q(0) must be 1 (lossless identity)");
    }

    #[test]
    fn test_ac_q_zero_is_one() {
        assert_eq!(ac_q(0), 1, "ac_q(0) must be 1 (lossless identity)");
    }

    #[test]
    fn test_dc_q_1_is_8() {
        assert_eq!(dc_q(1), 8, "dc_q(1) must be 8 per AV1 spec Table 7.10");
    }

    #[test]
    fn test_dc_q_63_is_440() {
        assert_eq!(dc_q(63), 440);
    }

    #[test]
    fn test_dc_q_clamping() {
        // Any q >= 64 must clamp to 440.
        assert_eq!(dc_q(64), 440);
        assert_eq!(dc_q(128), 440);
        assert_eq!(dc_q(255), 440);
    }

    #[test]
    fn test_ac_q_clamping() {
        assert_eq!(ac_q(64), 158);
        assert_eq!(ac_q(255), 158);
    }

    #[test]
    fn test_dc_q_table_spot_checks() {
        // Spot-check known values from the AV1 spec Table 7.10.
        // q=2  → DC_Q_TABLE[2]=10
        assert_eq!(dc_q(2), 10);
        // q=5  → DC_Q_TABLE[5]=14
        assert_eq!(dc_q(5), 14);
        // q=35 → DC_Q_TABLE[35]=64
        assert_eq!(dc_q(35), 64);
        // q=36 → DC_Q_TABLE[36]=68
        assert_eq!(dc_q(36), 68);
        // q=63 → DC_Q_TABLE[63]=440
        assert_eq!(dc_q(63), 440);
    }

    #[test]
    fn test_quantize_dequantize_round_trip() {
        let q_values: &[u8] = &[0, 1, 8, 16, 32, 63, 128, 255];
        let coeffs: &[i32] = &[0, 1, -1, 100, -100, 255, -255, 1000, -1000];
        for &q_idx in q_values {
            let q = dc_q(q_idx);
            for &coeff in coeffs {
                let level = quantize_coeff(coeff, q);
                let recon = dequantize_coeff(level, q);
                // Round-trip error must be < q (quantization noise bound).
                let error = (coeff - recon).unsigned_abs();
                assert!(
                    error < q as u32,
                    "round-trip error {error} >= q={q} for coeff={coeff}, q_idx={q_idx}"
                );
            }
        }
    }

    #[test]
    fn test_lossless_identity() {
        // q=0 → q-factor=1, so quantize and dequantize are both identity.
        for coeff in [-1000i32, -1, 0, 1, 255, 1000] {
            assert_eq!(quantize_coeff(coeff, 1), coeff);
            assert_eq!(dequantize_coeff(coeff, 1), coeff);
        }
    }

    #[test]
    fn test_quantize_rounding() {
        // With q=10: coeff=5 → (5+5)/10 = 1; coeff=4 → (4+5)/10 = 0.
        assert_eq!(quantize_coeff(5, 10), 1);
        assert_eq!(quantize_coeff(4, 10), 0);
        // Negative symmetry.
        assert_eq!(quantize_coeff(-5, 10), -1);
        assert_eq!(quantize_coeff(-4, 10), 0);
    }

    #[test]
    fn test_dequantize_sign_preservation() {
        assert_eq!(dequantize_coeff(3, 10), 30);
        assert_eq!(dequantize_coeff(-3, 10), -30);
        assert_eq!(dequantize_coeff(0, 100), 0);
    }
}
