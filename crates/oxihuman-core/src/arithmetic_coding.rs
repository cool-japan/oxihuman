// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Arithmetic coding stub (encode/decode with probability model).

#![allow(dead_code)]

use std::collections::HashMap;

/// Symbol probability model.
#[allow(dead_code)]
pub struct ProbModel {
    pub symbols: Vec<u8>,
    pub cumulative: Vec<f64>,
    pub total: u64,
}

impl ProbModel {
    /// Build from frequency map.
    #[allow(dead_code)]
    pub fn from_freq(freq: &HashMap<u8, u64>) -> Self {
        let mut pairs: Vec<(u8, u64)> = freq.iter().map(|(&k, &v)| (k, v)).collect();
        pairs.sort_by_key(|&(k, _)| k);
        let total: u64 = pairs.iter().map(|&(_, f)| f).sum();
        let mut cumulative = vec![0.0f64];
        for (_, f) in &pairs {
            let prev = cumulative.last().copied().unwrap_or(0.0);
            cumulative.push(prev + *f as f64 / total as f64);
        }
        Self {
            symbols: pairs.iter().map(|&(k, _)| k).collect(),
            cumulative,
            total,
        }
    }

    /// Get [low, high) for symbol.
    #[allow(dead_code)]
    pub fn range(&self, sym: u8) -> Option<(f64, f64)> {
        let idx = self.symbols.iter().position(|&s| s == sym)?;
        Some((self.cumulative[idx], self.cumulative[idx + 1]))
    }

    /// Decode symbol from value in [0,1).
    #[allow(dead_code)]
    pub fn decode_symbol(&self, value: f64) -> Option<u8> {
        for (i, sym) in self.symbols.iter().enumerate() {
            if value >= self.cumulative[i] && value < self.cumulative[i + 1] {
                return Some(*sym);
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}

/// Arithmetic encoder state.
#[allow(dead_code)]
pub struct ArithEncoder {
    low: f64,
    high: f64,
}

impl ArithEncoder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { low: 0.0, high: 1.0 }
    }

    #[allow(dead_code)]
    pub fn encode_symbol(&mut self, model: &ProbModel, sym: u8) -> bool {
        let range = self.high - self.low;
        if let Some((sym_low, sym_high)) = model.range(sym) {
            self.high = self.low + range * sym_high;
            self.low += range * sym_low;
            true
        } else {
            false
        }
    }

    /// Finalize: return midpoint of current interval.
    #[allow(dead_code)]
    pub fn finalize(&self) -> f64 {
        (self.low + self.high) / 2.0
    }

    #[allow(dead_code)]
    pub fn interval(&self) -> (f64, f64) {
        (self.low, self.high)
    }
}

impl Default for ArithEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Arithmetic decoder.
#[allow(dead_code)]
pub struct ArithDecoder {
    value: f64,
    low: f64,
    high: f64,
}

impl ArithDecoder {
    #[allow(dead_code)]
    pub fn new(encoded: f64) -> Self {
        Self { value: encoded, low: 0.0, high: 1.0 }
    }

    #[allow(dead_code)]
    pub fn decode_symbol(&mut self, model: &ProbModel) -> Option<u8> {
        let range = self.high - self.low;
        let scaled = (self.value - self.low) / range;
        let sym = model.decode_symbol(scaled)?;
        let (sym_low, sym_high) = model.range(sym)?;
        self.high = self.low + range * sym_high;
        self.low += range * sym_low;
        Some(sym)
    }

    #[allow(dead_code)]
    pub fn decode_n(&mut self, model: &ProbModel, n: usize) -> Vec<u8> {
        (0..n).filter_map(|_| self.decode_symbol(model)).collect()
    }
}

/// High-level encode: returns encoded value.
#[allow(dead_code)]
pub fn encode(data: &[u8], model: &ProbModel) -> f64 {
    let mut enc = ArithEncoder::new();
    for &b in data {
        enc.encode_symbol(model, b);
    }
    enc.finalize()
}

/// High-level decode: returns decoded bytes.
#[allow(dead_code)]
pub fn decode(value: f64, model: &ProbModel, length: usize) -> Vec<u8> {
    let mut dec = ArithDecoder::new(value);
    dec.decode_n(model, length)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model(data: &[u8]) -> ProbModel {
        let mut freq = HashMap::new();
        for &b in data {
            *freq.entry(b).or_insert(0u64) += 1;
        }
        ProbModel::from_freq(&freq)
    }

    #[test]
    fn test_model_range_covers_zero_one() {
        let m = make_model(b"aabb");
        let (lo, hi) = m.range(b'a').expect("should succeed");
        assert!(lo >= 0.0 && hi <= 1.0);
    }

    #[test]
    fn test_model_decode_symbol() {
        let m = make_model(b"ab");
        assert!(m.decode_symbol(0.1).is_some());
        assert!(m.decode_symbol(0.9).is_some());
    }

    #[test]
    fn test_encoder_interval_shrinks() {
        let m = make_model(b"aabb");
        let mut enc = ArithEncoder::new();
        enc.encode_symbol(&m, b'a');
        let (lo, hi) = enc.interval();
        assert!(hi - lo < 1.0);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let data = b"aab";
        let m = make_model(data);
        let encoded = encode(data, &m);
        let decoded = decode(encoded, &m, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_symbol_count() {
        let m = make_model(b"abcde");
        assert_eq!(m.symbol_count(), 5);
    }

    #[test]
    fn test_encoder_finalize_in_range() {
        let m = make_model(b"ab");
        let mut enc = ArithEncoder::new();
        enc.encode_symbol(&m, b'a');
        let val = enc.finalize();
        assert!((0.0..=1.0).contains(&val));
    }

    #[test]
    fn test_single_symbol_encode_decode() {
        let data = b"aaa";
        let m = make_model(data);
        let encoded = encode(data, &m);
        let decoded = decode(encoded, &m, 3);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_model_unknown_symbol() {
        let m = make_model(b"ab");
        assert!(m.range(b'z').is_none());
    }

    #[test]
    fn test_cumulative_starts_at_zero() {
        let m = make_model(b"abc");
        assert!((m.cumulative[0]).abs() < 1e-9);
    }

    #[test]
    fn test_cumulative_ends_at_one() {
        let m = make_model(b"abc");
        assert!((*m.cumulative.last().expect("should succeed") - 1.0).abs() < 1e-9);
    }
}
