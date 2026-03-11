// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gap buffer for text editing — maintains a contiguous gap at the cursor
//! position so that insertions and deletions near the cursor are O(1).

/// Gap buffer for `char` elements.
pub struct GapBuffer {
    buf: Vec<char>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    /// Create an empty gap buffer with an initial gap of `gap_size`.
    pub fn new(gap_size: usize) -> Self {
        let buf = vec!['\0'; gap_size];
        Self {
            buf,
            gap_start: 0,
            gap_end: gap_size,
        }
    }

    fn gap_len(&self) -> usize {
        self.gap_end - self.gap_start
    }

    /// Total logical length (excluding gap).
    pub fn len(&self) -> usize {
        self.buf.len() - self.gap_len()
    }

    /// True if the buffer contains no characters.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Move the cursor (gap) to position `pos`.
    pub fn move_cursor(&mut self, pos: usize) {
        let pos = pos.min(self.len());
        if pos < self.gap_start {
            /* move gap left */
            let move_count = self.gap_start - pos;
            for i in (0..move_count).rev() {
                self.buf[self.gap_end - move_count + i] = self.buf[pos + i];
            }
            self.gap_end -= move_count;
            self.gap_start = pos;
        } else if pos > self.gap_start {
            /* move gap right */
            let move_count = pos - self.gap_start;
            for i in 0..move_count {
                self.buf[self.gap_start + i] = self.buf[self.gap_end + i];
            }
            self.gap_start += move_count;
            self.gap_end += move_count;
        }
    }

    fn ensure_gap(&mut self, needed: usize) {
        if self.gap_len() < needed {
            let extra = needed - self.gap_len() + 16;
            let new_len = self.buf.len() + extra;
            let mut new_buf = vec!['\0'; new_len];
            new_buf[..self.gap_start].copy_from_slice(&self.buf[..self.gap_start]);
            let old_after = &self.buf[self.gap_end..];
            let new_gap_end = self.gap_end + extra;
            new_buf[new_gap_end..].copy_from_slice(old_after);
            self.buf = new_buf;
            self.gap_end = new_gap_end;
        }
    }

    /// Insert a character at the current cursor position.
    pub fn insert(&mut self, ch: char) {
        self.ensure_gap(1);
        self.buf[self.gap_start] = ch;
        self.gap_start += 1;
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_before(&mut self) -> Option<char> {
        if self.gap_start == 0 {
            return None;
        }
        self.gap_start -= 1;
        Some(self.buf[self.gap_start])
    }

    /// Collect the logical content into a String.
    pub fn as_string(&self) -> String {
        let mut s = String::with_capacity(self.len());
        s.extend(self.buf[..self.gap_start].iter());
        s.extend(self.buf[self.gap_end..].iter());
        s
    }

    /// Current cursor position.
    pub fn cursor(&self) -> usize {
        self.gap_start
    }
}

/// Create a new gap buffer with the given initial gap size.
pub fn new_gap_buffer(gap_size: usize) -> GapBuffer {
    GapBuffer::new(gap_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_to_string() {
        let mut gb = GapBuffer::new(8);
        gb.insert('a');
        gb.insert('b');
        gb.insert('c');
        assert_eq!(gb.as_string(), "abc"); /* inserted chars appear in order */
    }

    #[test]
    fn test_len() {
        let mut gb = GapBuffer::new(8);
        gb.insert('x');
        gb.insert('y');
        assert_eq!(gb.len(), 2); /* logical length */
    }

    #[test]
    fn test_is_empty() {
        let gb = GapBuffer::new(8);
        assert!(gb.is_empty()); /* fresh buffer is empty */
    }

    #[test]
    fn test_delete_before() {
        let mut gb = GapBuffer::new(8);
        gb.insert('a');
        let ch = gb.delete_before();
        assert_eq!(ch, Some('a')); /* deleted char returned */
        assert!(gb.is_empty()); /* buffer now empty */
    }

    #[test]
    fn test_delete_at_start() {
        let mut gb = GapBuffer::new(8);
        assert!(gb.delete_before().is_none()); /* nothing to delete */
    }

    #[test]
    fn test_move_cursor_and_insert() {
        let mut gb = GapBuffer::new(8);
        gb.insert('a');
        gb.insert('c');
        gb.move_cursor(1); /* between 'a' and 'c' */
        gb.insert('b');
        assert_eq!(gb.as_string(), "abc"); /* 'b' inserted in middle */
    }

    #[test]
    fn test_cursor_position() {
        let mut gb = GapBuffer::new(8);
        gb.insert('x');
        assert_eq!(gb.cursor(), 1); /* cursor advances after insert */
    }

    #[test]
    fn test_grow_beyond_initial_gap() {
        let mut gb = GapBuffer::new(2);
        for ch in "hello".chars() {
            gb.insert(ch);
        }
        assert_eq!(gb.as_string(), "hello"); /* gap grew to accommodate */
    }

    #[test]
    fn test_new_helper() {
        let gb = new_gap_buffer(4);
        assert!(gb.is_empty()); /* helper creates empty buffer */
    }

    #[test]
    fn test_move_cursor_left() {
        let mut gb = GapBuffer::new(8);
        gb.insert('a');
        gb.insert('b');
        gb.move_cursor(0);
        gb.delete_before(); /* nothing before position 0 */
        assert_eq!(gb.len(), 2); /* both chars remain */
    }
}
