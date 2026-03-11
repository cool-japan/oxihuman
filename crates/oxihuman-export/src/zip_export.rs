//! ZIP bundle export stub — tracks files to be bundled without actual compression (pure metadata).

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for ZIP bundle export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ZipExportConfig {
    /// Comment embedded in the ZIP central directory (stub only; not written to disk).
    pub comment: String,
    /// Whether to use compression (stub: recorded but not applied).
    pub compress: bool,
    /// Compression level 0–9 (stub: recorded but not applied).
    pub compression_level: u8,
}

/// A single entry tracked by the bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ZipEntry {
    /// Name / path inside the archive.
    pub name: String,
    /// Uncompressed size in bytes.
    pub size_bytes: u64,
    /// Raw data payload.
    pub data: Vec<u8>,
}

/// A collection of entries representing a ZIP bundle (stub — no real compression).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ZipBundle {
    /// Export configuration used when this bundle was created.
    pub config: ZipExportConfig,
    /// Tracked entries.
    pub entries: Vec<ZipEntry>,
}

// ── Construction ──────────────────────────────────────────────────────────────

/// Returns a default [`ZipExportConfig`].
#[allow(dead_code)]
pub fn default_zip_config() -> ZipExportConfig {
    ZipExportConfig {
        comment: String::new(),
        compress: false,
        compression_level: 0,
    }
}

/// Creates a new empty [`ZipBundle`] with the given config.
#[allow(dead_code)]
pub fn new_zip_bundle(cfg: &ZipExportConfig) -> ZipBundle {
    ZipBundle {
        config: cfg.clone(),
        entries: Vec::new(),
    }
}

// ── Mutation ──────────────────────────────────────────────────────────────────

/// Adds an entry to the bundle (replaces an existing entry with the same name).
#[allow(dead_code)]
pub fn zip_add_entry(bundle: &mut ZipBundle, name: &str, size_bytes: u64, data: Vec<u8>) {
    // Replace if already present.
    if let Some(entry) = bundle.entries.iter_mut().find(|e| e.name == name) {
        entry.size_bytes = size_bytes;
        entry.data = data;
        return;
    }
    bundle.entries.push(ZipEntry {
        name: name.to_string(),
        size_bytes,
        data,
    });
}

/// Removes the entry with the given name. Returns `true` if it was found and removed.
#[allow(dead_code)]
pub fn zip_remove_entry(bundle: &mut ZipBundle, name: &str) -> bool {
    let before = bundle.entries.len();
    bundle.entries.retain(|e| e.name != name);
    bundle.entries.len() < before
}

/// Removes all entries from the bundle.
#[allow(dead_code)]
pub fn zip_bundle_clear(bundle: &mut ZipBundle) {
    bundle.entries.clear();
}

// ── Query ─────────────────────────────────────────────────────────────────────

/// Returns the number of entries in the bundle.
#[allow(dead_code)]
pub fn zip_entry_count(bundle: &ZipBundle) -> usize {
    bundle.entries.len()
}

/// Returns the sum of all entry sizes in bytes.
#[allow(dead_code)]
pub fn zip_total_size(bundle: &ZipBundle) -> u64 {
    bundle.entries.iter().map(|e| e.size_bytes).sum()
}

/// Returns a reference to the entry with the given name, if present.
#[allow(dead_code)]
pub fn zip_find_entry<'a>(bundle: &'a ZipBundle, name: &str) -> Option<&'a ZipEntry> {
    bundle.entries.iter().find(|e| e.name == name)
}

// ── Serialisation ─────────────────────────────────────────────────────────────

/// Produces a minimal stub byte sequence representing the bundle (not a real ZIP).
/// Format: 4-byte magic `b"OXZP"` + 4-byte LE entry count + for each entry: 2-byte name length
/// + name bytes + 8-byte LE size + data length as 4-byte LE + data.
#[allow(dead_code)]
pub fn zip_to_bytes_stub(bundle: &ZipBundle) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"OXZP");
    out.extend_from_slice(&(bundle.entries.len() as u32).to_le_bytes());
    for entry in &bundle.entries {
        let name_bytes = entry.name.as_bytes();
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(&entry.size_bytes.to_le_bytes());
        out.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
        out.extend_from_slice(&entry.data);
    }
    out
}

/// Writes the stub byte representation of the bundle to a file at the given path.
#[allow(dead_code)]
pub fn zip_write_to_file(bundle: &ZipBundle, path: &str) -> Result<(), String> {
    use std::io::Write;
    let bytes = zip_to_bytes_stub(bundle);
    let mut file =
        std::fs::File::create(path).map_err(|e| format!("zip_write_to_file: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("zip_write_to_file write: {}", e))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_zip_config();
        assert!(!cfg.compress);
        assert_eq!(cfg.compression_level, 0);
        assert!(cfg.comment.is_empty());
    }

    #[test]
    fn test_new_bundle_empty() {
        let cfg = default_zip_config();
        let bundle = new_zip_bundle(&cfg);
        assert_eq!(zip_entry_count(&bundle), 0);
        assert_eq!(zip_total_size(&bundle), 0);
    }

    #[test]
    fn test_add_entry_and_count() {
        let cfg = default_zip_config();
        let mut bundle = new_zip_bundle(&cfg);
        zip_add_entry(&mut bundle, "mesh.glb", 1024, vec![0u8; 16]);
        assert_eq!(zip_entry_count(&bundle), 1);
        assert_eq!(zip_total_size(&bundle), 1024);
    }

    #[test]
    fn test_add_entry_replaces_existing() {
        let cfg = default_zip_config();
        let mut bundle = new_zip_bundle(&cfg);
        zip_add_entry(&mut bundle, "file.txt", 100, vec![1u8; 4]);
        zip_add_entry(&mut bundle, "file.txt", 200, vec![2u8; 8]);
        assert_eq!(zip_entry_count(&bundle), 1);
        assert_eq!(zip_total_size(&bundle), 200);
    }

    #[test]
    fn test_find_entry() {
        let cfg = default_zip_config();
        let mut bundle = new_zip_bundle(&cfg);
        zip_add_entry(&mut bundle, "model.obj", 512, vec![42u8; 10]);
        let found = zip_find_entry(&bundle, "model.obj");
        assert!(found.is_some());
        assert_eq!(found.unwrap().size_bytes, 512);
    }

    #[test]
    fn test_find_missing_entry() {
        let cfg = default_zip_config();
        let bundle = new_zip_bundle(&cfg);
        assert!(zip_find_entry(&bundle, "nope.glb").is_none());
    }

    #[test]
    fn test_remove_entry() {
        let cfg = default_zip_config();
        let mut bundle = new_zip_bundle(&cfg);
        zip_add_entry(&mut bundle, "a.txt", 10, vec![0u8; 2]);
        zip_add_entry(&mut bundle, "b.txt", 20, vec![0u8; 4]);
        let removed = zip_remove_entry(&mut bundle, "a.txt");
        assert!(removed);
        assert_eq!(zip_entry_count(&bundle), 1);
    }

    #[test]
    fn test_remove_missing_returns_false() {
        let cfg = default_zip_config();
        let mut bundle = new_zip_bundle(&cfg);
        assert!(!zip_remove_entry(&mut bundle, "ghost.txt"));
    }

    #[test]
    fn test_bundle_clear() {
        let cfg = default_zip_config();
        let mut bundle = new_zip_bundle(&cfg);
        zip_add_entry(&mut bundle, "x.txt", 5, vec![0u8]);
        zip_add_entry(&mut bundle, "y.txt", 7, vec![0u8]);
        zip_bundle_clear(&mut bundle);
        assert_eq!(zip_entry_count(&bundle), 0);
    }

    #[test]
    fn test_to_bytes_stub_magic() {
        let cfg = default_zip_config();
        let bundle = new_zip_bundle(&cfg);
        let bytes = zip_to_bytes_stub(&bundle);
        assert_eq!(&bytes[0..4], b"OXZP");
    }
}
