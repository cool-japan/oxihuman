// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV copy/paste between meshes or UV layers.

/// A clipboard holding copied UV coordinates.
pub struct UvClipboard {
    pub uvs: Vec<[f32; 2]>,
    pub source_label: String,
}

/// Create a new empty UV clipboard.
pub fn new_uv_clipboard() -> UvClipboard {
    UvClipboard {
        uvs: Vec::new(),
        source_label: String::new(),
    }
}

/// Copy UVs from a source buffer into the clipboard.
pub fn copy_uvs(clipboard: &mut UvClipboard, source: &[[f32; 2]], label: &str) {
    clipboard.uvs = source.to_vec();
    clipboard.source_label = label.to_string();
}

/// Paste UVs from the clipboard into the destination buffer.
/// Returns the number of UVs pasted (min of clipboard and destination size).
pub fn paste_uvs(clipboard: &UvClipboard, destination: &mut [[f32; 2]]) -> usize {
    let count = clipboard.uvs.len().min(destination.len());
    destination[..count].copy_from_slice(&clipboard.uvs[..count]);
    count
}

/// Number of UVs currently in the clipboard.
pub fn clipboard_size(clipboard: &UvClipboard) -> usize {
    clipboard.uvs.len()
}

/// Clear the clipboard.
pub fn clear_clipboard(clipboard: &mut UvClipboard) {
    clipboard.uvs.clear();
    clipboard.source_label.clear();
}

/// Check if the clipboard has compatible size for the destination.
pub fn clipboard_fits(clipboard: &UvClipboard, dest_len: usize) -> bool {
    clipboard.uvs.len() == dest_len
}

/// Paste only if sizes match exactly; returns true on success.
pub fn paste_exact(clipboard: &UvClipboard, destination: &mut [[f32; 2]]) -> bool {
    if !clipboard_fits(clipboard, destination.len()) {
        return false;
    }
    destination.copy_from_slice(&clipboard.uvs);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clipboard_empty() {
        let cb = new_uv_clipboard();
        assert_eq!(clipboard_size(&cb), 0 /* empty */);
    }

    #[test]
    fn copy_fills_clipboard() {
        let mut cb = new_uv_clipboard();
        let src = vec![[0.1f32, 0.2], [0.3, 0.4]];
        copy_uvs(&mut cb, &src, "mesh_a");
        assert_eq!(clipboard_size(&cb), 2 /* two UVs */);
        assert_eq!(cb.source_label, "mesh_a" /* label */);
    }

    #[test]
    fn paste_copies_uvs() {
        let mut cb = new_uv_clipboard();
        let src = vec![[0.5f32, 0.6], [0.7, 0.8]];
        copy_uvs(&mut cb, &src, "src");
        let mut dst = vec![[0.0f32, 0.0], [0.0, 0.0]];
        let count = paste_uvs(&cb, &mut dst);
        assert_eq!(count, 2 /* two pasted */);
        assert!((dst[0][0] - 0.5).abs() < 1e-6 /* U correct */);
    }

    #[test]
    fn paste_truncates_to_destination_size() {
        let mut cb = new_uv_clipboard();
        let src = vec![[0.1f32, 0.1], [0.2, 0.2], [0.3, 0.3]];
        copy_uvs(&mut cb, &src, "x");
        let mut dst = vec![[0.0f32, 0.0]];
        let count = paste_uvs(&cb, &mut dst);
        assert_eq!(count, 1 /* only one pasted */);
    }

    #[test]
    fn clear_empties_clipboard() {
        let mut cb = new_uv_clipboard();
        copy_uvs(&mut cb, &[[0.0, 0.0]], "y");
        clear_clipboard(&mut cb);
        assert_eq!(clipboard_size(&cb), 0 /* cleared */);
    }

    #[test]
    fn clipboard_fits_exact() {
        let mut cb = new_uv_clipboard();
        copy_uvs(&mut cb, &[[0.0f32, 0.0], [1.0, 1.0]], "z");
        assert!(clipboard_fits(&cb, 2) /* fits */);
        assert!(!clipboard_fits(&cb, 3) /* too many */);
    }

    #[test]
    fn paste_exact_success() {
        let mut cb = new_uv_clipboard();
        let src = vec![[0.25f32, 0.75]];
        copy_uvs(&mut cb, &src, "exact");
        let mut dst = vec![[0.0f32, 0.0]];
        let ok = paste_exact(&cb, &mut dst);
        assert!(ok /* success */);
        assert!((dst[0][0] - 0.25).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn paste_exact_size_mismatch() {
        let mut cb = new_uv_clipboard();
        copy_uvs(&mut cb, &[[0.0f32, 0.0]], "m");
        let mut dst = vec![[0.0f32, 0.0], [0.0, 0.0]];
        let ok = paste_exact(&cb, &mut dst);
        assert!(!ok /* size mismatch */);
    }

    #[test]
    fn copy_overwrites_previous() {
        let mut cb = new_uv_clipboard();
        copy_uvs(&mut cb, &[[0.0f32, 0.0]], "first");
        copy_uvs(&mut cb, &[[0.1f32, 0.1], [0.2, 0.2]], "second");
        assert_eq!(clipboard_size(&cb), 2 /* updated */);
    }
}
