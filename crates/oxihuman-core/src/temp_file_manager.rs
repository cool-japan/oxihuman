// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Temporary file lifecycle manager.

/// A temp file handle (stub — no actual OS file).
#[derive(Debug, Clone)]
pub struct TempFile {
    pub path: String,
    pub size_bytes: u64,
    pub deleted: bool,
}

impl TempFile {
    pub fn new(path: &str) -> Self {
        TempFile {
            path: path.to_string(),
            size_bytes: 0,
            deleted: false,
        }
    }

    pub fn write_bytes(&mut self, bytes: u64) {
        self.size_bytes += bytes;
    }

    pub fn delete(&mut self) {
        self.deleted = true;
        self.size_bytes = 0;
    }

    pub fn is_alive(&self) -> bool {
        !self.deleted
    }
}

/// Temporary file manager.
pub struct TempFileManager {
    prefix: String,
    counter: u64,
    files: Vec<TempFile>,
}

impl TempFileManager {
    pub fn new(prefix: &str) -> Self {
        TempFileManager {
            prefix: prefix.to_string(),
            counter: 0,
            files: Vec::new(),
        }
    }

    pub fn create(&mut self) -> &mut TempFile {
        let path = format!("{}{}.tmp", self.prefix, self.counter);
        self.counter += 1;
        self.files.push(TempFile::new(&path));
        let len = self.files.len();
        &mut self.files[len - 1]
    }

    pub fn cleanup_all(&mut self) {
        for f in &mut self.files {
            f.delete();
        }
    }

    pub fn alive_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_alive()).count()
    }

    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).sum()
    }

    pub fn all_files(&self) -> &[TempFile] {
        &self.files
    }
}

impl Default for TempFileManager {
    fn default() -> Self {
        Self::new("/tmp/oxihuman_")
    }
}

/// Create a default temp file manager.
pub fn new_temp_manager() -> TempFileManager {
    TempFileManager::default()
}

/// Create N temp files and return their paths.
pub fn create_n(mgr: &mut TempFileManager, n: usize) -> Vec<String> {
    (0..n).map(|_| mgr.create().path.clone()).collect()
}

/// Total bytes across all alive temp files.
pub fn alive_total_bytes(mgr: &TempFileManager) -> u64 {
    mgr.files
        .iter()
        .filter(|f| f.is_alive())
        .map(|f| f.size_bytes)
        .sum()
}

/// Delete a temp file by path.
pub fn delete_by_path(mgr: &mut TempFileManager, path: &str) {
    if let Some(f) = mgr.files.iter_mut().find(|f| f.path == path) {
        f.delete();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_file() {
        let mut m = new_temp_manager();
        let f = m.create();
        assert!(f.is_alive());
    }

    #[test]
    fn test_alive_count() {
        let mut m = new_temp_manager();
        create_n(&mut m, 3);
        assert_eq!(m.alive_count(), 3);
    }

    #[test]
    fn test_cleanup_all() {
        let mut m = new_temp_manager();
        create_n(&mut m, 2);
        m.cleanup_all();
        assert_eq!(m.alive_count(), 0);
    }

    #[test]
    fn test_write_bytes() {
        let mut m = new_temp_manager();
        m.create().write_bytes(512);
        assert_eq!(m.total_size(), 512);
    }

    #[test]
    fn test_alive_total_bytes() {
        let mut m = new_temp_manager();
        m.create().write_bytes(100);
        assert_eq!(alive_total_bytes(&m), 100);
    }

    #[test]
    fn test_delete_by_path() {
        let mut m = new_temp_manager();
        let paths = create_n(&mut m, 2);
        delete_by_path(&mut m, &paths[0]);
        assert_eq!(m.alive_count(), 1);
    }

    #[test]
    fn test_create_n_paths_unique() {
        let mut m = new_temp_manager();
        let paths = create_n(&mut m, 3);
        let unique: std::collections::HashSet<_> = paths.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_total_size_after_delete() {
        let mut m = new_temp_manager();
        m.create().write_bytes(200);
        m.cleanup_all();
        assert_eq!(m.total_size(), 0);
    }

    #[test]
    fn test_all_files_accessible() {
        let mut m = new_temp_manager();
        create_n(&mut m, 2);
        assert_eq!(m.all_files().len(), 2);
    }

    #[test]
    fn test_default_prefix() {
        let m = TempFileManager::default();
        assert!(m.prefix.contains("oxihuman"));
    }
}
