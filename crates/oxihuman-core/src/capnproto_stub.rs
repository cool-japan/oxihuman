// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cap'n Proto stub.

/// A Cap'n Proto message segment.
#[derive(Debug, Clone, Default)]
pub struct CapnSegment {
    words: Vec<u64>,
}

impl CapnSegment {
    /// Create an empty segment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a segment with pre-allocated capacity (in 64-bit words).
    pub fn with_capacity(words: usize) -> Self {
        CapnSegment {
            words: Vec::with_capacity(words),
        }
    }

    /// Append a 64-bit word.
    pub fn push_word(&mut self, w: u64) {
        self.words.push(w);
    }

    /// Return the number of words.
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Return the byte length (each word is 8 bytes).
    pub fn byte_len(&self) -> usize {
        self.words.len() * 8
    }

    /// Return a reference to the underlying words.
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    /// Read a word at the given index.
    pub fn read_word(&self, index: usize) -> Option<u64> {
        self.words.get(index).copied()
    }
}

/// A Cap'n Proto message containing one or more segments.
#[derive(Debug, Clone, Default)]
pub struct CapnMessage {
    segments: Vec<CapnSegment>,
}

impl CapnMessage {
    /// Create an empty message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a segment to the message.
    pub fn add_segment(&mut self, seg: CapnSegment) {
        self.segments.push(seg);
    }

    /// Return the number of segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Return the total word count across all segments.
    pub fn total_words(&self) -> usize {
        self.segments.iter().map(|s| s.word_count()).sum()
    }

    /// Return the total byte length.
    pub fn total_bytes(&self) -> usize {
        self.segments.iter().map(|s| s.byte_len()).sum()
    }

    /// Access a segment by index.
    pub fn segment(&self, index: usize) -> Option<&CapnSegment> {
        self.segments.get(index)
    }
}

/// Serialise a message to a flat byte vector (stub: raw word bytes).
pub fn serialize_message(msg: &CapnMessage) -> Vec<u8> {
    let mut out = Vec::with_capacity(msg.total_bytes());
    for seg in &msg.segments {
        for &word in seg.words() {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out
}

/// Compute the traversal limit check (stub: just returns total words).
pub fn traversal_limit_words(msg: &CapnMessage) -> usize {
    msg.total_words()
}

/// Return `true` if the message is empty (no segments or all segments empty).
pub fn message_is_empty(msg: &CapnMessage) -> bool {
    msg.total_words() == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_empty() {
        /* new segment is empty */
        let s = CapnSegment::new();
        assert_eq!(s.word_count(), 0);
    }

    #[test]
    fn test_push_word() {
        /* push_word increments word count */
        let mut s = CapnSegment::new();
        s.push_word(0xDEAD_BEEF_0000_0001);
        assert_eq!(s.word_count(), 1);
    }

    #[test]
    fn test_byte_len() {
        /* byte_len is 8x word count */
        let mut s = CapnSegment::new();
        s.push_word(1);
        s.push_word(2);
        assert_eq!(s.byte_len(), 16);
    }

    #[test]
    fn test_read_word() {
        /* read_word retrieves correct value */
        let mut s = CapnSegment::new();
        s.push_word(42);
        assert_eq!(s.read_word(0), Some(42));
        assert_eq!(s.read_word(1), None);
    }

    #[test]
    fn test_message_empty() {
        /* new message has no segments */
        let m = CapnMessage::new();
        assert!(message_is_empty(&m));
    }

    #[test]
    fn test_message_add_segment() {
        /* add_segment increments count */
        let mut m = CapnMessage::new();
        m.add_segment(CapnSegment::new());
        assert_eq!(m.segment_count(), 1);
    }

    #[test]
    fn test_total_words() {
        /* total_words sums across segments */
        let mut m = CapnMessage::new();
        let mut s = CapnSegment::new();
        s.push_word(1);
        s.push_word(2);
        m.add_segment(s);
        assert_eq!(m.total_words(), 2);
    }

    #[test]
    fn test_serialize_message() {
        /* serialised length equals total bytes */
        let mut m = CapnMessage::new();
        let mut s = CapnSegment::new();
        s.push_word(0);
        m.add_segment(s);
        let data = serialize_message(&m);
        assert_eq!(data.len(), m.total_bytes());
    }

    #[test]
    fn test_traversal_limit() {
        /* traversal limit matches total words */
        let mut m = CapnMessage::new();
        let mut s = CapnSegment::new();
        s.push_word(1);
        m.add_segment(s);
        assert_eq!(traversal_limit_words(&m), 1);
    }

    #[test]
    fn test_segment_access() {
        /* segment() returns correct segment */
        let mut m = CapnMessage::new();
        let mut s = CapnSegment::new();
        s.push_word(99);
        m.add_segment(s);
        assert_eq!(m.segment(0).unwrap().read_word(0), Some(99));
    }
}
