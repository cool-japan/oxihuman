// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ICO icon stub export.

/// A single ICO image entry.
#[derive(Debug, Clone)]
pub struct IcoEntry {
    pub size: u32,
    pub pixels: Vec<[u8; 4]>,
}

impl IcoEntry {
    /// Create a new ICO entry filled with solid color.
    pub fn new_solid(size: u32, color: [u8; 4]) -> Self {
        let pixels = vec![color; (size * size) as usize];
        Self { size, pixels }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }

    /// Check if size is a standard ICO size.
    pub fn is_standard_size(&self) -> bool {
        matches!(self.size, 16 | 24 | 32 | 48 | 64 | 128 | 256)
    }
}

/// ICO file stub.
#[derive(Debug, Clone)]
pub struct IcoExport {
    pub entries: Vec<IcoEntry>,
}

impl IcoExport {
    /// Create an empty ICO export.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry.
    pub fn add_entry(&mut self, entry: IcoEntry) {
        self.entries.push(entry);
    }

    /// Return entry count.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Largest icon size present.
    pub fn max_size(&self) -> u32 {
        self.entries.iter().map(|e| e.size).max().unwrap_or(0)
    }
}

impl Default for IcoExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate ICO export (all entries must be square with matching pixel count).
pub fn validate_ico(ico: &IcoExport) -> bool {
    ico.entries
        .iter()
        .all(|e| e.pixel_count() == (e.size * e.size) as usize && e.size > 0)
}

/// Estimate ICO file size (stub).
pub fn estimate_ico_bytes(ico: &IcoExport) -> usize {
    let header = 6 + ico.entries.len() * 16;
    let pixel_data: usize = ico.entries.iter().map(|e| e.pixel_count() * 4 + 40).sum();
    header + pixel_data
}

/// Serialize ICO metadata to JSON (stub).
pub fn ico_metadata_json(ico: &IcoExport) -> String {
    let sizes: Vec<String> = ico.entries.iter().map(|e| e.size.to_string()).collect();
    format!("{{\"entries\":[{}]}}", sizes.join(","))
}

/// Find entry with the given size.
pub fn find_ico_entry(ico: &IcoExport, size: u32) -> Option<&IcoEntry> {
    ico.entries.iter().find(|e| e.size == size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ico() -> IcoExport {
        let mut ico = IcoExport::new();
        ico.add_entry(IcoEntry::new_solid(16, [255, 0, 0, 255]));
        ico.add_entry(IcoEntry::new_solid(32, [0, 255, 0, 255]));
        ico.add_entry(IcoEntry::new_solid(256, [0, 0, 255, 255]));
        ico
    }

    #[test]
    fn test_entry_count() {
        /* entry count is correct */
        assert_eq!(sample_ico().entry_count(), 3);
    }

    #[test]
    fn test_max_size() {
        /* max size is the largest entry */
        assert_eq!(sample_ico().max_size(), 256);
    }

    #[test]
    fn test_validate_valid() {
        /* valid ICO passes validation */
        assert!(validate_ico(&sample_ico()));
    }

    #[test]
    fn test_is_standard_size() {
        /* standard sizes are recognized */
        let e = IcoEntry::new_solid(32, [0; 4]);
        assert!(e.is_standard_size());
        let e2 = IcoEntry::new_solid(100, [0; 4]);
        assert!(!e2.is_standard_size());
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* size estimate is positive */
        assert!(estimate_ico_bytes(&sample_ico()) > 0);
    }

    #[test]
    fn test_metadata_json() {
        /* metadata JSON contains size values */
        let json = ico_metadata_json(&sample_ico());
        assert!(json.contains("256"));
    }

    #[test]
    fn test_find_ico_entry() {
        /* find entry by size works */
        let ico = sample_ico();
        assert!(find_ico_entry(&ico, 32).is_some());
        assert!(find_ico_entry(&ico, 64).is_none());
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count is size^2 */
        let e = IcoEntry::new_solid(16, [0; 4]);
        assert_eq!(e.pixel_count(), 256);
    }
}
