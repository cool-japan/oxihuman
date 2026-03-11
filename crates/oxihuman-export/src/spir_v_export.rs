// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SPIR-V binary stub export.

/// SPIR-V magic number.
pub const SPIRV_MAGIC: u32 = 0x07230203;
/// SPIR-V version 1.5.
pub const SPIRV_VERSION_1_5: u32 = 0x00010500;

/// A SPIR-V binary stub.
pub struct SpirVExport {
    pub magic: u32,
    pub version: u32,
    pub bound: u32,
    pub words: Vec<u32>,
    pub entry_points: Vec<String>,
}

/// Create a minimal SPIR-V export stub.
pub fn new_spirv_export() -> SpirVExport {
    SpirVExport {
        magic: SPIRV_MAGIC,
        version: SPIRV_VERSION_1_5,
        bound: 1,
        words: vec![SPIRV_MAGIC, SPIRV_VERSION_1_5, 0, 1, 0],
        entry_points: Vec::new(),
    }
}

/// Add an entry point name.
pub fn add_spirv_entry_point(exp: &mut SpirVExport, name: &str) {
    exp.entry_points.push(name.to_string());
}

/// Entry point count.
pub fn spirv_entry_point_count(exp: &SpirVExport) -> usize {
    exp.entry_points.len()
}

/// Word count of the binary.
pub fn spirv_word_count(exp: &SpirVExport) -> usize {
    exp.words.len()
}

/// Byte size of the binary.
pub fn spirv_byte_size(exp: &SpirVExport) -> usize {
    exp.words.len() * 4
}

/// Serialize words to little-endian bytes.
pub fn spirv_to_bytes(exp: &SpirVExport) -> Vec<u8> {
    exp.words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// Validate the magic number.
pub fn validate_spirv_magic(exp: &SpirVExport) -> bool {
    exp.magic == SPIRV_MAGIC
}

/// Check that the header has correct minimal structure.
pub fn spirv_has_valid_header(exp: &SpirVExport) -> bool {
    exp.words.len() >= 5 && exp.words[0] == SPIRV_MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_magic() {
        let exp = new_spirv_export();
        assert_eq!(exp.magic, SPIRV_MAGIC /* magic correct */);
    }

    #[test]
    fn validate_magic_passes() {
        let exp = new_spirv_export();
        assert!(validate_spirv_magic(&exp) /* valid magic */);
    }

    #[test]
    fn new_export_has_valid_header() {
        let exp = new_spirv_export();
        assert!(spirv_has_valid_header(&exp) /* valid header */);
    }

    #[test]
    fn byte_size_is_word_count_times_4() {
        let exp = new_spirv_export();
        assert_eq!(
            spirv_byte_size(&exp),
            spirv_word_count(&exp) * 4 /* 4 bytes per word */
        );
    }

    #[test]
    fn to_bytes_starts_with_magic() {
        let exp = new_spirv_export();
        let bytes = spirv_to_bytes(&exp);
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, SPIRV_MAGIC /* magic in bytes */);
    }

    #[test]
    fn add_entry_point_increments() {
        let mut exp = new_spirv_export();
        add_spirv_entry_point(&mut exp, "main");
        assert_eq!(spirv_entry_point_count(&exp), 1 /* one entry point */);
    }

    #[test]
    fn version_is_1_5() {
        let exp = new_spirv_export();
        assert_eq!(exp.version, SPIRV_VERSION_1_5 /* version */);
    }

    #[test]
    fn to_bytes_length_correct() {
        let exp = new_spirv_export();
        let bytes = spirv_to_bytes(&exp);
        assert_eq!(
            bytes.len(),
            spirv_byte_size(&exp) /* correct byte count */
        );
    }

    #[test]
    fn multiple_entry_points() {
        let mut exp = new_spirv_export();
        add_spirv_entry_point(&mut exp, "vs_main");
        add_spirv_entry_point(&mut exp, "fs_main");
        assert_eq!(spirv_entry_point_count(&exp), 2 /* two entry points */);
    }
}
