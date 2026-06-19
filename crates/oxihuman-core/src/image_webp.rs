// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! WebP VP8L (lossless) encoder and decoder — pure Rust, no external codec deps.
//!
//! # Format overview
//!
//! ```text
//! RIFF header  : "RIFF" + file_size(u32 LE) + "WEBP"
//! VP8L chunk   : "VP8L" + chunk_size(u32 LE) + signature(0x2F) + VP8L bitstream
//!
//! VP8L bitstream (LSB-first bit packing):
//!   [14 bits] width  - 1
//!   [14 bits] height - 1
//!   [ 1 bit ] alpha_is_used
//!   [ 3 bits] version (must be 0)
//!   [ 1 bit ] transform_present = 0   (no transforms)
//!   [ 1 bit ] color_cache_code_bits = 0 (no color cache)
//!   [ 1 bit ] huffman_meta = 0        (no meta Huffman image)
//!   [ 5 Huffman trees: G, R, B, A, Distance ]
//!   [ pixel data encoded via those trees ]
//! ```
//!
//! ## Huffman tree encoding
//!
//! Each Huffman tree is written in one of two forms:
//!
//! ### Simple tree (≤ 2 distinct symbols)
//! ```text
//! [1]  is_simple = 1
//! [1]  num_symbols_minus1   (0 → 1 symbol, 1 → 2 symbols)
//! if 1 symbol: [8] symbol_value
//! if 2 symbols:
//!    [1]  first_symbol_uses_8bits  (always 1 here — we never rely on the short 1-bit path)
//!    [8]  first_symbol
//!    [8]  second_symbol
//! ```
//!
//! ### Standard tree (> 2 distinct symbols, up to 256)
//! ```text
//! [1]  is_simple = 0
//! [1]  use_length_limit = 0  (max_symbol = full alphabet)
//! [4]  num_code_length_codes_minus4   (value 0..=15 → means 4..=19 CL-codes written)
//! For each of the 19 code-length permuted positions (order 17,18,0,1,2,3,4,5,16,6..15):
//!    [3]  code_length_code_length  (only the first num_code_length_codes_minus4+4 are non-zero)
//! Then: encode `alphabet_size` code lengths using those 19-symbol meta-Huffman codes.
//! ```
//!
//! ## Pixel encoding
//! For each pixel (ARGB packed):
//!   encode G using G-tree, then R using R-tree, then B using B-tree, then A using A-tree.
//! Distance tree is never used (literal-only mode).

#![allow(dead_code)]

use super::image_codec::RawDecodeResult;

// ── Error ─────────────────────────────────────────────────────────────────────

/// Errors that can occur during WebP encode/decode.
#[derive(Debug, thiserror::Error)]
pub enum WebpError {
    #[error("Invalid WebP: {0}")]
    Invalid(String),
    #[error("Unsupported WebP feature: {0}")]
    Unsupported(String),
    #[error("Truncated data")]
    Truncated,
}

// ── Bit writer (LSB-first, little-endian) ─────────────────────────────────────

struct BitWriter {
    data: Vec<u8>,
    cur_byte: u32,
    bit_count: u8, // bits filled in cur_byte so far (0..=7)
}

impl BitWriter {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            cur_byte: 0,
            bit_count: 0,
        }
    }

    /// Write `n` bits from `value` (LSB first).
    fn write_bits(&mut self, value: u64, n: u8) {
        debug_assert!(n <= 64);
        let mut remaining = n;
        let mut val = value;
        while remaining > 0 {
            let can_write = 8 - self.bit_count;
            let to_write = remaining.min(can_write);
            let mask = (1u64 << to_write) - 1;
            let bits = (val & mask) as u32;
            self.cur_byte |= bits << self.bit_count;
            self.bit_count += to_write;
            val >>= to_write;
            remaining -= to_write;
            if self.bit_count == 8 {
                self.data.push(self.cur_byte as u8);
                self.cur_byte = 0;
                self.bit_count = 0;
            }
        }
    }

    /// Flush any partial byte with zero padding.
    fn flush(&mut self) {
        if self.bit_count > 0 {
            self.data.push(self.cur_byte as u8);
            self.cur_byte = 0;
            self.bit_count = 0;
        }
    }

    fn into_bytes(mut self) -> Vec<u8> {
        self.flush();
        self.data
    }
}

// ── Bit reader (LSB-first) ────────────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // bits consumed in current byte (0..=7)
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Read `n` bits (LSB first). Returns error on truncation.
    fn read_bits(&mut self, n: u8) -> Result<u64, WebpError> {
        debug_assert!(n <= 64);
        let mut result: u64 = 0;
        let mut filled = 0u8;
        let mut remaining = n;
        while remaining > 0 {
            if self.byte_pos >= self.data.len() {
                return Err(WebpError::Truncated);
            }
            let cur = self.data[self.byte_pos];
            let available = 8 - self.bit_pos;
            let to_read = remaining.min(available);
            let mask = if to_read == 8 {
                0xFFu8
            } else {
                (1u8 << to_read) - 1
            };
            let bits = ((cur >> self.bit_pos) & mask) as u64;
            result |= bits << filled;
            filled += to_read;
            self.bit_pos += to_read;
            remaining -= to_read;
            if self.bit_pos == 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }
        }
        Ok(result)
    }
}

// ── Canonical Huffman code generation ─────────────────────────────────────────

/// Compute Huffman code lengths for `symbols` (0..alphabet_size) from frequency counts.
/// Returns a Vec<u8> of length `alphabet_size` where entry `i` is the code length for symbol `i`.
/// Lengths are capped at `max_bits` (≤ 15 for VP8L).
///
/// Algorithm: standard optimal Huffman tree built with a min-heap, then depths are capped
/// at `max_bits` and the Kraft inequality is restored by shortening codes of the least-frequent
/// symbols (which are already long).  For VP8L's ≤256 symbols with max_bits=15 this is always
/// feasible — optimal depths never exceed ~log2(256)=8.
fn compute_code_lengths(freqs: &[u32], alphabet_size: usize, max_bits: u8) -> Vec<u8> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let mut lengths = vec![0u8; alphabet_size];

    // Non-zero symbols, sorted by ascending frequency (for tiebreaking, use symbol index).
    let mut non_zero: Vec<usize> = (0..alphabet_size)
        .filter(|&i| freqs.get(i).copied().unwrap_or(0) > 0)
        .collect();

    match non_zero.len() {
        0 => return lengths,
        1 => {
            lengths[non_zero[0]] = 1;
            return lengths;
        }
        _ => {}
    }

    non_zero.sort_unstable_by_key(|&i| (freqs[i], i));

    let n = non_zero.len();

    // We represent the tree as a flat array of (freq, parent, depth) for each node.
    // Leaves: indices 0..n (mapped to symbols via non_zero[i]).
    // Internal nodes: indices n..2n-1.
    let mut node_freq = vec![0u64; 2 * n];
    let mut depth = vec![0u8; 2 * n];

    for (i, &sym) in non_zero.iter().enumerate() {
        node_freq[i] = freqs[sym] as u64;
    }

    // Min-heap: (freq, counter, node_id) — counter for stable ordering.
    let mut heap: BinaryHeap<Reverse<(u64, usize, usize)>> =
        (0..n).map(|i| Reverse((node_freq[i], i, i))).collect();

    let mut next_node = n;
    let mut counter = n;
    let mut parent = vec![usize::MAX; 2 * n];

    while heap.len() > 1 {
        let Reverse((f1, _, id1)) = heap.pop().expect("heap non-empty");
        let Reverse((f2, _, id2)) = heap.pop().expect("heap non-empty");

        let new_id = next_node;
        next_node += 1;
        node_freq[new_id] = f1 + f2;
        depth[new_id] = 0; // set by tree traversal below
        parent[id1] = new_id;
        parent[id2] = new_id;

        heap.push(Reverse((node_freq[new_id], counter, new_id)));
        counter += 1;
    }

    // Compute depths by traversing from each leaf to the root.
    let root = if next_node > n { next_node - 1 } else { 0 };
    parent[root] = root;

    for i in 0..n {
        let mut d = 0u8;
        let mut cur = i;
        while parent[cur] != cur {
            d += 1;
            cur = parent[cur];
            if d > max_bits + 1 {
                break; // safety cap; handled below
            }
        }
        lengths[non_zero[i]] = d.min(max_bits);
    }

    // ── Restore the Kraft inequality after capping ────────────────────────────
    //
    // Canonical Huffman trees satisfy sum(2^{-l_i}) = 1.  Capping lengths can only
    // increase this sum (shorter codes → more "weight").  We restore it by
    // increassing the lengths of the cheapest (most-frequent) symbols, which shortens
    // their codes and thus decreases the Kraft sum — but we need to make the tree
    // *valid*, so we increase codes of the *least-frequent* symbols (already the
    // longest) to free up code space.
    //
    // In practice, for ≤256 symbols and max_bits=15 the optimal depth is at most 8,
    // so no adjustment is ever needed.  The loop below handles corner cases.

    // Use integer arithmetic scaled to 2^max_bits.
    let scale: i64 = 1i64 << max_bits;
    loop {
        let kraft: i64 = non_zero
            .iter()
            .map(|&sym| {
                let l = lengths[sym] as u32;
                if l > 0 {
                    scale >> l
                } else {
                    0i64
                }
            })
            .sum();

        if kraft <= scale {
            break;
        }

        // Kraft sum > 1: increase the length of the cheapest (first in `non_zero`) symbol
        // that hasn't yet hit `max_bits`.
        let mut changed = false;
        for &sym in non_zero.iter() {
            if lengths[sym] < max_bits {
                lengths[sym] += 1;
                changed = true;
                break;
            }
        }
        if !changed {
            break; // all at max_bits; can't do more
        }
    }

    lengths
}

/// Given code lengths, assign canonical Huffman codes (shorter = smaller symbol index).
/// Returns a Vec<(code: u16, length: u8)> of length `alphabet_size`.
/// Symbols with length=0 get code=0 and are not used.
/// Assign canonical Huffman codes to each symbol and **bit-reverse** each code so it is
/// ready for use with a LSB-first bitstream.
///
/// Standard canonical Huffman code assignment (MSB-first) followed by per-code bit-reversal:
///   code_for_symbol_s = reverse_bits(canonical_code(s), len_s)
///
/// This is required because VP8L writes and reads bits LSB-first, but Huffman code values
/// are assigned canonically (MSB-first ordering).  The bit-reversal ensures the decoder
/// reading `len` bits from an LSB-first stream recovers the canonical code value.
fn canonical_codes_from_lengths(lengths: &[u8]) -> Vec<(u16, u8)> {
    let alphabet_size = lengths.len();
    let max_len = *lengths.iter().max().unwrap_or(&0) as usize;
    let mut codes = vec![(0u16, 0u8); alphabet_size];

    if max_len == 0 {
        return codes;
    }

    // Count symbols per length.
    let mut bl_count = vec![0u32; max_len + 1];
    for &l in lengths {
        if l > 0 {
            bl_count[l as usize] += 1;
        }
    }

    // Compute starting (MSB-first) canonical code for each length group.
    let mut next_code = vec![0u32; max_len + 2];
    let mut code: u32 = 0;
    bl_count[0] = 0;
    for bits in 1..=max_len {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }

    // Assign codes and bit-reverse each one for LSB-first use.
    for s in 0..alphabet_size {
        let l = lengths[s] as usize;
        if l > 0 {
            let canon = next_code[l] as u16;
            next_code[l] += 1;
            // Reverse the `l` bits of `canon` to get the LSB-first code.
            let reversed = reverse_bits_u16(canon, l as u8);
            codes[s] = (reversed, lengths[s]);
        }
    }

    codes
}

/// Reverse the low `n` bits of `v`.
#[inline]
fn reverse_bits_u16(v: u16, n: u8) -> u16 {
    debug_assert!(n <= 16);
    let mut r = 0u16;
    let mut x = v;
    for _ in 0..n {
        r = (r << 1) | (x & 1);
        x >>= 1;
    }
    r
}

// ── Huffman decode table ──────────────────────────────────────────────────────

/// A flat, direct-lookup Huffman decode table.
///
/// Indexed by `max_bits`-bit window (read LSB-first from the bitstream).
/// Each entry stores (symbol, actual_code_length).  We read `max_bits` bits,
/// index the table, get the symbol and how many bits it actually consumed,
/// then "unread" the remaining bits via the bit reader.
struct HuffTable {
    /// `table[i]` = (symbol, code_length) for the code whose low `code_length`
    /// bits equal `i & ((1 << code_length) - 1)`.  Entries with `code_length == 0`
    /// indicate invalid/unused slots (should not be reached for valid streams).
    table: Vec<(u16, u8)>,
    /// Maximum code length present in this table.
    max_bits: u8,
    /// Number of distinct symbols (for the single-symbol special case).
    single_symbol: Option<u16>,
}

impl HuffTable {
    /// Build a decode table from canonical code lengths.
    fn build(lengths: &[u8]) -> Self {
        let max_bits = lengths.iter().copied().max().unwrap_or(0);

        // Count non-zero symbols.
        let non_zero_syms: Vec<u16> = (0..lengths.len() as u16)
            .filter(|&i| lengths[i as usize] > 0)
            .collect();

        if non_zero_syms.is_empty() {
            return HuffTable {
                table: vec![(0, 0); 1],
                max_bits: 0,
                single_symbol: None,
            };
        }

        // Single-symbol degenerate case: no bits read, always return this symbol.
        if non_zero_syms.len() == 1 {
            return HuffTable {
                table: vec![(non_zero_syms[0], 1); 2],
                max_bits: 1,
                single_symbol: Some(non_zero_syms[0]),
            };
        }

        // Build canonical codes.
        let codes = canonical_codes_from_lengths(lengths);

        // Allocate the full decode table: 2^max_bits entries.
        let table_size = 1usize << max_bits;
        let mut table = vec![(0u16, 0u8); table_size];

        for (sym, &(code, len)) in codes.iter().enumerate() {
            if len == 0 {
                continue;
            }
            // For each entry whose low `len` bits equal `code`, fill the table.
            // The remaining `max_bits - len` bits are "don't care" → enumerate all.
            let extra_bits = max_bits - len;
            let num_fill = 1usize << extra_bits;
            for fill in 0..num_fill {
                // LSB-first: code occupies the low `len` bits; `fill` occupies the upper bits.
                let idx = (fill << len) | (code as usize);
                table[idx] = (sym as u16, len);
            }
        }

        HuffTable {
            table,
            max_bits,
            single_symbol: None,
        }
    }

    /// Decode one symbol: reads `max_bits` bits, looks up the table, then "unreads" the
    /// bits that belong to the next symbol.
    fn decode(&self, reader: &mut BitReader<'_>) -> Result<u16, WebpError> {
        if self.max_bits == 0 {
            return Err(WebpError::Invalid("Huffman table has max_bits=0".into()));
        }

        // Single-symbol shortcut: read 1 bit (always 0), return the symbol.
        if let Some(sym) = self.single_symbol {
            let _ = reader.read_bits(1)?; // consume 1 bit (code = 0b0 or 0b1 doesn't matter)
            return Ok(sym);
        }

        let bits = reader.read_bits(self.max_bits)? as usize;
        let (sym, code_len) = self.table[bits & (self.table.len() - 1)];

        if code_len == 0 {
            return Err(WebpError::Invalid(format!(
                "invalid Huffman prefix bits={:b}",
                bits
            )));
        }

        // Unread the bits that weren't part of this code.
        let unused = self.max_bits - code_len;
        if unused > 0 {
            reader.unread_bits(unused);
        }

        Ok(sym)
    }
}

// Alias for consistency with the rest of the code.
type HuffTree = HuffTable;

impl<'a> BitReader<'a> {
    /// Undo `n` bits that were just read (wind the position back).
    ///
    /// The bits are guaranteed to be at or very near the current position, so
    /// unwinding the byte_pos/bit_pos bookkeeping is straightforward.
    fn unread_bits(&mut self, n: u8) {
        let mut remaining = n as u32;
        while remaining > 0 {
            if (self.bit_pos as u32) >= remaining {
                self.bit_pos -= remaining as u8;
                remaining = 0;
            } else {
                remaining -= self.bit_pos as u32;
                if self.byte_pos > 0 {
                    self.byte_pos -= 1;
                    self.bit_pos = 8;
                } else {
                    self.bit_pos = 0;
                    break;
                }
            }
        }
    }
}

// ── Huffman tree encoding (write to VP8L bitstream) ───────────────────────────

/// Description of a Huffman tree to be written or decoded.
#[derive(Debug)]
enum HuffSpec {
    /// 1 distinct symbol.
    Simple1 { symbol: u16 },
    /// 2 distinct symbols.
    Simple2 { sym0: u16, sym1: u16 },
    /// Standard (code-length-coded) tree.
    Standard { lengths: Vec<u8> },
}

/// Analyse `freqs[0..alphabet_size]` and choose the simplest HuffSpec.
fn analyse_freqs(freqs: &[u32], alphabet_size: usize) -> HuffSpec {
    let mut distinct: Vec<u16> = (0..alphabet_size as u16)
        .filter(|&i| freqs.get(i as usize).copied().unwrap_or(0) > 0)
        .collect();
    distinct.sort_unstable();

    match distinct.len() {
        0 => HuffSpec::Simple1 { symbol: 0 }, // degenerate: single symbol=0
        1 => HuffSpec::Simple1 {
            symbol: distinct[0],
        },
        2 => HuffSpec::Simple2 {
            sym0: distinct[0],
            sym1: distinct[1],
        },
        _ => {
            let lengths = compute_code_lengths(freqs, alphabet_size, 15);
            HuffSpec::Standard { lengths }
        }
    }
}

/// Write a HuffSpec into `bw`.
fn write_huff_tree(bw: &mut BitWriter, spec: &HuffSpec) {
    match spec {
        HuffSpec::Simple1 { symbol } => {
            bw.write_bits(1, 1); // is_simple = 1
            bw.write_bits(0, 1); // num_symbols_minus1 = 0  (one symbol)
            bw.write_bits(*symbol as u64, 8); // 8-bit symbol value
        }
        HuffSpec::Simple2 { sym0, sym1 } => {
            bw.write_bits(1, 1); // is_simple = 1
            bw.write_bits(1, 1); // num_symbols_minus1 = 1  (two symbols)
            bw.write_bits(1, 1); // first_symbol_uses_8bits = 1
            bw.write_bits(*sym0 as u64, 8);
            bw.write_bits(*sym1 as u64, 8);
        }
        HuffSpec::Standard { lengths } => {
            write_standard_huff_tree(bw, lengths);
        }
    }
}

/// Write the standard (code-length-coded) Huffman tree spec.
fn write_standard_huff_tree(bw: &mut BitWriter, lengths: &[u8]) {
    bw.write_bits(0, 1); // is_simple = 0

    let alphabet_size = lengths.len();

    // use_length_limit: 0 (we always write full alphabet)
    bw.write_bits(0, 1);

    // Build meta-Huffman (code-length codes) from lengths.
    // The 19 possible code-length values (0..=18), where 16/17/18 are RLE codes.
    // We only use values 0..=15 (no RLE for simplicity), so 19 meta-symbol freqs.
    const NUM_META: usize = 19;
    let mut cl_freqs = [0u32; NUM_META];
    for &l in lengths.iter().take(alphabet_size) {
        cl_freqs[l as usize] += 1;
    }

    // Permuted order for code-length codes per VP8L spec.
    const CL_ORDER: [usize; 19] = [
        17, 18, 0, 1, 2, 3, 4, 5, 16, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ];

    let cl_lengths = compute_code_lengths(&cl_freqs, NUM_META, 7);

    // Find how many code-length codes to write (trailing zeros trimmed, min 4).
    let num_cl = {
        let mut last = 4usize;
        for i in 0..19 {
            if cl_lengths[CL_ORDER[i]] > 0 {
                last = i + 1;
            }
        }
        last.max(4)
    };

    // num_code_length_codes_minus4 (4 bits)
    bw.write_bits((num_cl - 4) as u64, 4);

    // Write the code-length code lengths.
    for i in 0..num_cl {
        bw.write_bits(cl_lengths[CL_ORDER[i]] as u64, 3);
    }

    // Encode `lengths` using the meta-Huffman.
    let cl_codes = canonical_codes_from_lengths(&cl_lengths);
    for &l in &lengths[..alphabet_size] {
        let (code, len) = cl_codes[l as usize];
        if len == 0 {
            bw.write_bits(0, 1);
        } else {
            bw.write_bits(code as u64, len);
        }
    }
}

/// Read a HuffSpec from the bitstream and build a HuffTree.
fn read_huff_tree(br: &mut BitReader<'_>, alphabet_size: usize) -> Result<HuffTree, WebpError> {
    let is_simple = br.read_bits(1)? != 0;
    if is_simple {
        let num_syms_minus1 = br.read_bits(1)?;
        if num_syms_minus1 == 0 {
            // One symbol.
            let sym = br.read_bits(8)? as usize;
            let mut lengths = vec![0u8; alphabet_size];
            if sym < alphabet_size {
                lengths[sym] = 1;
            }
            return Ok(HuffTree::build(&lengths));
        } else {
            // Two symbols.
            let _first_uses_8bits = br.read_bits(1)?; // always 1 in our encoder
            let sym0 = br.read_bits(8)? as usize;
            let sym1 = br.read_bits(8)? as usize;
            let mut lengths = vec![0u8; alphabet_size];
            if sym0 < alphabet_size {
                lengths[sym0] = 1;
            }
            if sym1 < alphabet_size && sym1 != sym0 {
                lengths[sym1] = 1;
            }
            return Ok(HuffTree::build(&lengths));
        }
    }

    // Standard tree.
    let _use_length_limit = br.read_bits(1)?; // we wrote 0; ignoring for decoding

    const NUM_META: usize = 19;
    const CL_ORDER: [usize; 19] = [
        17, 18, 0, 1, 2, 3, 4, 5, 16, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ];

    let num_cl = (br.read_bits(4)? as usize) + 4;
    let mut cl_lengths = [0u8; NUM_META];
    for i in 0..num_cl {
        cl_lengths[CL_ORDER[i]] = br.read_bits(3)? as u8;
    }

    let cl_tree = HuffTree::build(&cl_lengths);

    // Decode `alphabet_size` code lengths using cl_tree.
    let mut lengths = vec![0u8; alphabet_size];
    for i in 0..alphabet_size {
        let sym = cl_tree.decode(br)?;
        // Values 0..=15 → direct code length; 16/17/18 → RLE (our encoder never emits these)
        match sym {
            0..=15 => lengths[i] = sym as u8,
            16 => {
                // Repeat previous length 3+extra(2bits) times
                let extra = br.read_bits(2)? as usize;
                let prev = if i > 0 { lengths[i - 1] } else { 0 };
                for j in 0..(3 + extra) {
                    if i + j < alphabet_size {
                        lengths[i + j] = prev;
                    }
                }
            }
            17 => {
                // Repeat zero length 3+extra(3bits) times
                let extra = br.read_bits(3)? as usize;
                for j in 0..(3 + extra) {
                    if i + j < alphabet_size {
                        lengths[i + j] = 0;
                    }
                }
            }
            18 => {
                // Repeat zero length 11+extra(7bits) times
                let extra = br.read_bits(7)? as usize;
                for j in 0..(11 + extra) {
                    if i + j < alphabet_size {
                        lengths[i + j] = 0;
                    }
                }
            }
            _ => return Err(WebpError::Invalid(format!("unexpected CL code {}", sym))),
        }
    }

    Ok(HuffTree::build(&lengths))
}

// ── VP8L alphabet sizes ────────────────────────────────────────────────────────

/// Size of the "green" alphabet: 256 literal green values + 24 copy-length codes = 280.
/// For literal-only encoding we use 256.
const ALPHA_G: usize = 256;
const ALPHA_R: usize = 256;
const ALPHA_B: usize = 256;
const ALPHA_A: usize = 256;
const ALPHA_D: usize = 40; // distance alphabet (backward references — unused in literal mode)

// ── Encoder ───────────────────────────────────────────────────────────────────

/// Encode raw RGB24 pixels (row-major, top-to-bottom) as a WebP file containing a VP8L stream.
///
/// `pixels` must be exactly `width * height * 3` bytes.
pub fn webp_encode_rgb(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>, WebpError> {
    let pixel_count = (width as usize) * (height as usize);
    if pixels.len() != pixel_count * 3 {
        return Err(WebpError::Invalid(format!(
            "pixel buffer length {} != expected {}",
            pixels.len(),
            pixel_count * 3
        )));
    }
    if width == 0 || height == 0 {
        return Err(WebpError::Invalid("zero dimension".into()));
    }
    if width > 16384 || height > 16384 {
        return Err(WebpError::Invalid(
            "dimension exceeds VP8L max of 16384".into(),
        ));
    }

    // Convert RGB → ARGB (alpha = 255).
    let mut argb: Vec<[u8; 4]> = Vec::with_capacity(pixel_count);
    for px in pixels.chunks_exact(3) {
        argb.push([255, px[0], px[1], px[2]]); // A, R, G, B
    }

    // Build frequency tables for G, R, B, A channels.
    let mut freq_g = [0u32; ALPHA_G];
    let mut freq_r = [0u32; ALPHA_R];
    let mut freq_b = [0u32; ALPHA_B];
    let mut freq_a = [0u32; ALPHA_A];
    let freq_d = [0u32; ALPHA_D]; // no backward refs

    for &[a, r, g, b] in &argb {
        freq_g[g as usize] += 1;
        freq_r[r as usize] += 1;
        freq_b[b as usize] += 1;
        freq_a[a as usize] += 1;
    }

    // Choose HuffSpec for each channel.
    let spec_g = analyse_freqs(&freq_g, ALPHA_G);
    let spec_r = analyse_freqs(&freq_r, ALPHA_R);
    let spec_b = analyse_freqs(&freq_b, ALPHA_B);
    let spec_a = analyse_freqs(&freq_a, ALPHA_A);
    let spec_d = analyse_freqs(&freq_d, ALPHA_D);

    // Build encode code tables.
    let codes_g = build_encode_table(&spec_g, ALPHA_G);
    let codes_r = build_encode_table(&spec_r, ALPHA_R);
    let codes_b = build_encode_table(&spec_b, ALPHA_B);
    let codes_a = build_encode_table(&spec_a, ALPHA_A);

    // ── Write VP8L bitstream ──────────────────────────────────────────────────
    let mut bw = BitWriter::new();

    // Dimensions (14 bits each, value = actual_dim - 1).
    bw.write_bits((width - 1) as u64, 14);
    bw.write_bits((height - 1) as u64, 14);

    // alpha_is_used = 0 (all pixels have alpha=255, so not "meaningful" alpha).
    bw.write_bits(0, 1);

    // version = 0 (3 bits).
    bw.write_bits(0, 3);

    // transform_present = 0.
    bw.write_bits(0, 1);

    // color_cache_code_bits = 0 (no color cache; 1 bit).
    bw.write_bits(0, 1);

    // huffman_meta = 0 (single meta group, 1 bit).
    bw.write_bits(0, 1);

    // Write 5 Huffman trees in order: G, R, B, A, Distance.
    write_huff_tree(&mut bw, &spec_g);
    write_huff_tree(&mut bw, &spec_r);
    write_huff_tree(&mut bw, &spec_b);
    write_huff_tree(&mut bw, &spec_a);
    write_huff_tree(&mut bw, &spec_d);

    // Write pixel data: for each pixel, emit G, R, B, A symbols.
    for &[a, r, g, b] in &argb {
        encode_symbol(&mut bw, &codes_g, g as usize);
        encode_symbol(&mut bw, &codes_r, r as usize);
        encode_symbol(&mut bw, &codes_b, b as usize);
        encode_symbol(&mut bw, &codes_a, a as usize);
    }

    let vp8l_data = bw.into_bytes();

    // ── Assemble RIFF/WEBP container ─────────────────────────────────────────
    // VP8L chunk payload = 1-byte signature (0x2F) + vp8l_data
    let chunk_payload_size = 1 + vp8l_data.len();
    let chunk_size = chunk_payload_size as u32;

    // RIFF total file size = 4 (WEBP) + 4 (VP8L) + 4 (chunk_size) + chunk_payload
    // Chunks must be padded to even size per RIFF spec.
    let padded_chunk = (chunk_payload_size + 1) & !1;
    let riff_size = 4 + 4 + 4 + padded_chunk;

    let mut out: Vec<u8> = Vec::with_capacity(12 + padded_chunk);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(riff_size as u32).to_le_bytes());
    out.extend_from_slice(b"WEBP");
    out.extend_from_slice(b"VP8L");
    out.extend_from_slice(&chunk_size.to_le_bytes());
    out.push(0x2F); // VP8L signature byte
    out.extend_from_slice(&vp8l_data);
    if chunk_payload_size & 1 == 1 {
        out.push(0x00); // pad to even
    }

    Ok(out)
}

/// Build a flat encode table (symbol → (code: u16, len: u8)) from a HuffSpec.
fn build_encode_table(spec: &HuffSpec, alphabet_size: usize) -> Vec<(u16, u8)> {
    match spec {
        HuffSpec::Simple1 { symbol } => {
            let mut table = vec![(0u16, 0u8); alphabet_size];
            if (*symbol as usize) < alphabet_size {
                table[*symbol as usize] = (0, 1);
            }
            table
        }
        HuffSpec::Simple2 { sym0, sym1 } => {
            let mut table = vec![(0u16, 0u8); alphabet_size];
            if (*sym0 as usize) < alphabet_size {
                table[*sym0 as usize] = (0, 1); // code 0
            }
            if (*sym1 as usize) < alphabet_size {
                table[*sym1 as usize] = (1, 1); // code 1
            }
            table
        }
        HuffSpec::Standard { lengths } => canonical_codes_from_lengths(lengths),
    }
}

/// Emit one symbol from a pre-built encode table.
#[inline]
fn encode_symbol(bw: &mut BitWriter, codes: &[(u16, u8)], sym: usize) {
    if sym < codes.len() {
        let (code, len) = codes[sym];
        if len > 0 {
            bw.write_bits(code as u64, len);
        } else {
            // len=0 means single-symbol simple tree → write nothing (implicit).
        }
    }
}

// ── Decoder ───────────────────────────────────────────────────────────────────

/// Decode a WebP file (VP8L lossless only) into raw RGB24 pixels.
pub fn webp_decode(bytes: &[u8]) -> Result<RawDecodeResult, WebpError> {
    // ── Parse RIFF container ──────────────────────────────────────────────────
    if bytes.len() < 12 {
        return Err(WebpError::Truncated);
    }
    if &bytes[0..4] != b"RIFF" {
        return Err(WebpError::Invalid("missing RIFF header".into()));
    }
    if &bytes[8..12] != b"WEBP" {
        return Err(WebpError::Invalid("missing WEBP fourcc".into()));
    }

    // Find VP8L chunk.
    let mut pos = 12usize;
    let file_end = bytes.len();

    while pos + 8 <= file_end {
        let fourcc = &bytes[pos..pos + 4];
        let chunk_size = u32::from_le_bytes(
            bytes[pos + 4..pos + 8]
                .try_into()
                .map_err(|_| WebpError::Truncated)?,
        ) as usize;
        let data_start = pos + 8;
        let data_end = data_start + chunk_size;

        if fourcc == b"VP8L" {
            if bytes.len() < data_end {
                return Err(WebpError::Truncated);
            }
            let chunk_data = &bytes[data_start..data_end];
            return decode_vp8l(chunk_data);
        } else if fourcc == b"VP8 " || fourcc == b"VP8X" {
            return Err(WebpError::Unsupported(
                "only VP8L (lossless) is supported".into(),
            ));
        }

        // Advance to next chunk (RIFF pads chunks to even size).
        let padded = (chunk_size + 1) & !1;
        pos = data_start + padded;
    }

    Err(WebpError::Invalid("VP8L chunk not found".into()))
}

/// Decode the VP8L chunk payload (after the chunk header, starting with 0x2F signature).
fn decode_vp8l(data: &[u8]) -> Result<RawDecodeResult, WebpError> {
    if data.is_empty() {
        return Err(WebpError::Truncated);
    }
    if data[0] != 0x2F {
        return Err(WebpError::Invalid(format!(
            "VP8L signature byte expected 0x2F, got 0x{:02X}",
            data[0]
        )));
    }

    let mut br = BitReader::new(&data[1..]);

    // Dimensions.
    let width = (br.read_bits(14)? as u32) + 1;
    let height = (br.read_bits(14)? as u32) + 1;

    let _alpha_used = br.read_bits(1)?;
    let version = br.read_bits(3)?;
    if version != 0 {
        return Err(WebpError::Invalid(format!(
            "VP8L version must be 0, got {}",
            version
        )));
    }

    // Transform present flag.
    let transform_present = br.read_bits(1)?;
    if transform_present != 0 {
        return Err(WebpError::Unsupported(
            "VP8L transforms are not supported".into(),
        ));
    }

    // Color cache code bits.
    let color_cache_code_bits = br.read_bits(1)?;
    if color_cache_code_bits != 0 {
        return Err(WebpError::Unsupported(
            "VP8L color cache not supported".into(),
        ));
    }

    // Huffman meta.
    let huffman_meta = br.read_bits(1)?;
    if huffman_meta != 0 {
        return Err(WebpError::Unsupported(
            "VP8L meta Huffman not supported".into(),
        ));
    }

    // Read 5 Huffman trees: G, R, B, A, Distance.
    let tree_g = read_huff_tree(&mut br, ALPHA_G)?;
    let tree_r = read_huff_tree(&mut br, ALPHA_R)?;
    let tree_b = read_huff_tree(&mut br, ALPHA_B)?;
    let tree_a = read_huff_tree(&mut br, ALPHA_A)?;
    let _tree_d = read_huff_tree(&mut br, ALPHA_D)?;

    // Decode pixel data.
    let pixel_count = (width as usize) * (height as usize);
    let mut rgb_pixels: Vec<u8> = Vec::with_capacity(pixel_count * 3);

    for _ in 0..pixel_count {
        let g = tree_g.decode(&mut br)? as u8;
        let r = tree_r.decode(&mut br)? as u8;
        let b = tree_b.decode(&mut br)? as u8;
        let _a = tree_a.decode(&mut br)? as u8;
        rgb_pixels.push(r);
        rgb_pixels.push(g);
        rgb_pixels.push(b);
    }

    Ok(RawDecodeResult {
        width: width as usize,
        height: height as usize,
        pixels: rgb_pixels,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solid_rgb(w: usize, h: usize, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 3);
        for _ in 0..(w * h) {
            v.push(r);
            v.push(g);
            v.push(b);
        }
        v
    }

    fn make_gradient(w: usize, h: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                v.push(((x * 255) / w.max(1)) as u8);
                v.push(((y * 255) / h.max(1)) as u8);
                v.push(((x + y) % 256) as u8);
            }
        }
        v
    }

    #[test]
    fn test_webp_header() {
        let pixels = make_solid_rgb(4, 4, 200, 100, 50);
        let encoded = webp_encode_rgb(4, 4, &pixels).expect("encode must succeed");
        assert!(encoded.starts_with(b"RIFF"), "must start with RIFF");
        assert_eq!(&encoded[8..12], b"WEBP", "must contain WEBP");
        assert!(
            encoded.windows(4).any(|w| w == b"VP8L"),
            "must contain VP8L chunk marker"
        );
    }

    #[test]
    fn test_webp_roundtrip_solid_color() {
        let pixels = make_solid_rgb(4, 4, 127, 63, 200);
        let encoded = webp_encode_rgb(4, 4, &pixels).expect("encode");
        let decoded = webp_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 4);
        assert_eq!(decoded.height, 4);
        assert_eq!(
            decoded.pixels, pixels,
            "solid-color round-trip must be exact"
        );
    }

    #[test]
    fn test_webp_roundtrip_gradient() {
        let pixels = make_gradient(8, 8);
        let encoded = webp_encode_rgb(8, 8, &pixels).expect("encode");
        let decoded = webp_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 8);
        assert_eq!(decoded.height, 8);
        assert_eq!(decoded.pixels, pixels, "gradient round-trip must be exact");
    }

    #[test]
    fn test_webp_decode_invalid_returns_error() {
        let result = webp_decode(b"not a webp file at all 12345678");
        assert!(result.is_err(), "invalid input must return an error");
    }

    #[test]
    fn test_webp_roundtrip_single_pixel() {
        let pixels = vec![42u8, 84u8, 168u8]; // 1×1 RGB
        let encoded = webp_encode_rgb(1, 1, &pixels).expect("encode");
        let decoded = webp_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 1);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.pixels, pixels);
    }

    #[test]
    fn test_webp_roundtrip_checkerboard() {
        // 4×4 checkerboard of two colors — exercises 2-symbol simple trees.
        let mut pixels = Vec::with_capacity(4 * 4 * 3);
        for y in 0..4usize {
            for x in 0..4usize {
                if (x + y) % 2 == 0 {
                    pixels.extend_from_slice(&[255, 0, 0]);
                } else {
                    pixels.extend_from_slice(&[0, 0, 255]);
                }
            }
        }
        let encoded = webp_encode_rgb(4, 4, &pixels).expect("encode");
        let decoded = webp_decode(&encoded).expect("decode");
        assert_eq!(decoded.pixels, pixels);
    }
}
