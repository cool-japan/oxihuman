// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! OXB — OxiHuman Bundle: a simple binary multi-file asset bundle format.
//!
//! Format specification:
//! ```text
//! Header (16 bytes):
//!   [0..4]   magic: b"OXB1"
//!   [4..8]   entry_count: u32 LE
//!   [8..16]  reserved: [u8; 8] = 0
//!
//! Directory (entry_count entries, each 72 bytes):
//!   [0..64]  name: null-padded UTF-8 string (max 63 chars + null)
//!   [64..68] offset: u32 LE  (byte offset from start of bundle)
//!   [68..72] length: u32 LE  (byte length of entry data)
//!
//! Data section:
//!   Raw bytes of all entries, concatenated in directory order
//! ```

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

/// Magic bytes that identify an OXB bundle file.
pub const OXB_MAGIC: &[u8; 4] = b"OXB1";

/// Maximum number of characters in an entry name (not counting the terminating null byte).
pub const MAX_ENTRY_NAME: usize = 63;

const HEADER_SIZE: usize = 16;
const DIR_ENTRY_SIZE: usize = 72;
const NAME_FIELD_SIZE: usize = 64;

// ── BundleEntry ──────────────────────────────────────────────────────────────

/// A single named data entry inside an [`AssetBundle`].
pub struct BundleEntry {
    pub name: String,
    pub data: Vec<u8>,
}

impl BundleEntry {
    /// Create a new entry from an in-memory byte vector.
    ///
    /// Returns an error if `name` is empty, longer than [`MAX_ENTRY_NAME`] bytes,
    /// or contains a null byte.
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Result<Self> {
        let name = name.into();
        validate_name(&name)?;
        Ok(Self { name, data })
    }

    /// Create a new entry by reading `path` from disk.
    pub fn from_file(name: impl Into<String>, path: &Path) -> Result<Self> {
        let name = name.into();
        validate_name(&name)?;
        let data =
            fs::read(path).with_context(|| format!("failed to read file: {}", path.display()))?;
        Ok(Self { name, data })
    }

    /// Returns the byte length of the entry data.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

// ── AssetBundle ───────────────────────────────────────────────────────────────

/// An in-memory collection of named binary entries that can be exported to / loaded
/// from a `.oxb` bundle file.
pub struct AssetBundle {
    entries: Vec<BundleEntry>,
}

impl AssetBundle {
    /// Create an empty bundle.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a pre-built [`BundleEntry`] to the bundle.
    ///
    /// Returns an error if an entry with the same name already exists.
    pub fn add(&mut self, entry: BundleEntry) -> Result<()> {
        if self.contains(&entry.name) {
            bail!("duplicate entry name: {}", entry.name);
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Add a named byte slice to the bundle.
    pub fn add_bytes(&mut self, name: impl Into<String>, data: Vec<u8>) -> Result<()> {
        let entry = BundleEntry::new(name, data)?;
        self.add(entry)
    }

    /// Add a named file from disk to the bundle.
    pub fn add_file(&mut self, name: impl Into<String>, path: &Path) -> Result<()> {
        let entry = BundleEntry::from_file(name, path)?;
        self.add(entry)
    }

    /// Add a named UTF-8 string to the bundle (stored as raw UTF-8 bytes).
    pub fn add_str(&mut self, name: impl Into<String>, text: &str) -> Result<()> {
        self.add_bytes(name, text.as_bytes().to_vec())
    }

    /// Returns the number of entries in the bundle.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the total number of data bytes across all entries.
    pub fn total_size(&self) -> usize {
        self.entries.iter().map(|e| e.size()).sum()
    }

    /// Look up an entry by name.
    pub fn get(&self, name: &str) -> Option<&BundleEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Returns a list of all entry names in insertion order.
    pub fn entry_names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }

    /// Returns `true` if the bundle contains an entry with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }

    /// Remove an entry by name.  Returns `true` if an entry was found and removed.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.name == name) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for AssetBundle {
    fn default() -> Self {
        Self::new()
    }
}

// ── I/O ──────────────────────────────────────────────────────────────────────

/// Serialise `bundle` to a `.oxb` file at `path`.
pub fn export_bundle(bundle: &AssetBundle, path: &Path) -> Result<()> {
    let entry_count = bundle.entries.len();
    let data_offset = HEADER_SIZE + entry_count * DIR_ENTRY_SIZE;

    let mut file = fs::File::create(path)
        .with_context(|| format!("cannot create bundle file: {}", path.display()))?;

    // ── Header ──────────────────────────────────────────────────────────────
    file.write_all(OXB_MAGIC)?;
    file.write_all(&(entry_count as u32).to_le_bytes())?;
    file.write_all(&[0u8; 8])?; // reserved

    // ── Directory ───────────────────────────────────────────────────────────
    let mut current_offset = data_offset as u32;
    for entry in &bundle.entries {
        let mut name_buf = [0u8; NAME_FIELD_SIZE];
        let name_bytes = entry.name.as_bytes();
        name_buf[..name_bytes.len()].copy_from_slice(name_bytes);
        file.write_all(&name_buf)?;
        file.write_all(&current_offset.to_le_bytes())?;
        file.write_all(&(entry.data.len() as u32).to_le_bytes())?;
        current_offset += entry.data.len() as u32;
    }

    // ── Data section ────────────────────────────────────────────────────────
    for entry in &bundle.entries {
        file.write_all(&entry.data)?;
    }

    file.flush()?;
    Ok(())
}

/// Deserialise a `.oxb` file from `path` into an [`AssetBundle`].
pub fn load_bundle(path: &Path) -> Result<AssetBundle> {
    let raw =
        fs::read(path).with_context(|| format!("cannot read bundle file: {}", path.display()))?;

    if raw.len() < HEADER_SIZE {
        bail!("bundle file too small to contain a valid header");
    }

    // ── Header ──────────────────────────────────────────────────────────────
    if &raw[0..4] != OXB_MAGIC.as_ref() {
        bail!("invalid OXB magic bytes");
    }
    let entry_count = u32::from_le_bytes(
        raw[4..8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    ) as usize;

    let dir_end = HEADER_SIZE + entry_count * DIR_ENTRY_SIZE;
    if raw.len() < dir_end {
        bail!("bundle file truncated: directory extends past end of file");
    }

    // ── Directory + data ────────────────────────────────────────────────────
    let mut bundle = AssetBundle::new();
    for i in 0..entry_count {
        let base = HEADER_SIZE + i * DIR_ENTRY_SIZE;
        let name_field = &raw[base..base + NAME_FIELD_SIZE];
        let null_pos = name_field
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(NAME_FIELD_SIZE);
        let name = std::str::from_utf8(&name_field[..null_pos])
            .with_context(|| format!("entry {} has invalid UTF-8 name", i))?
            .to_owned();

        let offset = u32::from_le_bytes(
            raw[base + 64..base + 68]
                .try_into()
                .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
        ) as usize;
        let length = u32::from_le_bytes(
            raw[base + 68..base + 72]
                .try_into()
                .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
        ) as usize;

        if offset + length > raw.len() {
            bail!(
                "entry '{}' data range [{}, {}) exceeds file size {}",
                name,
                offset,
                offset + length,
                raw.len()
            );
        }

        let data = raw[offset..offset + length].to_vec();
        bundle
            .add(BundleEntry { name, data })
            .with_context(|| format!("failed to add entry {}", i))?;
    }

    Ok(bundle)
}

/// Validate a `.oxb` file without fully loading its data.
///
/// Checks the magic bytes, the directory structure, and that every entry's
/// data range lies within the file.  Returns the number of entries on success.
pub fn validate_bundle(path: &Path) -> Result<usize> {
    let raw =
        fs::read(path).with_context(|| format!("cannot read bundle file: {}", path.display()))?;

    if raw.len() < HEADER_SIZE {
        bail!("bundle file too small");
    }
    if &raw[0..4] != OXB_MAGIC.as_ref() {
        bail!("invalid OXB magic bytes");
    }

    let entry_count = u32::from_le_bytes(
        raw[4..8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    ) as usize;
    let dir_end = HEADER_SIZE + entry_count * DIR_ENTRY_SIZE;

    if raw.len() < dir_end {
        bail!("directory extends past end of file");
    }

    for i in 0..entry_count {
        let base = HEADER_SIZE + i * DIR_ENTRY_SIZE;
        let offset = u32::from_le_bytes(
            raw[base + 64..base + 68]
                .try_into()
                .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
        ) as usize;
        let length = u32::from_le_bytes(
            raw[base + 68..base + 72]
                .try_into()
                .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
        ) as usize;

        if offset < dir_end {
            bail!(
                "entry {} offset {} is inside the header/directory region",
                i,
                offset
            );
        }
        if offset.checked_add(length).is_none_or(|end| end > raw.len()) {
            bail!(
                "entry {} data range [{}, {}) exceeds file size",
                i,
                offset,
                offset + length
            );
        }
    }

    Ok(entry_count)
}

/// Extract all entries from the bundle at `path` to `output_dir`.
///
/// `output_dir` is created if it does not exist.  Returns the list of extracted
/// entry names.
pub fn extract_bundle(path: &Path, output_dir: &Path) -> Result<Vec<String>> {
    let bundle = load_bundle(path)?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("cannot create output directory: {}", output_dir.display()))?;

    let mut names = Vec::new();
    for entry in &bundle.entries {
        let out_path = output_dir.join(&entry.name);
        fs::write(&out_path, &entry.data)
            .with_context(|| format!("cannot write extracted file: {}", out_path.display()))?;
        names.push(entry.name.clone());
    }
    Ok(names)
}

/// Build a bundle from all *files* (non-recursive) in `dir`.
///
/// Each file's name (not full path) is used as the entry name.  Entries are
/// added in directory-read order (which is filesystem-dependent).
pub fn bundle_from_dir(dir: &Path) -> Result<AssetBundle> {
    let mut bundle = AssetBundle::new();
    let read_dir =
        fs::read_dir(dir).with_context(|| format!("cannot read directory: {}", dir.display()))?;

    for result in read_dir {
        let entry = result.with_context(|| "failed to read directory entry")?;
        let meta = entry.metadata()?;
        if !meta.is_file() {
            continue;
        }
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("non-UTF-8 file name in directory"))?;
        let data = fs::read(entry.path())?;
        bundle.add_bytes(file_name, data)?;
    }

    Ok(bundle)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("entry name must not be empty");
    }
    if name.len() > MAX_ENTRY_NAME {
        bail!(
            "entry name '{}' is {} bytes, exceeds MAX_ENTRY_NAME ({})",
            name,
            name.len(),
            MAX_ENTRY_NAME
        );
    }
    if name.contains('\0') {
        bail!("entry name must not contain null bytes");
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_path(filename: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/{}", filename))
    }

    // ── BundleEntry ──────────────────────────────────────────────────────────

    #[test]
    fn test_bundle_entry_new() {
        let entry = BundleEntry::new("mesh.bin", vec![1, 2, 3]).unwrap();
        assert_eq!(entry.name, "mesh.bin");
        assert_eq!(entry.data, vec![1, 2, 3]);
        assert_eq!(entry.size(), 3);
    }

    #[test]
    fn test_bundle_entry_name_too_long() {
        let long_name = "a".repeat(MAX_ENTRY_NAME + 1);
        let result = BundleEntry::new(long_name, vec![]);
        assert!(result.is_err());
    }

    // ── AssetBundle ──────────────────────────────────────────────────────────

    #[test]
    fn test_asset_bundle_new() {
        let bundle = AssetBundle::new();
        assert_eq!(bundle.entry_count(), 0);
        assert_eq!(bundle.total_size(), 0);
    }

    #[test]
    fn test_add_bytes() {
        let mut bundle = AssetBundle::new();
        bundle.add_bytes("alpha", vec![0xFF, 0x00]).unwrap();
        bundle.add_bytes("beta", vec![1, 2, 3, 4]).unwrap();
        assert_eq!(bundle.entry_count(), 2);
        assert_eq!(bundle.total_size(), 6);
    }

    #[test]
    fn test_add_str() {
        let mut bundle = AssetBundle::new();
        bundle.add_str("readme.txt", "Hello, world!").unwrap();
        let entry = bundle.get("readme.txt").unwrap();
        assert_eq!(entry.data, b"Hello, world!");
    }

    #[test]
    fn test_contains_and_get() {
        let mut bundle = AssetBundle::new();
        bundle.add_bytes("x", vec![42]).unwrap();
        assert!(bundle.contains("x"));
        assert!(!bundle.contains("y"));
        assert_eq!(bundle.get("x").unwrap().data, vec![42]);
        assert!(bundle.get("y").is_none());
    }

    #[test]
    fn test_remove_entry() {
        let mut bundle = AssetBundle::new();
        bundle.add_bytes("keep", vec![1]).unwrap();
        bundle.add_bytes("drop", vec![2]).unwrap();
        assert!(bundle.remove("drop"));
        assert!(!bundle.contains("drop"));
        assert_eq!(bundle.entry_count(), 1);
        assert!(!bundle.remove("drop")); // second remove returns false
    }

    #[test]
    fn test_total_size() {
        let mut bundle = AssetBundle::new();
        bundle.add_bytes("a", vec![0u8; 100]).unwrap();
        bundle.add_bytes("b", vec![0u8; 200]).unwrap();
        assert_eq!(bundle.total_size(), 300);
    }

    // ── I/O roundtrip ────────────────────────────────────────────────────────

    #[test]
    fn test_export_and_load_roundtrip() {
        let path = tmp_path("oxihuman_test_roundtrip.oxb");

        let mut bundle = AssetBundle::new();
        bundle.add_str("hello.txt", "Hello OXB").unwrap();
        bundle
            .add_bytes("data.bin", vec![0xDE, 0xAD, 0xBE, 0xEF])
            .unwrap();
        bundle.add_bytes("empty.bin", vec![]).unwrap();
        export_bundle(&bundle, &path).unwrap();

        let loaded = load_bundle(&path).unwrap();
        assert_eq!(loaded.entry_count(), 3);
        assert_eq!(loaded.get("hello.txt").unwrap().data, b"Hello OXB");
        assert_eq!(
            loaded.get("data.bin").unwrap().data,
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
        assert_eq!(loaded.get("empty.bin").unwrap().data, Vec::<u8>::new());
    }

    #[test]
    fn test_validate_bundle() {
        let path = tmp_path("oxihuman_test_validate.oxb");

        let mut bundle = AssetBundle::new();
        bundle.add_bytes("a", vec![1, 2]).unwrap();
        bundle.add_bytes("b", vec![3, 4, 5]).unwrap();
        export_bundle(&bundle, &path).unwrap();

        let count = validate_bundle(&path).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_validate_bad_magic() {
        let path = tmp_path("oxihuman_test_bad_magic.oxb");
        fs::write(&path, b"NOTOXB1\x00\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
        let result = validate_bundle(&path);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("magic"));
    }

    #[test]
    fn test_extract_bundle() {
        let bundle_path = tmp_path("oxihuman_test_extract.oxb");
        let out_dir = PathBuf::from("/tmp/oxihuman_test_extract_out");

        let mut bundle = AssetBundle::new();
        bundle.add_str("file1.txt", "content one").unwrap();
        bundle.add_str("file2.txt", "content two").unwrap();
        export_bundle(&bundle, &bundle_path).unwrap();

        let names = extract_bundle(&bundle_path, &out_dir).unwrap();
        assert_eq!(names.len(), 2);

        let f1 = fs::read_to_string(out_dir.join("file1.txt")).unwrap();
        let f2 = fs::read_to_string(out_dir.join("file2.txt")).unwrap();
        assert_eq!(f1, "content one");
        assert_eq!(f2, "content two");
    }

    #[test]
    fn test_bundle_from_dir() {
        let dir = PathBuf::from("/tmp/oxihuman_test_bundle_from_dir");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("asset_a.bin"), b"aaa").unwrap();
        fs::write(dir.join("asset_b.bin"), b"bbbb").unwrap();

        let bundle = bundle_from_dir(&dir).unwrap();
        assert_eq!(bundle.entry_count(), 2);
        assert!(bundle.contains("asset_a.bin"));
        assert!(bundle.contains("asset_b.bin"));
        assert_eq!(bundle.get("asset_a.bin").unwrap().data, b"aaa");
    }
}
