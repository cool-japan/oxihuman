#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Diff and patch operations on byte slices.

/// A single patch operation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PatchOp {
    /// Insert bytes at offset.
    Insert { offset: usize, data: Vec<u8> },
    /// Delete `len` bytes at offset.
    Delete { offset: usize, len: usize },
    /// Replace `len` bytes at offset with new data.
    Replace { offset: usize, len: usize, data: Vec<u8> },
}

/// A collection of patch operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffPatch {
    ops: Vec<PatchOp>,
}

#[allow(dead_code)]
pub fn create_patch(old: &[u8], new: &[u8]) -> DiffPatch {
    let mut ops = Vec::new();
    if old == new {
        return DiffPatch { ops };
    }
    // Simple diff: single replace of the whole content if different.
    ops.push(PatchOp::Replace {
        offset: 0,
        len: old.len(),
        data: new.to_vec(),
    });
    DiffPatch { ops }
}

#[allow(dead_code)]
pub fn apply_patch(data: &[u8], patch: &DiffPatch) -> Vec<u8> {
    let mut result = data.to_vec();
    for op in &patch.ops {
        match op {
            PatchOp::Insert { offset, data: d } => {
                let off = (*offset).min(result.len());
                for (i, &b) in d.iter().enumerate() {
                    result.insert(off + i, b);
                }
            }
            PatchOp::Delete { offset, len } => {
                let off = (*offset).min(result.len());
                let end = (off + len).min(result.len());
                result.drain(off..end);
            }
            PatchOp::Replace { offset, len, data: d } => {
                let off = (*offset).min(result.len());
                let end = (off + len).min(result.len());
                result.drain(off..end);
                for (i, &b) in d.iter().enumerate() {
                    result.insert(off + i, b);
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn patch_op_count(patch: &DiffPatch) -> usize {
    patch.ops.len()
}

#[allow(dead_code)]
pub fn patch_to_bytes(patch: &DiffPatch) -> Vec<u8> {
    // Simple serialization: op count + each op.
    let mut out = Vec::new();
    out.extend_from_slice(&(patch.ops.len() as u32).to_le_bytes());
    for op in &patch.ops {
        match op {
            PatchOp::Insert { offset, data } => {
                out.push(0);
                out.extend_from_slice(&(*offset as u32).to_le_bytes());
                out.extend_from_slice(&(data.len() as u32).to_le_bytes());
                out.extend_from_slice(data);
            }
            PatchOp::Delete { offset, len } => {
                out.push(1);
                out.extend_from_slice(&(*offset as u32).to_le_bytes());
                out.extend_from_slice(&(*len as u32).to_le_bytes());
            }
            PatchOp::Replace { offset, len, data } => {
                out.push(2);
                out.extend_from_slice(&(*offset as u32).to_le_bytes());
                out.extend_from_slice(&(*len as u32).to_le_bytes());
                out.extend_from_slice(&(data.len() as u32).to_le_bytes());
                out.extend_from_slice(data);
            }
        }
    }
    out
}

#[allow(dead_code)]
pub fn patch_from_bytes(data: &[u8]) -> Option<DiffPatch> {
    if data.len() < 4 {
        return None;
    }
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut ops = Vec::with_capacity(count);
    let mut pos = 4;
    for _ in 0..count {
        if pos >= data.len() {
            return None;
        }
        let kind = data[pos];
        pos += 1;
        match kind {
            0 => {
                if pos + 8 > data.len() { return None; }
                let offset = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                let dlen = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                if pos + dlen > data.len() { return None; }
                let d = data[pos..pos+dlen].to_vec();
                pos += dlen;
                ops.push(PatchOp::Insert { offset, data: d });
            }
            1 => {
                if pos + 8 > data.len() { return None; }
                let offset = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                let len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                ops.push(PatchOp::Delete { offset, len });
            }
            2 => {
                if pos + 12 > data.len() { return None; }
                let offset = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                let len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                let dlen = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                if pos + dlen > data.len() { return None; }
                let d = data[pos..pos+dlen].to_vec();
                pos += dlen;
                ops.push(PatchOp::Replace { offset, len, data: d });
            }
            _ => return None,
        }
    }
    Some(DiffPatch { ops })
}

#[allow(dead_code)]
pub fn invert_patch(patch: &DiffPatch) -> DiffPatch {
    let mut ops = Vec::new();
    for op in patch.ops.iter().rev() {
        match op {
            PatchOp::Insert { offset, data } => {
                ops.push(PatchOp::Delete { offset: *offset, len: data.len() });
            }
            PatchOp::Delete { offset, len } => {
                ops.push(PatchOp::Insert { offset: *offset, data: vec![0; *len] });
            }
            PatchOp::Replace { offset, len, data } => {
                ops.push(PatchOp::Replace { offset: *offset, len: data.len(), data: vec![0; *len] });
            }
        }
    }
    DiffPatch { ops }
}

#[allow(dead_code)]
pub fn patch_is_empty(patch: &DiffPatch) -> bool {
    patch.ops.is_empty()
}

#[allow(dead_code)]
pub fn patch_summary(patch: &DiffPatch) -> String {
    let inserts = patch.ops.iter().filter(|o| matches!(o, PatchOp::Insert { .. })).count();
    let deletes = patch.ops.iter().filter(|o| matches!(o, PatchOp::Delete { .. })).count();
    let replaces = patch.ops.iter().filter(|o| matches!(o, PatchOp::Replace { .. })).count();
    format!("inserts={}, deletes={}, replaces={}", inserts, deletes, replaces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_patch() {
        let p = create_patch(b"abc", b"abc");
        assert!(patch_is_empty(&p));
    }

    #[test]
    fn test_apply_identity() {
        let p = create_patch(b"abc", b"abc");
        let result = apply_patch(b"abc", &p);
        assert_eq!(result, b"abc");
    }

    #[test]
    fn test_apply_replace() {
        let p = create_patch(b"abc", b"xyz");
        let result = apply_patch(b"abc", &p);
        assert_eq!(result, b"xyz");
    }

    #[test]
    fn test_op_count() {
        let p = create_patch(b"a", b"b");
        assert_eq!(patch_op_count(&p), 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let p = create_patch(b"hello", b"world");
        let bytes = patch_to_bytes(&p);
        let p2 = patch_from_bytes(&bytes).unwrap();
        assert_eq!(patch_op_count(&p2), patch_op_count(&p));
    }

    #[test]
    fn test_invert() {
        let p = create_patch(b"a", b"b");
        let inv = invert_patch(&p);
        assert_eq!(patch_op_count(&inv), 1);
    }

    #[test]
    fn test_summary() {
        let p = create_patch(b"a", b"b");
        let s = patch_summary(&p);
        assert!(s.contains("replaces=1"));
    }

    #[test]
    fn test_from_bytes_invalid() {
        assert!(patch_from_bytes(&[]).is_none());
    }

    #[test]
    fn test_insert_op() {
        let p = DiffPatch {
            ops: vec![PatchOp::Insert { offset: 1, data: vec![b'X'] }],
        };
        let result = apply_patch(b"ab", &p);
        assert_eq!(result, b"aXb");
    }

    #[test]
    fn test_delete_op() {
        let p = DiffPatch {
            ops: vec![PatchOp::Delete { offset: 1, len: 1 }],
        };
        let result = apply_patch(b"abc", &p);
        assert_eq!(result, b"ac");
    }
}
