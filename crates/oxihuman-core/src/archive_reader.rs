// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ZIP archive reader — parses End-of-Central-Directory, Central Directory, and Local File Headers.
//! Supports compression methods 0 (Stored) and 8 (Deflate via oxiarc_deflate).

use oxiarc_deflate::inflate;
use std::fs;
use thiserror::Error;

// ─── Error ────────────────────────────────────────────────────────────────────

/// Errors that can arise while opening or reading a ZIP archive.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(String),

    /// The bytes do not conform to the ZIP specification.
    #[error("Invalid ZIP format: {0}")]
    InvalidFormat(String),

    /// A compression method other than 0 (Stored) or 8 (Deflate) was found.
    #[error("Unsupported compression method: {method}")]
    UnsupportedCompression {
        /// The raw method identifier from the Central Directory.
        method: u16,
    },

    /// `oxiarc_deflate::inflate` returned an error.
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
}

// ─── Data structures ──────────────────────────────────────────────────────────

/// A file entry loaded from an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// Path / name exactly as stored in the Central Directory.
    pub name: String,
    /// On-disk byte count (after any compression).
    pub compressed_size: u64,
    /// Logical byte count (uncompressed content).
    pub uncompressed_size: u64,
    /// Decompressed file content.
    pub data: Vec<u8>,
}

impl ArchiveEntry {
    /// Construct an entry whose compressed size equals its uncompressed size
    /// (i.e. a Stored entry that was created in memory, not read from a file).
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        let len = data.len() as u64;
        ArchiveEntry {
            name: name.to_string(),
            compressed_size: len,
            uncompressed_size: len,
            data,
        }
    }

    /// Ratio of compressed to uncompressed size.  Returns 1.0 for empty entries.
    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            1.0
        } else {
            self.compressed_size as f64 / self.uncompressed_size as f64
        }
    }
}

/// An in-memory view of all entries in a ZIP archive.
#[derive(Debug)]
pub struct ArchiveReader {
    entries: Vec<ArchiveEntry>,
    source: String,
}

impl ArchiveReader {
    /// Create an empty reader not backed by a file.
    pub fn new(source: &str) -> Self {
        ArchiveReader {
            entries: Vec::new(),
            source: source.to_string(),
        }
    }

    /// Push an entry (used when constructing an in-memory archive).
    pub fn load_entry(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    /// All entry names, in the order they appear in the Central Directory.
    pub fn entry_names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }

    /// Find an entry by its exact name.
    pub fn find_entry(&self, name: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Sum of all uncompressed sizes.
    pub fn total_uncompressed(&self) -> u64 {
        self.entries.iter().map(|e| e.uncompressed_size).sum()
    }

    /// Number of entries in this reader.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// The path that was passed to `open_archive` / `open_archive_stub`.
    pub fn source(&self) -> &str {
        &self.source
    }
}

// ─── Little-endian helpers ────────────────────────────────────────────────────

#[inline]
fn read_u16_le(buf: &[u8], offset: usize) -> Result<u16, ArchiveError> {
    if offset + 2 > buf.len() {
        return Err(ArchiveError::InvalidFormat(format!(
            "u16 read at offset {} overflows buffer len {}",
            offset,
            buf.len()
        )));
    }
    Ok(u16::from_le_bytes([buf[offset], buf[offset + 1]]))
}

#[inline]
fn read_u32_le(buf: &[u8], offset: usize) -> Result<u32, ArchiveError> {
    if offset + 4 > buf.len() {
        return Err(ArchiveError::InvalidFormat(format!(
            "u32 read at offset {} overflows buffer len {}",
            offset,
            buf.len()
        )));
    }
    Ok(u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]))
}

// ─── ZIP format constants ─────────────────────────────────────────────────────

const SIG_LOCAL_FILE_HEADER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const SIG_CENTRAL_DIR_ENTRY: [u8; 4] = [0x50, 0x4B, 0x01, 0x02];
const SIG_EOCD: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];

const COMPRESSION_STORED: u16 = 0;
const COMPRESSION_DEFLATE: u16 = 8;

/// Minimum sizes of fixed-width fields (not including variable-length names).
const EOCD_FIXED_SIZE: usize = 22;
const CD_ENTRY_FIXED_SIZE: usize = 46;
const LFH_FIXED_SIZE: usize = 30;

// ─── EOCD parsing ─────────────────────────────────────────────────────────────

/// Offsets within an EOCD record (relative to start of signature).
mod eocd {
    pub const TOTAL_ENTRIES: usize = 10;
    pub const CD_OFFSET: usize = 16;
    pub const COMMENT_LEN: usize = 20;
}

struct EocdRecord {
    total_entries: u16,
    cd_offset: u32,
}

/// Scan backwards through `buf` for the EOCD signature.
///
/// The EOCD record must end at the end of the file (possibly followed by a
/// zip comment whose length is encoded in the record itself).  The maximum
/// comment length is 0xFFFF bytes, so we search the last 65536 + 22 bytes.
fn find_eocd(buf: &[u8]) -> Result<EocdRecord, ArchiveError> {
    let file_len = buf.len();
    if file_len < EOCD_FIXED_SIZE {
        return Err(ArchiveError::InvalidFormat(
            "file is too small to contain an EOCD record".to_string(),
        ));
    }

    // The earliest possible position for the EOCD signature.
    let search_start = file_len.saturating_sub(EOCD_FIXED_SIZE + 0xFFFF);

    // Scan backwards.
    let mut pos = file_len - EOCD_FIXED_SIZE;
    loop {
        if buf[pos..pos + 4] == SIG_EOCD {
            // Validate that the comment length matches.
            let comment_len = read_u16_le(buf, pos + eocd::COMMENT_LEN)? as usize;
            if pos + EOCD_FIXED_SIZE + comment_len == file_len {
                let total_entries = read_u16_le(buf, pos + eocd::TOTAL_ENTRIES)?;
                let cd_offset = read_u32_le(buf, pos + eocd::CD_OFFSET)?;
                return Ok(EocdRecord {
                    total_entries,
                    cd_offset,
                });
            }
        }
        if pos == search_start {
            break;
        }
        pos -= 1;
    }

    Err(ArchiveError::InvalidFormat(
        "EOCD signature (PK\\x05\\x06) not found".to_string(),
    ))
}

// ─── Central Directory parsing ────────────────────────────────────────────────

/// Offsets within a Central Directory entry (relative to start of signature).
mod cd {
    pub const COMPRESSION_METHOD: usize = 10;
    pub const CRC32: usize = 16;
    pub const COMPRESSED_SIZE: usize = 20;
    pub const UNCOMPRESSED_SIZE: usize = 24;
    pub const NAME_LEN: usize = 28;
    pub const EXTRA_LEN: usize = 30;
    pub const COMMENT_LEN: usize = 32;
    pub const LOCAL_HEADER_OFFSET: usize = 42;
    pub const NAME_START: usize = 46;
}

struct CdEntry {
    compression_method: u16,
    #[allow(dead_code)] // parsed for completeness; CRC validation is a future enhancement
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    name: String,
    local_header_offset: u32,
    /// Total byte width of this CD entry (signature + fixed fields + variable).
    record_size: usize,
}

fn parse_cd_entry(buf: &[u8], cd_start: usize) -> Result<CdEntry, ArchiveError> {
    if cd_start + CD_ENTRY_FIXED_SIZE > buf.len() {
        return Err(ArchiveError::InvalidFormat(format!(
            "CD entry at offset {} is truncated (file len {})",
            cd_start,
            buf.len()
        )));
    }

    if buf[cd_start..cd_start + 4] != SIG_CENTRAL_DIR_ENTRY {
        return Err(ArchiveError::InvalidFormat(format!(
            "Expected CD entry signature at offset {}, got {:02x?}",
            cd_start,
            &buf[cd_start..cd_start + 4]
        )));
    }

    let compression_method = read_u16_le(buf, cd_start + cd::COMPRESSION_METHOD)?;
    let crc32 = read_u32_le(buf, cd_start + cd::CRC32)?;
    let compressed_size = read_u32_le(buf, cd_start + cd::COMPRESSED_SIZE)?;
    let uncompressed_size = read_u32_le(buf, cd_start + cd::UNCOMPRESSED_SIZE)?;
    let name_len = read_u16_le(buf, cd_start + cd::NAME_LEN)? as usize;
    let extra_len = read_u16_le(buf, cd_start + cd::EXTRA_LEN)? as usize;
    let comment_len = read_u16_le(buf, cd_start + cd::COMMENT_LEN)? as usize;
    let local_header_offset = read_u32_le(buf, cd_start + cd::LOCAL_HEADER_OFFSET)?;

    let name_end = cd_start + cd::NAME_START + name_len;
    if name_end > buf.len() {
        return Err(ArchiveError::InvalidFormat(format!(
            "CD entry name extends beyond EOF at offset {}",
            cd_start
        )));
    }

    let name = String::from_utf8_lossy(&buf[cd_start + cd::NAME_START..name_end]).into_owned();
    let record_size = CD_ENTRY_FIXED_SIZE + name_len + extra_len + comment_len;

    Ok(CdEntry {
        compression_method,
        crc32,
        compressed_size,
        uncompressed_size,
        name,
        local_header_offset,
        record_size,
    })
}

// ─── Local File Header parsing ────────────────────────────────────────────────

/// Offsets within a Local File Header (relative to start of signature).
mod lfh {
    pub const NAME_LEN: usize = 26;
    pub const EXTRA_LEN: usize = 28;
}

/// Return the byte offset at which file data begins inside `buf`, given that
/// the Local File Header starts at `lfh_start`.
fn data_start_from_lfh(buf: &[u8], lfh_start: usize) -> Result<usize, ArchiveError> {
    if lfh_start + LFH_FIXED_SIZE > buf.len() {
        return Err(ArchiveError::InvalidFormat(format!(
            "Local File Header at offset {} is truncated",
            lfh_start
        )));
    }

    if buf[lfh_start..lfh_start + 4] != SIG_LOCAL_FILE_HEADER {
        return Err(ArchiveError::InvalidFormat(format!(
            "Expected Local File Header signature at offset {}, got {:02x?}",
            lfh_start,
            &buf[lfh_start..lfh_start + 4]
        )));
    }

    let name_len = read_u16_le(buf, lfh_start + lfh::NAME_LEN)? as usize;
    let extra_len = read_u16_le(buf, lfh_start + lfh::EXTRA_LEN)? as usize;
    Ok(lfh_start + LFH_FIXED_SIZE + name_len + extra_len)
}

// ─── Decompression dispatch ───────────────────────────────────────────────────

fn decompress_entry(
    method: u16,
    compressed_data: &[u8],
    uncompressed_size: u32,
) -> Result<Vec<u8>, ArchiveError> {
    match method {
        COMPRESSION_STORED => {
            // Stored: data is already the raw file bytes.
            if compressed_data.len() != uncompressed_size as usize {
                return Err(ArchiveError::InvalidFormat(format!(
                    "Stored entry: compressed data len {} != declared uncompressed size {}",
                    compressed_data.len(),
                    uncompressed_size
                )));
            }
            Ok(compressed_data.to_vec())
        }
        COMPRESSION_DEFLATE => {
            inflate(compressed_data).map_err(|e| ArchiveError::DecompressionFailed(e.to_string()))
        }
        other => Err(ArchiveError::UnsupportedCompression { method: other }),
    }
}

// ─── Public entry points ──────────────────────────────────────────────────────

/// Open a ZIP file from `path`, parse all entries, and return an `ArchiveReader`
/// whose entries contain the fully decompressed file data.
///
/// # Errors
///
/// Returns `ArchiveError::Io` if the file cannot be read, `ArchiveError::InvalidFormat`
/// if the bytes do not conform to the ZIP specification, `ArchiveError::UnsupportedCompression`
/// for any compression method other than 0 or 8, and `ArchiveError::DecompressionFailed`
/// when `oxiarc_deflate::inflate` returns an error.
pub fn open_archive(path: &str) -> Result<ArchiveReader, ArchiveError> {
    let buf = fs::read(path).map_err(|e| ArchiveError::Io(e.to_string()))?;
    parse_zip_bytes(&buf, path)
}

/// Convenience wrapper that calls `open_archive` and returns an empty
/// `ArchiveReader` on any error (backward-compatible with the old stub API).
pub fn open_archive_stub(path: &str) -> ArchiveReader {
    open_archive(path).unwrap_or_else(|_| ArchiveReader::new(path))
}

/// Parse ZIP bytes without needing a file path (useful for in-memory blobs or
/// bytes that were already loaded).  `source` is stored in the resulting reader.
fn parse_zip_bytes(buf: &[u8], source: &str) -> Result<ArchiveReader, ArchiveError> {
    let eocd = find_eocd(buf)?;
    let mut reader = ArchiveReader::new(source);

    let total_entries = eocd.total_entries as usize;
    let cd_start = eocd.cd_offset as usize;

    if cd_start > buf.len() {
        return Err(ArchiveError::InvalidFormat(format!(
            "Central Directory offset {} is beyond file length {}",
            cd_start,
            buf.len()
        )));
    }

    let mut cd_cursor = cd_start;
    for entry_idx in 0..total_entries {
        let cd_entry = parse_cd_entry(buf, cd_cursor)?;

        // Advance past this CD entry before we error on any data problem,
        // so we can at least attempt the remaining entries.
        cd_cursor += cd_entry.record_size;

        let lfh_start = cd_entry.local_header_offset as usize;
        let data_start = data_start_from_lfh(buf, lfh_start)
            .map_err(|e| ArchiveError::InvalidFormat(format!("entry {}: {}", entry_idx, e)))?;

        let compressed_size = cd_entry.compressed_size as usize;
        let data_end = data_start + compressed_size;

        if data_end > buf.len() {
            return Err(ArchiveError::InvalidFormat(format!(
                "Entry '{}': compressed data [{}..{}] extends beyond file length {}",
                cd_entry.name,
                data_start,
                data_end,
                buf.len()
            )));
        }

        let compressed_bytes = &buf[data_start..data_end];
        let decompressed = decompress_entry(
            cd_entry.compression_method,
            compressed_bytes,
            cd_entry.uncompressed_size,
        )?;

        reader.load_entry(ArchiveEntry {
            name: cd_entry.name,
            compressed_size: cd_entry.compressed_size as u64,
            uncompressed_size: cd_entry.uncompressed_size as u64,
            data: decompressed,
        });
    }

    Ok(reader)
}

// ─── Accessor helpers ─────────────────────────────────────────────────────────

/// Read the decompressed bytes of a named entry.
pub fn read_entry_bytes(reader: &ArchiveReader, name: &str) -> Option<Vec<u8>> {
    reader.find_entry(name).map(|e| e.data.clone())
}

/// Read a named entry and decode it as UTF-8.  Returns `None` if the entry
/// does not exist or its bytes are not valid UTF-8.
pub fn read_entry_text(reader: &ArchiveReader, name: &str) -> Option<String> {
    let bytes = read_entry_bytes(reader, name)?;
    String::from_utf8(bytes).ok()
}

/// All entry names that contain `pattern` as a substring.
pub fn list_matching<'a>(reader: &'a ArchiveReader, pattern: &str) -> Vec<&'a str> {
    reader
        .entries
        .iter()
        .filter(|e| e.name.contains(pattern))
        .map(|e| e.name.as_str())
        .collect()
}

/// Sum of all compressed sizes.
pub fn total_compressed(reader: &ArchiveReader) -> u64 {
    reader.entries.iter().map(|e| e.compressed_size).sum()
}

// ─── Test helpers (ZIP byte builders) ────────────────────────────────────────

#[cfg(test)]
mod zip_builder {
    //! Minimal hand-crafted ZIP builder for unit tests.
    //! Only supports Stored (method 0) entries.  Deflated entries are built
    //! separately by the individual test using `oxiarc_deflate::deflate`.

    pub struct ZipFile {
        local_sections: Vec<u8>,
        central_dirs: Vec<u8>,
        entry_count: u16,
    }

    impl ZipFile {
        pub fn new() -> Self {
            ZipFile {
                local_sections: Vec::new(),
                central_dirs: Vec::new(),
                entry_count: 0,
            }
        }

        /// Add a Stored (method 0) entry.
        pub fn add_stored(&mut self, name: &str, data: &[u8]) {
            self.add_entry(0, name, data, data);
        }

        /// Add an entry with explicit compression method, giving both the
        /// raw compressed bytes and the original data (used to derive sizes).
        pub fn add_entry(&mut self, method: u16, name: &str, compressed: &[u8], original: &[u8]) {
            let name_bytes = name.as_bytes();
            let local_offset = self.local_sections.len() as u32;
            let crc = crc32_simple(original);
            let comp_size = compressed.len() as u32;
            let uncomp_size = original.len() as u32;

            // Local file header
            self.local_sections
                .extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]); // sig
            self.local_sections.extend_from_slice(&[0x14, 0x00]); // version needed: 20
            self.local_sections.extend_from_slice(&[0x00, 0x00]); // flags
            self.local_sections.extend_from_slice(&method.to_le_bytes()); // compression method
            self.local_sections.extend_from_slice(&[0x00, 0x00]); // last mod time
            self.local_sections.extend_from_slice(&[0x00, 0x00]); // last mod date
            self.local_sections.extend_from_slice(&crc.to_le_bytes()); // crc-32
            self.local_sections
                .extend_from_slice(&comp_size.to_le_bytes()); // compressed size
            self.local_sections
                .extend_from_slice(&uncomp_size.to_le_bytes()); // uncompressed size
            self.local_sections
                .extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // name len
            self.local_sections.extend_from_slice(&[0x00, 0x00]); // extra len
            self.local_sections.extend_from_slice(name_bytes); // name
            self.local_sections.extend_from_slice(compressed); // file data

            // Central directory entry
            self.central_dirs
                .extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]); // sig
            self.central_dirs.extend_from_slice(&[0x1E, 0x03]); // version made by (Unix, 30)
            self.central_dirs.extend_from_slice(&[0x14, 0x00]); // version needed
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // flags
            self.central_dirs.extend_from_slice(&method.to_le_bytes()); // compression method
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // last mod time
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // last mod date
            self.central_dirs.extend_from_slice(&crc.to_le_bytes()); // crc-32
            self.central_dirs
                .extend_from_slice(&comp_size.to_le_bytes()); // compressed size
            self.central_dirs
                .extend_from_slice(&uncomp_size.to_le_bytes()); // uncompressed size
            self.central_dirs
                .extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // name len
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // extra len
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // comment len
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // disk number start
            self.central_dirs.extend_from_slice(&[0x00, 0x00]); // internal attributes
            self.central_dirs
                .extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // external attributes
            self.central_dirs
                .extend_from_slice(&local_offset.to_le_bytes()); // local header offset
            self.central_dirs.extend_from_slice(name_bytes); // name

            self.entry_count += 1;
        }

        /// Serialise into a complete ZIP byte vector.
        pub fn finish(self) -> Vec<u8> {
            let mut out = self.local_sections;
            let cd_offset = out.len() as u32;
            let cd_size = self.central_dirs.len() as u32;
            out.extend_from_slice(&self.central_dirs);

            // EOCD record
            out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]); // sig
            out.extend_from_slice(&[0x00, 0x00]); // disk number
            out.extend_from_slice(&[0x00, 0x00]); // disk with CD
            out.extend_from_slice(&self.entry_count.to_le_bytes()); // entries on disk
            out.extend_from_slice(&self.entry_count.to_le_bytes()); // total entries
            out.extend_from_slice(&cd_size.to_le_bytes()); // CD size
            out.extend_from_slice(&cd_offset.to_le_bytes()); // CD offset
            out.extend_from_slice(&[0x00, 0x00]); // comment length
            out
        }
    }

    /// CRC-32 (ISO 3309 / ITU-T V.42) — simple table-free implementation sufficient
    /// for building small test fixtures.
    pub fn crc32_simple(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFF_FFFF;
        for &byte in data {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB8_8320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive_reader::zip_builder::ZipFile;
    use oxiarc_deflate::deflate as oxiarc_deflate_compress;
    use std::io::Write;

    // ── Existing tests (must keep passing) ──────────────────────────────────

    #[test]
    fn test_open_archive_stub() {
        // Non-existent file: stub silently returns an empty reader.
        let r = open_archive_stub("/tmp/oxihuman_nonexistent_test_stub_file_99999.zip");
        assert_eq!(r.entry_count(), 0);
        assert_eq!(
            r.source(),
            "/tmp/oxihuman_nonexistent_test_stub_file_99999.zip"
        );
    }

    #[test]
    fn test_load_and_find_entry() {
        let mut r = open_archive_stub("/tmp/a.zip");
        r.load_entry(ArchiveEntry::new("hello.txt", b"hello".to_vec()));
        assert!(r.find_entry("hello.txt").is_some());
        assert!(r.find_entry("nope.txt").is_none());
    }

    #[test]
    fn test_read_entry_bytes() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("data.bin", vec![1, 2, 3]));
        let bytes = read_entry_bytes(&r, "data.bin").expect("should succeed");
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    #[test]
    fn test_read_entry_text() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("note.txt", b"hello world".to_vec()));
        let text = read_entry_text(&r, "note.txt").expect("should succeed");
        assert_eq!(text, "hello world");
    }

    #[test]
    fn test_total_uncompressed() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("a", vec![0u8; 100]));
        r.load_entry(ArchiveEntry::new("b", vec![0u8; 200]));
        assert_eq!(r.total_uncompressed(), 300);
    }

    #[test]
    fn test_list_matching() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("src/main.rs", vec![]));
        r.load_entry(ArchiveEntry::new("tests/test.rs", vec![]));
        let found = list_matching(&r, "src/");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_compression_ratio_empty() {
        let e = ArchiveEntry::new("empty", vec![]);
        assert!((e.compression_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_entry_names() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("a.txt", vec![]));
        r.load_entry(ArchiveEntry::new("b.txt", vec![]));
        let names = r.entry_names();
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"b.txt"));
    }

    #[test]
    fn test_total_compressed() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("f", vec![0u8; 50]));
        assert_eq!(total_compressed(&r), 50);
    }

    #[test]
    fn test_entry_count() {
        let mut r = open_archive_stub("/tmp/x.zip");
        for i in 0..5 {
            r.load_entry(ArchiveEntry::new(&format!("f{}.txt", i), vec![]));
        }
        assert_eq!(r.entry_count(), 5);
    }

    // ── New real-ZIP tests ────────────────────────────────────────────────────

    fn temp_zip_path(suffix: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("oxihuman_test_{}.zip", suffix));
        p
    }

    fn write_zip_to_temp(bytes: Vec<u8>, suffix: &str) -> std::path::PathBuf {
        let path = temp_zip_path(suffix);
        let mut f = std::fs::File::create(&path).expect("create temp zip");
        f.write_all(&bytes).expect("write temp zip");
        path
    }

    /// Build a two-entry Stored ZIP in memory and verify round-trip.
    #[test]
    fn test_open_archive_real_zip() {
        let mut builder = ZipFile::new();
        builder.add_stored("readme.txt", b"Hello from OxiHuman");
        builder.add_stored("data/config.toml", b"[section]\nkey = \"value\"");
        let zip_bytes = builder.finish();
        let path = write_zip_to_temp(zip_bytes, "real_zip");

        let reader = open_archive(path.to_str().expect("valid path"))
            .expect("open_archive should succeed on well-formed ZIP");

        assert_eq!(reader.entry_count(), 2);

        let text = read_entry_text(&reader, "readme.txt").expect("readme.txt should be readable");
        assert_eq!(text, "Hello from OxiHuman");

        let text2 =
            read_entry_text(&reader, "data/config.toml").expect("config.toml should be readable");
        assert_eq!(text2, "[section]\nkey = \"value\"");

        let _ = std::fs::remove_file(&path);
    }

    /// Verify Stored (method 0) entries are read verbatim.
    #[test]
    fn test_open_archive_stored() {
        let payload: Vec<u8> = (0u8..=255).collect();
        let mut builder = ZipFile::new();
        builder.add_stored("binary.bin", &payload);
        let zip_bytes = builder.finish();
        let path = write_zip_to_temp(zip_bytes, "stored");

        let reader = open_archive(path.to_str().expect("valid path")).expect("open_archive stored");
        assert_eq!(reader.entry_count(), 1);
        let data = read_entry_bytes(&reader, "binary.bin").expect("binary.bin");
        assert_eq!(data, payload);

        let _ = std::fs::remove_file(&path);
    }

    /// Verify Deflate (method 8) entries are correctly inflated.
    #[test]
    fn test_open_archive_deflated() {
        // Produce a compressible payload (repeated pattern).
        let original: Vec<u8> = (0u8..64).cycle().take(512).collect();
        let compressed = oxiarc_deflate_compress(&original, 6).expect("deflate test data");

        let mut builder = ZipFile::new();
        builder.add_entry(8, "deflated.bin", &compressed, &original);
        let zip_bytes = builder.finish();
        let path = write_zip_to_temp(zip_bytes, "deflated");

        let reader =
            open_archive(path.to_str().expect("valid path")).expect("open_archive deflated");
        assert_eq!(reader.entry_count(), 1);
        let data = read_entry_bytes(&reader, "deflated.bin").expect("deflated.bin");
        assert_eq!(data, original, "decompressed data must match original");

        let _ = std::fs::remove_file(&path);
    }

    /// Non-existent file must yield `ArchiveError::Io`.
    #[test]
    fn test_open_archive_not_found_returns_error() {
        let result = open_archive("/tmp/oxihuman_definitely_does_not_exist_99999.zip");
        assert!(
            matches!(result, Err(ArchiveError::Io(_))),
            "expected Err(ArchiveError::Io), got {:?}",
            result
        );
    }

    /// Bytes that are not a ZIP file must yield `ArchiveError::InvalidFormat`.
    #[test]
    fn test_open_archive_invalid_zip_returns_error() {
        let path = {
            let mut p = std::env::temp_dir();
            p.push("oxihuman_test_invalid_zip.bin");
            p
        };
        std::fs::write(
            &path,
            b"This is not a ZIP file at all, no signatures here!!",
        )
        .expect("write invalid file");

        let result = open_archive(path.to_str().expect("valid path"));
        assert!(
            matches!(result, Err(ArchiveError::InvalidFormat(_))),
            "expected Err(ArchiveError::InvalidFormat), got {:?}",
            result
        );

        let _ = std::fs::remove_file(&path);
    }

    /// Verify that `open_archive_stub` silently degrades to an empty reader on
    /// a well-formed ZIP that does not exist on disk (non-existent path).
    #[test]
    fn test_stub_falls_back_on_missing_file() {
        let r = open_archive_stub("/tmp/oxihuman_stub_fallback_test_nope.zip");
        assert_eq!(
            r.entry_count(),
            0,
            "stub must return empty reader on missing file"
        );
    }

    /// Verify that `open_archive_stub` succeeds on a real ZIP when the file
    /// actually exists on disk.
    #[test]
    fn test_stub_reads_real_zip_when_present() {
        let mut builder = ZipFile::new();
        builder.add_stored("hello.txt", b"stub test");
        let zip_bytes = builder.finish();
        let path = write_zip_to_temp(zip_bytes, "stub_real");

        let reader = open_archive_stub(path.to_str().expect("valid path"));
        assert_eq!(reader.entry_count(), 1);
        let text = read_entry_text(&reader, "hello.txt").expect("hello.txt");
        assert_eq!(text, "stub test");

        let _ = std::fs::remove_file(&path);
    }

    /// Verify multi-entry round-trip: 5 Stored entries with varied content.
    #[test]
    fn test_open_archive_multi_entry() {
        let entries: Vec<(&str, Vec<u8>)> = vec![
            ("a.txt", b"aaa".to_vec()),
            ("b.txt", b"bbbbb".to_vec()),
            ("c/d.txt", b"nested file".to_vec()),
            ("empty.txt", vec![]),
            ("binary.bin", vec![0xDE, 0xAD, 0xBE, 0xEF]),
        ];

        let mut builder = ZipFile::new();
        for (name, data) in &entries {
            builder.add_stored(name, data);
        }
        let zip_bytes = builder.finish();
        let path = write_zip_to_temp(zip_bytes, "multi_entry");

        let reader =
            open_archive(path.to_str().expect("valid path")).expect("open_archive multi_entry");
        assert_eq!(reader.entry_count(), entries.len());

        for (name, expected) in &entries {
            let got = read_entry_bytes(&reader, name)
                .unwrap_or_else(|| panic!("entry '{}' not found", name));
            assert_eq!(&got, expected, "content mismatch for '{}'", name);
        }

        let _ = std::fs::remove_file(&path);
    }
}
