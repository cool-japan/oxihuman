// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Line number to byte offset indexer.
//!
//! Builds an index mapping 0-based line numbers to byte offsets in a text
//! buffer, enabling O(log n) conversions between line/column and byte offset.

/// Maps line numbers to byte offsets in a source text.
#[derive(Debug, Clone)]
pub struct LineIndexer {
    /// Byte offset of the start of each line (line 0 → offset 0).
    offsets: Vec<usize>,
    /// Total byte length of the indexed text.
    total_len: usize,
}

impl LineIndexer {
    /// Build an index for the given text.
    pub fn new(text: &str) -> Self {
        let mut offsets = vec![0usize];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                offsets.push(i + 1);
            }
        }
        Self {
            offsets,
            total_len: text.len(),
        }
    }

    /// Number of lines in the indexed text.
    pub fn line_count(&self) -> usize {
        self.offsets.len()
    }

    /// Return the byte offset of the start of `line` (0-based).
    pub fn line_offset(&self, line: usize) -> Option<usize> {
        self.offsets.get(line).copied()
    }

    /// Return the length of a line (not including the newline).
    pub fn line_len(&self, line: usize) -> Option<usize> {
        let start = *self.offsets.get(line)?;
        let end = self
            .offsets
            .get(line + 1)
            .copied()
            .unwrap_or(self.total_len);
        Some(end.saturating_sub(start).saturating_sub(1))
    }

    /// Convert a byte offset to a (line, column) pair (both 0-based).
    pub fn offset_to_line_col(&self, offset: usize) -> Option<(usize, usize)> {
        if offset > self.total_len {
            return None;
        }
        let line = match self.offsets.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let col = offset.saturating_sub(self.offsets[line]);
        Some((line, col))
    }

    /// Convert a (line, column) pair to a byte offset.
    pub fn line_col_to_offset(&self, line: usize, col: usize) -> Option<usize> {
        let start = self.offsets.get(line).copied()?;
        Some(start + col)
    }

    /// Return all line start offsets.
    pub fn all_offsets(&self) -> &[usize] {
        &self.offsets
    }

    /// Check whether a byte offset is valid for the indexed text.
    pub fn is_valid_offset(&self, offset: usize) -> bool {
        offset <= self.total_len
    }
}

/// Build a `LineIndexer` from a string slice.
pub fn build_line_indexer(text: &str) -> LineIndexer {
    LineIndexer::new(text)
}

/// Convert a line number to a byte offset, returning `None` on out-of-range.
pub fn line_to_offset(indexer: &LineIndexer, line: usize) -> Option<usize> {
    indexer.line_offset(line)
}

/// Convert a byte offset to (line, column).
pub fn offset_to_position(indexer: &LineIndexer, offset: usize) -> Option<(usize, usize)> {
    indexer.offset_to_line_col(offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "hello\nworld\nfoo\n";

    #[test]
    fn test_line_count() {
        let idx = LineIndexer::new(TEXT);
        assert_eq!(idx.line_count(), 4); /* 3 newlines → 4 offsets including empty trailing */
    }

    #[test]
    fn test_line_zero_offset_is_zero() {
        let idx = LineIndexer::new(TEXT);
        assert_eq!(idx.line_offset(0), Some(0));
    }

    #[test]
    fn test_line_one_offset() {
        let idx = LineIndexer::new(TEXT);
        /* "hello\n" is 6 bytes, so line 1 starts at 6 */
        assert_eq!(idx.line_offset(1), Some(6));
    }

    #[test]
    fn test_offset_to_line_col_start() {
        let idx = LineIndexer::new(TEXT);
        assert_eq!(idx.offset_to_line_col(0), Some((0, 0)));
    }

    #[test]
    fn test_offset_to_line_col_mid_line() {
        let idx = LineIndexer::new(TEXT);
        assert_eq!(idx.offset_to_line_col(2), Some((0, 2)));
    }

    #[test]
    fn test_line_col_to_offset_round_trip() {
        let idx = LineIndexer::new(TEXT);
        let (l, c) = idx.offset_to_line_col(8).expect("should succeed");
        let back = idx.line_col_to_offset(l, c).expect("should succeed");
        assert_eq!(back, 8);
    }

    #[test]
    fn test_out_of_bounds_returns_none() {
        let idx = LineIndexer::new("hi");
        assert!(idx.line_offset(100).is_none());
    }

    #[test]
    fn test_is_valid_offset() {
        let idx = LineIndexer::new("ab");
        assert!(idx.is_valid_offset(2));
        assert!(!idx.is_valid_offset(99));
    }

    #[test]
    fn test_build_line_indexer_helper() {
        let idx = build_line_indexer("one\ntwo\n");
        assert_eq!(idx.line_count(), 3);
    }

    #[test]
    fn test_line_len() {
        let idx = LineIndexer::new("abc\ndef\n");
        assert_eq!(idx.line_len(0), Some(3));
    }
}
