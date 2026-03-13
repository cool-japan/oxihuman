// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Fuzz target: GLB/binary container parsing with arbitrary byte streams.
//
// Exercises:
//   - The GLB 2.0 binary-container framing logic (magic, version, length,
//     chunk-header iteration) implemented inline following the glTF spec.
//   - `detect_from_bytes` — the format-sniffing path.
//   - `read_pc2` and `read_mdd` — the point-cache binary readers.
//
// Any input that triggers a panic (rather than returning a value or an `Err`)
// would indicate a bug.

#![no_main]
use libfuzzer_sys::fuzz_target;
use oxihuman_export::{detect_from_bytes, read_mdd, read_pc2};

/// GLB 2.0 header size in bytes (magic u32 + version u32 + totalLength u32).
const GLB_HEADER_SIZE: usize = 12;
/// GLB chunk header size (chunkLength u32 + chunkType u32).
const CHUNK_HEADER_SIZE: usize = 8;
/// Expected magic value for a valid GLB file ("glTF" in LE u32).
const GLB_MAGIC: u32 = 0x46546C67;

/// Walk the GLB chunk list in `bytes` without panicking on any input.
///
/// Returns the number of syntactically well-formed chunks found, or 0 if the
/// header is absent/invalid.  All arithmetic uses checked / saturating ops to
/// avoid overflow on malformed length fields.
fn walk_glb_chunks(bytes: &[u8]) -> usize {
    if bytes.len() < GLB_HEADER_SIZE {
        return 0;
    }

    // Parse the 12-byte GLB header.
    let magic = u32::from_le_bytes(match bytes[0..4].try_into() {
        Ok(b) => b,
        Err(_) => return 0,
    });
    if magic != GLB_MAGIC {
        return 0;
    }

    // version field — we accept any version to exercise future-format paths.
    let _version = u32::from_le_bytes(match bytes[4..8].try_into() {
        Ok(b) => b,
        Err(_) => return 0,
    });

    let total_length = u32::from_le_bytes(match bytes[8..12].try_into() {
        Ok(b) => b,
        Err(_) => return 0,
    }) as usize;

    // Clamp the effective range to the actual buffer length.
    let effective_end = total_length.min(bytes.len());

    let mut cursor = GLB_HEADER_SIZE;
    let mut chunk_count = 0usize;

    while cursor + CHUNK_HEADER_SIZE <= effective_end {
        let chunk_length = u32::from_le_bytes(match bytes[cursor..cursor + 4].try_into() {
            Ok(b) => b,
            Err(_) => break,
        }) as usize;

        let _chunk_type = u32::from_le_bytes(match bytes[cursor + 4..cursor + 8].try_into() {
            Ok(b) => b,
            Err(_) => break,
        });

        cursor += CHUNK_HEADER_SIZE;

        // Align payload length to 4 bytes as required by the spec.
        let aligned = (chunk_length.saturating_add(3)) & !3;

        // Consume the payload bytes (or as many as are available).
        let available = effective_end.saturating_sub(cursor);
        let consume = aligned.min(available);
        cursor = cursor.saturating_add(consume);

        chunk_count += 1;

        // Hard cap to avoid infinite loops on crafted inputs.
        if chunk_count > 64 {
            break;
        }
    }

    chunk_count
}

fuzz_target!(|data: &[u8]| {
    // ── GLB binary framing walk ────────────────────────────────────────────────
    let _ = walk_glb_chunks(data);

    // ── Format detection from magic bytes ─────────────────────────────────────
    let _ = detect_from_bytes(data);

    // ── PC2 point-cache binary reader ─────────────────────────────────────────
    // `read_pc2` must return Err on malformed input, never panic.
    let _ = read_pc2(data);

    // ── MDD point-cache binary reader ─────────────────────────────────────────
    // `read_mdd` must return Err on malformed input, never panic.
    let _ = read_mdd(data);
});
