// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A growable text buffer supporting append, line-level access, and search.

/// A mutable text buffer backed by a String.
#[allow(dead_code)]
pub struct TextBuffer {
    buf: String,
    append_count: u64,
}

#[allow(dead_code)]
impl TextBuffer {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            append_count: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: String::with_capacity(cap),
            append_count: 0,
        }
    }

    /// Append a string slice.
    pub fn append(&mut self, s: &str) {
        self.buf.push_str(s);
        self.append_count += 1;
    }

    /// Append a line (adds newline at end).
    pub fn append_line(&mut self, s: &str) {
        self.buf.push_str(s);
        self.buf.push('\n');
        self.append_count += 1;
    }

    /// Total character count.
    pub fn char_count(&self) -> usize {
        self.buf.chars().count()
    }

    /// Total byte count.
    pub fn byte_len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns the buffer as a string slice.
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        if self.buf.is_empty() {
            return 0;
        }
        self.buf.lines().count()
    }

    /// Returns the nth line (0-indexed).
    pub fn line(&self, n: usize) -> Option<&str> {
        self.buf.lines().nth(n)
    }

    /// Returns all lines as a vector.
    pub fn lines_vec(&self) -> Vec<&str> {
        self.buf.lines().collect()
    }

    /// Search for a substring; returns the byte offset of the first match.
    pub fn find(&self, needle: &str) -> Option<usize> {
        self.buf.find(needle)
    }

    /// Count occurrences of a substring.
    pub fn count_occurrences(&self, needle: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let mut count = 0;
        let mut start = 0;
        while let Some(pos) = self.buf[start..].find(needle) {
            count += 1;
            start += pos + needle.len();
        }
        count
    }

    /// Replace all occurrences of `from` with `to`.
    pub fn replace_all(&mut self, from: &str, to: &str) {
        self.buf = self.buf.replace(from, to);
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Truncate to the first `n` bytes (must be on a char boundary).
    pub fn truncate(&mut self, n: usize) {
        if n < self.buf.len() {
            // Find the last valid char boundary at or before n
            let n_safe = (0..=n)
                .rev()
                .find(|&i| self.buf.is_char_boundary(i))
                .unwrap_or(0);
            self.buf.truncate(n_safe);
        }
    }

    pub fn append_count(&self) -> u64 {
        self.append_count
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_text_buffer() -> TextBuffer {
    TextBuffer::new()
}

pub fn tb_append(buf: &mut TextBuffer, s: &str) {
    buf.append(s);
}

pub fn tb_append_line(buf: &mut TextBuffer, s: &str) {
    buf.append_line(s);
}

pub fn tb_as_str(buf: &TextBuffer) -> &str {
    buf.as_str()
}

pub fn tb_line_count(buf: &TextBuffer) -> usize {
    buf.line_count()
}

pub fn tb_find(buf: &TextBuffer, needle: &str) -> Option<usize> {
    buf.find(needle)
}

pub fn tb_clear(buf: &mut TextBuffer) {
    buf.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_on_creation() {
        let b = new_text_buffer();
        assert!(b.is_empty());
        assert_eq!(b.byte_len(), 0);
    }

    #[test]
    fn append_and_as_str() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "hello");
        tb_append(&mut b, " world");
        assert_eq!(tb_as_str(&b), "hello world");
    }

    #[test]
    fn append_line_adds_newline() {
        let mut b = new_text_buffer();
        tb_append_line(&mut b, "line1");
        tb_append_line(&mut b, "line2");
        assert_eq!(tb_line_count(&b), 2);
    }

    #[test]
    fn line_access() {
        let mut b = new_text_buffer();
        tb_append_line(&mut b, "alpha");
        tb_append_line(&mut b, "beta");
        assert_eq!(b.line(0), Some("alpha"));
        assert_eq!(b.line(1), Some("beta"));
        assert_eq!(b.line(2), None);
    }

    #[test]
    fn find_substring() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "abcdef");
        assert_eq!(tb_find(&b, "cde"), Some(2));
        assert_eq!(tb_find(&b, "xyz"), None);
    }

    #[test]
    fn count_occurrences() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "aababab");
        assert_eq!(b.count_occurrences("ab"), 3);
    }

    #[test]
    fn replace_all() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "foo bar foo");
        b.replace_all("foo", "baz");
        assert_eq!(tb_as_str(&b), "baz bar baz");
    }

    #[test]
    fn clear_resets() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "data");
        tb_clear(&mut b);
        assert!(b.is_empty());
    }

    #[test]
    fn truncate() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "hello world");
        b.truncate(5);
        assert_eq!(tb_as_str(&b), "hello");
    }

    #[test]
    fn append_count() {
        let mut b = new_text_buffer();
        tb_append(&mut b, "a");
        tb_append_line(&mut b, "b");
        assert_eq!(b.append_count(), 2);
    }
}
