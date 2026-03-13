// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ZIP pack scanning utilities for loading asset packs in-memory.

use anyhow::Result;

/// Read a `u16` from `data` at `offset` (little-endian).
#[inline]
fn zip_u16(data: &[u8], offset: usize) -> u16 {
    (data[offset] as u16) | ((data[offset + 1] as u16) << 8)
}

/// Read a `u32` from `data` at `offset` (little-endian).
#[inline]
fn zip_u32(data: &[u8], offset: usize) -> u32 {
    (data[offset] as u32)
        | ((data[offset + 1] as u32) << 8)
        | ((data[offset + 2] as u32) << 16)
        | ((data[offset + 3] as u32) << 24)
}

/// Scan a ZIP byte slice for all local file entries (signature `0x04034B50`).
///
/// Only STORE compression (method 0) is supported; compressed entries are
/// included in the result with their raw (== uncompressed) bytes.
/// Returns `Vec<(filename, data)>`.
pub fn scan_zip_local_entries(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    const LOCAL_SIG: u32 = 0x04034B50;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut pos = 0usize;

    while pos + 30 <= data.len() {
        let sig = zip_u32(data, pos);
        if sig != LOCAL_SIG {
            // Skip one byte and keep scanning.
            pos += 1;
            continue;
        }

        // Local file header:
        //  +0  signature (4)
        //  +4  version needed (2)
        //  +6  flags (2)
        //  +8  compression (2)
        //  +10 mod time (2)
        //  +12 mod date (2)
        //  +14 crc-32 (4)
        //  +18 compressed size (4)
        //  +22 uncompressed size (4)
        //  +26 filename length (2)
        //  +28 extra field length (2)
        //  +30 filename ...
        let compression = zip_u16(data, pos + 8);
        let compressed_size = zip_u32(data, pos + 18) as usize;
        let fname_len = zip_u16(data, pos + 26) as usize;
        let extra_len = zip_u16(data, pos + 28) as usize;

        let fname_start = pos + 30;
        let data_start = fname_start + fname_len + extra_len;
        let data_end = data_start + compressed_size;

        if data_end > data.len() {
            // Truncated -- stop scanning.
            break;
        }

        let fname_bytes = &data[fname_start..fname_start + fname_len];
        let filename = String::from_utf8_lossy(fname_bytes).into_owned();

        if compression == 0 {
            // STORE: raw data is the file content.
            entries.push((filename, data[data_start..data_end].to_vec()));
        }
        // Compressed entries (deflate etc.) are skipped -- we advance past them.

        pos = data_end;
    }

    Ok(entries)
}
