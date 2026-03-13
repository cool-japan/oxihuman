// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Bitstream writer for compact binary export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitstreamExport {
    data: Vec<u8>,
    bit_pos: usize,
}

#[allow(dead_code)]
impl BitstreamExport {
    /// Create a new empty bitstream.
    pub fn new() -> Self {
        Self { data: Vec::new(), bit_pos: 0 }
    }

    /// Write N bits from a u32 value.
    pub fn write_bits(&mut self, value: u32, num_bits: u8) {
        for i in 0..num_bits {
            let byte_idx = self.bit_pos / 8;
            let bit_idx = self.bit_pos % 8;
            if byte_idx >= self.data.len() {
                self.data.push(0);
            }
            if (value >> i) & 1 == 1 {
                self.data[byte_idx] |= 1 << bit_idx;
            }
            self.bit_pos += 1;
        }
    }

    /// Write a full u8.
    pub fn write_u8(&mut self, value: u8) {
        self.write_bits(value as u32, 8);
    }

    /// Write a full u16.
    pub fn write_u16(&mut self, value: u16) {
        self.write_bits(value as u32, 16);
    }

    /// Write a full u32.
    pub fn write_u32(&mut self, value: u32) {
        self.write_bits(value, 32);
    }

    /// Write a float as 32-bit IEEE.
    pub fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    /// Total bytes written.
    pub fn byte_count(&self) -> usize {
        self.data.len()
    }

    /// Total bits written.
    pub fn bit_count(&self) -> usize {
        self.bit_pos
    }

    /// Get the underlying bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Consume and return bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Reset the stream.
    pub fn clear(&mut self) {
        self.data.clear();
        self.bit_pos = 0;
    }
}

impl Default for BitstreamExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Export mesh indices as variable-bit-width stream.
#[allow(dead_code)]
pub fn export_indices_bitstream(indices: &[u32], bits_per_index: u8) -> BitstreamExport {
    let mut bs = BitstreamExport::new();
    bs.write_u32(indices.len() as u32);
    for &idx in indices {
        bs.write_bits(idx, bits_per_index);
    }
    bs
}

/// Compute minimum bits needed for a max value.
#[allow(dead_code)]
pub fn bits_needed(max_value: u32) -> u8 {
    if max_value == 0 { return 1; }
    32 - max_value.leading_zeros() as u8
}

/// Serialize bitstream stats to JSON.
#[allow(dead_code)]
pub fn bitstream_to_json(bs: &BitstreamExport) -> String {
    format!("{{\"bytes\":{},\"bits\":{}}}", bs.byte_count(), bs.bit_count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_bits() {
        let mut bs = BitstreamExport::new();
        bs.write_bits(5, 3);
        assert_eq!(bs.bit_count(), 3);
    }

    #[test]
    fn test_write_u8() {
        let mut bs = BitstreamExport::new();
        bs.write_u8(0xFF);
        assert_eq!(bs.byte_count(), 1);
        assert_eq!(bs.as_bytes()[0], 0xFF);
    }

    #[test]
    fn test_write_u16() {
        let mut bs = BitstreamExport::new();
        bs.write_u16(1000);
        assert_eq!(bs.bit_count(), 16);
    }

    #[test]
    fn test_write_f32() {
        let mut bs = BitstreamExport::new();
        bs.write_f32(1.0);
        assert_eq!(bs.bit_count(), 32);
    }

    #[test]
    fn test_bits_needed() {
        assert_eq!(bits_needed(0), 1);
        assert_eq!(bits_needed(1), 1);
        assert_eq!(bits_needed(255), 8);
        assert_eq!(bits_needed(256), 9);
    }

    #[test]
    fn test_export_indices() {
        let bs = export_indices_bitstream(&[0, 1, 2], 8);
        assert!(bs.byte_count() > 0);
    }

    #[test]
    fn test_clear() {
        let mut bs = BitstreamExport::new();
        bs.write_u8(42);
        bs.clear();
        assert_eq!(bs.bit_count(), 0);
    }

    #[test]
    fn test_into_bytes() {
        let mut bs = BitstreamExport::new();
        bs.write_u8(123);
        let bytes = bs.into_bytes();
        assert_eq!(bytes[0], 123);
    }

    #[test]
    fn test_to_json() {
        let mut bs = BitstreamExport::new();
        bs.write_u32(42);
        let json = bitstream_to_json(&bs);
        assert!(json.contains("bytes"));
    }

    #[test]
    fn test_default() {
        let bs = BitstreamExport::default();
        assert_eq!(bs.bit_count(), 0);
    }
}
