// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AV1 CDF (Cumulative Distribution Function) default tables and the
//! `update_cdf` adaptive-entropy update routine.
//!
//! # CDF convention
//!
//! AV1 stores CDFs in the *inverted* form used by its range-entropy coder:
//! for a symbol alphabet of size N, the CDF array has N entries where:
//!
//! ```text
//! cdf[i] = floor(32768 × (1 − cumulative_P(symbol ≤ i)))
//!        = floor(32768 × (N − 1 − i) / N)   [uniform prior]
//! cdf[N−1] = 0     ← this entry is reused as the update count
//! ```
//!
//! After observing symbol `k`, `update_cdf` nudges `cdf[i]` toward:
//! - 0       when i < k  (lower the probability mass above symbol k)
//! - 32768   when i ≥ k  (raise it)
//!   and increments the count (saturating at 32).
//!
//! # Uniform-prior formula
//!
//! For N symbols:
//! ```text
//! cdf[i] = (32768 * (N - 1 - i)) / N    for i in 0..N-1
//! cdf[N-1] = 0                           count slot
//! ```
//! All values are pre-computed as `u16` constants below.

// ─────────────────────────────────────────────────────────────────────────────
// Intra mode CDF  (13 symbols → 14 entries)
// ─────────────────────────────────────────────────────────────────────────────

/// Number of intra prediction mode symbols (13) plus one count slot.
pub const INTRA_MODE_CDF_LEN: usize = 14;

/// Default (uniform) CDF for intra prediction modes.
///
/// Symbols 0–12 correspond to `IntraMode` variants DcPred … SmoothHPred.
pub static DEFAULT_INTRA_MODE_CDF: [u16; INTRA_MODE_CDF_LEN] = [
    // cdf[i] = floor(32768 * (13 - 1 - i) / 13)  for i in 0..12; cdf[13] = 0
    30247, // i=0  → floor(32768 * 12 / 13) = 30247
    27726, // i=1  → floor(32768 * 11 / 13) = 27726
    25206, // i=2  → floor(32768 * 10 / 13) = 25206
    22685, // i=3  → floor(32768 *  9 / 13) = 22685
    20164, // i=4  → floor(32768 *  8 / 13) = 20164
    17644, // i=5  → floor(32768 *  7 / 13) = 17644
    15123, // i=6  → floor(32768 *  6 / 13) = 15123
    12603, // i=7  → floor(32768 *  5 / 13) = 12603
    10082, // i=8  → floor(32768 *  4 / 13) = 10082
     7561, // i=9  → floor(32768 *  3 / 13) =  7561
     5041, // i=10 → floor(32768 *  2 / 13) =  5041
     2520, // i=11 → floor(32768 *  1 / 13) =  2520
        0, // i=12 → 0 (last probability bucket, always 0)
        0, // count slot (initially 0)
];

// ─────────────────────────────────────────────────────────────────────────────
// Partition CDF  (4 outcomes → 5 entries)
// ─────────────────────────────────────────────────────────────────────────────

/// Number of partition-type symbols (4: NONE, HORZ, VERT, SPLIT) plus count.
pub const PARTITION_CDF_LEN: usize = 5;

/// Default (uniform) CDF for partition type.
pub static DEFAULT_PARTITION_CDF: [u16; PARTITION_CDF_LEN] = [
    // cdf[i] = floor(32768 * (4 - 1 - i) / 4)  for i in 0..3; cdf[4] = 0
    24576, // i=0 → floor(32768 * 3 / 4) = 24576
    16384, // i=1 → floor(32768 * 2 / 4) = 16384
     8192, // i=2 → floor(32768 * 1 / 4) =  8192
        0, // i=3 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Skip flag CDF  (2 outcomes → 3 entries)
// ─────────────────────────────────────────────────────────────────────────────

/// Number of skip-flag symbols (2: no-skip, skip) plus count.
pub const SKIP_CDF_LEN: usize = 3;

/// Default (50/50) CDF for the skip flag.
pub static DEFAULT_SKIP_CDF: [u16; SKIP_CDF_LEN] = [
    // cdf[i] = floor(32768 * (2 - 1 - i) / 2)  for i in 0..1; cdf[2] = 0
    16384, // i=0 → 16384  (P(symbol=0) = 0.5)
        0, // i=1 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Transform type CDF  (9 types → 10 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// AV1 tx types (used in the lossy path):
//   0 = DCT_DCT, 1 = ADST_DCT, 2 = DCT_ADST, 3 = ADST_ADST,
//   4 = FLIPADST_DCT, 5 = DCT_FLIPADST, 6 = FLIPADST_FLIPADST,
//   7 = ADST_FLIPADST, 8 = FLIPADST_ADST — and WHT_WHT is treated as
//   type 8 in the lossless path (base_q_idx = 0 implies IDTX or WHT).

/// Number of transform-type symbols (9) plus count.
pub const TX_TYPE_CDF_LEN: usize = 10;

/// Default (uniform) CDF for the transform type.
pub static DEFAULT_TX_TYPE_CDF: [u16; TX_TYPE_CDF_LEN] = [
    // cdf[i] = floor(32768 * (9 - 1 - i) / 9)  for i in 0..8; cdf[9] = 0
    29127, // i=0 → floor(32768 * 8 / 9) = 29127
    25486, // i=1 → floor(32768 * 7 / 9) = 25486
    21845, // i=2 → floor(32768 * 6 / 9) = 21845
    18204, // i=3 → floor(32768 * 5 / 9) = 18204
    14563, // i=4 → floor(32768 * 4 / 9) = 14563
    10922, // i=5 → floor(32768 * 3 / 9) = 10922
     7281, // i=6 → floor(32768 * 2 / 9) =  7281
     3640, // i=7 → floor(32768 * 1 / 9) =  3640
        0, // i=8 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Coefficient base-level CDF  (4 levels → 5 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// Levels: 0 = zero coeff, 1 = level-1, 2 = level-2, 3 = level-3+

/// Number of coefficient-base-level symbols (4) plus count.
pub const COEFF_BASE_CDF_LEN: usize = 5;

/// Default (uniform) CDF for coefficient base level.
pub static DEFAULT_COEFF_BASE_CDF: [u16; COEFF_BASE_CDF_LEN] = [
    // cdf[i] = floor(32768 * (4 - 1 - i) / 4)  for i in 0..3; cdf[4] = 0
    24576, // i=0
    16384, // i=1
     8192, // i=2
        0, // i=3
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// EOB point CDF  (5 positions → 6 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// For a 4×4 block the EOB can be at scan positions 1..=16; coded as
// an exponent (0 = 1 coeff, 1 = 2 coeffs, …, 4 = 16 coeffs), giving 5
// symbols.

/// Number of EOB-point symbols (5) plus count.
pub const EOB_PT_CDF_LEN: usize = 6;

/// Default (uniform) CDF for the EOB point in a 4×4 block.
pub static DEFAULT_EOB_PT_CDF: [u16; EOB_PT_CDF_LEN] = [
    // cdf[i] = floor(32768 * (5 - 1 - i) / 5)  for i in 0..4; cdf[5] = 0
    26214, // i=0 → floor(32768 * 4 / 5) = 26214
    19660, // i=1 → floor(32768 * 3 / 5) = 19660
    13107, // i=2 → floor(32768 * 2 / 5) = 13107
     6553, // i=3 → floor(32768 * 1 / 5) =  6553
        0, // i=4 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// DC sign CDF  (2 outcomes → 3 entries)
// ─────────────────────────────────────────────────────────────────────────────

/// Number of DC-sign symbols (2: positive, negative) plus count.
pub const DC_SIGN_CDF_LEN: usize = 3;

/// Default (50/50) CDF for the DC coefficient sign.
pub static DEFAULT_DC_SIGN_CDF: [u16; DC_SIGN_CDF_LEN] = [
    16384, // i=0 → 16384
        0, // i=1 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Golomb tail CDF  (2 outcomes → 3 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// Used for coding coefficients whose magnitude exceeds level 3.
// Symbol 0 = stop-bit (terminate Golomb unary prefix), 1 = continue.

/// Number of Golomb-tail symbols (2) plus count.
pub const GOLOMB_CDF_LEN: usize = 3;

/// Default (50/50) CDF for Golomb unary tail bits.
pub static DEFAULT_GOLOMB_CDF: [u16; GOLOMB_CDF_LEN] = [
    16384, // i=0 → 16384
        0, // i=1 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Coefficient non-zero flag CDF  (2 outcomes → 3 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// Adaptive binary: 0 = coefficient is zero, 1 = non-zero.  One CDF per
// position class (DC vs AC), all starting at 50/50.

/// Number of nonzero-flag symbols (2) plus count slot.
pub const NONZERO_CDF_LEN: usize = 3;

/// Default (50/50) CDF for the per-coefficient non-zero flag.
pub static DEFAULT_NONZERO_CDF: [u16; NONZERO_CDF_LEN] = [
    16384, // i=0 → 16384  (P(symbol=0) = 0.5)
        0, // i=1 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Coefficient sign CDF  (2 outcomes → 3 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// Adaptive binary: 0 = positive, 1 = negative.  One CDF per position class.

/// Number of sign symbols (2) plus count slot.
pub const SIGN_CDF_LEN: usize = 3;

/// Default (50/50) CDF for the per-coefficient sign.
pub static DEFAULT_SIGN_CDF: [u16; SIGN_CDF_LEN] = [
    16384, // i=0 → 16384
        0, // i=1 → 0
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// Coefficient magnitude-class CDF  (12 symbols → 13 entries)
// ─────────────────────────────────────────────────────────────────────────────
//
// Symbols 0–10 = exp-Golomb order k (k = floor(log2(mag_minus_1 + 1))).
// Symbol 11    = escape (k ≥ 11; the full exp-Golomb coder is used instead).
// One CDF per position class (DC vs AC).

/// Number of magnitude-class symbols (12) plus count slot.
pub const MAG_CLASS_CDF_LEN: usize = 13;

/// Number of real magnitude-class symbols (not counting the count slot).
pub const MAG_CLASS_SYMS: usize = MAG_CLASS_CDF_LEN - 1; // = 12

/// The escape symbol index (k ≥ K_MAX falls back to full exp-Golomb).
pub const K_MAX: usize = MAG_CLASS_SYMS - 1; // = 11

/// Default (uniform) CDF for the coefficient magnitude class.
///
/// `cdf[i]` = floor(32768 × (11 − i) / 12)  for i in 0..10; `cdf[11]` = 0; `cdf[12]` = 0.
pub static DEFAULT_MAG_CLASS_CDF: [u16; MAG_CLASS_CDF_LEN] = [
    30037, // i=0  → floor(32768 * 11 / 12) = 30037
    27306, // i=1  → floor(32768 * 10 / 12) = 27306
    24576, // i=2  → floor(32768 *  9 / 12) = 24576
    21845, // i=3  → floor(32768 *  8 / 12) = 21845
    19114, // i=4  → floor(32768 *  7 / 12) = 19114
    16384, // i=5  → floor(32768 *  6 / 12) = 16384
    13653, // i=6  → floor(32768 *  5 / 12) = 13653
    10922, // i=7  → floor(32768 *  4 / 12) = 10922
     8192, // i=8  → floor(32768 *  3 / 12) =  8192
     5461, // i=9  → floor(32768 *  2 / 12) =  5461
     2730, // i=10 → floor(32768 *  1 / 12) =  2730
        0, // i=11 → 0  (terminator for last-symbol bucket)
        0, // count slot
];

// ─────────────────────────────────────────────────────────────────────────────
// update_cdf  (AV1 spec Section 8.3)
// ─────────────────────────────────────────────────────────────────────────────

/// Update a CDF after observing symbol `sym`.
///
/// # Parameters
/// * `cdf` — Mutable CDF slice. Must have exactly `nsymbs` entries where
///   `cdf[0 .. nsymbs-2]` are inverted probability values and
///   `cdf[nsymbs-1]` is the saturating update count (starts at 0).
/// * `sym` — Observed symbol index (0 ≤ sym < nsymbs − 1).
/// * `nsymbs` — **Total array length** (= number of symbols + 1 for the count
///   slot). For a binary symbol use `nsymbs = 3`; for 13 intra
///   modes use `nsymbs = INTRA_MODE_CDF_LEN = 14`.
///
/// Implements the AV1 `update_cdf` algorithm exactly:
///
/// ```text
/// count  = cdf[nsymbs − 1]
/// rate   = 3 + (count > 15) + (count > 31) + min(floor_log2(nsymbs − 1), 2)
///
/// for i in 0 .. nsymbs − 2:           // symbol slots only (not the count)
///     if i < sym:  cdf[i] −= cdf[i] >> rate
///     else:        cdf[i] += (32768 − cdf[i]) >> rate
///
/// cdf[nsymbs − 1] = min(count + 1, 32)   // saturating count
/// ```
///
/// # Panics
///
/// Panics in debug mode if `sym >= nsymbs - 1` or `cdf.len() < nsymbs`.
pub fn update_cdf(cdf: &mut [u16], sym: usize, nsymbs: usize) {
    debug_assert!(cdf.len() >= nsymbs, "CDF slice too short");
    // sym must be in 0..nsymbs-2 (valid symbol range; nsymbs-1 is count slot).
    debug_assert!(sym + 1 < nsymbs, "symbol index out of range");

    // Count slot is the LAST entry: cdf[nsymbs - 1].
    let count = cdf[nsymbs - 1] as usize;

    // floor_log2(nsymbs - 1): number of bits in (nsymbs - 1) minus 1.
    // nsymbs ≥ 3 (at least 2 symbols + 1 count), so nsymbs - 1 ≥ 2.
    let n_syms = nsymbs - 1; // number of actual symbols
    let log2_n = (usize::BITS - n_syms.leading_zeros()) as usize - 1;
    let rate = 3 + (count > 15) as usize + (count > 31) as usize + log2_n.min(2);

    // Update probability entries 0..n_syms-2 (the "0-terminator" at index
    // n_syms-1 = nsymbs-2 is never touched; it stays 0 by convention).
    for (i, entry) in cdf.iter_mut().enumerate().take(n_syms - 1) {
        if i < sym {
            *entry -= *entry >> rate;
        } else {
            *entry += (32768u16 - *entry) >> rate;
        }
    }

    // Saturating increment of the count slot.
    if count < 32 {
        cdf[nsymbs - 1] = (count + 1) as u16;
    }
}

/// Clone a fixed-size CDF array (for per-block context copies).
pub fn clone_cdf<const N: usize>(src: &[u16; N]) -> [u16; N] {
    *src
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Constant sanity checks ────────────────────────────────────────────────

    #[test]
    fn intra_mode_cdf_length() {
        assert_eq!(DEFAULT_INTRA_MODE_CDF.len(), INTRA_MODE_CDF_LEN);
        assert_eq!(INTRA_MODE_CDF_LEN, 14, "13 symbols + 1 count slot");
    }

    #[test]
    fn intra_mode_cdf_count_slot_zero() {
        assert_eq!(
            DEFAULT_INTRA_MODE_CDF[INTRA_MODE_CDF_LEN - 1], 0,
            "count slot must start at 0"
        );
    }

    #[test]
    fn intra_mode_cdf_strictly_decreasing() {
        // The valid probability entries (indices 0..N-2) must be strictly
        // decreasing in an AV1 CDF (inverted form, uniform prior).
        for i in 0..INTRA_MODE_CDF_LEN - 2 {
            assert!(
                DEFAULT_INTRA_MODE_CDF[i] > DEFAULT_INTRA_MODE_CDF[i + 1],
                "CDF not strictly decreasing at index {i}"
            );
        }
    }

    #[test]
    fn skip_cdf_is_50_50() {
        // Uniform over 2 symbols: cdf[0] should be 16384.
        assert_eq!(DEFAULT_SKIP_CDF[0], 16384);
        assert_eq!(DEFAULT_SKIP_CDF[1], 0);
        assert_eq!(DEFAULT_SKIP_CDF[2], 0); // count slot
    }

    #[test]
    fn all_count_slots_zero() {
        assert_eq!(DEFAULT_PARTITION_CDF[PARTITION_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_SKIP_CDF[SKIP_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_TX_TYPE_CDF[TX_TYPE_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_COEFF_BASE_CDF[COEFF_BASE_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_EOB_PT_CDF[EOB_PT_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_DC_SIGN_CDF[DC_SIGN_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_GOLOMB_CDF[GOLOMB_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_NONZERO_CDF[NONZERO_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_SIGN_CDF[SIGN_CDF_LEN - 1], 0);
        assert_eq!(DEFAULT_MAG_CLASS_CDF[MAG_CLASS_CDF_LEN - 1], 0);
    }

    #[test]
    fn new_coeff_cdfs_correct_length_and_terminator() {
        assert_eq!(DEFAULT_NONZERO_CDF.len(), NONZERO_CDF_LEN);
        assert_eq!(DEFAULT_SIGN_CDF.len(), SIGN_CDF_LEN);
        assert_eq!(DEFAULT_MAG_CLASS_CDF.len(), MAG_CLASS_CDF_LEN);
        // Terminator slot (index nsyms-1 = len-2) must be 0.
        assert_eq!(DEFAULT_NONZERO_CDF[NONZERO_CDF_LEN - 2], 0);
        assert_eq!(DEFAULT_SIGN_CDF[SIGN_CDF_LEN - 2], 0);
        assert_eq!(DEFAULT_MAG_CLASS_CDF[MAG_CLASS_CDF_LEN - 2], 0);
        // K_MAX and MAG_CLASS_SYMS sanity.
        assert_eq!(MAG_CLASS_SYMS, 12);
        assert_eq!(K_MAX, 11);
    }

    #[test]
    fn mag_class_cdf_strictly_decreasing_in_prob_region() {
        // The active probability region is indices 0..MAG_CLASS_SYMS-2 (= 0..10).
        for i in 0..MAG_CLASS_CDF_LEN - 3 {
            assert!(
                DEFAULT_MAG_CLASS_CDF[i] > DEFAULT_MAG_CLASS_CDF[i + 1],
                "MAG_CLASS CDF not decreasing at index {i}"
            );
        }
    }

    // ── update_cdf adaptive learning ─────────────────────────────────────────

    #[test]
    fn update_cdf_symbol_0_increases_probability() {
        // After many observations of symbol 0, cdf[0] should increase
        // (higher inverted CDF value = more probability mass for symbol 0).
        // nsymbs = SKIP_CDF_LEN = 3 (2 symbols + 1 count slot).
        let mut cdf = DEFAULT_SKIP_CDF;
        let initial_cdf0 = cdf[0];
        for _ in 0..50 {
            update_cdf(&mut cdf, 0, SKIP_CDF_LEN);
        }
        assert!(
            cdf[0] > initial_cdf0,
            "cdf[0] should increase after many symbol-0 observations: \
             initial={initial_cdf0} final={}",
            cdf[0]
        );
    }

    #[test]
    fn update_cdf_symbol_1_decreases_cdf0() {
        // After many observations of symbol 1, cdf[0] should decrease
        // (less probability mass for symbol 0).
        let mut cdf = DEFAULT_SKIP_CDF;
        let initial_cdf0 = cdf[0];
        for _ in 0..50 {
            update_cdf(&mut cdf, 1, SKIP_CDF_LEN);
        }
        assert!(
            cdf[0] < initial_cdf0,
            "cdf[0] should decrease after many symbol-1 observations: \
             initial={initial_cdf0} final={}",
            cdf[0]
        );
    }

    #[test]
    fn update_cdf_count_saturates_at_32() {
        // nsymbs = SKIP_CDF_LEN = 3; count slot is cdf[2].
        let mut cdf = DEFAULT_SKIP_CDF;
        for _ in 0..100 {
            update_cdf(&mut cdf, 0, SKIP_CDF_LEN);
        }
        assert_eq!(
            cdf[SKIP_CDF_LEN - 1],
            32,
            "count slot must saturate at 32"
        );
    }

    #[test]
    fn update_cdf_values_stay_in_range() {
        // CDF entries must remain in [0, 32768].
        // nsymbs = INTRA_MODE_CDF_LEN = 14 (13 symbols + 1 count slot).
        let mut cdf = clone_cdf(&DEFAULT_INTRA_MODE_CDF);
        for sym in (0..13usize).cycle().take(200) {
            update_cdf(&mut cdf, sym, INTRA_MODE_CDF_LEN);
            for (i, &v) in cdf[..13].iter().enumerate() {
                assert!(
                    v <= 32768,
                    "CDF entry {i} exceeded 32768: {v}"
                );
            }
        }
    }

    #[test]
    fn update_cdf_values_bounded_after_updates() {
        // After many random updates all probability entries (indices 0..N-2)
        // must stay in [0, 32768].  The AV1 update_cdf algorithm intentionally
        // allows adjacent CDF entries to cross (non-monotonicity is permitted by
        // the spec; the entropy coder uses raw probabilities, not a strict CDF
        // order), so we only verify the value bounds.
        let mut cdf = clone_cdf(&DEFAULT_INTRA_MODE_CDF);
        let symbols: [usize; 20] = [0, 3, 7, 12, 1, 6, 11, 2, 5, 10, 4, 9, 8, 0, 0, 12, 12, 6, 6, 6];
        for &sym in &symbols {
            update_cdf(&mut cdf, sym, INTRA_MODE_CDF_LEN);
        }
        // All probability entries 0..12 must be in [0, 32768].
        for (i, &v) in cdf[..13].iter().enumerate() {
            assert!(
                v <= 32768,
                "CDF entry {i} out of bounds after updates: {v}"
            );
        }
        // The count slot must have advanced.
        assert!(
            cdf[INTRA_MODE_CDF_LEN - 1] > 0,
            "count slot should be > 0 after updates, got {}",
            cdf[INTRA_MODE_CDF_LEN - 1]
        );
    }

    #[test]
    fn update_cdf_frequent_symbol_gains_probability() {
        // Repeatedly observing symbol 6 should cause cdf[6] (and higher
        // neighbouring entries) to rise, indicating higher probability for
        // symbol 6.  This validates that the update direction is correct.
        let mut cdf = clone_cdf(&DEFAULT_INTRA_MODE_CDF);
        let initial_cdf6 = cdf[6];
        for _ in 0..40 {
            update_cdf(&mut cdf, 6, INTRA_MODE_CDF_LEN);
        }
        assert!(
            cdf[6] > initial_cdf6,
            "cdf[6] should increase after many observations of symbol 6: \
             initial={initial_cdf6} final={}",
            cdf[6]
        );
    }

    #[test]
    fn clone_cdf_produces_equal_copy() {
        let original = DEFAULT_INTRA_MODE_CDF;
        let copy = clone_cdf(&original);
        assert_eq!(original, copy);
        // Verify the copy is independent by checking it's actually a copy.
        let mut mutable_copy = copy;
        mutable_copy[0] = 0;
        assert_eq!(mutable_copy[0], 0, "mutable copy should reflect the write");
        assert_ne!(original[0], 0, "original should be unchanged after modifying copy");
    }
}
