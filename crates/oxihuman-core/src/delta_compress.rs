// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Delta compression for sequences of integer values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeltaCompress {
    base: i64,
    deltas: Vec<i64>,
}

#[allow(dead_code)]
impl DeltaCompress {
    pub fn new() -> Self {
        Self {
            base: 0,
            deltas: Vec::new(),
        }
    }

    pub fn encode(values: &[i64]) -> Self {
        if values.is_empty() {
            return Self::new();
        }
        let base = values[0];
        let mut deltas = Vec::with_capacity(values.len().saturating_sub(1));
        for i in 1..values.len() {
            deltas.push(values[i] - values[i - 1]);
        }
        Self { base, deltas }
    }

    pub fn decode(&self) -> Vec<i64> {
        let mut result = Vec::with_capacity(self.deltas.len() + 1);
        result.push(self.base);
        let mut current = self.base;
        for &d in &self.deltas {
            current += d;
            result.push(current);
        }
        result
    }

    pub fn base(&self) -> i64 {
        self.base
    }

    pub fn num_deltas(&self) -> usize {
        self.deltas.len()
    }

    pub fn original_len(&self) -> usize {
        if self.deltas.is_empty() && self.base == 0 {
            0
        } else {
            self.deltas.len() + 1
        }
    }

    pub fn max_delta(&self) -> Option<i64> {
        self.deltas.iter().copied().max()
    }

    pub fn min_delta(&self) -> Option<i64> {
        self.deltas.iter().copied().min()
    }

    pub fn compression_ratio(&self) -> f64 {
        let orig = self.original_len() as f64;
        if orig < 1e-12 {
            return 0.0;
        }
        let compressed = 1.0 + self.deltas.len() as f64;
        compressed / orig
    }

    pub fn encode_f32(values: &[f32], scale: f32) -> Self {
        let ints: Vec<i64> = values.iter().map(|&v| (v * scale) as i64).collect();
        Self::encode(&ints)
    }

    pub fn decode_f32(&self, scale: f32) -> Vec<f32> {
        self.decode().iter().map(|&v| v as f32 / scale).collect()
    }
}

impl Default for DeltaCompress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let dc = DeltaCompress::encode(&[]);
        assert_eq!(dc.original_len(), 0);
    }

    #[test]
    fn test_single() {
        let dc = DeltaCompress::encode(&[42]);
        assert_eq!(dc.base(), 42);
        assert_eq!(dc.num_deltas(), 0);
        assert_eq!(dc.decode(), vec![42]);
    }

    #[test]
    fn test_roundtrip() {
        let vals = vec![10, 20, 25, 30, 50];
        let dc = DeltaCompress::encode(&vals);
        assert_eq!(dc.decode(), vals);
    }

    #[test]
    fn test_negative_deltas() {
        let vals = vec![100, 90, 80, 70];
        let dc = DeltaCompress::encode(&vals);
        assert_eq!(dc.decode(), vals);
        assert_eq!(dc.min_delta(), Some(-10));
    }

    #[test]
    fn test_max_delta() {
        let vals = vec![0, 100, 101];
        let dc = DeltaCompress::encode(&vals);
        assert_eq!(dc.max_delta(), Some(100));
    }

    #[test]
    fn test_compression_ratio() {
        let vals = vec![1, 2, 3, 4, 5];
        let dc = DeltaCompress::encode(&vals);
        let ratio = dc.compression_ratio();
        assert!((ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_f32_roundtrip() {
        let vals = vec![1.0f32, 2.0, 3.0];
        let dc = DeltaCompress::encode_f32(&vals, 1000.0);
        let decoded = dc.decode_f32(1000.0);
        for (a, b) in vals.iter().zip(decoded.iter()) {
            assert!((a - b).abs() < 0.01);
        }
    }

    #[test]
    fn test_original_len() {
        let vals = vec![1, 2, 3];
        let dc = DeltaCompress::encode(&vals);
        assert_eq!(dc.original_len(), 3);
    }

    #[test]
    fn test_constant_sequence() {
        let vals = vec![5, 5, 5, 5];
        let dc = DeltaCompress::encode(&vals);
        assert_eq!(dc.max_delta(), Some(0));
        assert_eq!(dc.decode(), vals);
    }

    #[test]
    fn test_default() {
        let dc = DeltaCompress::default();
        assert_eq!(dc.original_len(), 0);
    }
}
