// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! MXL compressed MusicXML stub export.

/// MXL archive entry (filename + content).
#[derive(Debug, Clone)]
pub struct MxlEntry {
    pub filename: String,
    pub content: Vec<u8>,
}

impl MxlEntry {
    pub fn new(filename: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            content,
        }
    }
}

/// An MXL container (stub: stores entries uncompressed).
#[derive(Debug, Clone, Default)]
pub struct MxlContainer {
    pub entries: Vec<MxlEntry>,
    pub rootfile: String,
}

impl MxlContainer {
    pub fn new(rootfile: impl Into<String>) -> Self {
        Self {
            rootfile: rootfile.into(),
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: MxlEntry) {
        self.entries.push(entry);
    }

    pub fn total_bytes(&self) -> usize {
        self.entries.iter().map(|e| e.content.len()).sum()
    }
}

/// Generate a container.xml manifest for an MXL archive.
pub fn generate_container_xml(rootfile: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <container>\n\
           <rootfiles>\n\
             <rootfile full-path=\"{}\" media-type=\"application/vnd.recordare.musicxml+xml\"/>\n\
           </rootfiles>\n\
         </container>\n",
        rootfile
    )
}

/// Build an MXL container stub from MusicXML content.
pub fn build_mxl_container(musicxml: &str, score_filename: &str) -> MxlContainer {
    /* Stub: store entries without actual ZIP compression */
    let mut container = MxlContainer::new(score_filename);
    let manifest = generate_container_xml(score_filename);
    container.add_entry(MxlEntry::new(
        "META-INF/container.xml",
        manifest.into_bytes(),
    ));
    container.add_entry(MxlEntry::new(score_filename, musicxml.as_bytes().to_vec()));
    container
}

/// Serialize the MXL container to a stub byte stream (not a real ZIP).
pub fn serialize_mxl_stub(container: &MxlContainer) -> Vec<u8> {
    /* Stub format: header + entry list */
    let mut buf = Vec::new();
    buf.extend_from_slice(b"MXL_STUB_V1\n");
    buf.extend_from_slice(&(container.entries.len() as u32).to_le_bytes());
    for entry in &container.entries {
        let name_bytes = entry.filename.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&(entry.content.len() as u32).to_le_bytes());
        buf.extend_from_slice(&entry.content);
    }
    buf
}

/// Validate that a stub byte stream starts with the MXL stub header.
pub fn is_mxl_stub(data: &[u8]) -> bool {
    data.starts_with(b"MXL_STUB_V1")
}

/// Count entries in an MXL container.
pub fn count_mxl_entries(container: &MxlContainer) -> usize {
    container.entries.len()
}

/// Find an entry by filename in the container.
pub fn find_entry<'a>(container: &'a MxlContainer, filename: &str) -> Option<&'a MxlEntry> {
    container.entries.iter().find(|e| e.filename == filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_container_xml() {
        let xml = generate_container_xml("score.xml");
        assert!(xml.contains("rootfile") /* rootfile element */);
        assert!(xml.contains("score.xml") /* filename */);
    }

    #[test]
    fn test_build_mxl_container_entry_count() {
        let container = build_mxl_container("<score-partwise/>", "score.xml");
        assert_eq!(count_mxl_entries(&container), 2 /* manifest + score */);
    }

    #[test]
    fn test_find_entry_manifest() {
        let container = build_mxl_container("<score-partwise/>", "score.xml");
        let entry = find_entry(&container, "META-INF/container.xml");
        assert!(entry.is_some() /* manifest found */);
    }

    #[test]
    fn test_find_entry_score() {
        let container = build_mxl_container("<score-partwise/>", "score.xml");
        let entry = find_entry(&container, "score.xml");
        assert!(entry.is_some() /* score entry found */);
    }

    #[test]
    fn test_serialize_mxl_stub_header() {
        let container = build_mxl_container("<score-partwise/>", "score.xml");
        let bytes = serialize_mxl_stub(&container);
        assert!(is_mxl_stub(&bytes) /* stub header present */);
    }

    #[test]
    fn test_total_bytes_positive() {
        let container = build_mxl_container("<score-partwise/>", "score.xml");
        assert!(container.total_bytes() > 0 /* non-empty content */);
    }

    #[test]
    fn test_find_entry_missing() {
        let container = MxlContainer::new("x.xml");
        assert!(find_entry(&container, "missing.xml").is_none() /* not found */);
    }

    #[test]
    fn test_mxl_container_rootfile() {
        let container = MxlContainer::new("my_score.xml");
        assert_eq!(container.rootfile, "my_score.xml" /* rootfile set */);
    }

    #[test]
    fn test_is_mxl_stub_false() {
        assert!(!is_mxl_stub(b"not an mxl stub") /* invalid header */);
    }
}
