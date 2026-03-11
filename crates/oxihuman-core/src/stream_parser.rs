// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Streaming byte-level parser for reading structured binary or text data.

/// The parser result.
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum ParseResult<T> {
    Ok(T),
    NeedMore,
    Error(String),
}

/// A streaming cursor over a byte buffer.
#[allow(dead_code)]
pub struct StreamParser {
    buf: Vec<u8>,
    pos: usize,
    parse_count: u64,
}

#[allow(dead_code)]
impl StreamParser {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            pos: 0,
            parse_count: 0,
        }
    }

    /// Feed more data into the parser buffer.
    pub fn feed(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// Remaining unread bytes.
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    /// Total bytes fed so far.
    pub fn total_fed(&self) -> usize {
        self.buf.len()
    }

    /// Current read cursor position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Read a single byte.
    pub fn read_u8(&mut self) -> ParseResult<u8> {
        if self.pos >= self.buf.len() {
            return ParseResult::NeedMore;
        }
        let b = self.buf[self.pos];
        self.pos += 1;
        self.parse_count += 1;
        ParseResult::Ok(b)
    }

    /// Read a little-endian u16.
    pub fn read_u16_le(&mut self) -> ParseResult<u16> {
        if self.remaining() < 2 {
            return ParseResult::NeedMore;
        }
        let lo = self.buf[self.pos] as u16;
        let hi = self.buf[self.pos + 1] as u16;
        self.pos += 2;
        self.parse_count += 1;
        ParseResult::Ok(lo | (hi << 8))
    }

    /// Read a little-endian u32.
    pub fn read_u32_le(&mut self) -> ParseResult<u32> {
        if self.remaining() < 4 {
            return ParseResult::NeedMore;
        }
        let b = &self.buf[self.pos..self.pos + 4];
        let v = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        self.pos += 4;
        self.parse_count += 1;
        ParseResult::Ok(v)
    }

    /// Read `n` raw bytes.
    pub fn read_bytes(&mut self, n: usize) -> ParseResult<Vec<u8>> {
        if self.remaining() < n {
            return ParseResult::NeedMore;
        }
        let slice = self.buf[self.pos..self.pos + n].to_vec();
        self.pos += n;
        self.parse_count += 1;
        ParseResult::Ok(slice)
    }

    /// Read a null-terminated UTF-8 string.
    pub fn read_cstring(&mut self) -> ParseResult<String> {
        let start = self.pos;
        let null_pos = self.buf[start..].iter().position(|&b| b == 0);
        match null_pos {
            None => ParseResult::NeedMore,
            Some(rel) => {
                let s = std::str::from_utf8(&self.buf[start..start + rel])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|e| format!("<utf8 error: {e}>"));
                self.pos = start + rel + 1;
                self.parse_count += 1;
                ParseResult::Ok(s)
            }
        }
    }

    /// Skip `n` bytes.
    pub fn skip(&mut self, n: usize) -> bool {
        if self.remaining() < n {
            return false;
        }
        self.pos += n;
        true
    }

    /// Reset the cursor to the beginning (keeps buffer data).
    pub fn reset_cursor(&mut self) {
        self.pos = 0;
    }

    /// Discard already-consumed bytes to free memory.
    pub fn compact(&mut self) {
        self.buf.drain(..self.pos);
        self.pos = 0;
    }

    /// Number of successful parse calls.
    pub fn parse_count(&self) -> u64 {
        self.parse_count
    }
}

impl Default for StreamParser {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_stream_parser() -> StreamParser {
    StreamParser::new()
}

pub fn sp_feed(sp: &mut StreamParser, data: &[u8]) {
    sp.feed(data);
}

pub fn sp_read_u8(sp: &mut StreamParser) -> ParseResult<u8> {
    sp.read_u8()
}

pub fn sp_read_u32_le(sp: &mut StreamParser) -> ParseResult<u32> {
    sp.read_u32_le()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_parser_needs_more() {
        let mut sp = new_stream_parser();
        assert_eq!(sp_read_u8(&mut sp), ParseResult::NeedMore);
    }

    #[test]
    fn feed_and_read_u8() {
        let mut sp = new_stream_parser();
        sp_feed(&mut sp, &[0xAB]);
        assert_eq!(sp_read_u8(&mut sp), ParseResult::Ok(0xAB));
    }

    #[test]
    fn read_u16_le() {
        let mut sp = new_stream_parser();
        sp.feed(&[0x01, 0x02]);
        assert_eq!(sp.read_u16_le(), ParseResult::Ok(0x0201));
    }

    #[test]
    fn read_u32_le() {
        let mut sp = new_stream_parser();
        sp.feed(&[0x01, 0x00, 0x00, 0x00]);
        assert_eq!(sp_read_u32_le(&mut sp), ParseResult::Ok(1));
    }

    #[test]
    fn read_cstring() {
        let mut sp = new_stream_parser();
        sp.feed(b"hello\0");
        assert_eq!(sp.read_cstring(), ParseResult::Ok("hello".to_string()));
    }

    #[test]
    fn partial_u32_needs_more() {
        let mut sp = new_stream_parser();
        sp.feed(&[0x01, 0x02]);
        assert_eq!(sp_read_u32_le(&mut sp), ParseResult::NeedMore);
    }

    #[test]
    fn skip_bytes() {
        let mut sp = new_stream_parser();
        sp.feed(&[0, 0, 42]);
        assert!(sp.skip(2));
        assert_eq!(sp_read_u8(&mut sp), ParseResult::Ok(42));
    }

    #[test]
    fn compact_frees_consumed() {
        let mut sp = new_stream_parser();
        sp.feed(&[1, 2, 3]);
        sp_read_u8(&mut sp);
        sp_read_u8(&mut sp);
        sp.compact();
        assert_eq!(sp.total_fed(), 1);
        assert_eq!(sp.position(), 0);
    }

    #[test]
    fn parse_count_increments() {
        let mut sp = new_stream_parser();
        sp.feed(&[1, 2, 3]);
        sp_read_u8(&mut sp);
        sp_read_u8(&mut sp);
        assert_eq!(sp.parse_count(), 2);
    }

    #[test]
    fn read_bytes_exact() {
        let mut sp = new_stream_parser();
        sp.feed(&[10, 20, 30]);
        assert_eq!(sp.read_bytes(3), ParseResult::Ok(vec![10, 20, 30]));
    }
}
