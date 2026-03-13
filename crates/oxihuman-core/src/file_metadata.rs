// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! File metadata (size, mtime) reader stub.

/// File metadata record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    pub path: String,
    pub size_bytes: u64,
    pub mtime_secs: i64,
    pub is_readonly: bool,
    pub permissions: u32,
}

impl FileMetadata {
    pub fn new(path: &str, size_bytes: u64, mtime_secs: i64) -> Self {
        FileMetadata {
            path: path.to_string(),
            size_bytes,
            mtime_secs,
            is_readonly: false,
            permissions: 0o644,
        }
    }

    pub fn is_newer_than(&self, other: &FileMetadata) -> bool {
        self.mtime_secs > other.mtime_secs
    }

    pub fn size_kb(&self) -> f64 {
        self.size_bytes as f64 / 1024.0
    }
}

/// Metadata store stub.
pub struct MetadataStore {
    entries: Vec<FileMetadata>,
}

impl MetadataStore {
    pub fn new() -> Self {
        MetadataStore {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, meta: FileMetadata) {
        self.entries.push(meta);
    }

    pub fn get(&self, path: &str) -> Option<&FileMetadata> {
        self.entries.iter().find(|m| m.path == path)
    }

    pub fn remove(&mut self, path: &str) {
        self.entries.retain(|m| m.path != path);
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for MetadataStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Read metadata stub (returns synthetic entry).
pub fn read_metadata_stub(path: &str) -> FileMetadata {
    FileMetadata::new(path, 0, 0)
}

/// Find the largest file in a list.
pub fn largest_file(metas: &[FileMetadata]) -> Option<&FileMetadata> {
    metas.iter().max_by_key(|m| m.size_bytes)
}

/// Find the most recently modified file.
pub fn newest_file(metas: &[FileMetadata]) -> Option<&FileMetadata> {
    metas.iter().max_by_key(|m| m.mtime_secs)
}

/// Filter files larger than `min_bytes`.
pub fn filter_large(metas: &[FileMetadata], min_bytes: u64) -> Vec<&FileMetadata> {
    metas.iter().filter(|m| m.size_bytes >= min_bytes).collect()
}

/// Total size across all entries.
pub fn total_size(metas: &[FileMetadata]) -> u64 {
    metas.iter().map(|m| m.size_bytes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metadata() {
        let m = FileMetadata::new("/tmp/x", 1024, 1000);
        assert_eq!(m.size_bytes, 1024);
        assert_eq!(m.mtime_secs, 1000);
    }

    #[test]
    fn test_is_newer_than() {
        let a = FileMetadata::new("/a", 0, 200);
        let b = FileMetadata::new("/b", 0, 100);
        assert!(a.is_newer_than(&b));
        assert!(!b.is_newer_than(&a));
    }

    #[test]
    fn test_size_kb() {
        let m = FileMetadata::new("/f", 2048, 0);
        assert!((m.size_kb() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_store_insert_get() {
        let mut s = MetadataStore::new();
        s.insert(FileMetadata::new("/a", 10, 0));
        assert!(s.get("/a").is_some());
        assert!(s.get("/b").is_none());
    }

    #[test]
    fn test_store_remove() {
        let mut s = MetadataStore::new();
        s.insert(FileMetadata::new("/x", 5, 0));
        s.remove("/x");
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_largest_file() {
        let metas = vec![
            FileMetadata::new("/a", 100, 0),
            FileMetadata::new("/b", 500, 0),
            FileMetadata::new("/c", 200, 0),
        ];
        let l = largest_file(&metas).expect("should succeed");
        assert_eq!(l.path, "/b");
    }

    #[test]
    fn test_newest_file() {
        let metas = vec![
            FileMetadata::new("/a", 0, 1000),
            FileMetadata::new("/b", 0, 9000),
        ];
        let n = newest_file(&metas).expect("should succeed");
        assert_eq!(n.path, "/b");
    }

    #[test]
    fn test_filter_large() {
        let metas = vec![
            FileMetadata::new("/small", 50, 0),
            FileMetadata::new("/big", 5000, 0),
        ];
        let big = filter_large(&metas, 100);
        assert_eq!(big.len(), 1);
    }

    #[test]
    fn test_total_size() {
        let metas = vec![
            FileMetadata::new("/a", 100, 0),
            FileMetadata::new("/b", 200, 0),
        ];
        assert_eq!(total_size(&metas), 300);
    }

    #[test]
    fn test_read_metadata_stub() {
        let m = read_metadata_stub("/any/path");
        assert_eq!(m.size_bytes, 0);
    }
}
