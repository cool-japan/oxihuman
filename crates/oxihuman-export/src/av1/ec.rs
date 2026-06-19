// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Self-consistent arithmetic range coder for AV1-like codec.
//!
//! # Design
//!
//! A byte-emission range coder with 32-bit internal state (u64 arithmetic to
//! prevent overflow).  The encoder and decoder share identical normalization
//! logic, guaranteeing a bit-exact round-trip.
//!
//! ## State invariant
//! At all times (outside of encode/decode operations):
//! - `lo` and `range` satisfy `0 <= lo < 2^32` and `0 < range <= 2^32`.
//! - The "current interval" is `[lo/2^32, (lo+range)/2^32)`.
//!
//! ## Normalization
//! After each symbol, if `(lo >> 24) == ((lo + range - 1) >> 24)`:
//! - The top byte has been "committed" — emit it.
//! - Shift both `lo` and `range` left by 8.
//!
//! ## Probability precision
//! All probabilities are represented as 15-bit fractions (denominator 32768).
//!
//! ## Finish / Initialize
//! The encoder appends exactly 4 padding bytes (the bottom 4 bytes of `lo`) so
//! the decoder can always recover the last symbol.  The decoder reads exactly 4
//! initial bytes to seed its `val` register.

// adapt_cdf (defined below) is the descending-CDF variant used by the adaptive
// wrappers.  The AOM ascending update_cdf from cdf_tables is used by coeffs.rs
// for the skip-flag CDF path; importing it here is not needed.

const SCALE: u64 = 1 << 15; // 32768

// ─────────────────────────────────────────────────────────────────────────────
// BoolEncoder
// ─────────────────────────────────────────────────────────────────────────────

/// Arithmetic encoder.
pub struct BoolEncoder {
    /// Low end of the current interval (low 32 bits meaningful, u64 for arithmetic).
    lo: u64,
    /// Width of the current interval.
    range: u64,
    /// Output buffer.
    out: Vec<u8>,
}

impl Default for BoolEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BoolEncoder {
    pub fn new() -> Self {
        Self {
            lo: 0,
            // Initial range = full 32-bit space.
            range: 1u64 << 32,
            out: Vec::new(),
        }
    }

    /// Encode symbol `sym` (0-indexed) from CDF `cdf` with `nsyms` symbols.
    ///
    /// `cdf.len()` must be at least `nsyms`.  `cdf[nsyms-1]` is the count slot
    /// (ignored here).  CDF values use the standard AV1 inverted form:
    /// `cdf[i]` = 32768 × P(symbol > i).
    pub fn encode_symbol(&mut self, sym: usize, cdf: &[u16], nsyms: usize) {
        let (fl, fh) = prob_window(sym, cdf, nsyms);
        self.narrow(fl, fh);
    }

    /// Encode symbol `sym` then adapt `cdf` toward it (`update_cdf`).
    ///
    /// # CDF array layout (critical — identical on encoder and decoder)
    ///
    /// Arrays must be `[p_0 … p_{nsyms-2}, terminator=0, count]`, length
    /// `nsyms + 1`.  The terminator slot at index `nsyms-1` starts at 0 and is
    /// **never updated** by `update_cdf` (it loops `take(n_syms-1)` = `nsyms-2`
    /// entries).  The count slot is at index `nsyms`.
    ///
    /// Call this with `nsyms` = number of real symbols; `update_cdf` is called
    /// internally with `nsyms + 1` (the full array length including count).
    ///
    /// **Encoder and decoder must call their respective adaptive pair in
    /// identical order and with identical `(sym, cdf-state)` so that `update_cdf`
    /// fires symmetrically on both sides (code-then-update on each).**
    pub fn encode_symbol_adapt(&mut self, sym: usize, cdf: &mut [u16], nsyms: usize) {
        self.encode_symbol(sym, cdf, nsyms);
        adapt_cdf(cdf, sym, nsyms + 1);
        repair_cdf_monotone(cdf, nsyms);
    }

    /// Encode a boolean bit (50/50): `true` = upper half.
    pub fn encode_bit(&mut self, bit: bool) {
        if bit {
            self.narrow(SCALE / 2, SCALE);
        } else {
            self.narrow(0, SCALE / 2);
        }
    }

    /// Write raw unsigned integer `val` using `bits` bits (MSB first).
    pub fn write_bypass_uint(&mut self, val: u64, bits: u32) {
        for i in (0..bits).rev() {
            self.encode_bit((val >> i) & 1 != 0);
        }
    }

    /// Finalise and return the encoded bytes.
    ///
    /// Appends 4 padding bytes (the 32-bit `lo` in big-endian order) so the
    /// decoder has enough data to decode the very last symbol.
    pub fn finish(mut self) -> Vec<u8> {
        // The decoder will read exactly 4 initial bytes.  We emit lo as 4
        // big-endian bytes.  Any pending renorm state is included because lo
        // already contains the lower end of the final interval.
        self.out.push((self.lo >> 24) as u8);
        self.out.push((self.lo >> 16) as u8);
        self.out.push((self.lo >>  8) as u8);
        self.out.push((self.lo      ) as u8);
        self.out
    }

    // ── internals ────────────────────────────────────────────────────────────

    /// Narrow the interval to [lo + range*fl/SCALE, lo + range*fh/SCALE).
    fn narrow(&mut self, fl: u64, fh: u64) {
        debug_assert!(fl < fh, "fl={fl} must be < fh={fh}");
        debug_assert!(fh <= SCALE, "fh={fh} must be <= SCALE");

        // Compute new sub-interval using u64 arithmetic (range <= 2^32, fl/fh <= 2^15).
        let lo_delta = self.range * fl / SCALE;
        let hi_delta = self.range * fh / SCALE;
        let range_new = (hi_delta - lo_delta).max(1);

        self.lo += lo_delta;
        self.range = range_new;
        self.renorm();
    }

    /// Emit committed top bytes and expand the interval.
    fn renorm(&mut self) {
        loop {
            // Top byte of lo and top byte of (lo + range - 1).
            let top_lo = (self.lo >> 24) & 0xFF;
            let top_hi = ((self.lo + self.range - 1) >> 24) & 0xFF;
            if top_lo != top_hi {
                break;
            }
            // Emit the agreed-upon top byte.
            self.out.push(top_lo as u8);
            // Remove top byte from both lo and range, then scale up by 256.
            self.lo  = (self.lo  & 0x00FF_FFFF) << 8;
            self.range <<= 8;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BoolDecoder
// ─────────────────────────────────────────────────────────────────────────────

/// Arithmetic decoder.
pub struct BoolDecoder<'a> {
    data: &'a [u8],
    pos: usize,
    /// Low end of the current interval (mirrors encoder's lo).
    lo: u64,
    /// Width of the current interval (mirrors encoder's range).
    range: u64,
    /// Current stream value in [lo, lo + range).
    val: u64,
}

impl<'a> BoolDecoder<'a> {
    /// Create a decoder from bytes produced by `BoolEncoder::finish`.
    pub fn new(data: &'a [u8]) -> Self {
        let mut dec = Self {
            data,
            pos: 0,
            lo: 0,
            range: 1u64 << 32,
            val: 0,
        };
        // Read 4 initial bytes to seed val (matching finish()'s 4-byte output).
        for _ in 0..4 {
            dec.val = (dec.val << 8) | dec.read_byte() as u64;
        }
        dec
    }

    /// Decode a symbol from CDF `cdf` with `nsyms` symbols.
    pub fn decode_symbol(&mut self, cdf: &[u16], nsyms: usize) -> usize {
        // val is always in [lo, lo+range).  We need to find which symbol's
        // sub-interval contains val.
        for sym in 0..nsyms {
            let (fl, fh) = prob_window(sym, cdf, nsyms);
            let lo_delta = self.range * fl / SCALE;
            let hi       = self.range * fh / SCALE;

            // The symbol interval within the current range is [lo + lo_delta, lo + hi).
            let sym_lo = self.lo + lo_delta;
            let sym_hi = self.lo + hi;  // exclusive upper bound (clamped to lo+range below)

            if self.val >= sym_lo && self.val < sym_hi.max(sym_lo + 1) {
                self.lo = sym_lo;
                self.range = (hi - lo_delta).max(1);
                self.renorm();
                return sym;
            }
        }

        // Fallback: last symbol.
        let last = nsyms - 1;
        let (fl, fh) = prob_window(last, cdf, nsyms);
        let lo_delta = self.range * fl / SCALE;
        let hi       = self.range * fh / SCALE;
        self.lo   += lo_delta;
        self.range = (hi - lo_delta).max(1);
        self.renorm();
        last
    }

    /// Decode a boolean bit.
    pub fn decode_bit(&mut self) -> bool {
        // True occupies upper half: fl = SCALE/2, fh = SCALE.
        let mid_delta = self.range * (SCALE / 2) / SCALE;

        if self.val < self.lo + mid_delta {
            // val is in the lower half → bit = false.
            self.range = mid_delta.max(1);
            self.renorm();
            false
        } else {
            // val is in the upper half → bit = true.
            self.lo   += mid_delta;
            self.range = (self.range - mid_delta).max(1);
            self.renorm();
            true
        }
    }

    /// Decode a symbol then adapt `cdf` toward the decoded value.
    ///
    /// Symmetric counterpart of `BoolEncoder::encode_symbol_adapt` — both
    /// call `update_cdf` **after** coding, with identical `(sym, cdf-state)`,
    /// so the CDF evolves identically on both sides (lockstep guarantee).
    ///
    /// See `encode_symbol_adapt` for the array layout convention.
    pub fn decode_symbol_adapt(&mut self, cdf: &mut [u16], nsyms: usize) -> usize {
        let sym = self.decode_symbol(cdf, nsyms);
        adapt_cdf(cdf, sym, nsyms + 1);
        repair_cdf_monotone(cdf, nsyms);
        sym
    }

    /// Read raw unsigned integer using `bits` bits (MSB first).
    pub fn read_bypass_uint(&mut self, bits: u32) -> u64 {
        let mut val = 0u64;
        for _ in 0..bits {
            val = (val << 1) | self.decode_bit() as u64;
        }
        val
    }

    // ── internals ────────────────────────────────────────────────────────────

    fn read_byte(&mut self) -> u8 {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            b
        } else {
            0x00 // pad with zeros past end of stream
        }
    }

    /// Remove committed top bytes from the state and refill from the stream.
    fn renorm(&mut self) {
        loop {
            let top_lo = (self.lo >> 24) & 0xFF;
            let top_hi = ((self.lo + self.range - 1) >> 24) & 0xFF;
            if top_lo != top_hi {
                break;
            }
            // Consume the agreed-upon top byte and shift in a new byte from stream.
            self.lo    = (self.lo    & 0x00FF_FFFF) << 8;
            self.range <<= 8;
            self.val   = (self.val   & 0x00FF_FFFF) << 8 | self.read_byte() as u64;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helper
// ─────────────────────────────────────────────────────────────────────────────

/// Return `(fl, fh)` — the probability window for symbol `sym`.
///
/// Uses the inverted AV1 CDF convention:
/// - `fh = cdf[sym-1]` (or SCALE when sym==0)
/// - `fl = cdf[sym]`   (or 0 when sym is the last valid symbol)
///
/// Callers that mutate the CDF (adaptive coding) must call
/// `repair_cdf_monotone` after each `update_cdf` call so that the CDF
/// presented here is always non-increasing — that guarantees disjoint
/// intervals and correct decoder symbol selection.
#[inline(always)]
fn prob_window(sym: usize, cdf: &[u16], nsyms: usize) -> (u64, u64) {
    debug_assert!(sym < nsyms, "sym={sym} out of range for nsyms={nsyms}");
    let fh: u64 = if sym == 0 { SCALE } else { cdf[sym - 1] as u64 };
    let fl: u64 = if sym + 1 < nsyms { cdf[sym] as u64 } else { 0 };
    (fl, fh)
}

/// Adapt a **descending** CDF toward the observed symbol `sym`.
///
/// Our `prob_window` uses the **descending** convention: `cdf[0] > cdf[1] > …`
/// where `P(sym=k) = (cdf[k-1] − cdf[k]) / SCALE`.
///
/// To *increase* `P(sym=k)` after observing it, `cdf[k-1]` must rise and
/// `cdf[k]` must fall.  Concretely:
/// - `i < sym`: move `cdf[i]` **toward 32768** (increase).
/// - `i >= sym`: move `cdf[i]` **toward 0** (decrease).
///
/// This is the mirror of the AOM `update_cdf` (which is designed for ascending
/// CDFs).  The rate and count-slot logic are identical to `_update_cdf_aom`.
///
/// `n = nsyms + 1` (total array length including the count slot at index n-1).
#[inline]
fn adapt_cdf(cdf: &mut [u16], sym: usize, n: usize) {
    let n_syms = n - 1;
    let rate = 4 + (cdf[n - 1] as usize >> 4).min(2);
    for (i, c) in cdf.iter_mut().take(n_syms - 1).enumerate() {
        if i < sym {
            *c += (32768 - *c) >> rate;
        } else {
            *c -= *c >> rate;
        }
    }
    cdf[n - 1] += (cdf[n - 1] < 32) as u16;
}

/// Repair the CDF probability entries to be *strictly* decreasing with
/// a minimum value of 1.
///
/// `update_cdf` (AV1 spec) permits non-monotone CDFs after a single update.
/// Non-monotone or equal-adjacent entries produce zero-width probability
/// windows, breaking the decoder's linear-scan symbol search.  This repair
/// is applied **after every `update_cdf` call** in the adaptive wrappers so
/// that both encoder and decoder always see a strictly-decreasing CDF with
/// positive-width, disjoint intervals.
///
/// Two-pass algorithm (both sides apply identically → lockstep preserved):
/// 1. Forward pass: clamp each entry to be at most the previous (non-increasing).
/// 2. Backward pass: propagate a minimum-gap of 1 from the right, ensuring
///    `cdf[i] >= cdf[i+1] + 1` for all i, and `cdf[n-1] >= 1`.
///    This also resolves equal pairs left by pass 1.
///
/// `nsyms` = number of real symbols.  Probability entries: `cdf[0..nsyms-2]`.
/// `cdf[nsyms-1]` (terminator = 0) and `cdf[nsyms]` (count) are not touched.
#[inline]
fn repair_cdf_monotone(cdf: &mut [u16], nsyms: usize) {
    if nsyms <= 1 {
        return;
    }
    // n = number of probability entries = nsyms - 1.
    let n = (nsyms - 1).min(cdf.len());
    if n == 0 {
        return;
    }
    // Forward pass: enforce non-increasing order.
    for i in 1..n {
        if cdf[i] > cdf[i - 1] {
            cdf[i] = cdf[i - 1];
        }
    }
    // Backward pass: enforce strict decrease and minimum probability 1.
    if cdf[n - 1] == 0 {
        cdf[n - 1] = 1;
    }
    for i in (0..n - 1).rev() {
        let min_needed = cdf[i + 1].saturating_add(1);
        if cdf[i] < min_needed {
            cdf[i] = min_needed;
        }
        // Hard cap at SCALE — in practice never hit (n <= 11, so max push is 11).
        if cdf[i] > SCALE as u16 {
            cdf[i] = SCALE as u16;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct Prng {
        s: u64,
    }
    impl Prng {
        fn new(seed: u64) -> Self {
            Self { s: seed.max(1) }
        }
        fn next(&mut self) -> u64 {
            let mut x = self.s;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.s = x;
            x
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    const CDF3: [u16; 4] = [21845, 10922, 0, 0];
    const N3: usize = 3;

    fn enc_dec_syms(syms: &[usize], cdf: &[u16], n: usize) -> Vec<usize> {
        let mut enc = BoolEncoder::new();
        for &s in syms {
            enc.encode_symbol(s, cdf, n);
        }
        let bytes = enc.finish();
        let mut dec = BoolDecoder::new(&bytes);
        (0..syms.len()).map(|_| dec.decode_symbol(cdf, n)).collect()
    }

    fn enc_dec_bits(bits: &[bool]) -> Vec<bool> {
        let mut enc = BoolEncoder::new();
        for &b in bits {
            enc.encode_bit(b);
        }
        let bytes = enc.finish();
        let mut dec = BoolDecoder::new(&bytes);
        (0..bits.len()).map(|_| dec.decode_bit()).collect()
    }

    #[test]
    fn test_single_symbol_round_trip() {
        for sym in 0..N3 {
            let got = enc_dec_syms(&[sym], &CDF3, N3);
            assert_eq!(got[0], sym, "single symbol {sym}: got {}", got[0]);
        }
    }

    #[test]
    fn test_1000_random_symbols_round_trip() {
        let mut rng = Prng::new(0xDEAD_BEEF_1234_5678);
        let syms: Vec<usize> = (0..1000).map(|_| rng.below(N3)).collect();
        let got = enc_dec_syms(&syms, &CDF3, N3);
        for (i, (&e, &g)) in syms.iter().zip(got.iter()).enumerate() {
            assert_eq!(g, e, "position {i}: expected {e} got {g}");
        }
    }

    #[test]
    fn test_1000_random_bits_round_trip() {
        let mut rng = Prng::new(0xCAFE_BABE_DEAD_0001);
        let bits: Vec<bool> = (0..1000).map(|_| rng.next() & 1 != 0).collect();
        let got = enc_dec_bits(&bits);
        for (i, (&e, &g)) in bits.iter().zip(got.iter()).enumerate() {
            assert_eq!(g, e, "position {i}: expected {e} got {g}");
        }
    }

    #[test]
    fn test_empty_sequence() {
        let enc = BoolEncoder::new();
        let bytes = enc.finish();
        assert!(bytes.len() >= 4);
        let _dec = BoolDecoder::new(&bytes);
    }

    #[test]
    fn test_bypass_uint_round_trip() {
        for (val, bits) in [
            (0u64, 1u32),
            (1, 1),
            (0b1010_0101, 8),
            (0xDEAD, 16),
            (0, 7),
            (127, 7),
        ] {
            let mut enc = BoolEncoder::new();
            enc.write_bypass_uint(val, bits);
            let bytes = enc.finish();
            let mut dec = BoolDecoder::new(&bytes);
            let got = dec.read_bypass_uint(bits);
            assert_eq!(got, val, "val={val:#x} bits={bits} got={got:#x}");
        }
    }

    #[test]
    fn test_mixed_round_trip() {
        let mut enc = BoolEncoder::new();
        enc.encode_symbol(2, &CDF3, N3);
        enc.encode_bit(true);
        enc.encode_symbol(0, &CDF3, N3);
        enc.encode_bit(false);
        enc.encode_symbol(1, &CDF3, N3);
        enc.write_bypass_uint(0b1011, 4);
        let bytes = enc.finish();

        let mut dec = BoolDecoder::new(&bytes);
        assert_eq!(dec.decode_symbol(&CDF3, N3), 2, "sym 2");
        assert!(dec.decode_bit(), "bit true");
        assert_eq!(dec.decode_symbol(&CDF3, N3), 0, "sym 0");
        assert!(!dec.decode_bit(), "bit false");
        assert_eq!(dec.decode_symbol(&CDF3, N3), 1, "sym 1");
        assert_eq!(dec.read_bypass_uint(4), 0b1011u64, "bypass uint");
    }

    #[test]
    fn test_skewed_cdf_round_trip() {
        let cdf: [u16; 4] = [100, 50, 0, 0];
        let n = 3;
        let syms: Vec<usize> = vec![0; 200];
        let got = enc_dec_syms(&syms, &cdf, n);
        for (i, &g) in got.iter().enumerate() {
            assert_eq!(g, 0, "position {i}");
        }
    }

    #[test]
    fn test_4_symbol_cdf() {
        let cdf: [u16; 5] = [24576, 16384, 8192, 0, 0];
        let n = 4;
        let seq = [0usize, 1, 2, 3, 2, 1, 0, 3, 3, 0];
        let got = enc_dec_syms(&seq, &cdf, n);
        for (i, (&e, &g)) in seq.iter().zip(got.iter()).enumerate() {
            assert_eq!(g, e, "4-sym pos {i}: expected {e} got {g}");
        }
    }

    #[test]
    fn test_compression() {
        let cdf: [u16; 4] = [100, 50, 0, 0];
        let mut enc = BoolEncoder::new();
        for _ in 0..5000 {
            enc.encode_symbol(0, &cdf, 3);
        }
        let bytes = enc.finish();
        assert!(bytes.len() < 500, "got {} bytes", bytes.len());
    }

    #[test]
    fn test_all_same_symbols() {
        for sym in 0..N3 {
            let syms: Vec<usize> = vec![sym; 100];
            let got = enc_dec_syms(&syms, &CDF3, N3);
            for (i, &g) in got.iter().enumerate() {
                assert_eq!(g, sym, "sym={sym} at {i}: got {g}");
            }
        }
    }

    #[test]
    fn test_single_true_bit() {
        assert!(enc_dec_bits(&[true])[0]);
    }

    #[test]
    fn test_single_false_bit() {
        assert!(!enc_dec_bits(&[false])[0]);
    }

    #[test]
    fn test_alternating_bits() {
        let bits: Vec<bool> = (0..50).map(|i| i % 2 == 0).collect();
        let got = enc_dec_bits(&bits);
        for (i, (&e, &g)) in bits.iter().zip(got.iter()).enumerate() {
            assert_eq!(g, e, "bit at {i}");
        }
    }

    #[test]
    fn test_adaptive_symbol_round_trip() {
        // Encode 500 symbols adaptively with a shared evolving CDF on each side.
        // Both sides must decode identically despite CDF mutation.
        let mut rng = Prng::new(0xADA_0ADD_FADE_1234);
        let syms: Vec<usize> = (0..500).map(|_| rng.below(4)).collect();

        // Encode with adapting CDF.
        let mut enc = BoolEncoder::new();
        let mut enc_cdf: [u16; 5] = [24576, 16384, 8192, 0, 0];
        for &s in &syms {
            enc.encode_symbol_adapt(s, &mut enc_cdf, 4);
        }
        let bytes = enc.finish();

        // Decode with identically-initialized adapting CDF.
        let mut dec = BoolDecoder::new(&bytes);
        let mut dec_cdf: [u16; 5] = [24576, 16384, 8192, 0, 0];
        for (i, &expected) in syms.iter().enumerate() {
            let got = dec.decode_symbol_adapt(&mut dec_cdf, 4);
            assert_eq!(got, expected, "adaptive round-trip failed at pos {i}");
        }
    }

    #[test]
    fn test_adaptive_converges_and_compresses() {
        // After repeatedly encoding symbol 0, the adaptive CDF should learn a
        // high probability for it, resulting in fewer bits than the non-adaptive
        // baseline with a flat (50/50) CDF.
        const N: usize = 2000;
        let syms = vec![0usize; N];

        // Adaptive encoding.
        let mut enc = BoolEncoder::new();
        let mut cdf: [u16; 3] = [16384, 0, 0]; // binary, uniform start
        for &s in &syms {
            enc.encode_symbol_adapt(s, &mut cdf, 2);
        }
        let adaptive_bytes = enc.finish().len();

        // Non-adaptive encoding with the same flat CDF.
        let flat_cdf: [u16; 3] = [16384, 0, 0];
        let mut enc2 = BoolEncoder::new();
        for &s in &syms {
            enc2.encode_symbol(s, &flat_cdf, 2);
        }
        let flat_bytes = enc2.finish().len();

        assert!(
            adaptive_bytes < flat_bytes,
            "adaptive ({adaptive_bytes} bytes) must compress better than flat ({flat_bytes} bytes)"
        );

        // Also verify round-trip correctness.
        let mut enc3 = BoolEncoder::new();
        let mut enc_cdf: [u16; 3] = [16384, 0, 0];
        for &s in &syms {
            enc3.encode_symbol_adapt(s, &mut enc_cdf, 2);
        }
        let bytes = enc3.finish();
        let mut dec = BoolDecoder::new(&bytes);
        let mut dec_cdf: [u16; 3] = [16384, 0, 0];
        for i in 0..N {
            let got = dec.decode_symbol_adapt(&mut dec_cdf, 2);
            assert_eq!(got, 0usize, "adaptive decode failed at pos {i}");
        }
    }
}
