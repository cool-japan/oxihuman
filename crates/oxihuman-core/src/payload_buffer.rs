// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Named payload slots with typed tags.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PayloadEntry {
    pub tag: String,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
pub struct PayloadBuffer {
    entries: Vec<PayloadEntry>,
    max_entries: usize,
    total_bytes: usize,
}

#[allow(dead_code)]
impl PayloadBuffer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries: max_entries.max(1),
            total_bytes: 0,
        }
    }
    pub fn push(&mut self, tag: &str, data: &[u8]) -> bool {
        if self.entries.len() >= self.max_entries {
            return false;
        }
        self.total_bytes += data.len();
        self.entries.push(PayloadEntry {
            tag: tag.to_string(),
            data: data.to_vec(),
        });
        true
    }
    pub fn pop(&mut self) -> Option<PayloadEntry> {
        let e = self.entries.pop();
        if let Some(ref entry) = e {
            self.total_bytes = self.total_bytes.saturating_sub(entry.data.len());
        }
        e
    }
    pub fn peek(&self) -> Option<&PayloadEntry> {
        self.entries.last()
    }
    pub fn get(&self, idx: usize) -> Option<&PayloadEntry> {
        self.entries.get(idx)
    }
    pub fn find_by_tag(&self, tag: &str) -> Vec<&PayloadEntry> {
        self.entries.iter().filter(|e| e.tag == tag).collect()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.max_entries
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_bytes = 0;
    }
    pub fn drain(&mut self) -> Vec<PayloadEntry> {
        self.total_bytes = 0;
        std::mem::take(&mut self.entries)
    }
}

#[allow(dead_code)]
pub fn new_payload_buffer(max: usize) -> PayloadBuffer {
    PayloadBuffer::new(max)
}
#[allow(dead_code)]
pub fn pybuf_push(b: &mut PayloadBuffer, tag: &str, data: &[u8]) -> bool {
    b.push(tag, data)
}
#[allow(dead_code)]
pub fn pybuf_pop(b: &mut PayloadBuffer) -> Option<PayloadEntry> {
    b.pop()
}
#[allow(dead_code)]
pub fn pybuf_peek(b: &PayloadBuffer) -> Option<&PayloadEntry> {
    b.peek()
}
#[allow(dead_code)]
pub fn pybuf_len(b: &PayloadBuffer) -> usize {
    b.len()
}
#[allow(dead_code)]
pub fn pybuf_is_empty(b: &PayloadBuffer) -> bool {
    b.is_empty()
}
#[allow(dead_code)]
pub fn pybuf_total_bytes(b: &PayloadBuffer) -> usize {
    b.total_bytes()
}
#[allow(dead_code)]
pub fn pybuf_is_full(b: &PayloadBuffer) -> bool {
    b.is_full()
}
#[allow(dead_code)]
pub fn pybuf_clear(b: &mut PayloadBuffer) {
    b.clear();
}
#[allow(dead_code)]
pub fn pybuf_drain(b: &mut PayloadBuffer) -> Vec<PayloadEntry> {
    b.drain()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_push_pop() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "msg", &[1, 2, 3]);
        let e = pybuf_pop(&mut b).expect("should succeed");
        assert_eq!(e.tag, "msg".to_string());
        assert_eq!(e.data, vec![1, 2, 3]);
    }
    #[test]
    fn test_max_entries() {
        let mut b = new_payload_buffer(2);
        pybuf_push(&mut b, "a", &[1]);
        pybuf_push(&mut b, "b", &[2]);
        assert!(!pybuf_push(&mut b, "c", &[3]));
    }
    #[test]
    fn test_is_full() {
        let mut b = new_payload_buffer(1);
        pybuf_push(&mut b, "x", &[0]);
        assert!(pybuf_is_full(&b));
    }
    #[test]
    fn test_total_bytes() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "a", &[1, 2]);
        pybuf_push(&mut b, "b", &[3, 4, 5]);
        assert_eq!(pybuf_total_bytes(&b), 5);
    }
    #[test]
    fn test_find_by_tag() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "foo", &[1]);
        pybuf_push(&mut b, "bar", &[2]);
        pybuf_push(&mut b, "foo", &[3]);
        assert_eq!(b.find_by_tag("foo").len(), 2);
    }
    #[test]
    fn test_clear() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "a", &[1]);
        pybuf_clear(&mut b);
        assert!(pybuf_is_empty(&b));
    }
    #[test]
    fn test_drain() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "a", &[1]);
        pybuf_push(&mut b, "b", &[2]);
        let drained = pybuf_drain(&mut b);
        assert_eq!(drained.len(), 2);
        assert!(pybuf_is_empty(&b));
    }
    #[test]
    fn test_is_empty_initially() {
        let b = new_payload_buffer(8);
        assert!(pybuf_is_empty(&b));
    }
    #[test]
    fn test_peek() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "top", &[99]);
        assert_eq!(pybuf_peek(&b).map(|e| e.tag.as_str()), Some("top"));
    }
    #[test]
    fn test_bytes_decrease_after_pop() {
        let mut b = new_payload_buffer(8);
        pybuf_push(&mut b, "a", &[1, 2, 3]);
        pybuf_pop(&mut b);
        assert_eq!(pybuf_total_bytes(&b), 0);
    }
}
