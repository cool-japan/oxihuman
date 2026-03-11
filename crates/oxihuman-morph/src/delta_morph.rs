// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! DeltaMorph — per-vertex position delta morphing.

#![allow(dead_code)]

/// A per-vertex position delta (dx, dy, dz).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeltaVert {
    pub index: usize,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

/// A named collection of vertex deltas forming one morph target.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DeltaMorph {
    pub name: String,
    pub deltas: Vec<DeltaVert>,
}

/// Create an empty `DeltaMorph` with the given name.
#[allow(dead_code)]
pub fn new_delta_morph(name: &str) -> DeltaMorph {
    DeltaMorph { name: name.to_owned(), deltas: Vec::new() }
}

/// Append a vertex delta to the morph.
#[allow(dead_code)]
pub fn add_delta(morph: &mut DeltaMorph, index: usize, dx: f32, dy: f32, dz: f32) {
    morph.deltas.push(DeltaVert { index, dx, dy, dz });
}

/// Apply morph deltas at `weight` into `positions` (flat [x,y,z,...] array).
#[allow(dead_code)]
pub fn apply_delta_morph(morph: &DeltaMorph, positions: &mut [f32], weight: f32) {
    for d in &morph.deltas {
        let base = d.index * 3;
        if base + 2 < positions.len() {
            positions[base] += d.dx * weight;
            positions[base + 1] += d.dy * weight;
            positions[base + 2] += d.dz * weight;
        }
    }
}

/// Return the maximum Euclidean delta magnitude across all entries.
#[allow(dead_code)]
pub fn delta_magnitude(morph: &DeltaMorph) -> f32 {
    morph
        .deltas
        .iter()
        .map(|d| (d.dx * d.dx + d.dy * d.dy + d.dz * d.dz).sqrt())
        .fold(0.0_f32, f32::max)
}

/// Return a copy of the morph with all deltas multiplied by `scale`.
#[allow(dead_code)]
pub fn scale_delta(morph: &DeltaMorph, scale: f32) -> DeltaMorph {
    DeltaMorph {
        name: morph.name.clone(),
        deltas: morph
            .deltas
            .iter()
            .map(|d| DeltaVert { index: d.index, dx: d.dx * scale, dy: d.dy * scale, dz: d.dz * scale })
            .collect(),
    }
}

/// Linearly blend two morph targets at `t` (0 = a, 1 = b).
/// Both morphs must have the same number of deltas.
#[allow(dead_code)]
pub fn blend_deltas(a: &DeltaMorph, b: &DeltaMorph, t: f32) -> DeltaMorph {
    let n = a.deltas.len().min(b.deltas.len());
    let deltas = (0..n)
        .map(|i| {
            let da = a.deltas[i];
            let db = b.deltas[i];
            DeltaVert {
                index: da.index,
                dx: da.dx + (db.dx - da.dx) * t,
                dy: da.dy + (db.dy - da.dy) * t,
                dz: da.dz + (db.dz - da.dz) * t,
            }
        })
        .collect();
    DeltaMorph { name: format!("blend_{}_{}", a.name, b.name), deltas }
}

/// Return the number of delta entries in the morph.
#[allow(dead_code)]
pub fn delta_count(morph: &DeltaMorph) -> usize {
    morph.deltas.len()
}

/// Return the delta at `index`, if present.
#[allow(dead_code)]
pub fn delta_at(morph: &DeltaMorph, index: usize) -> Option<DeltaVert> {
    morph.deltas.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_delta_morph_empty() {
        let m = new_delta_morph("smile");
        assert_eq!(delta_count(&m), 0);
        assert_eq!(m.name, "smile");
    }

    #[test]
    fn test_add_delta_and_count() {
        let mut m = new_delta_morph("test");
        add_delta(&mut m, 0, 1.0, 0.0, 0.0);
        add_delta(&mut m, 1, 0.0, 1.0, 0.0);
        assert_eq!(delta_count(&m), 2);
    }

    #[test]
    fn test_delta_at_some() {
        let mut m = new_delta_morph("t");
        add_delta(&mut m, 3, 0.1, 0.2, 0.3);
        let d = delta_at(&m, 0).unwrap();
        assert_eq!(d.index, 3);
        assert!((d.dx - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_delta_at_none() {
        let m = new_delta_morph("t");
        assert!(delta_at(&m, 0).is_none());
    }

    #[test]
    fn test_apply_delta_morph() {
        let mut m = new_delta_morph("x");
        add_delta(&mut m, 0, 1.0, 0.0, 0.0);
        let mut pos = vec![0.0_f32; 3];
        apply_delta_morph(&m, &mut pos, 0.5);
        assert!((pos[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_delta_magnitude_single() {
        let mut m = new_delta_morph("x");
        add_delta(&mut m, 0, 3.0, 4.0, 0.0);
        assert!((delta_magnitude(&m) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_delta() {
        let mut m = new_delta_morph("x");
        add_delta(&mut m, 0, 1.0, 2.0, 3.0);
        let s = scale_delta(&m, 2.0);
        let d = delta_at(&s, 0).unwrap();
        assert!((d.dx - 2.0).abs() < 1e-6);
        assert!((d.dz - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_deltas_midpoint() {
        let mut a = new_delta_morph("a");
        add_delta(&mut a, 0, 0.0, 0.0, 0.0);
        let mut b = new_delta_morph("b");
        add_delta(&mut b, 0, 2.0, 0.0, 0.0);
        let c = blend_deltas(&a, &b, 0.5);
        let d = delta_at(&c, 0).unwrap();
        assert!((d.dx - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_deltas_name() {
        let a = new_delta_morph("a");
        let b = new_delta_morph("b");
        let c = blend_deltas(&a, &b, 0.0);
        assert!(c.name.contains("a"));
        assert!(c.name.contains("b"));
    }
}
