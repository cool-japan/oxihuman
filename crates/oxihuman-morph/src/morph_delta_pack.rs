#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A single sparse morph delta entry.
#[derive(Debug, Clone)]
pub struct DeltaEntry {
    pub vert_idx: u32,
    pub offset: [f32; 3],
}

/// Sparse morph delta storage.
#[derive(Debug, Clone)]
pub struct MorphDeltaPack {
    pub deltas: Vec<DeltaEntry>,
    pub threshold: f32,
}

#[allow(dead_code)]
pub fn new_morph_delta_pack(threshold: f32) -> MorphDeltaPack {
    MorphDeltaPack {
        deltas: Vec::new(),
        threshold,
    }
}

#[allow(dead_code)]
pub fn pack_deltas(offsets: &[[f32; 3]], threshold: f32) -> MorphDeltaPack {
    let deltas: Vec<DeltaEntry> = offsets
        .iter()
        .enumerate()
        .filter_map(|(i, &o)| {
            let mag = (o[0] * o[0] + o[1] * o[1] + o[2] * o[2]).sqrt();
            if mag > threshold {
                Some(DeltaEntry {
                    vert_idx: i as u32,
                    offset: o,
                })
            } else {
                None
            }
        })
        .collect();
    MorphDeltaPack { deltas, threshold }
}

#[allow(dead_code)]
pub fn unpack_deltas(pack: &MorphDeltaPack, n_verts: usize) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0f32; 3]; n_verts];
    for entry in &pack.deltas {
        let idx = entry.vert_idx as usize;
        if idx < n_verts {
            out[idx] = entry.offset;
        }
    }
    out
}

#[allow(dead_code)]
pub fn delta_count(pack: &MorphDeltaPack) -> usize {
    pack.deltas.len()
}

#[allow(dead_code)]
pub fn compression_ratio_pack(pack: &MorphDeltaPack, n_verts: usize) -> f32 {
    if n_verts == 0 {
        return 1.0;
    }
    1.0 - (pack.deltas.len() as f32 / n_verts as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pack_empty() {
        let p = new_morph_delta_pack(0.01);
        assert!(p.deltas.is_empty());
        assert!((p.threshold - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_pack_deltas_all_zero() {
        let offsets = vec![[0.0f32; 3]; 10];
        let p = pack_deltas(&offsets, 0.001);
        assert_eq!(delta_count(&p), 0);
    }

    #[test]
    fn test_pack_deltas_some_nonzero() {
        let offsets = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let p = pack_deltas(&offsets, 0.001);
        assert_eq!(delta_count(&p), 1);
    }

    #[test]
    fn test_unpack_round_trip() {
        let offsets = vec![[0.0f32; 3], [1.0, 2.0, 3.0], [0.0, 0.0, 0.0]];
        let p = pack_deltas(&offsets, 0.001);
        let out = unpack_deltas(&p, 3);
        assert!((out[1][0] - 1.0).abs() < 1e-6);
        assert!((out[0][0]).abs() < 1e-6);
    }

    #[test]
    fn test_compression_ratio_all_zero() {
        let offsets = vec![[0.0f32; 3]; 100];
        let p = pack_deltas(&offsets, 0.001);
        let ratio = compression_ratio_pack(&p, 100);
        assert!((ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compression_ratio_all_nonzero() {
        let offsets = vec![[1.0f32, 0.0, 0.0]; 10];
        let p = pack_deltas(&offsets, 0.001);
        let ratio = compression_ratio_pack(&p, 10);
        assert!((ratio).abs() < 1e-6); // all kept
    }

    #[test]
    fn test_compression_ratio_zero_verts() {
        let p = new_morph_delta_pack(0.001);
        assert!((compression_ratio_pack(&p, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_delta_entry_idx() {
        let offsets = vec![[0.0f32; 3], [5.0, 0.0, 0.0]];
        let p = pack_deltas(&offsets, 0.001);
        assert_eq!(p.deltas[0].vert_idx, 1);
    }

    #[test]
    fn test_unpack_out_of_range_ignored() {
        let mut p = new_morph_delta_pack(0.0);
        p.deltas.push(crate::morph_delta_pack::DeltaEntry {
            vert_idx: 999,
            offset: [1.0, 0.0, 0.0],
        });
        let out = unpack_deltas(&p, 3);
        assert_eq!(out.len(), 3);
        // idx 999 is out of range, should be ignored
        assert!((out[0][0]).abs() < 1e-6);
    }
}
