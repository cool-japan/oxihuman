// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Piece table text buffer — stores original and added text in two buffers,
//! with a sequence of pieces describing the logical document.

/// Source of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceSource {
    Original,
    Added,
}

/// A piece describing a run of characters from a source buffer.
#[derive(Debug, Clone)]
pub struct Piece {
    pub source: PieceSource,
    pub start: usize,
    pub length: usize,
}

/// Piece table text buffer.
pub struct PieceTable {
    original: Vec<char>,
    added: Vec<char>,
    pieces: Vec<Piece>,
}

impl PieceTable {
    /// Create a piece table from an initial string.
    pub fn new(initial: &str) -> Self {
        let original: Vec<char> = initial.chars().collect();
        let len = original.len();
        let pieces = if len > 0 {
            vec![Piece {
                source: PieceSource::Original,
                start: 0,
                length: len,
            }]
        } else {
            Vec::new()
        };
        Self {
            original,
            added: Vec::new(),
            pieces,
        }
    }

    /// Total character count of the document.
    pub fn len(&self) -> usize {
        self.pieces.iter().map(|p| p.length).sum()
    }

    /// True if the document is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert `text` at logical character position `pos`.
    pub fn insert(&mut self, pos: usize, text: &str) {
        let add_start = self.added.len();
        self.added.extend(text.chars());
        let add_len = self.added.len() - add_start;
        if add_len == 0 {
            return;
        }
        let new_piece = Piece {
            source: PieceSource::Added,
            start: add_start,
            length: add_len,
        };
        /* find the piece containing pos */
        let mut cur = 0usize;
        let mut insert_idx = self.pieces.len();
        for (i, p) in self.pieces.iter().enumerate() {
            if cur + p.length >= pos {
                let offset = pos - cur;
                if offset == 0 {
                    insert_idx = i;
                } else if offset == p.length {
                    insert_idx = i + 1;
                } else {
                    /* split piece at offset */
                    let left = Piece {
                        source: p.source,
                        start: p.start,
                        length: offset,
                    };
                    let right = Piece {
                        source: p.source,
                        start: p.start + offset,
                        length: p.length - offset,
                    };
                    self.pieces.remove(i);
                    self.pieces.insert(i, right);
                    self.pieces.insert(i, new_piece.clone());
                    self.pieces.insert(i, left);
                    return;
                }
                break;
            }
            cur += p.length;
        }
        self.pieces.insert(insert_idx, new_piece);
    }

    /// Collect the full document text.
    pub fn as_string(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for p in &self.pieces {
            let src = match p.source {
                PieceSource::Original => &self.original,
                PieceSource::Added => &self.added,
            };
            s.extend(src[p.start..p.start + p.length].iter());
        }
        s
    }

    /// Number of pieces in the table.
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }
}

/// Create a piece table from initial text.
pub fn new_piece_table(initial: &str) -> PieceTable {
    PieceTable::new(initial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_content() {
        let pt = PieceTable::new("hello");
        assert_eq!(pt.as_string(), "hello"); /* initial text preserved */
    }

    #[test]
    fn test_len() {
        let pt = PieceTable::new("abc");
        assert_eq!(pt.len(), 3); /* correct length */
    }

    #[test]
    fn test_is_empty() {
        let pt = PieceTable::new("");
        assert!(pt.is_empty()); /* empty initial */
    }

    #[test]
    fn test_insert_at_end() {
        let mut pt = PieceTable::new("hello");
        pt.insert(5, " world");
        assert_eq!(pt.as_string(), "hello world"); /* appended */
    }

    #[test]
    fn test_insert_at_start() {
        let mut pt = PieceTable::new("world");
        pt.insert(0, "hello ");
        assert_eq!(pt.as_string(), "hello world"); /* prepended */
    }

    #[test]
    fn test_insert_in_middle() {
        let mut pt = PieceTable::new("helo");
        pt.insert(3, "l");
        assert_eq!(pt.as_string(), "hello"); /* mid-insert */
    }

    #[test]
    fn test_multiple_inserts() {
        let mut pt = PieceTable::new("");
        pt.insert(0, "c");
        pt.insert(0, "b");
        pt.insert(0, "a");
        assert_eq!(pt.as_string(), "abc"); /* three prepends */
    }

    #[test]
    fn test_piece_count_grows() {
        let mut pt = PieceTable::new("ab");
        pt.insert(1, "X");
        assert!(pt.piece_count() > 1); /* split created more pieces */
    }

    #[test]
    fn test_new_helper() {
        let pt = new_piece_table("test");
        assert_eq!(pt.as_string(), "test"); /* helper works */
    }

    #[test]
    fn test_insert_empty_string() {
        let mut pt = PieceTable::new("abc");
        pt.insert(1, "");
        assert_eq!(pt.as_string(), "abc"); /* no change for empty insert */
    }
}
