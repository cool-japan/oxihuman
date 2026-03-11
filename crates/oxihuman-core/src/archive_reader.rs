// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ZIP archive reader stub.

/// A file entry inside an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub data: Vec<u8>,
}

impl ArchiveEntry {
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        let len = data.len() as u64;
        ArchiveEntry {
            name: name.to_string(),
            compressed_size: len,
            uncompressed_size: len,
            data,
        }
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            1.0
        } else {
            self.compressed_size as f64 / self.uncompressed_size as f64
        }
    }
}

/// Archive reader stub.
pub struct ArchiveReader {
    entries: Vec<ArchiveEntry>,
    source: String,
}

impl ArchiveReader {
    pub fn new(source: &str) -> Self {
        ArchiveReader {
            entries: Vec::new(),
            source: source.to_string(),
        }
    }

    pub fn load_entry(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    pub fn entry_names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }

    pub fn find_entry(&self, name: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn total_uncompressed(&self) -> u64 {
        self.entries.iter().map(|e| e.uncompressed_size).sum()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Create a reader stub.
pub fn open_archive_stub(path: &str) -> ArchiveReader {
    ArchiveReader::new(path)
}

/// Read bytes from a named entry.
pub fn read_entry_bytes(reader: &ArchiveReader, name: &str) -> Option<Vec<u8>> {
    reader.find_entry(name).map(|e| e.data.clone())
}

/// Read entry as UTF-8 string.
pub fn read_entry_text(reader: &ArchiveReader, name: &str) -> Option<String> {
    let bytes = read_entry_bytes(reader, name)?;
    String::from_utf8(bytes).ok()
}

/// List all entry names containing `pattern`.
pub fn list_matching<'a>(reader: &'a ArchiveReader, pattern: &str) -> Vec<&'a str> {
    reader
        .entries
        .iter()
        .filter(|e| e.name.contains(pattern))
        .map(|e| e.name.as_str())
        .collect()
}

/// Total compressed size of all entries.
pub fn total_compressed(reader: &ArchiveReader) -> u64 {
    reader.entries.iter().map(|e| e.compressed_size).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_archive_stub() {
        let r = open_archive_stub("/tmp/test.zip");
        assert_eq!(r.entry_count(), 0);
        assert_eq!(r.source(), "/tmp/test.zip");
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
        let bytes = read_entry_bytes(&r, "data.bin").unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    #[test]
    fn test_read_entry_text() {
        let mut r = open_archive_stub("/tmp/x.zip");
        r.load_entry(ArchiveEntry::new("note.txt", b"hello world".to_vec()));
        let text = read_entry_text(&r, "note.txt").unwrap();
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
}
