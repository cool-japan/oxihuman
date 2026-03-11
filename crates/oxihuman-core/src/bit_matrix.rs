// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compact boolean matrix (bitset rows with u64 words).

#![allow(dead_code)]

/// A compact boolean matrix stored as rows of u64 bit-words.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitMatrix {
    pub rows: usize,
    pub cols: usize,
    words_per_row: usize,
    data: Vec<u64>,
}

#[allow(dead_code)]
impl BitMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        let words_per_row = cols.div_ceil(64);
        let data = vec![0u64; rows * words_per_row];
        Self {
            rows,
            cols,
            words_per_row,
            data,
        }
    }

    fn index(&self, row: usize, col: usize) -> (usize, usize) {
        let word = row * self.words_per_row + col / 64;
        let bit = col % 64;
        (word, bit)
    }

    /// Set bit at (row, col) to true.
    pub fn set(&mut self, row: usize, col: usize) {
        assert!(row < self.rows && col < self.cols);
        let (w, b) = self.index(row, col);
        self.data[w] |= 1u64 << b;
    }

    /// Clear bit at (row, col).
    pub fn clear_bit(&mut self, row: usize, col: usize) {
        assert!(row < self.rows && col < self.cols);
        let (w, b) = self.index(row, col);
        self.data[w] &= !(1u64 << b);
    }

    /// Toggle bit at (row, col).
    pub fn toggle(&mut self, row: usize, col: usize) {
        assert!(row < self.rows && col < self.cols);
        let (w, b) = self.index(row, col);
        self.data[w] ^= 1u64 << b;
    }

    /// Get bit at (row, col).
    pub fn get(&self, row: usize, col: usize) -> bool {
        assert!(row < self.rows && col < self.cols);
        let (w, b) = self.index(row, col);
        (self.data[w] >> b) & 1 == 1
    }

    /// Count set bits in a row.
    pub fn row_popcount(&self, row: usize) -> u32 {
        assert!(row < self.rows);
        let start = row * self.words_per_row;
        let end = start + self.words_per_row;
        self.data[start..end].iter().map(|w| w.count_ones()).sum()
    }

    /// Count all set bits.
    pub fn total_popcount(&self) -> u32 {
        self.data.iter().map(|w| w.count_ones()).sum()
    }

    /// Zero all bits.
    pub fn clear_all(&mut self) {
        self.data.fill(0);
    }

    /// Fill all bits.
    pub fn fill_all(&mut self) {
        self.data.fill(u64::MAX);
        // Zero out padding bits in last word of each row
        if !self.cols.is_multiple_of(64) {
            let mask = (1u64 << (self.cols % 64)) - 1;
            for row in 0..self.rows {
                let last = (row + 1) * self.words_per_row - 1;
                self.data[last] &= mask;
            }
        }
    }

    /// Bitwise AND row r1 with r2, result in r1.
    pub fn row_and(&mut self, r1: usize, r2: usize) {
        assert!(r1 < self.rows && r2 < self.rows);
        for w in 0..self.words_per_row {
            self.data[r1 * self.words_per_row + w] &= self.data[r2 * self.words_per_row + w];
        }
    }

    /// Bitwise OR row r1 with r2, result in r1.
    pub fn row_or(&mut self, r1: usize, r2: usize) {
        assert!(r1 < self.rows && r2 < self.rows);
        for w in 0..self.words_per_row {
            self.data[r1 * self.words_per_row + w] |= self.data[r2 * self.words_per_row + w];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_all_clear() {
        let m = BitMatrix::new(4, 64);
        assert_eq!(m.total_popcount(), 0);
    }

    #[test]
    fn set_and_get() {
        let mut m = BitMatrix::new(4, 64);
        m.set(2, 33);
        assert!(m.get(2, 33));
        assert!(!m.get(2, 32));
    }

    #[test]
    fn clear_bit() {
        let mut m = BitMatrix::new(4, 64);
        m.set(1, 10);
        m.clear_bit(1, 10);
        assert!(!m.get(1, 10));
    }

    #[test]
    fn toggle() {
        let mut m = BitMatrix::new(4, 64);
        m.toggle(0, 0);
        assert!(m.get(0, 0));
        m.toggle(0, 0);
        assert!(!m.get(0, 0));
    }

    #[test]
    fn row_popcount() {
        let mut m = BitMatrix::new(3, 64);
        m.set(1, 0);
        m.set(1, 10);
        m.set(1, 63);
        assert_eq!(m.row_popcount(1), 3);
    }

    #[test]
    fn total_popcount() {
        let mut m = BitMatrix::new(3, 64);
        m.set(0, 0);
        m.set(1, 1);
        m.set(2, 2);
        assert_eq!(m.total_popcount(), 3);
    }

    #[test]
    fn fill_all_popcount() {
        let mut m = BitMatrix::new(2, 10);
        m.fill_all();
        assert_eq!(m.total_popcount(), 20);
    }

    #[test]
    fn row_and() {
        let mut m = BitMatrix::new(2, 64);
        m.set(0, 5);
        m.set(0, 10);
        m.set(1, 5);
        m.row_and(0, 1);
        assert!(m.get(0, 5));
        assert!(!m.get(0, 10));
    }

    #[test]
    fn row_or() {
        let mut m = BitMatrix::new(2, 64);
        m.set(0, 5);
        m.set(1, 10);
        m.row_or(0, 1);
        assert!(m.get(0, 5));
        assert!(m.get(0, 10));
    }

    #[test]
    fn wide_matrix() {
        let mut m = BitMatrix::new(2, 200);
        m.set(0, 199);
        m.set(1, 65);
        assert!(m.get(0, 199));
        assert!(m.get(1, 65));
        assert!(!m.get(0, 65));
    }
}
