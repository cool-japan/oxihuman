// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Binary serialization helpers using little-endian byte encoding.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for the binary serializer/deserializer.
#[derive(Debug, Clone)]
pub struct BinSerConfig {
    /// Initial capacity (bytes) for new write buffers.
    pub initial_capacity: usize,
    /// Whether to write a 4-byte magic header.
    pub write_magic: bool,
    /// Magic bytes used when `write_magic` is true.
    pub magic: [u8; 4],
}

/// Return a sensible default `BinSerConfig`.
#[allow(dead_code)]
pub fn default_bin_config() -> BinSerConfig {
    BinSerConfig {
        initial_capacity: 64,
        write_magic: false,
        magic: [0x4F, 0x58, 0x49, 0x42], // "OXIB"
    }
}

/// A growing byte buffer for binary serialization.
#[derive(Debug, Clone)]
pub struct BinWriter {
    pub buf: Vec<u8>,
}

/// A cursor over a byte slice for binary deserialization.
#[derive(Debug, Clone)]
pub struct BinReader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

/// Error type returned by reader functions when there is not enough data.
#[derive(Debug, Clone)]
pub struct SerializeError {
    pub message: String,
}

impl SerializeError {
    fn new(msg: impl Into<String>) -> Self {
        SerializeError { message: msg.into() }
    }
}

// ---------------------------------------------------------------------------
// Writer functions
// ---------------------------------------------------------------------------

/// Create a new, empty `BinWriter`.
#[allow(dead_code)]
pub fn new_writer() -> BinWriter {
    BinWriter { buf: Vec::new() }
}

/// Create a new `BinWriter` using the given config (capacity hint; ignores magic for simplicity).
#[allow(dead_code)]
pub fn new_bin_writer(config: &BinSerConfig) -> BinWriter {
    BinWriter {
        buf: Vec::with_capacity(config.initial_capacity),
    }
}

/// Append a `u8` to the writer.
#[allow(dead_code)]
pub fn write_u8(w: &mut BinWriter, v: u8) {
    w.buf.push(v);
}

/// Append a `u32` as 4 little-endian bytes.
#[allow(dead_code)]
pub fn write_u32(w: &mut BinWriter, v: u32) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

/// Append a `u64` as 8 little-endian bytes.
#[allow(dead_code)]
pub fn write_u64(w: &mut BinWriter, v: u64) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

/// Append an `f32` as 4 little-endian bytes.
#[allow(dead_code)]
pub fn write_f32(w: &mut BinWriter, v: f32) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

/// Append an `f64` as 8 little-endian bytes.
#[allow(dead_code)]
pub fn write_f64(w: &mut BinWriter, v: f64) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

/// Append a raw byte slice verbatim.
#[allow(dead_code)]
pub fn write_bytes(w: &mut BinWriter, data: &[u8]) {
    w.buf.extend_from_slice(data);
}

/// Write a length-prefixed UTF-8 string: `u32` byte-length then raw UTF-8 bytes.
#[allow(dead_code)]
pub fn write_string(w: &mut BinWriter, s: &str) {
    let bytes = s.as_bytes();
    write_u32(w, bytes.len() as u32);
    w.buf.extend_from_slice(bytes);
}

/// Return a slice over all bytes written so far.
#[allow(dead_code)]
pub fn writer_to_bytes(w: &BinWriter) -> &[u8] {
    &w.buf
}

// ---------------------------------------------------------------------------
// Reader functions
// ---------------------------------------------------------------------------

/// Create a new `BinReader` positioned at the beginning of `data`.
#[allow(dead_code)]
pub fn new_reader(data: &[u8]) -> BinReader<'_> {
    BinReader { data, pos: 0 }
}

/// Create a new `BinReader` (config-aware alias; config is unused but accepted for API symmetry).
#[allow(dead_code)]
pub fn new_bin_reader<'a>(_config: &BinSerConfig, data: &'a [u8]) -> BinReader<'a> {
    BinReader { data, pos: 0 }
}

/// Current byte position of the reader.
#[allow(dead_code)]
pub fn reader_position(r: &BinReader<'_>) -> usize {
    r.pos
}

/// How many bytes remain unread.
#[allow(dead_code)]
pub fn reader_remaining(r: &BinReader<'_>) -> usize {
    r.data.len().saturating_sub(r.pos)
}

/// Read a single `u8`.
#[allow(dead_code)]
pub fn read_u8(r: &mut BinReader<'_>) -> Result<u8, SerializeError> {
    if r.pos >= r.data.len() {
        return Err(SerializeError::new("read_u8: not enough data"));
    }
    let v = r.data[r.pos];
    r.pos += 1;
    Ok(v)
}

/// Read a `u32` from 4 little-endian bytes.
#[allow(dead_code)]
pub fn read_u32(r: &mut BinReader<'_>) -> Result<u32, SerializeError> {
    let bytes = read_n::<4>(r, "read_u32")?;
    Ok(u32::from_le_bytes(bytes))
}

/// Read a `u64` from 8 little-endian bytes.
#[allow(dead_code)]
pub fn read_u64(r: &mut BinReader<'_>) -> Result<u64, SerializeError> {
    let bytes = read_n::<8>(r, "read_u64")?;
    Ok(u64::from_le_bytes(bytes))
}

/// Read an `f32` from 4 little-endian bytes.
#[allow(dead_code)]
pub fn read_f32(r: &mut BinReader<'_>) -> Result<f32, SerializeError> {
    let bytes = read_n::<4>(r, "read_f32")?;
    Ok(f32::from_le_bytes(bytes))
}

/// Read an `f64` from 8 little-endian bytes.
#[allow(dead_code)]
pub fn read_f64(r: &mut BinReader<'_>) -> Result<f64, SerializeError> {
    let bytes = read_n::<8>(r, "read_f64")?;
    Ok(f64::from_le_bytes(bytes))
}

/// Read exactly `n` raw bytes.
#[allow(dead_code)]
pub fn read_bytes(r: &mut BinReader<'_>, n: usize) -> Result<Vec<u8>, SerializeError> {
    let end = r.pos.checked_add(n).ok_or_else(|| SerializeError::new("read_bytes: overflow"))?;
    if end > r.data.len() {
        return Err(SerializeError::new(format!(
            "read_bytes: need {} bytes, only {} remain",
            n,
            reader_remaining(r)
        )));
    }
    let v = r.data[r.pos..end].to_vec();
    r.pos = end;
    Ok(v)
}

/// Read a length-prefixed UTF-8 string (written by `write_string`).
#[allow(dead_code)]
pub fn read_string(r: &mut BinReader<'_>) -> Result<String, SerializeError> {
    let len = read_u32(r)? as usize;
    let bytes = read_bytes(r, len)?;
    String::from_utf8(bytes).map_err(|e| SerializeError::new(format!("read_string: {}", e)))
}

// ---------------------------------------------------------------------------
// Private helper
// ---------------------------------------------------------------------------

fn read_n<const N: usize>(r: &mut BinReader<'_>, ctx: &str) -> Result<[u8; N], SerializeError> {
    let end = r.pos + N;
    if end > r.data.len() {
        return Err(SerializeError::new(format!(
            "{}: need {} bytes, only {} remain",
            ctx,
            N,
            reader_remaining(r)
        )));
    }
    let mut arr = [0u8; N];
    arr.copy_from_slice(&r.data[r.pos..end]);
    r.pos = end;
    Ok(arr)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. write/read u8 roundtrip
    #[test]
    fn roundtrip_u8() {
        let mut w = new_writer();
        write_u8(&mut w, 0xAB);
        let mut r = new_reader(writer_to_bytes(&w));
        assert_eq!(read_u8(&mut r).expect("should succeed"), 0xAB);
    }

    // 2. write/read u32 roundtrip
    #[test]
    fn roundtrip_u32() {
        let mut w = new_writer();
        write_u32(&mut w, 0xDEAD_BEEF);
        let mut r = new_reader(writer_to_bytes(&w));
        assert_eq!(read_u32(&mut r).expect("should succeed"), 0xDEAD_BEEF);
    }

    // 3. write/read u64 roundtrip
    #[test]
    fn roundtrip_u64() {
        let mut w = new_writer();
        write_u64(&mut w, u64::MAX);
        let mut r = new_reader(writer_to_bytes(&w));
        assert_eq!(read_u64(&mut r).expect("should succeed"), u64::MAX);
    }

    // 4. write/read f32 roundtrip
    #[test]
    fn roundtrip_f32() {
        let mut w = new_writer();
        write_f32(&mut w, std::f32::consts::PI);
        let mut r = new_reader(writer_to_bytes(&w));
        let v = read_f32(&mut r).expect("should succeed");
        assert!((v - std::f32::consts::PI).abs() < 1e-6);
    }

    // 5. write/read f64 roundtrip
    #[test]
    fn roundtrip_f64() {
        let mut w = new_writer();
        write_f64(&mut w, std::f64::consts::E);
        let mut r = new_reader(writer_to_bytes(&w));
        let v = read_f64(&mut r).expect("should succeed");
        assert!((v - std::f64::consts::E).abs() < 1e-12);
    }

    // 6. write/read bytes roundtrip
    #[test]
    fn roundtrip_bytes() {
        let data = vec![0u8, 1, 2, 3, 255];
        let mut w = new_writer();
        write_bytes(&mut w, &data);
        let mut r = new_reader(writer_to_bytes(&w));
        let out = read_bytes(&mut r, data.len()).expect("should succeed");
        assert_eq!(out, data);
    }

    // 7. write/read string roundtrip
    #[test]
    fn roundtrip_string() {
        let mut w = new_writer();
        write_string(&mut w, "hello, world!");
        let mut r = new_reader(writer_to_bytes(&w));
        assert_eq!(read_string(&mut r).expect("should succeed"), "hello, world!");
    }

    // 8. multiple values in sequence
    #[test]
    fn multiple_values_sequence() {
        let mut w = new_writer();
        write_u8(&mut w, 7);
        write_u32(&mut w, 1234);
        write_string(&mut w, "test");
        write_f32(&mut w, 1.5);

        let bytes = writer_to_bytes(&w).to_vec();
        let mut r = new_reader(&bytes);
        assert_eq!(read_u8(&mut r).expect("should succeed"), 7);
        assert_eq!(read_u32(&mut r).expect("should succeed"), 1234);
        assert_eq!(read_string(&mut r).expect("should succeed"), "test");
        assert!((read_f32(&mut r).expect("should succeed") - 1.5).abs() < 1e-6);
        assert_eq!(reader_remaining(&r), 0);
    }

    // 9. read past end returns Err
    #[test]
    fn read_past_end_is_error() {
        let mut r = new_reader(&[]);
        assert!(read_u8(&mut r).is_err());
    }

    // 10. read_u32 with too few bytes
    #[test]
    fn read_u32_too_few_bytes() {
        let mut r = new_reader(&[0x01, 0x02]);
        assert!(read_u32(&mut r).is_err());
    }

    // 11. reader_remaining decreases correctly
    #[test]
    fn reader_remaining_decreases() {
        let data = vec![1u8, 2, 3, 4];
        let mut r = new_reader(&data);
        assert_eq!(reader_remaining(&r), 4);
        read_u8(&mut r).expect("should succeed");
        assert_eq!(reader_remaining(&r), 3);
        assert!(read_u32(&mut r).is_err()); // only 3 left — error, pos unchanged
        // After failed read, pos should still be at 1
        assert_eq!(reader_remaining(&r), 3);
    }

    // 12. write_string empty string
    #[test]
    fn roundtrip_empty_string() {
        let mut w = new_writer();
        write_string(&mut w, "");
        let mut r = new_reader(writer_to_bytes(&w));
        assert_eq!(read_string(&mut r).expect("should succeed"), "");
    }

    // 13. writer_to_bytes slice has correct length
    #[test]
    fn writer_to_bytes_length() {
        let mut w = new_writer();
        write_u32(&mut w, 0);
        write_u64(&mut w, 0);
        // 4 + 8 = 12 bytes
        assert_eq!(writer_to_bytes(&w).len(), 12);
    }

    // 14. read_bytes with n=0 returns empty vec
    #[test]
    fn read_bytes_zero() {
        let data = vec![1u8, 2, 3];
        let mut r = new_reader(&data);
        let out = read_bytes(&mut r, 0).expect("should succeed");
        assert!(out.is_empty());
        assert_eq!(reader_remaining(&r), 3);
    }

    // 15. default_bin_config has expected defaults
    #[test]
    fn default_bin_config_sane() {
        let cfg = default_bin_config();
        assert!(cfg.initial_capacity > 0);
        assert!(!cfg.write_magic);
        assert_eq!(&cfg.magic, b"OXIB");
    }

    // 16. new_bin_writer respects initial_capacity
    #[test]
    fn new_bin_writer_capacity() {
        let cfg = default_bin_config();
        let w = new_bin_writer(&cfg);
        assert!(w.buf.capacity() >= cfg.initial_capacity);
        assert!(writer_to_bytes(&w).is_empty());
    }

    // 17. new_bin_reader starts at position 0
    #[test]
    fn new_bin_reader_starts_at_zero() {
        let cfg = default_bin_config();
        let data = vec![1u8, 2, 3];
        let r = new_bin_reader(&cfg, &data);
        assert_eq!(reader_position(&r), 0);
        assert_eq!(reader_remaining(&r), 3);
    }

    // 18. reader_position advances after each read
    #[test]
    fn reader_position_advances() {
        let mut w = new_writer();
        write_u8(&mut w, 0xFF);
        write_u32(&mut w, 42);
        let bytes = writer_to_bytes(&w).to_vec();
        let mut r = new_reader(&bytes);
        assert_eq!(reader_position(&r), 0);
        read_u8(&mut r).expect("should succeed");
        assert_eq!(reader_position(&r), 1);
        read_u32(&mut r).expect("should succeed");
        assert_eq!(reader_position(&r), 5);
    }

    // 19. new_bin_writer / write / read round-trip
    #[test]
    fn bin_writer_reader_round_trip() {
        let cfg = default_bin_config();
        let mut w = new_bin_writer(&cfg);
        write_f32(&mut w, std::f32::consts::E);
        let data = writer_to_bytes(&w).to_vec();
        let mut r = new_bin_reader(&cfg, &data);
        let v = read_f32(&mut r).expect("should succeed");
        assert!((v - std::f32::consts::E).abs() < 1e-5);
    }

    // 20. BinSerConfig is Clone
    #[test]
    fn bin_ser_config_clone() {
        let cfg = default_bin_config();
        let cfg2 = cfg.clone();
        assert_eq!(cfg.initial_capacity, cfg2.initial_capacity);
        assert_eq!(cfg.write_magic, cfg2.write_magic);
    }
}
