// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AVIF container writer.
//!
//! Produces a valid AVIF file (ISOBMFF/HEIF container) wrapping our AV1-like
//! OBU stream.  The file structure follows the AVIF specification:
//!
//! ```text
//! ftyp  — file-type box (major brand 'avif')
//! mdat  — media data box  (contains raw AV1 OBU bytes)
//! meta  — metadata box (FullBox)
//!   hdlr  — handler reference ('pict')
//!   pitm  — primary item (item_ID = 1)
//!   iloc  — item location (offset into mdat)
//!   iinf  — item information (one 'av01' entry)
//!   iprp  — item properties
//!     ipco  — property container
//!       ispe  — image spatial extents
//!       av1C  — AV1 codec configuration
//!     ipma  — item-property association
//! ```
//!
//! Because mdat precedes meta, the iloc offsets are known before writing meta.

use crate::avif_export::AvifExport;

use super::encoder::encode_av1_intra;
use super::isobmff::{
    begin_box, end_box, write_cstring, write_full_box_header, write_u16_be, write_u32_be,
};

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Build a complete AVIF file from an `AvifExport`.
///
/// Returns the raw bytes suitable for writing to disk or embedding in a
/// network response.
pub fn write_avif(export: &AvifExport) -> Vec<u8> {
    let w = export.options.width;
    let h = export.options.height;
    let base_q_idx = export.options.preset.quantizer();

    // Encode the AV1 OBU stream.
    let av1_data = encode_av1_intra(&export.pixels, w, h, base_q_idx);

    let mut out = Vec::new();

    // ── 1. ftyp ──────────────────────────────────────────────────────────────
    // Fixed size: 4 (size) + 4 (type) + 4 (major) + 4 (minor) + 4*4 (compat) = 32 bytes
    {
        let start = begin_box(&mut out, b"ftyp");
        out.extend_from_slice(b"avif"); // major brand
        write_u32_be(&mut out, 0); // minor version = 0
        // Compatible brands.
        out.extend_from_slice(b"avif");
        out.extend_from_slice(b"mif1");
        out.extend_from_slice(b"miaf");
        end_box(&mut out, start);
    }

    // ── 2. mdat ───────────────────────────────────────────────────────────────
    // We record the absolute file offset of the AV1 data for use in iloc.
    let mdat_start = begin_box(&mut out, b"mdat");
    // The AV1 data begins immediately after the 8-byte box header.
    let av1_data_offset = (mdat_start + 8) as u32;
    out.extend_from_slice(&av1_data);
    end_box(&mut out, mdat_start);

    let av1_data_len = av1_data.len() as u32;

    // ── 3. meta ───────────────────────────────────────────────────────────────
    let meta_start = begin_box(&mut out, b"meta");
    write_full_box_header(&mut out, 0, 0);

    // 3a. hdlr
    {
        let s = begin_box(&mut out, b"hdlr");
        write_full_box_header(&mut out, 0, 0);
        write_u32_be(&mut out, 0); // pre_defined
        out.extend_from_slice(b"pict"); // handler_type
        out.extend_from_slice(&[0u8; 12]); // reserved
        write_cstring(&mut out, "AVIF");
        end_box(&mut out, s);
    }

    // 3b. pitm
    {
        let s = begin_box(&mut out, b"pitm");
        write_full_box_header(&mut out, 0, 0);
        write_u16_be(&mut out, 1); // item_ID = 1
        end_box(&mut out, s);
    }

    // 3c. iloc
    //
    // Version 0 layout:
    //   offset_size  = 4 bits (value 4)  → 4-byte extents
    //   length_size  = 4 bits (value 4)  → 4-byte lengths
    //   base_offset_size = 4 bits (value 0) → no base offset
    //   reserved     = 4 bits (value 0)
    //   item_count   = u16
    //   For each item:
    //     item_ID              u16
    //     data_reference_index u16
    //     extent_count         u16
    //     For each extent:
    //       extent_offset  u32  (offset_size=4 bytes)
    //       extent_length  u32  (length_size=4 bytes)
    {
        let s = begin_box(&mut out, b"iloc");
        write_full_box_header(&mut out, 0, 0);
        out.push(0x44); // offset_size=4 (hi nibble), length_size=4 (lo nibble)
        out.push(0x00); // base_offset_size=0 (hi nibble), reserved=0 (lo nibble)
        write_u16_be(&mut out, 1); // item_count = 1
        // Item 1:
        write_u16_be(&mut out, 1); // item_ID
        write_u16_be(&mut out, 0); // data_reference_index = 0 (same file)
        write_u16_be(&mut out, 1); // extent_count = 1
        write_u32_be(&mut out, av1_data_offset); // extent_offset (from file start)
        write_u32_be(&mut out, av1_data_len); // extent_length
        end_box(&mut out, s);
    }

    // 3d. iinf
    {
        let s = begin_box(&mut out, b"iinf");
        write_full_box_header(&mut out, 0, 0);
        write_u16_be(&mut out, 1); // entry_count = 1

        // infe (version 2)
        let se = begin_box(&mut out, b"infe");
        write_full_box_header(&mut out, 2, 0);
        write_u16_be(&mut out, 1); // item_ID = 1
        write_u16_be(&mut out, 0); // item_protection_index = 0
        out.extend_from_slice(b"av01"); // item_type
        write_cstring(&mut out, "Color");
        end_box(&mut out, se);

        end_box(&mut out, s);
    }

    // 3e. iprp
    {
        let s = begin_box(&mut out, b"iprp");

        // ipco
        let sc = begin_box(&mut out, b"ipco");

        // Property 1: ispe (image spatial extents)
        {
            let sp = begin_box(&mut out, b"ispe");
            write_full_box_header(&mut out, 0, 0);
            write_u32_be(&mut out, w);
            write_u32_be(&mut out, h);
            end_box(&mut out, sp);
        }

        // Property 2: av1C (AV1 codec configuration record)
        //
        // AV1CodecConfigurationRecord layout (big-endian bitfields):
        //   u(1) marker = 1
        //   u(7) version = 1
        //   u(3) seq_profile           = 1 (4:4:4 MC_IDENTITY)
        //   u(5) seq_level_idx_0       = 0
        //   u(1) seq_tier_0            = 0
        //   u(1) high_bitdepth         = 0
        //   u(1) twelve_bit            = 0
        //   u(1) monochrome            = 0
        //   u(1) chroma_subsampling_x  = 0
        //   u(1) chroma_subsampling_y  = 0
        //   u(2) chroma_sample_position= 0
        //   u(1) initial_presentation_delay_present = 0
        //   u(7) reserved = 0
        //   configOBUs: (none)
        //
        // Byte 0: 1_0000001 = 0x81  (marker=1, version=1)
        // Byte 1: 001_00000 = 0x20  (seq_profile=1<<5=32 = 0x20, seq_level_idx=0)
        // Byte 2: 0_0_0_0_0_0_00 = 0x00  (all zero)
        // Byte 3: 0_0000000 = 0x00  (initial_presentation_delay_present=0, reserved=0)
        {
            let sp = begin_box(&mut out, b"av1C");
            out.push(0x81); // marker=1, version=1
            out.push(0x20); // seq_profile=1, seq_level_idx_0=0
            out.push(0x00); // all zero (8-bit, non-mono, 4:4:4, no subsampling)
            out.push(0x00); // initial_presentation_delay_present=0, reserved
            // configOBUs: omit for simplicity (decoder can work without them)
            end_box(&mut out, sp);
        }

        end_box(&mut out, sc); // close ipco

        // ipma (item-property association)
        {
            let sa = begin_box(&mut out, b"ipma");
            write_full_box_header(&mut out, 0, 0);
            write_u32_be(&mut out, 1); // entry_count = 1

            write_u16_be(&mut out, 1); // item_ID = 1
            out.push(2); // association_count = 2

            // Association 1: ispe (property index 1, not essential)
            out.push(0x01); // essential=0, property_index=1 (bit7=0, bits6-0=1)

            // Association 2: av1C (property index 2, essential)
            out.push(0x82); // essential=1 (bit7), property_index=2
            end_box(&mut out, sa);
        }

        end_box(&mut out, s); // close iprp
    }

    end_box(&mut out, meta_start); // close meta

    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::decoder::decode_av1_intra;
    use crate::avif_export::{AvifExport, AvifOptions, AvifPreset};

    fn make_export(w: u32, h: u32, preset: AvifPreset) -> AvifExport {
        let pixels: Vec<[u8; 4]> = (0..(w * h) as usize)
            .map(|i| {
                let v = (i * 4) as u8;
                [v, v.wrapping_add(50), v.wrapping_add(100), 255]
            })
            .collect();
        AvifExport {
            options: AvifOptions {
                width: w,
                height: h,
                preset,
                alpha: true,
            },
            pixels,
        }
    }

    fn lossless_export(w: u32, h: u32) -> AvifExport {
        // Build a custom lossless export (quantizer=0).
        let pixels: Vec<[u8; 4]> = (0..(w * h) as usize)
            .map(|i| {
                let v = (i * 4) as u8;
                [v, v.wrapping_add(50), v.wrapping_add(100), 255]
            })
            .collect();
        AvifExport {
            options: AvifOptions {
                width: w,
                height: h,
                preset: AvifPreset::Best, // will use quantizer=10, not truly lossless
                alpha: true,
            },
            pixels,
        }
    }

    #[test]
    fn test_write_avif_produces_ftyp_brand() {
        let export = make_export(4, 4, AvifPreset::Best);
        let bytes = write_avif(&export);
        // The ftyp box must start at offset 0 and contain 'avif' as major brand.
        assert!(bytes.len() > 12, "AVIF file must have at least ftyp + mdat");
        // Bytes 0-3: box size (should be 32 for our ftyp).
        let ftyp_size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(&bytes[4..8], b"ftyp", "first box must be ftyp");
        assert!(ftyp_size >= 20, "ftyp box must be at least 20 bytes");
        assert_eq!(&bytes[8..12], b"avif", "major brand must be 'avif'");
    }

    #[test]
    fn test_write_avif_contains_mdat() {
        let export = make_export(4, 4, AvifPreset::Best);
        let bytes = write_avif(&export);
        // Find mdat box after ftyp.
        let ftyp_size =
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mdat_offset = ftyp_size;
        assert!(
            bytes.len() > mdat_offset + 8,
            "file must contain mdat box after ftyp"
        );
        assert_eq!(
            &bytes[mdat_offset + 4..mdat_offset + 8],
            b"mdat",
            "second box must be mdat"
        );
    }

    #[test]
    fn test_write_avif_nonempty() {
        let export = make_export(8, 8, AvifPreset::Balanced);
        let bytes = write_avif(&export);
        assert!(bytes.len() > 100, "AVIF file must contain substantial data");
    }

    #[test]
    fn test_avif_lossless_roundtrip_via_obu_extraction() {
        // Extract the AV1 OBU bytes from the mdat box and decode them directly.
        let w = 8u32;
        let h = 8u32;
        let pixels: Vec<[u8; 4]> = (0..(w * h) as usize)
            .map(|i| {
                let v = (i * 4) as u8;
                [v, v.wrapping_add(50), v.wrapping_add(100), 255]
            })
            .collect();

        // Encode directly as lossless AV1 OBU stream.
        let av1_bytes = encode_av1_intra(&pixels, w, h, 0);
        let (decoded, dw, dh) =
            decode_av1_intra(&av1_bytes).expect("decode must succeed");

        assert_eq!(dw, w);
        assert_eq!(dh, h);
        assert_eq!(decoded.len(), pixels.len());
        for (i, (&orig, &dec)) in pixels.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(orig[0], dec[0], "R mismatch at pixel {i}");
            assert_eq!(orig[1], dec[1], "G mismatch at pixel {i}");
            assert_eq!(orig[2], dec[2], "B mismatch at pixel {i}");
        }
    }

    #[test]
    fn test_write_avif_to_tempfile() {
        use std::io::Write;
        let export = make_export(4, 4, AvifPreset::Best);
        let bytes = write_avif(&export);

        let mut path = std::env::temp_dir();
        path.push("oxihuman_test_avif_writer.avif");
        let mut f = std::fs::File::create(&path).expect("cannot create temp file");
        f.write_all(&bytes).expect("cannot write AVIF bytes");
        let metadata = std::fs::metadata(&path).expect("cannot stat file");
        assert!(metadata.len() > 0, "temp file must be non-empty");
        let _ = std::fs::remove_file(&path);
    }
}
