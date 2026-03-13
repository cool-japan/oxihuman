// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ZIP archive writer stub.

/// A pending write entry.
#[derive(Debug, Clone)]
pub struct WriteEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub compression_level: u8,
}

impl WriteEntry {
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        WriteEntry {
            name: name.to_string(),
            data,
            compression_level: 6,
        }
    }

    pub fn with_compression(mut self, level: u8) -> Self {
        self.compression_level = level.min(9);
        self
    }
}

/// Archive writer stub — accumulates entries in memory.
pub struct ArchiveWriter {
    destination: String,
    entries: Vec<WriteEntry>,
    comment: String,
}

impl ArchiveWriter {
    pub fn new(destination: &str) -> Self {
        ArchiveWriter {
            destination: destination.to_string(),
            entries: Vec::new(),
            comment: String::new(),
        }
    }

    pub fn add_entry(&mut self, entry: WriteEntry) {
        self.entries.push(entry);
    }

    pub fn add_bytes(&mut self, name: &str, data: Vec<u8>) {
        self.entries.push(WriteEntry::new(name, data));
    }

    pub fn add_text(&mut self, name: &str, text: &str) {
        self.add_bytes(name, text.as_bytes().to_vec());
    }

    pub fn set_comment(&mut self, comment: &str) {
        self.comment = comment.to_string();
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_uncompressed_size(&self) -> u64 {
        self.entries.iter().map(|e| e.data.len() as u64).sum()
    }

    /// Finalize (stub — returns byte count that would be written).
    pub fn finalize(&self) -> u64 {
        self.total_uncompressed_size()
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn has_entry(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }
}

impl Default for ArchiveWriter {
    fn default() -> Self {
        Self::new("/tmp/out.zip")
    }
}

/// Create a new archive writer.
pub fn new_archive_writer(path: &str) -> ArchiveWriter {
    ArchiveWriter::new(path)
}

/// Write a directory of entries (stub: just adds them all).
pub fn write_entries(writer: &mut ArchiveWriter, entries: Vec<WriteEntry>) {
    for e in entries {
        writer.add_entry(e);
    }
}

/// Build an archive from name/data pairs.
pub fn build_archive(path: &str, items: &[(&str, &[u8])]) -> ArchiveWriter {
    let mut w = new_archive_writer(path);
    for (name, data) in items {
        w.add_bytes(name, data.to_vec());
    }
    w
}

/// Total bytes that would be written.
pub fn estimate_size(writer: &ArchiveWriter) -> u64 {
    writer.total_uncompressed_size()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_writer() {
        let w = new_archive_writer("/tmp/x.zip");
        assert_eq!(w.entry_count(), 0);
        assert_eq!(w.destination(), "/tmp/x.zip");
    }

    #[test]
    fn test_add_bytes() {
        let mut w = new_archive_writer("/tmp/x.zip");
        w.add_bytes("a.txt", b"hello".to_vec());
        assert_eq!(w.entry_count(), 1);
        assert!(w.has_entry("a.txt"));
    }

    #[test]
    fn test_add_text() {
        let mut w = new_archive_writer("/tmp/x.zip");
        w.add_text("note.txt", "text content");
        assert!(w.has_entry("note.txt"));
    }

    #[test]
    fn test_total_uncompressed_size() {
        let mut w = new_archive_writer("/tmp/x.zip");
        w.add_bytes("a", vec![0u8; 100]);
        w.add_bytes("b", vec![0u8; 200]);
        assert_eq!(w.total_uncompressed_size(), 300);
    }

    #[test]
    fn test_finalize() {
        let mut w = new_archive_writer("/tmp/x.zip");
        w.add_bytes("f", vec![0u8; 50]);
        assert_eq!(w.finalize(), 50);
    }

    #[test]
    fn test_set_comment() {
        let mut w = new_archive_writer("/tmp/x.zip");
        w.set_comment("archive comment");
        assert_eq!(w.comment, "archive comment");
    }

    #[test]
    fn test_build_archive() {
        let w = build_archive("/tmp/out.zip", &[("a.txt", b"aa"), ("b.txt", b"bb")]);
        assert_eq!(w.entry_count(), 2);
    }

    #[test]
    fn test_write_entry_compression_level_clamped() {
        let e = WriteEntry::new("f", vec![]).with_compression(20);
        assert_eq!(e.compression_level, 9);
    }

    #[test]
    fn test_estimate_size() {
        let mut w = new_archive_writer("/tmp/x.zip");
        w.add_bytes("x", vec![0u8; 77]);
        assert_eq!(estimate_size(&w), 77);
    }

    #[test]
    fn test_has_entry_false() {
        let w = new_archive_writer("/tmp/x.zip");
        assert!(!w.has_entry("nonexistent"));
    }
}
