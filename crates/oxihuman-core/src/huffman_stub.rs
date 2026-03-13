// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Real Huffman coding implementation with tree construction, canonical codes,
//! bit-level encoding/decoding, and length-limited codes (max 15 bits).

#![allow(dead_code)]

use std::collections::BinaryHeap;

/// Maximum allowed code length (like DEFLATE).
const MAX_CODE_LEN: u8 = 15;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A node in the Huffman tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HuffNode {
    pub symbol: Option<u8>,
    pub freq: u64,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

/// The full Huffman tree stored as a flat node array.
#[derive(Debug, Clone)]
pub struct HuffmanTree {
    pub nodes: Vec<HuffNode>,
    /// Index of the root node.
    root: Option<usize>,
}

/// Lookup table: for each symbol 0..=255, `(code_bits, code_length)`.
/// Symbols that do not appear have `code_length == 0`.
#[derive(Debug, Clone)]
pub struct HuffmanCodeTable {
    /// Indexed by symbol value (0..=255). `(code_bits, code_length)`.
    pub codes: Vec<(u32, u8)>,
}

/// A symbol with frequency and assigned code length (legacy public API).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HuffmanSymbol {
    pub byte: u8,
    pub frequency: usize,
    pub code_len: u8,
}

/// A frequency table mapping bytes to HuffmanSymbol entries (legacy API).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HuffmanTable {
    pub symbols: Vec<HuffmanSymbol>,
}

/// Bit-level writer: packs bits into a `Vec<u8>`.
#[derive(Debug, Clone)]
pub struct BitWriter {
    pub buffer: Vec<u8>,
    pub bit_pos: usize,
}

/// Bit-level reader over a byte slice.
#[derive(Debug, Clone)]
pub struct BitReader<'a> {
    pub data: &'a [u8],
    pub bit_pos: usize,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that may occur during Huffman operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HuffmanError {
    /// The input data is empty (no symbols to encode).
    EmptyInput,
    /// A symbol was not found in the table during encoding.
    SymbolNotFound(u8),
    /// Ran out of bits while decoding.
    UnexpectedEndOfStream,
    /// Decoded bit sequence does not match any symbol.
    InvalidCode,
}

impl std::fmt::Display for HuffmanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "empty input"),
            Self::SymbolNotFound(s) => write!(f, "symbol {s} not in table"),
            Self::UnexpectedEndOfStream => write!(f, "unexpected end of bit stream"),
            Self::InvalidCode => write!(f, "invalid huffman code in stream"),
        }
    }
}

impl std::error::Error for HuffmanError {}

// ---------------------------------------------------------------------------
// BitWriter
// ---------------------------------------------------------------------------

impl BitWriter {
    /// Create a new, empty bit writer.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            bit_pos: 0,
        }
    }

    /// Create a new writer with pre-allocated capacity (in bytes).
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(bytes),
            bit_pos: 0,
        }
    }

    /// Write `num_bits` least-significant bits of `value` to the stream,
    /// MSB first (big-endian bit order within each code word).
    pub fn write_bits(&mut self, value: u32, num_bits: u8) {
        for i in (0..num_bits).rev() {
            let bit = (value >> i) & 1;
            let byte_idx = self.bit_pos / 8;
            let bit_idx = 7 - (self.bit_pos % 8);
            if byte_idx >= self.buffer.len() {
                self.buffer.push(0);
            }
            if bit == 1 {
                self.buffer[byte_idx] |= 1 << bit_idx;
            }
            self.bit_pos += 1;
        }
    }

    /// Total number of bits written so far.
    pub fn total_bits(&self) -> usize {
        self.bit_pos
    }

    /// Consume the writer, returning `(byte_buffer, total_bit_count)`.
    pub fn finish(self) -> (Vec<u8>, usize) {
        (self.buffer, self.bit_pos)
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BitReader
// ---------------------------------------------------------------------------

impl<'a> BitReader<'a> {
    /// Create a new reader over the given byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    /// Read `num_bits` from the stream (MSB first), returning the value.
    /// Returns `None` if not enough bits remain.
    pub fn read_bits(&mut self, num_bits: u8) -> Option<u32> {
        let total_bits = self.data.len() * 8;
        if self.bit_pos + num_bits as usize > total_bits {
            return None;
        }
        let mut value: u32 = 0;
        for _ in 0..num_bits {
            let byte_idx = self.bit_pos / 8;
            let bit_idx = 7 - (self.bit_pos % 8);
            let bit = (self.data[byte_idx] >> bit_idx) & 1;
            value = (value << 1) | bit as u32;
            self.bit_pos += 1;
        }
        Some(value)
    }

    /// Read a single bit, returning 0 or 1, or `None` at end.
    pub fn read_bit(&mut self) -> Option<u32> {
        self.read_bits(1)
    }

    /// How many bits have been consumed.
    pub fn position(&self) -> usize {
        self.bit_pos
    }

    /// Set the reader position (in bits).
    pub fn set_position(&mut self, pos: usize) {
        self.bit_pos = pos;
    }
}

// ---------------------------------------------------------------------------
// HuffmanTree - build from frequencies using a min-heap
// ---------------------------------------------------------------------------

/// Entry used in the priority queue while building the tree.
#[derive(Debug, Clone, Eq, PartialEq)]
struct HeapEntry {
    freq: u64,
    /// Tie-break: lower index wins (keeps tree deterministic).
    node_idx: usize,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We want a min-heap, so reverse the natural order.
        other
            .freq
            .cmp(&self.freq)
            .then_with(|| other.node_idx.cmp(&self.node_idx))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl HuffmanTree {
    /// Build a Huffman tree from a frequency array (indexed by symbol 0..=255).
    /// Returns `None` if all frequencies are zero.
    pub fn build(freq: &[u64; 256]) -> Option<Self> {
        let mut nodes: Vec<HuffNode> = Vec::new();
        let mut heap = BinaryHeap::new();

        for (sym, &f) in freq.iter().enumerate() {
            if f > 0 {
                let idx = nodes.len();
                nodes.push(HuffNode {
                    symbol: Some(sym as u8),
                    freq: f,
                    left: None,
                    right: None,
                });
                heap.push(HeapEntry {
                    freq: f,
                    node_idx: idx,
                });
            }
        }

        if heap.is_empty() {
            return None;
        }

        // Special case: single symbol - we still need a tree with depth 1.
        if heap.len() == 1 {
            let entry = heap.pop()?;
            let root_idx = nodes.len();
            nodes.push(HuffNode {
                symbol: None,
                freq: entry.freq,
                left: Some(entry.node_idx),
                right: None,
            });
            return Some(Self {
                nodes,
                root: Some(root_idx),
            });
        }

        while heap.len() >= 2 {
            let a = heap.pop()?;
            let b = heap.pop()?;
            let combined_freq = a.freq + b.freq;
            let parent_idx = nodes.len();
            nodes.push(HuffNode {
                symbol: None,
                freq: combined_freq,
                left: Some(a.node_idx),
                right: Some(b.node_idx),
            });
            heap.push(HeapEntry {
                freq: combined_freq,
                node_idx: parent_idx,
            });
        }

        let root_entry = heap.pop()?;
        Some(Self {
            nodes,
            root: Some(root_entry.node_idx),
        })
    }

    /// Extract code lengths per symbol by walking the tree.
    /// Returns an array of 256 code lengths.
    pub fn code_lengths(&self) -> [u8; 256] {
        let mut lengths = [0u8; 256];
        if let Some(root) = self.root {
            self.walk(root, 0, &mut lengths);
        }
        lengths
    }

    fn walk(&self, idx: usize, depth: u8, lengths: &mut [u8; 256]) {
        let node = &self.nodes[idx];
        if let Some(sym) = node.symbol {
            // Leaf node - depth is the code length (min 1).
            lengths[sym as usize] = depth.max(1);
            return;
        }
        if let Some(left) = node.left {
            self.walk(left, depth.saturating_add(1), lengths);
        }
        if let Some(right) = node.right {
            self.walk(right, depth.saturating_add(1), lengths);
        }
    }
}

// ---------------------------------------------------------------------------
// Length-limited Huffman codes
// ---------------------------------------------------------------------------

/// Limit code lengths to `max_len` using a heuristic:
/// clamp all to max_len, then fix the Kraft inequality by lengthening short codes.
fn limit_code_lengths(lengths: &mut [u8; 256], max_len: u8) {
    let needs_limiting = lengths.iter().any(|&l| l > max_len);
    if !needs_limiting {
        return;
    }

    // Collect active symbols.
    let mut syms: Vec<(usize, u8)> = lengths
        .iter()
        .enumerate()
        .filter(|(_, &l)| l > 0)
        .map(|(s, &l)| (s, l))
        .collect();

    // Clamp all lengths to max_len.
    for (_, len) in &mut syms {
        if *len > max_len {
            *len = max_len;
        }
    }

    // Fix Kraft inequality: sum(2^(max_len - l_i)) <= 2^max_len
    loop {
        let kraft_sum: u64 = syms.iter().map(|(_, l)| 1u64 << (max_len - *l)).sum();
        let kraft_limit = 1u64 << max_len;

        if kraft_sum <= kraft_limit {
            break;
        }

        // Sort by length ascending, then symbol ascending.
        syms.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let mut fixed = false;
        for (_, len) in &mut syms {
            if *len < max_len {
                *len += 1;
                fixed = true;
                break;
            }
        }
        if !fixed {
            break;
        }
    }

    // Write back.
    for &(s, l) in &syms {
        lengths[s] = l;
    }
}

// ---------------------------------------------------------------------------
// Canonical Huffman code assignment
// ---------------------------------------------------------------------------

impl HuffmanCodeTable {
    /// Build a canonical Huffman table from code lengths.
    /// Symbols with `length == 0` are not encoded.
    pub fn from_lengths(lengths: &[u8; 256]) -> Self {
        let mut codes = vec![(0u32, 0u8); 256];

        // Collect (symbol, length) pairs for active symbols, then sort.
        let mut active: Vec<(u8, u8)> = lengths
            .iter()
            .enumerate()
            .filter(|(_, &l)| l > 0)
            .map(|(s, &l)| (s as u8, l))
            .collect();

        // Canonical ordering: by (length, symbol).
        active.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        if active.is_empty() {
            return Self { codes };
        }

        let mut code: u32 = 0;
        let mut prev_len = active[0].1;

        for (i, &(sym, len)) in active.iter().enumerate() {
            if i > 0 {
                code += 1;
                if len > prev_len {
                    code <<= len - prev_len;
                }
            }
            codes[sym as usize] = (code, len);
            prev_len = len;
        }

        Self { codes }
    }

    /// Build a Huffman table directly from raw data bytes.
    /// Returns `None` if `data` is empty.
    pub fn from_data(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        let mut freq = [0u64; 256];
        for &b in data {
            freq[b as usize] += 1;
        }
        let tree = HuffmanTree::build(&freq)?;
        let mut lengths = tree.code_lengths();
        limit_code_lengths(&mut lengths, MAX_CODE_LEN);
        Some(Self::from_lengths(&lengths))
    }

    /// Look up the code for a given symbol.
    /// Returns `None` if the symbol is not in the table.
    pub fn lookup(&self, symbol: u8) -> Option<(u32, u8)> {
        let (bits, len) = self.codes[symbol as usize];
        if len == 0 {
            None
        } else {
            Some((bits, len))
        }
    }
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/// Encode a slice of bytes into a packed bit stream using the given table.
/// Returns `(byte_buffer, total_bit_count)`.
pub fn huffman_encode(
    data: &[u8],
    table: &HuffmanCodeTable,
) -> Result<(Vec<u8>, usize), HuffmanError> {
    if data.is_empty() {
        return Err(HuffmanError::EmptyInput);
    }
    let mut writer = BitWriter::with_capacity(data.len());
    for &b in data {
        let (bits, len) = table.lookup(b).ok_or(HuffmanError::SymbolNotFound(b))?;
        writer.write_bits(bits, len);
    }
    Ok(writer.finish())
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// Decode lookup structure for efficient symbol resolution.
struct DecodeLookup {
    /// (code_bits, code_length, symbol), sorted by (length, code).
    entries: Vec<(u32, u8, u8)>,
}

impl DecodeLookup {
    fn from_table(table: &HuffmanCodeTable) -> Self {
        let mut entries: Vec<(u32, u8, u8)> = table
            .codes
            .iter()
            .enumerate()
            .filter(|(_, &(_, len))| len > 0)
            .map(|(sym, &(bits, len))| (bits, len, sym as u8))
            .collect();
        entries.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        Self { entries }
    }

    /// Decode one symbol from the reader.
    fn decode_one(&self, reader: &mut BitReader<'_>) -> Result<u8, HuffmanError> {
        let start = reader.position();
        let mut accumulated: u32 = 0;
        let mut bits_read: u8 = 0;

        for &(code, len, sym) in &self.entries {
            while bits_read < len {
                let bit = reader
                    .read_bit()
                    .ok_or(HuffmanError::UnexpectedEndOfStream)?;
                accumulated = (accumulated << 1) | bit;
                bits_read += 1;
            }
            if bits_read == len && accumulated == code {
                return Ok(sym);
            }
        }

        reader.set_position(start);
        Err(HuffmanError::InvalidCode)
    }
}

/// Decode a packed bit stream back to symbols.
///
/// - `data`: the byte buffer containing packed bits.
/// - `bit_count`: total number of valid bits in the stream.
/// - `symbol_count`: how many symbols to decode.
/// - `table`: the Huffman table used for encoding.
pub fn huffman_decode(
    data: &[u8],
    bit_count: usize,
    symbol_count: usize,
    table: &HuffmanCodeTable,
) -> Result<Vec<u8>, HuffmanError> {
    let lookup = DecodeLookup::from_table(table);
    let mut reader = BitReader::new(data);
    let mut output = Vec::with_capacity(symbol_count);

    for _ in 0..symbol_count {
        if reader.position() >= bit_count {
            return Err(HuffmanError::UnexpectedEndOfStream);
        }
        let sym = lookup.decode_one(&mut reader)?;
        output.push(sym);
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Legacy public API (kept for backward compatibility)
// ---------------------------------------------------------------------------

/// Build a frequency table from the given data slice (legacy API).
///
/// This now uses a real Huffman tree to assign code lengths.
#[allow(dead_code)]
pub fn build_frequency_table(data: &[u8]) -> HuffmanTable {
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    let mut symbols: Vec<HuffmanSymbol> = freq
        .iter()
        .enumerate()
        .filter(|(_, &f)| f > 0)
        .map(|(i, &f)| HuffmanSymbol {
            byte: i as u8,
            frequency: f as usize,
            code_len: 0,
        })
        .collect();

    // Build real tree and get lengths.
    if let Some(tree) = HuffmanTree::build(&freq) {
        let mut lengths = tree.code_lengths();
        limit_code_lengths(&mut lengths, MAX_CODE_LEN);
        for sym in &mut symbols {
            sym.code_len = lengths[sym.byte as usize];
        }
    }

    // Sort by frequency descending (legacy behavior).
    symbols.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    HuffmanTable { symbols }
}

/// Encode a byte to its stub code (index in table), or `None` if not present.
#[allow(dead_code)]
pub fn encode_symbol(table: &HuffmanTable, byte: u8) -> Option<u8> {
    table
        .symbols
        .iter()
        .enumerate()
        .find(|(_, s)| s.byte == byte)
        .map(|(i, _)| i as u8)
}

/// Decode a code back to the original byte, or `None` if code out of range.
#[allow(dead_code)]
pub fn decode_symbol(table: &HuffmanTable, code: u8) -> Option<u8> {
    table.symbols.get(code as usize).map(|s| s.byte)
}

/// Return the number of symbols in the table.
#[allow(dead_code)]
pub fn table_size(table: &HuffmanTable) -> usize {
    table.symbols.len()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Legacy API tests (kept from original stub)
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_data_gives_empty_table() {
        let t = build_frequency_table(&[]);
        assert_eq!(table_size(&t), 0);
    }

    #[test]
    fn test_single_byte_table() {
        let t = build_frequency_table(&[42u8; 10]);
        assert_eq!(table_size(&t), 1);
        assert_eq!(t.symbols[0].byte, 42);
        assert_eq!(t.symbols[0].frequency, 10);
    }

    #[test]
    fn test_multiple_bytes_sorted_by_frequency() {
        let data = [1u8, 1, 1, 2, 2, 3];
        let t = build_frequency_table(&data);
        assert!(t.symbols[0].frequency >= t.symbols[1].frequency);
    }

    #[test]
    fn test_encode_symbol_found() {
        let data = [5u8, 5, 5, 10, 10];
        let t = build_frequency_table(&data);
        // byte 5 has highest freq -> code 0
        assert_eq!(encode_symbol(&t, 5), Some(0));
    }

    #[test]
    fn test_encode_symbol_not_found() {
        let data = [1u8, 2, 3];
        let t = build_frequency_table(&data);
        assert_eq!(encode_symbol(&t, 99), None);
    }

    #[test]
    fn test_decode_symbol_roundtrip() {
        let data = [7u8, 7, 8, 9];
        let t = build_frequency_table(&data);
        let code = encode_symbol(&t, 7).expect("should succeed");
        assert_eq!(decode_symbol(&t, code), Some(7));
    }

    #[test]
    fn test_decode_out_of_range() {
        let t = build_frequency_table(&[1u8, 2]);
        assert_eq!(decode_symbol(&t, 200), None);
    }

    #[test]
    fn test_code_len_assigned() {
        let data = [0u8, 0, 1, 2];
        let t = build_frequency_table(&data);
        // All symbols get code_len >= 1
        for sym in &t.symbols {
            assert!(sym.code_len >= 1);
        }
    }

    #[test]
    fn test_table_size_matches_unique_bytes() {
        let data = [10u8, 20, 30, 10, 20];
        let t = build_frequency_table(&data);
        assert_eq!(table_size(&t), 3);
    }

    // -----------------------------------------------------------------------
    // BitWriter / BitReader tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bit_writer_single_byte() {
        let mut w = BitWriter::new();
        w.write_bits(0b10110011, 8);
        assert_eq!(w.total_bits(), 8);
        let (buf, bits) = w.finish();
        assert_eq!(bits, 8);
        assert_eq!(buf, vec![0b10110011]);
    }

    #[test]
    fn test_bit_writer_partial_byte() {
        let mut w = BitWriter::new();
        w.write_bits(0b101, 3);
        assert_eq!(w.total_bits(), 3);
        let (buf, bits) = w.finish();
        assert_eq!(bits, 3);
        // 101 written to top 3 bits of byte => 10100000
        assert_eq!(buf, vec![0b10100000]);
    }

    #[test]
    fn test_bit_roundtrip() {
        let mut w = BitWriter::new();
        w.write_bits(0b110, 3);
        w.write_bits(0b01011, 5);
        w.write_bits(0b1, 1);
        let (buf, total) = w.finish();
        assert_eq!(total, 9);

        let mut r = BitReader::new(&buf);
        assert_eq!(r.read_bits(3), Some(0b110));
        assert_eq!(r.read_bits(5), Some(0b01011));
        assert_eq!(r.read_bits(1), Some(0b1));
    }

    #[test]
    fn test_bit_reader_out_of_bounds() {
        let data = [0xFF];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read_bits(8), Some(0xFF));
        assert_eq!(r.read_bits(1), None);
    }

    // -----------------------------------------------------------------------
    // Huffman tree tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_tree_build_empty() {
        let freq = [0u64; 256];
        assert!(HuffmanTree::build(&freq).is_none());
    }

    #[test]
    fn test_tree_single_symbol() {
        let mut freq = [0u64; 256];
        freq[65] = 100; // 'A'
        let tree = HuffmanTree::build(&freq).expect("should succeed");
        let lengths = tree.code_lengths();
        assert_eq!(lengths[65], 1);
        for (i, &l) in lengths.iter().enumerate() {
            if i != 65 {
                assert_eq!(l, 0);
            }
        }
    }

    #[test]
    fn test_tree_two_symbols() {
        let mut freq = [0u64; 256];
        freq[0] = 10;
        freq[1] = 5;
        let tree = HuffmanTree::build(&freq).expect("should succeed");
        let lengths = tree.code_lengths();
        assert_eq!(lengths[0], 1);
        assert_eq!(lengths[1], 1);
    }

    #[test]
    fn test_tree_multiple_symbols_kraft_inequality() {
        let mut freq = [0u64; 256];
        freq[0] = 100;
        freq[1] = 50;
        freq[2] = 25;
        freq[3] = 12;
        let tree = HuffmanTree::build(&freq).expect("should succeed");
        let lengths = tree.code_lengths();

        let kraft: f64 = lengths
            .iter()
            .filter(|&&l| l > 0)
            .map(|&l| 2.0f64.powi(-(l as i32)))
            .sum();
        assert!(kraft <= 1.0 + 1e-10, "Kraft inequality violated: {kraft}");
    }

    // -----------------------------------------------------------------------
    // Canonical code tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_canonical_codes_simple() {
        let mut lengths = [0u8; 256];
        lengths[b'A' as usize] = 1;
        lengths[b'B' as usize] = 2;
        lengths[b'C' as usize] = 2;

        let table = HuffmanCodeTable::from_lengths(&lengths);

        let (a_bits, a_len) = table.codes[b'A' as usize];
        let (b_bits, b_len) = table.codes[b'B' as usize];
        let (c_bits, c_len) = table.codes[b'C' as usize];

        assert_eq!(a_len, 1);
        assert_eq!(b_len, 2);
        assert_eq!(c_len, 2);

        // Canonical assignment: A=0, B=10, C=11
        assert_eq!(a_bits, 0b0);
        assert_eq!(b_bits, 0b10);
        assert_eq!(c_bits, 0b11);
    }

    // -----------------------------------------------------------------------
    // Length limiting tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_length_limiting() {
        let mut lengths = [0u8; 256];
        lengths[..32].fill(20);
        limit_code_lengths(&mut lengths, MAX_CODE_LEN);
        for (i, &len) in lengths[..32].iter().enumerate() {
            assert!(
                len <= MAX_CODE_LEN,
                "symbol {i} has length {} > {MAX_CODE_LEN}",
                len
            );
        }
    }

    #[test]
    fn test_length_limiting_preserves_kraft() {
        let mut lengths = [0u8; 256];
        lengths[..16].fill(18);
        limit_code_lengths(&mut lengths, MAX_CODE_LEN);

        let kraft: f64 = lengths
            .iter()
            .filter(|&&l| l > 0)
            .map(|&l| 2.0f64.powi(-(l as i32)))
            .sum();
        assert!(
            kraft <= 1.0 + 1e-10,
            "Kraft inequality violated after limiting: {kraft}"
        );
    }

    // -----------------------------------------------------------------------
    // Encode / Decode roundtrip tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_decode_roundtrip_simple() {
        let data = b"aabbbc";
        let table = HuffmanCodeTable::from_data(data).expect("should succeed");
        let (encoded, bit_count) = huffman_encode(data, &table).expect("should succeed");
        let decoded =
            huffman_decode(&encoded, bit_count, data.len(), &table).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_decode_roundtrip_single_symbol() {
        let data = vec![42u8; 100];
        let table = HuffmanCodeTable::from_data(&data).expect("should succeed");
        let (encoded, bit_count) = huffman_encode(&data, &table).expect("should succeed");
        let decoded =
            huffman_decode(&encoded, bit_count, data.len(), &table).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_decode_roundtrip_all_bytes() {
        let mut data: Vec<u8> = (0..=255u8).collect();
        data.extend(std::iter::repeat_n(0u8, 50));
        data.extend(std::iter::repeat_n(1u8, 30));
        data.extend(std::iter::repeat_n(255u8, 20));

        let table = HuffmanCodeTable::from_data(&data).expect("should succeed");

        for &(_, len) in &table.codes {
            if len > 0 {
                assert!(len <= MAX_CODE_LEN);
            }
        }

        let (encoded, bit_count) = huffman_encode(&data, &table).expect("should succeed");
        let decoded =
            huffman_decode(&encoded, bit_count, data.len(), &table).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_decode_roundtrip_two_symbols() {
        let data = vec![0u8, 0, 0, 1, 1, 0, 1, 0, 0, 1];
        let table = HuffmanCodeTable::from_data(&data).expect("should succeed");
        let (encoded, bit_count) = huffman_encode(&data, &table).expect("should succeed");
        let decoded =
            huffman_decode(&encoded, bit_count, data.len(), &table).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_decode_large_data() {
        let mut data = Vec::new();
        for sym in 0u8..50 {
            let count = 1000 / (sym as usize + 1);
            for _ in 0..count {
                data.push(sym);
            }
        }
        let table = HuffmanCodeTable::from_data(&data).expect("should succeed");
        let (encoded, bit_count) = huffman_encode(&data, &table).expect("should succeed");
        let decoded =
            huffman_decode(&encoded, bit_count, data.len(), &table).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_empty_data_error() {
        let table = HuffmanCodeTable {
            codes: vec![(0, 0); 256],
        };
        assert_eq!(huffman_encode(&[], &table), Err(HuffmanError::EmptyInput));
    }

    #[test]
    fn test_encode_symbol_not_in_table_error() {
        let mut lengths = [0u8; 256];
        lengths[0] = 1;
        let table = HuffmanCodeTable::from_lengths(&lengths);
        let result = huffman_encode(&[0, 1], &table);
        assert_eq!(result, Err(HuffmanError::SymbolNotFound(1)));
    }

    #[test]
    fn test_decode_unexpected_end() {
        let data = b"ab";
        let table = HuffmanCodeTable::from_data(data).expect("should succeed");
        let (encoded, bit_count) = huffman_encode(data, &table).expect("should succeed");
        let result = huffman_decode(&encoded, bit_count, 100, &table);
        assert!(result.is_err());
    }

    #[test]
    fn test_huffman_compression_ratio() {
        let data: Vec<u8> = std::iter::repeat_n(0u8, 1000)
            .chain(std::iter::repeat_n(1u8, 10))
            .chain(std::iter::once(2u8))
            .collect();

        let table = HuffmanCodeTable::from_data(&data).expect("should succeed");
        let (_, bit_count) = huffman_encode(&data, &table).expect("should succeed");
        let original_bits = data.len() * 8;
        assert!(
            bit_count < original_bits,
            "Expected compression: {bit_count} bits < {original_bits} bits"
        );
    }

    #[test]
    fn test_canonical_codes_no_prefix_conflict() {
        let data: Vec<u8> = (0..10)
            .flat_map(|i| vec![i; (i as usize + 1) * 10])
            .collect();
        let table = HuffmanCodeTable::from_data(&data).expect("should succeed");
        let active: Vec<(u32, u8)> = table
            .codes
            .iter()
            .filter(|(_, len)| *len > 0)
            .copied()
            .collect();

        for (i, &(code_a, len_a)) in active.iter().enumerate() {
            for &(code_b, len_b) in &active[i + 1..] {
                if len_a <= len_b {
                    let shifted = code_b >> (len_b - len_a);
                    assert_ne!(
                        shifted, code_a,
                        "Prefix conflict: ({code_a:#b}, {len_a}) is prefix of ({code_b:#b}, {len_b})"
                    );
                } else {
                    let shifted = code_a >> (len_a - len_b);
                    assert_ne!(
                        shifted, code_b,
                        "Prefix conflict: ({code_b:#b}, {len_b}) is prefix of ({code_a:#b}, {len_a})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_from_data_none_on_empty() {
        assert!(HuffmanCodeTable::from_data(&[]).is_none());
    }

    #[test]
    fn test_table_lookup() {
        let data = b"aaabbc";
        let table = HuffmanCodeTable::from_data(data).expect("should succeed");
        assert!(table.lookup(b'a').is_some());
        assert!(table.lookup(b'z').is_none());
    }
}
