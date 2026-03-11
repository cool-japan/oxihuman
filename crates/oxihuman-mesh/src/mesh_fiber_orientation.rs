// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct FiberField {
    pub directions: Vec<[f32; 3]>,
    pub coherence: Vec<f32>,
}

pub fn new_fiber_field(n: usize) -> FiberField {
    FiberField {
        directions: vec![[1.0, 0.0, 0.0]; n],
        coherence: vec![1.0; n],
    }
}

pub fn fiber_set(f: &mut FiberField, i: usize, dir: [f32; 3], coherence: f32) {
    f.directions[i] = dir;
    f.coherence[i] = coherence;
}

pub fn fiber_get(f: &FiberField, i: usize) -> ([f32; 3], f32) {
    (f.directions[i], f.coherence[i])
}

pub fn fiber_mean_coherence(f: &FiberField) -> f32 {
    if f.coherence.is_empty() {
        return 0.0;
    }
    f.coherence.iter().sum::<f32>() / f.coherence.len() as f32
}

pub fn fiber_anisotropy_index(f: &FiberField) -> f32 {
    if f.coherence.is_empty() {
        return 0.0;
    }
    let mean = fiber_mean_coherence(f);
    let variance =
        f.coherence.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / f.coherence.len() as f32;
    variance.sqrt()
}

pub fn fiber_count(f: &FiberField) -> usize {
    f.directions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fiber_field() {
        /* basic construction */
        let f = new_fiber_field(5);
        assert_eq!(fiber_count(&f), 5);
    }

    #[test]
    fn test_fiber_set_get() {
        /* set and retrieve */
        let mut f = new_fiber_field(3);
        fiber_set(&mut f, 1, [0.0, 1.0, 0.0], 0.5);
        let (dir, coh) = fiber_get(&f, 1);
        assert_eq!(dir, [0.0, 1.0, 0.0]);
        assert!((coh - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_fiber_mean_coherence() {
        /* mean of uniform values */
        let mut f = new_fiber_field(4);
        for i in 0..4 {
            fiber_set(&mut f, i, [1.0, 0.0, 0.0], 0.5);
        }
        assert!((fiber_mean_coherence(&f) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_fiber_anisotropy_zero() {
        /* uniform coherence => zero std dev */
        let f = new_fiber_field(4);
        assert!(fiber_anisotropy_index(&f) < 1e-6);
    }

    #[test]
    fn test_fiber_anisotropy_nonzero() {
        /* mixed coherence */
        let mut f = new_fiber_field(2);
        fiber_set(&mut f, 0, [1.0, 0.0, 0.0], 0.0);
        fiber_set(&mut f, 1, [1.0, 0.0, 0.0], 1.0);
        assert!(fiber_anisotropy_index(&f) > 0.0);
    }

    #[test]
    fn test_fiber_count_empty() {
        /* empty field */
        let f = new_fiber_field(0);
        assert_eq!(fiber_count(&f), 0);
        assert!((fiber_mean_coherence(&f)).abs() < 1e-6);
    }
}
