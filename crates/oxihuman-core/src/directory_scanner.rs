// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Recursive directory scanner stub.

/// A directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

impl DirEntry {
    pub fn file(path: &str, size_bytes: u64) -> Self {
        DirEntry {
            path: path.to_string(),
            is_dir: false,
            size_bytes,
        }
    }

    pub fn dir(path: &str) -> Self {
        DirEntry {
            path: path.to_string(),
            is_dir: true,
            size_bytes: 0,
        }
    }
}

/// Scanner configuration.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub max_depth: usize,
    pub follow_symlinks: bool,
    pub include_hidden: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        ScanConfig {
            max_depth: 64,
            follow_symlinks: false,
            include_hidden: false,
        }
    }
}

/// Directory scanner stub — holds a virtual file tree.
pub struct DirectoryScanner {
    pub config: ScanConfig,
    virtual_tree: Vec<DirEntry>,
}

impl DirectoryScanner {
    pub fn new(config: ScanConfig) -> Self {
        DirectoryScanner {
            config,
            virtual_tree: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: DirEntry) {
        self.virtual_tree.push(entry);
    }

    pub fn scan(&self, _root: &str) -> Vec<DirEntry> {
        self.virtual_tree.clone()
    }

    pub fn file_count(&self) -> usize {
        self.virtual_tree.iter().filter(|e| !e.is_dir).count()
    }

    pub fn dir_count(&self) -> usize {
        self.virtual_tree.iter().filter(|e| e.is_dir).count()
    }
}

impl Default for DirectoryScanner {
    fn default() -> Self {
        Self::new(ScanConfig::default())
    }
}

/// Create a default scanner.
pub fn new_scanner() -> DirectoryScanner {
    DirectoryScanner::default()
}

/// Filter entries by extension.
pub fn filter_by_ext<'a>(entries: &'a [DirEntry], ext: &str) -> Vec<&'a DirEntry> {
    entries.iter().filter(|e| e.path.ends_with(ext)).collect()
}

/// Total size of all file entries.
pub fn total_size(entries: &[DirEntry]) -> u64 {
    entries.iter().map(|e| e.size_bytes).sum()
}

/// Find entries whose path contains `needle`.
pub fn find_by_name<'a>(entries: &'a [DirEntry], needle: &str) -> Vec<&'a DirEntry> {
    entries.iter().filter(|e| e.path.contains(needle)).collect()
}

/// Sort entries by path.
pub fn sort_entries(entries: &mut [DirEntry]) {
    entries.sort_by(|a, b| a.path.cmp(&b.path));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scanner_empty() {
        let s = new_scanner();
        assert_eq!(s.file_count(), 0);
        assert_eq!(s.dir_count(), 0);
    }

    #[test]
    fn test_add_and_scan() {
        let mut s = new_scanner();
        s.add_entry(DirEntry::file("/tmp/a.txt", 100));
        let entries = s.scan("/tmp");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_file_dir_count() {
        let mut s = new_scanner();
        s.add_entry(DirEntry::file("/tmp/x", 50));
        s.add_entry(DirEntry::dir("/tmp/subdir"));
        assert_eq!(s.file_count(), 1);
        assert_eq!(s.dir_count(), 1);
    }

    #[test]
    fn test_filter_by_ext() {
        let entries = vec![DirEntry::file("/a.rs", 10), DirEntry::file("/b.txt", 20)];
        let rs = filter_by_ext(&entries, ".rs");
        assert_eq!(rs.len(), 1);
    }

    #[test]
    fn test_total_size() {
        let entries = vec![DirEntry::file("/a", 100), DirEntry::file("/b", 200)];
        assert_eq!(total_size(&entries), 300);
    }

    #[test]
    fn test_find_by_name() {
        let entries = vec![
            DirEntry::file("/foo/bar.rs", 10),
            DirEntry::file("/baz/qux.rs", 20),
        ];
        let found = find_by_name(&entries, "foo");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_sort_entries() {
        let mut entries = vec![DirEntry::file("/z.txt", 1), DirEntry::file("/a.txt", 1)];
        sort_entries(&mut entries);
        assert_eq!(entries[0].path, "/a.txt");
    }

    #[test]
    fn test_scan_config_default() {
        let cfg = ScanConfig::default();
        assert!(!cfg.follow_symlinks);
        assert_eq!(cfg.max_depth, 64);
    }

    #[test]
    fn test_dir_entry_is_dir_flag() {
        let d = DirEntry::dir("/mydir");
        assert!(d.is_dir);
        let f = DirEntry::file("/myfile", 0);
        assert!(!f.is_dir);
    }

    #[test]
    fn test_total_size_dirs_excluded() {
        let entries = vec![DirEntry::dir("/subdir"), DirEntry::file("/file", 77)];
        /* dirs contribute 0 to size */
        assert_eq!(total_size(&entries), 77);
    }
}
