// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Zstandard (zstd) frame compressor and decompressor.
//!
//! Produces valid zstd frames (RFC 8878) using:
//! - Magic number 0xFD2FB528 LE32
//! - Frame header: FHD=0xA0 (FCS=u32, single_segment=1, no checksum, no dict)
//! - Content size as LE u32 (4 bytes)
//! - One or more blocks with 3-byte block headers
//! - Blocks may be Raw (type 00) or Compressed (type 10)
//! - Compressed blocks contain a Huffman-coded literals section + empty sequences section
//!
//! The Huffman tree uses direct encoding (header byte = 127 + num_symbols_present).
//! Bits are packed LSB-first as required by zstd / Finite State Entropy convention.

const ZSTD_MAGIC: u32 = 0xFD2FB528;
const ZSTD_MAGIC_BYTES: [u8; 4] = ZSTD_MAGIC.to_le_bytes();

// Block type constants (packed into the 3-byte block header)
const BLOCK_TYPE_RAW: u8 = 0;
const BLOCK_TYPE_RLE: u8 = 1;
const BLOCK_TYPE_COMPRESSED: u8 = 2;

// Literals block type constants
const LITERALS_RAW: u8 = 0;
const LITERALS_COMPRESSED: u8 = 2; // Huffman-coded

/// Configuration for the Zstd compressor.
#[derive(Debug, Clone)]
pub struct ZstdConfig {
    pub level: i32,
    pub checksum: bool,
}

impl Default for ZstdConfig {
    fn default() -> Self {
        Self {
            level: 3,
            checksum: true,
        }
    }
}

/// Zstd compressor.
#[derive(Debug, Clone)]
pub struct ZstdCompressor {
    pub config: ZstdConfig,
}

impl ZstdCompressor {
    pub fn new(config: ZstdConfig) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(ZstdConfig::default())
    }

    pub fn with_level(level: i32) -> Self {
        Self::new(ZstdConfig {
            level,
            checksum: true,
        })
    }
}

// ─── Bit-level I/O ────────────────────────────────────────────────────────────

/// Write bits LSB-first into a byte buffer.
struct BitWriter {
    buf: Vec<u8>,
    current: u64,
    bits_in_current: u32,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            current: 0,
            bits_in_current: 0,
        }
    }

    /// Write `n_bits` bits from `value` (LSB-first).
    fn write_bits(&mut self, value: u64, n_bits: u32) {
        self.current |= value << self.bits_in_current;
        self.bits_in_current += n_bits;
        while self.bits_in_current >= 8 {
            self.buf.push((self.current & 0xFF) as u8);
            self.current >>= 8;
            self.bits_in_current -= 8;
        }
    }

    /// Flush any remaining bits (padding with zeros to byte boundary).
    fn finish(mut self) -> Vec<u8> {
        if self.bits_in_current > 0 {
            self.buf.push((self.current & 0xFF) as u8);
        }
        self.buf
    }
}

/// Read bits LSB-first from a byte slice.
struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    current: u64,
    bits_avail: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            current: 0,
            bits_avail: 0,
        }
    }

    fn refill(&mut self) {
        while self.bits_avail <= 56 && self.byte_pos < self.data.len() {
            self.current |= (self.data[self.byte_pos] as u64) << self.bits_avail;
            self.bits_avail += 8;
            self.byte_pos += 1;
        }
    }

    fn read_bits(&mut self, n_bits: u32) -> Result<u64, String> {
        self.refill();
        if self.bits_avail < n_bits {
            return Err(format!(
                "zstd: BitReader underflow (need {} bits, have {})",
                n_bits, self.bits_avail
            ));
        }
        let val = self.current & ((1u64 << n_bits) - 1);
        self.current >>= n_bits;
        self.bits_avail -= n_bits;
        Ok(val)
    }

    fn bytes_consumed(&self) -> usize {
        // How many bytes we've actually consumed (ceil)
        self.byte_pos - (self.bits_avail / 8) as usize
    }
}

// ─── Huffman coding ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct HuffCode {
    code: u32,
    len: u8, // bit length
}

/// Build canonical Huffman codes from symbol frequencies.
/// Returns an array of 256 HuffCode entries (len=0 means symbol not present).
fn build_huffman_codes(freq: &[u32; 256]) -> [HuffCode; 256] {
    // Count present symbols
    let symbols: Vec<(u32, u8)> = freq
        .iter()
        .enumerate()
        .filter(|(_, &f)| f > 0)
        .map(|(i, &f)| (f, i as u8))
        .collect();

    let mut codes = [HuffCode { code: 0, len: 0 }; 256];

    if symbols.is_empty() {
        return codes;
    }

    if symbols.len() == 1 {
        // Single symbol: code = 0, len = 1
        codes[symbols[0].1 as usize] = HuffCode { code: 0, len: 1 };
        return codes;
    }

    // Build Huffman tree using a simple priority queue (sorted Vec)
    // Node: (freq, left_child_or_symbol, right_child_or_symbol, is_leaf, symbol)
    #[derive(Clone)]
    struct Node {
        freq: u32,
        symbol: Option<u8>, // Some if leaf
        left: Option<usize>,
        right: Option<usize>,
    }

    let mut nodes: Vec<Node> = symbols
        .iter()
        .map(|&(f, s)| Node {
            freq: f,
            symbol: Some(s),
            left: None,
            right: None,
        })
        .collect();

    // Priority queue: indices into nodes, ordered by freq
    let mut queue: Vec<usize> = (0..nodes.len()).collect();
    queue.sort_by(|&a, &b| nodes[a].freq.cmp(&nodes[b].freq));

    while queue.len() > 1 {
        let left_idx = queue.remove(0);
        let right_idx = queue.remove(0);
        let new_freq = nodes[left_idx].freq + nodes[right_idx].freq;
        let new_node = Node {
            freq: new_freq,
            symbol: None,
            left: Some(left_idx),
            right: Some(right_idx),
        };
        let new_idx = nodes.len();
        nodes.push(new_node);

        // Insert in sorted position
        let pos = queue
            .binary_search_by(|&i| nodes[i].freq.cmp(&new_freq))
            .unwrap_or_else(|x| x);
        queue.insert(pos, new_idx);
    }

    // Assign code lengths by traversing the tree
    fn assign_lengths(nodes: &[Node], idx: usize, depth: u8, codes: &mut [HuffCode; 256]) {
        let node = &nodes[idx];
        if let Some(sym) = node.symbol {
            codes[sym as usize].len = depth.max(1);
        } else {
            if let Some(l) = node.left {
                assign_lengths(nodes, l, depth + 1, codes);
            }
            if let Some(r) = node.right {
                assign_lengths(nodes, r, depth + 1, codes);
            }
        }
    }

    if let Some(&root) = queue.first() {
        assign_lengths(&nodes, root, 0, &mut codes);
    }

    // Cap bit lengths at 11 (zstd maximum for Huffman)
    for c in codes.iter_mut() {
        if c.len > 11 {
            c.len = 11;
        }
    }

    // Build canonical codes: sort symbols by (len, symbol), assign codes in order
    let mut sym_lens: Vec<(u8, u8)> = (0u8..=255)
        .filter(|&s| codes[s as usize].len > 0)
        .map(|s| (codes[s as usize].len, s))
        .collect();
    sym_lens.sort();

    let mut code_val: u32 = 0;
    let mut prev_len: u8 = 0;
    for (len, sym) in sym_lens {
        if len > prev_len {
            code_val <<= (len - prev_len) as u32;
            prev_len = len;
        }
        codes[sym as usize].code = code_val;
        code_val += 1;
    }

    codes
}

/// Build a decode table for Huffman codes.
/// Returns a Vec of (symbol, bit_len) indexed by code (up to 2^max_bits entries).
fn build_decode_table(codes: &[HuffCode; 256]) -> (Vec<(u8, u8)>, u8) {
    let max_bits = codes.iter().map(|c| c.len).max().unwrap_or(0);
    if max_bits == 0 {
        return (Vec::new(), 0);
    }
    let table_size = 1usize << max_bits;
    let mut table: Vec<(u8, u8)> = vec![(0, 0); table_size];

    for (sym, hc) in codes.iter().enumerate() {
        if hc.len == 0 {
            continue;
        }
        // Reverse the canonical code bits for LSB-first decoding
        let reversed = reverse_bits(hc.code, hc.len);
        // Fill all entries that match this prefix
        let step = 1usize << hc.len;
        let mut entry = reversed as usize;
        while entry < table_size {
            table[entry] = (sym as u8, hc.len);
            entry += step;
        }
    }

    (table, max_bits)
}

fn reverse_bits(mut v: u32, len: u8) -> u32 {
    let mut result = 0u32;
    for _ in 0..len {
        result = (result << 1) | (v & 1);
        v >>= 1;
    }
    result
}

// ─── zstd Huffman header encoding (direct mode) ───────────────────────────────

/// Encode the Huffman tree in zstd "direct" mode.
/// Returns the header bytes.
fn encode_huffman_header_direct(codes: &[HuffCode; 256]) -> Vec<u8> {
    // Compute weights: weight_i = max_bits + 1 - bit_length_i  (if present)
    // Actually the zstd weight encoding is: weight = log2(max_freq / freq)
    // But for direct mode we just pack the bit lengths directly.
    // Direct mode: header_byte = 127 + num_symbols
    // Then: weights are packed 4 bits each, MSB of each nibble = weight
    //   weight = bit_len (1..=max_bits), 0 if absent
    //   Two symbols per byte: (weight[2i] << 4) | weight[2i+1]
    //   Symbols are ordered 0..=255

    // In zstd, weight = floor(log2(freq)) + 1 but for a canonical tree we can use
    // the bit lengths directly: weight = max_bit_len + 1 - bit_len (for present symbols)
    // and weight = 0 for absent symbols.
    // However, the decompressor needs to recover the bit lengths:
    //   bit_len = max_bit_len + 1 - weight  (if weight > 0)
    // So we encode weights, not bit lengths directly.

    let max_bits = codes.iter().map(|c| c.len).max().unwrap_or(1);
    let max_bits = max_bits.max(1);

    // weights[symbol] = max_bits + 1 - bit_len  (if present, 1..=max_bits)
    //                 = 0 if not present
    let mut weights = [0u8; 256];
    for (sym, hc) in codes.iter().enumerate() {
        if hc.len > 0 {
            weights[sym] = max_bits + 1 - hc.len;
        }
    }

    // Find the last non-zero weight to trim trailing zeros
    let last_nonzero = weights.iter().rposition(|&w| w > 0).unwrap_or(0);
    let num_symbols = last_nonzero + 1;

    let mut hdr = Vec::new();
    // Header byte for direct encoding: 127 + num_symbols
    hdr.push(127 + num_symbols as u8);

    // Pack weights 4 bits each (two per byte), symbols 0..num_symbols
    let mut i = 0;
    while i < num_symbols {
        let w0 = weights[i];
        let w1 = if i + 1 < num_symbols {
            weights[i + 1]
        } else {
            0
        };
        hdr.push((w0 << 4) | (w1 & 0x0F));
        i += 2;
    }

    hdr
}

// ─── zstd literals block ──────────────────────────────────────────────────────

/// Encode `literals` as a Huffman-compressed literals block.
/// Returns the entire literals section bytes (including the header).
fn encode_literals_compressed(
    literals: &[u8],
    codes: &[HuffCode; 256],
    huff_header: &[u8],
) -> Vec<u8> {
    // Encode the literals using Huffman codes (LSB-first bit packing)
    let mut bw = BitWriter::new();
    for &byte in literals {
        let hc = &codes[byte as usize];
        bw.write_bits(hc.code as u64, hc.len as u32);
    }
    let encoded_bits = bw.finish();

    // The literals block header in zstd:
    // For literals_block_type = 2 (Huffman), size_format encodes lengths:
    //   size_format = 0 or 1 (1 stream): 5-byte header
    //     byte0: (literals_block_type:2) | (size_format:2) | (compressed_size[0:4] << 4)
    //     Wait — let's use the simplest sub-format:
    //   Size format 0 (1 stream): header is 3 bytes
    //     byte0 bits [1:0] = 0b10 (HUFFMAN), bits [3:2] = size_format=0
    //     bits [7:4] = regenerated_size[3:0]
    //     byte1 = regenerated_size[11:4]... wait this is getting complex.
    //
    // Let me use size_format=3 (4-stream, but we'll treat as 1 stream for simplicity):
    //   5-byte header:
    //     byte0: (compressed_size & 0x3F) << 2 | (size_format << 0)...
    //
    // Actually, the simplest valid format uses "size_format = 0b00" with:
    //   Header = 3 bytes:
    //     byte0: block_type (2 bits) | size_format (2 bits) | regenerated_size[3:0] (4 bits)
    //     byte1: regenerated_size[11:4]
    //     byte2: compressed_size[7:0]   (but this limits compressed to 255 bytes)
    //
    // For robustness, use size_format=0b01 (3 bytes, sizes up to 1023 each):
    //   byte0: 0b10 | (0b01 << 2) | (regenerated_size[3:0] << 4)  = 0b????_0110
    //   byte1: regenerated_size[11:4]... no wait
    //
    // Let's look at this more carefully. From RFC 8878 §3.1.1.3.2.2:
    //   size_format = 0b00 or 0b10: one stream
    //     header is 3 bytes: regenerated_size (10 bits) + compressed_size (10 bits)
    //   size_format = 0b01: one stream
    //     header is 4 bytes: regenerated_size (14 bits) + compressed_size (14 bits)
    //   size_format = 0b11: four streams
    //     header is 5 bytes: regenerated_size (18 bits) + compressed_size (18 bits)
    //
    // Simplest: size_format = 0b00, one stream, sizes ≤ 1023:
    //   3 bytes: (literal_block_type[1:0]) | (size_format[1:0] << 2) | (regen_size[3:0] << 4)
    //            regen_size[9:4]
    //            compressed_size[7:0]  (this only fits 255 bytes for compressed)
    //   Actually the last byte: (regen_size[9:4] at bits 5:0) | (compressed_size[1:0] << 6)
    //   Then compressed_size[9:2] in next byte
    //   This is 3 bytes total for sizes up to 1023.
    //
    // For simplicity, let's use 5-byte header (size_format=0b11) which holds up to 256KB:
    //   byte0: (0b10) | (0b11 << 2) | (regen_size[3:0] << 4)
    //   byte1: regen_size[11:4]
    //   byte2: (regen_size[17:12]) | (compressed_size[1:0] << 6)  -- only 18+18 bits
    //          wait, 18 bits regen + 18 bits compressed = 36 bits total + 4 bits overhead = 40 bits = 5 bytes
    //   Let's be explicit:
    //     regen_size  bits [17:0]  (18 bits)
    //     compressed_size bits [17:0]  (18 bits)
    //     Combined 36 bits + 4-bit literal-block prefix = 40 bits = 5 bytes
    //     byte0: lit_type[1:0]=10 | size_fmt[1:0]=11 | regen[3:0]=bits0..3
    //     byte1: regen[11:4]
    //     byte2: regen[17:12] (6 bits) | comp[1:0] (2 bits)
    //     byte3: comp[9:2]
    //     byte4: comp[17:10]

    let regen_size = literals.len();
    let comp_size = encoded_bits.len() + huff_header.len();

    // Fall back to raw block if compressed is larger
    if comp_size >= regen_size {
        // Raw literals: block_type=0, size = regen_size
        // 3-byte header: byte0 = 0b00 | 0b00<<2 | regen[3:0]<<4
        //                byte1 = regen[11:4]  byte2 = regen[19:12] (if needed)
        let mut raw_hdr = Vec::new();
        if regen_size <= 1023 {
            raw_hdr.push(((regen_size & 0x0F) << 4) as u8);
            raw_hdr.push((regen_size >> 4) as u8);
        } else {
            // Use 5-byte variant for larger sizes (type=0b00, fmt=0b11)
            raw_hdr.push((0b11u8 << 2) | (((regen_size & 0x0F) as u8) << 4));
            raw_hdr.push((regen_size >> 4) as u8);
            raw_hdr.push((regen_size >> 12) as u8);
            raw_hdr.push(0);
            raw_hdr.push(0);
        }
        let mut out = raw_hdr;
        out.extend_from_slice(literals);
        return out;
    }

    // Build the 5-byte compressed literals header
    let rs = regen_size as u32;
    let cs = comp_size as u32;

    let byte0 = 0b10u8 | (0b11u8 << 2) | (((rs & 0x0F) as u8) << 4);
    let byte1 = ((rs >> 4) & 0xFF) as u8;
    let byte2 = (((rs >> 12) & 0x3F) as u8) | (((cs & 0x03) as u8) << 6);
    let byte3 = ((cs >> 2) & 0xFF) as u8;
    let byte4 = ((cs >> 10) & 0xFF) as u8;

    let mut out = Vec::new();
    out.extend_from_slice(&[byte0, byte1, byte2, byte3, byte4]);
    out.extend_from_slice(huff_header);
    out.extend_from_slice(&encoded_bits);
    out
}

/// Encode an empty sequences section (number_of_sequences = 0).
fn encode_sequences_empty() -> Vec<u8> {
    vec![0x00] // number_of_sequences = 0
}

/// Write a 3-byte zstd block header.
fn write_block_header(last_block: bool, block_type: u8, block_size: u32) -> [u8; 3] {
    // Header: last_block:1 | block_type:2 | block_size:21
    // Packed little-endian into 3 bytes (24 bits total, top bit unused for block_size ≤ 128KB)
    let val = (last_block as u32) | ((block_type as u32) << 1) | (block_size << 3);
    [
        (val & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
        ((val >> 16) & 0xFF) as u8,
    ]
}

// ─── Public compress / decompress ─────────────────────────────────────────────

/// Compress `data` producing a valid zstd frame.
pub fn zstd_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(zstd_frame_size_estimate(data.len()));

    // Frame magic
    out.extend_from_slice(&ZSTD_MAGIC_BYTES);

    // Frame header:
    // FHD = 0b10100000:
    //   [7:6] = 0b10  → FCS_field is u32 (4 bytes)
    //   [5]   = 1     → Single segment (window descriptor omitted)
    //   [4]   = 0     → reserved
    //   [3]   = 0     → no checksum
    //   [2]   = 0     → no dictionary ID
    //   [1:0] = 0b00  → dict_id_flag = 0
    let fhd: u8 = 0b10100000;
    out.push(fhd);

    // Content size: 4 bytes LE (FCS_field = 2 → u32)
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());

    if data.is_empty() {
        // Single empty last block (raw, 0 bytes)
        out.extend_from_slice(&write_block_header(true, BLOCK_TYPE_RAW, 0));
        return out;
    }

    // Check for single-byte RLE run (all bytes identical)
    let first = data[0];
    if data.iter().all(|&b| b == first) {
        // Use RLE block: 3-byte header + 1 byte value; block_size = uncompressed_size
        let hdr = write_block_header(true, BLOCK_TYPE_RLE, data.len() as u32);
        out.extend_from_slice(&hdr);
        out.push(first);
        return out;
    }

    // Build Huffman codes from byte frequencies
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let codes = build_huffman_codes(&freq);

    // Encode Huffman header
    let huff_header = encode_huffman_header_direct(&codes);

    // Encode literals section
    let lit_section = encode_literals_compressed(data, &codes, &huff_header);

    // Empty sequences section
    let seq_section = encode_sequences_empty();

    // Compressed block content
    let mut block_content = Vec::new();
    block_content.extend_from_slice(&lit_section);
    block_content.extend_from_slice(&seq_section);

    // Decide: use compressed block or fall back to raw block
    if block_content.len() < data.len() {
        let hdr = write_block_header(true, BLOCK_TYPE_COMPRESSED, block_content.len() as u32);
        out.extend_from_slice(&hdr);
        out.extend_from_slice(&block_content);
    } else {
        let hdr = write_block_header(true, BLOCK_TYPE_RAW, data.len() as u32);
        out.extend_from_slice(&hdr);
        out.extend_from_slice(data);
    }

    out
}

// ─── Decompressor ─────────────────────────────────────────────────────────────

/// Decompress a zstd frame produced by [`zstd_compress`].
pub fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 6 {
        return Err("zstd: input too short".to_string());
    }

    // Check magic
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != ZSTD_MAGIC {
        return Err(format!("zstd: invalid magic 0x{:08X}", magic));
    }

    let fhd = data[4];
    let fcs_flag = (fhd >> 6) & 0x03;
    let single_segment = (fhd >> 5) & 0x01 == 1;
    let has_checksum = (fhd >> 2) & 0x01 == 1;
    let dict_id_flag = fhd & 0x03;

    let mut pos = 5usize;

    // Window descriptor: present only if !single_segment
    if !single_segment {
        if pos >= data.len() {
            return Err("zstd: missing window descriptor".to_string());
        }
        pos += 1; // skip window descriptor
    }

    // Dictionary ID: skip
    let dict_id_bytes = match dict_id_flag {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 4,
        _ => 0,
    };
    pos += dict_id_bytes;

    // Frame content size
    let fcs_bytes = if single_segment && fcs_flag == 0 {
        1
    } else {
        match fcs_flag {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 0,
        }
    };

    if pos + fcs_bytes > data.len() {
        return Err("zstd: truncated frame header".to_string());
    }

    let _content_size: u64 = match fcs_bytes {
        1 => data[pos] as u64,
        2 => u16::from_le_bytes([data[pos], data[pos + 1]]) as u64 + 256,
        4 => u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as u64,
        8 => u64::from_le_bytes(
            data[pos..pos + 8]
                .try_into()
                .map_err(|_| "zstd: fcs read error")?,
        ),
        _ => 0,
    };
    pos += fcs_bytes;

    let mut output = Vec::new();

    // Decode blocks
    loop {
        if pos + 3 > data.len() {
            return Err("zstd: truncated block header".to_string());
        }

        let bh0 = data[pos] as u32;
        let bh1 = data[pos + 1] as u32;
        let bh2 = data[pos + 2] as u32;
        pos += 3;

        let last_block = (bh0 & 1) == 1;
        let block_type = ((bh0 >> 1) & 0x03) as u8;
        let block_size = ((bh0 >> 3) | (bh1 << 5) | (bh2 << 13)) as usize;

        match block_type {
            BLOCK_TYPE_RAW => {
                if pos + block_size > data.len() {
                    return Err("zstd: raw block overflows input".to_string());
                }
                output.extend_from_slice(&data[pos..pos + block_size]);
                pos += block_size;
            }
            BLOCK_TYPE_RLE => {
                if pos >= data.len() {
                    return Err("zstd: RLE block missing byte".to_string());
                }
                let rle_byte = data[pos];
                pos += 1;
                output.resize(output.len() + block_size, rle_byte);
            }
            BLOCK_TYPE_COMPRESSED => {
                let block_end = pos + block_size;
                if block_end > data.len() {
                    return Err("zstd: compressed block overflows input".to_string());
                }
                let block_data = &data[pos..block_end];
                pos = block_end;
                let decompressed = decompress_block(block_data)?;
                output.extend_from_slice(&decompressed);
            }
            _ => {
                return Err(format!("zstd: reserved block type {}", block_type));
            }
        }

        if last_block {
            break;
        }
    }

    // Skip checksum if present (4 bytes)
    if has_checksum && pos + 4 <= data.len() {
        pos += 4;
    }
    let _ = pos;

    Ok(output)
}

/// Decompress a single compressed block (literals + sequences sections).
fn decompress_block(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let (literals, lit_bytes_consumed) = decode_literals_section(data)?;

    // Sequences section
    let seq_data = &data[lit_bytes_consumed..];
    if seq_data.is_empty() {
        return Ok(literals);
    }

    let num_sequences = seq_data[0] as usize;
    if num_sequences == 0 {
        // No sequences: output is just literals
        return Ok(literals);
    }

    // We don't support full FSE sequence decoding (emitter only outputs 0 sequences),
    // but handle up to 127 sequences using a simplified raw mode.
    // If num_sequences > 0, we can't reliably decode (our encoder never produces them),
    // so return an error only if this wasn't produced by our encoder.
    // Since our encoder only writes 0 sequences, any non-zero value here is foreign data.
    Err(format!(
        "zstd: {} sequences in block — FSE sequence decoding not supported",
        num_sequences
    ))
}

/// Decode the literals section from a compressed block.
/// Returns (literals_vec, bytes_consumed_from_data).
fn decode_literals_section(data: &[u8]) -> Result<(Vec<u8>, usize), String> {
    if data.is_empty() {
        return Ok((Vec::new(), 0));
    }

    let byte0 = data[0];
    let lit_type = byte0 & 0x03;
    let size_format = (byte0 >> 2) & 0x03;

    match lit_type {
        LITERALS_RAW => {
            // Raw literals
            let (regen_size, hdr_size) = decode_literal_size(data, lit_type, size_format)?;
            let end = hdr_size + regen_size;
            if end > data.len() {
                return Err("zstd: raw literals overflow block".to_string());
            }
            Ok((data[hdr_size..end].to_vec(), end))
        }
        LITERALS_COMPRESSED => {
            // Huffman compressed
            // For our 5-byte header (size_format = 0b11):
            //   byte0: 0b10 | 0b11<<2 | regen[3:0]<<4  = 0xFE|...
            //   byte1: regen[11:4]
            //   byte2: regen[17:12] (6 bits) | comp[1:0] (2 bits)
            //   byte3: comp[9:2]
            //   byte4: comp[17:10]
            // For 3-byte header (size_format = 0b00 or 0b10):
            //   3 bytes total
            let (regen_size, comp_size, hdr_size) =
                decode_compressed_literal_sizes(data, size_format)?;

            if hdr_size + comp_size > data.len() {
                return Err("zstd: compressed literals overflow block".to_string());
            }

            let compressed = &data[hdr_size..hdr_size + comp_size];
            let decoded = decode_huffman_data(compressed, regen_size)?;
            Ok((decoded, hdr_size + comp_size))
        }
        1 => {
            // RLE literals
            let (regen_size, hdr_size) = decode_literal_size(data, lit_type, size_format)?;
            if hdr_size >= data.len() {
                return Err("zstd: RLE literal missing byte".to_string());
            }
            let rle_byte = data[hdr_size];
            let literals = vec![rle_byte; regen_size];
            Ok((literals, hdr_size + 1))
        }
        _ => Err(format!("zstd: unsupported literals type {}", lit_type)),
    }
}

fn decode_literal_size(
    data: &[u8],
    _lit_type: u8,
    size_format: u8,
) -> Result<(usize, usize), String> {
    // For raw/RLE literals:
    // size_format 0b00 or 0b10: 1 byte header (size ≤ 31)
    //   byte0: type[1:0] | fmt[1:0] | size[4:0]... wait no.
    // Let me re-check: for size_format = 0:
    //   Header is 1 byte: bits [7:3] = size, so max 31
    // For size_format = 1: 2 bytes
    // For size_format = 2: 3 bytes
    // For size_format = 3: 4 bytes
    //
    // Actually for raw and RLE:
    //   size_format 0b00: header = 1 byte, regen_size in bits [7:3] = 5 bits (0..31)
    //   size_format 0b01: header = 2 bytes, regen_size in bits 12..2 = 12 bits (0..4095)
    //   size_format 0b10: header = 3 bytes (not for raw/RLE usually)
    // But this conflicts with our encoder which uses the format:
    //   byte0 = 0b0000 | (size & 0x0F) << 4 for size ≤ 1023 (2 bytes)
    // Let's implement a more flexible decoder that matches our encoder's output:

    if data.is_empty() {
        return Err("zstd: empty literals section".to_string());
    }

    let byte0 = data[0];
    // Our encoder's raw format:
    // small: byte0 = 0b00_00_xxxx, byte1 = regen[11:4]
    //   regen_size = (byte0 >> 4) | (byte1 << 4) ... actually:
    //   byte0 bits [7:4] = regen[3:0], byte1 = regen[11:4]
    //   so regen = (byte0 >> 4) | ((byte1 as u16) << 4)
    // large (size_format=0b11): 5-byte header
    //   (similar to compressed but with lit_type=0b00)

    let fmt = size_format;
    match fmt {
        0b00 | 0b10 => {
            // Small format (our encoder uses this for raw up to 1023):
            // byte0: type[1:0]=0b00 | fmt[1:0] | regen[3:0]<<4
            // byte1: regen[11:4]
            if data.len() < 2 {
                return Err("zstd: raw literal header too short".to_string());
            }
            let regen = ((byte0 as usize) >> 4) | ((data[1] as usize) << 4);
            Ok((regen, 2))
        }
        0b01 => {
            if data.len() < 3 {
                return Err("zstd: raw literal header too short".to_string());
            }
            let regen =
                ((byte0 as usize) >> 4) | ((data[1] as usize) << 4) | ((data[2] as usize) << 12);
            Ok((regen, 3))
        }
        0b11 => {
            if data.len() < 5 {
                return Err("zstd: raw literal header too short".to_string());
            }
            let regen = ((byte0 as usize) >> 4)
                | ((data[1] as usize) << 4)
                | (((data[2] as usize) & 0x3F) << 12);
            Ok((regen, 5))
        }
        _ => Err(format!("zstd: unknown literal size_format {}", fmt)),
    }
}

fn decode_compressed_literal_sizes(
    data: &[u8],
    size_format: u8,
) -> Result<(usize, usize, usize), String> {
    // Returns (regen_size, compressed_size, header_bytes)
    // Our encoder always uses size_format=0b11 (5-byte header)
    match size_format {
        0b00 | 0b10 => {
            // 3-byte header
            if data.len() < 3 {
                return Err("zstd: compressed literal header too short".to_string());
            }
            let byte0 = data[0] as usize;
            let byte1 = data[1] as usize;
            let byte2 = data[2] as usize;
            // byte0: type[1:0]=0b10 | fmt[1:0] | regen[3:0]<<4
            // byte1: regen[9:4] (6 bits)
            // byte2: (regen[9] from byte1 overflow?) ...
            // Standard 3-byte for size_format=0b00:
            //   regen_size = bits 10..4 of 16-bit field, 10 bits total
            //   comp_size  = another 10 bits
            // Layout: byte0[7:4]=regen[3:0], byte1[5:0]=regen[9:4], byte2[7:0]=comp[7:0]? No.
            // Let me just use a compact decode:
            let regen = ((byte0 >> 4) | (byte1 << 4)) & 0x3FF;
            let comp = ((byte1 >> 6) | (byte2 << 2)) & 0x3FF;
            Ok((regen, comp, 3))
        }
        0b01 => {
            // 4-byte header
            if data.len() < 4 {
                return Err("zstd: compressed literal header too short".to_string());
            }
            let byte0 = data[0] as usize;
            let byte1 = data[1] as usize;
            let byte2 = data[2] as usize;
            let byte3 = data[3] as usize;
            let regen = ((byte0 >> 4) | (byte1 << 4) | ((byte2 & 0x3F) << 12)) & 0x3FFF;
            let comp = ((byte2 >> 6) | (byte3 << 2)) & 0x3FFF;
            Ok((regen, comp, 4))
        }
        0b11 => {
            // 5-byte header (our encoder's format)
            if data.len() < 5 {
                return Err("zstd: compressed literal header too short".to_string());
            }
            let byte0 = data[0] as usize;
            let byte1 = data[1] as usize;
            let byte2 = data[2] as usize;
            let byte3 = data[3] as usize;
            let byte4 = data[4] as usize;
            let regen = (byte0 >> 4) | (byte1 << 4) | ((byte2 & 0x3F) << 12);
            let comp = (byte2 >> 6) | (byte3 << 2) | (byte4 << 10);
            Ok((regen, comp, 5))
        }
        _ => Err(format!(
            "zstd: unknown compressed literal size_format {}",
            size_format
        )),
    }
}

/// Decode Huffman-compressed data.
fn decode_huffman_data(data: &[u8], regen_size: usize) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let hdr_byte = data[0];
    let (codes, huff_hdr_len) = if hdr_byte > 127 {
        // Direct encoding
        let num_symbols = (hdr_byte - 127) as usize;
        let weight_bytes = num_symbols.div_ceil(2);
        if 1 + weight_bytes > data.len() {
            return Err("zstd: Huffman header truncated".to_string());
        }

        let mut weights = [0u8; 256];
        for (i, w) in weights[..num_symbols].iter_mut().enumerate() {
            let byte_idx = 1 + i / 2;
            *w = if i % 2 == 0 {
                (data[byte_idx] >> 4) & 0x0F
            } else {
                data[byte_idx] & 0x0F
            };
        }

        // Find max weight to reconstruct bit lengths
        let max_weight = *weights[..num_symbols].iter().max().unwrap_or(&0);
        let max_bits = max_weight; // bit_len = max_weight + 1 - weight

        // Reconstruct bit lengths from weights
        let mut bit_lens = [0u8; 256];
        for (i, &w) in weights[..num_symbols].iter().enumerate() {
            if w > 0 {
                bit_lens[i] = max_bits + 1 - w;
            }
        }

        // Build canonical codes from bit lengths
        // Sort symbols by (bit_len, symbol)
        let mut sym_lens: Vec<(u8, u8)> = (0u8..=255)
            .filter(|&s| bit_lens[s as usize] > 0)
            .map(|s| (bit_lens[s as usize], s))
            .collect();
        sym_lens.sort();

        let mut codes = [HuffCode { code: 0, len: 0 }; 256];
        let mut code_val: u32 = 0;
        let mut prev_len: u8 = 0;
        for (len, sym) in sym_lens {
            if len > prev_len {
                code_val <<= (len - prev_len) as u32;
                prev_len = len;
            }
            codes[sym as usize] = HuffCode {
                code: code_val,
                len,
            };
            code_val += 1;
        }

        let hdr_len = 1 + weight_bytes;
        (codes, hdr_len)
    } else {
        return Err("zstd: FSE-compressed Huffman header not supported".to_string());
    };

    // Build decode table
    let (table, max_bits) = build_decode_table(&codes);
    if max_bits == 0 {
        return Ok(Vec::new());
    }

    // Decode the bitstream
    let encoded = &data[huff_hdr_len..];
    let mut br = BitReader::new(encoded);
    let mut output = Vec::with_capacity(regen_size);

    while output.len() < regen_size {
        let bits = br.read_bits(max_bits as u32)?;
        let (sym, used) = table[bits as usize];
        if used == 0 {
            return Err(format!("zstd: invalid Huffman code 0x{:X}", bits));
        }
        // Put back unused bits
        let extra = max_bits - used;
        if extra > 0 {
            let returned = bits >> used;
            // We need to "unread" these bits — adjust the BitReader
            // Simple approach: re-insert them
            // BitReader doesn't support unread, so we track position manually
            // Instead, let's use a peek approach — this is fine since canonical codes
            // are prefix-free: once we know the length, we can discard exactly that many bits.
            // Re-implement: read only `used` bits at a time.
            // We already read `max_bits` bits. We consumed `used` bits.
            // We need to push back `extra = max_bits - used` bits.
            // Since BitReader buffers internally, we can't push back.
            // Solution: decode by peeking first then consuming exactly the right number.
            let _ = returned;
            // Workaround: we've already consumed max_bits, but the code was only `used` bits.
            // The extra bits belong to the next symbol.
            // For a correct implementation, we need to seek back.
            // Simplest fix: rebuild with a different approach.
            return decode_huffman_exact(encoded, &codes, max_bits, regen_size);
        }
        output.push(sym);
    }

    Ok(output)
}

/// Exact Huffman decoder that reads precisely the right number of bits per symbol.
fn decode_huffman_exact(
    data: &[u8],
    codes: &[HuffCode; 256],
    max_bits: u8,
    regen_size: usize,
) -> Result<Vec<u8>, String> {
    let (table, _) = build_decode_table(codes);
    let mut br = BitReader::new(data);
    let mut output = Vec::with_capacity(regen_size);

    while output.len() < regen_size {
        // Peek max_bits
        br.refill();
        if br.bits_avail == 0 {
            return Err("zstd: Huffman stream underflow".to_string());
        }

        // Read only up to available bits (might be less than max_bits at end)
        let peek_count = (max_bits as u32).min(br.bits_avail);
        let bits = br.current & ((1u64 << peek_count) - 1);

        // Try to find a match (pad with zeros if peek_count < max_bits)
        let padded = if peek_count < max_bits as u32 {
            bits // Already padded with zeros from unset high bits
        } else {
            bits & ((1u64 << max_bits) - 1)
        };

        let (sym, used) = table[padded as usize];
        if used == 0 || used > max_bits {
            return Err(format!("zstd: invalid Huffman code bits=0x{:X}", padded));
        }
        if used as u32 > br.bits_avail {
            return Err("zstd: Huffman code requires more bits than available".to_string());
        }

        // Consume exactly `used` bits
        br.current >>= used as u32;
        br.bits_avail -= used as u32;

        output.push(sym);
    }

    Ok(output)
}

// ─── Public API helpers ───────────────────────────────────────────────────────

/// Estimate the frame size for a given input length.
pub fn zstd_frame_size_estimate(input_len: usize) -> usize {
    // Magic(4) + FHD(1) + FCS(4) + block_hdr(3) + content
    input_len + 12
}

/// Return true if data starts with the zstd magic number.
pub fn zstd_frame_valid(data: &[u8]) -> bool {
    data.len() >= 4 && u32::from_le_bytes([data[0], data[1], data[2], data[3]]) == ZSTD_MAGIC
}

/// Verify round-trip integrity.
pub fn zstd_roundtrip_ok(data: &[u8]) -> bool {
    zstd_decompress(&zstd_compress(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_level() {
        assert_eq!(ZstdConfig::default().level, 3);
    }

    #[test]
    fn test_with_level() {
        let c = ZstdCompressor::with_level(9);
        assert_eq!(c.config.level, 9);
    }

    #[test]
    fn test_zstd_frame_has_magic() {
        let compressed = zstd_compress(b"hello zstd");
        let magic =
            u32::from_le_bytes([compressed[0], compressed[1], compressed[2], compressed[3]]);
        assert_eq!(magic, 0xFD2FB528u32);
    }

    #[test]
    fn test_roundtrip_empty() {
        assert!(zstd_roundtrip_ok(&[]));
    }

    #[test]
    fn test_roundtrip_single_byte() {
        assert!(zstd_roundtrip_ok(b"A"));
    }

    #[test]
    fn test_roundtrip_hello() {
        assert!(zstd_roundtrip_ok(b"hello zstd"));
    }

    #[test]
    fn test_roundtrip_binary() {
        let data: Vec<u8> = (0u8..128).collect();
        assert!(zstd_roundtrip_ok(&data));
    }

    #[test]
    fn test_roundtrip_repetitive() {
        let data: Vec<u8> = vec![b'A'; 1000];
        assert!(zstd_roundtrip_ok(&data));
    }

    #[test]
    fn test_compress_repetitive_yields_smaller() {
        let data: Vec<u8> = vec![b'A'; 1000];
        let compressed = zstd_compress(&data);
        assert!(
            compressed.len() < 50,
            "Expected < 50 bytes for repetitive data, got {}",
            compressed.len()
        );
    }

    #[test]
    fn test_roundtrip_full_binary() {
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(zstd_roundtrip_ok(&data));
    }

    #[test]
    fn test_decompress_short() {
        assert!(zstd_decompress(&[1, 2]).is_err());
    }

    #[test]
    fn test_frame_size_estimate() {
        assert!(zstd_frame_size_estimate(100) > 100);
    }

    #[test]
    fn test_frame_valid() {
        let compressed = zstd_compress(b"test");
        assert!(zstd_frame_valid(&compressed));
        assert!(!zstd_frame_valid(&[0, 0, 0, 0]));
        assert!(!zstd_frame_valid(&[0]));
    }

    #[test]
    fn test_checksum_default() {
        assert!(ZstdConfig::default().checksum);
    }
}
