// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! FlatBuffers builder stub.

/// A FlatBuffers builder that accumulates bytes.
#[derive(Debug, Default)]
pub struct FlatBuilder {
    buf: Vec<u8>,
    vtables: Vec<usize>,
    finished: bool,
}

/// FlatBuffers error.
#[derive(Debug, Clone, PartialEq)]
pub enum FlatError {
    AlreadyFinished,
    NotFinished,
    InvalidOffset,
}

impl std::fmt::Display for FlatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyFinished => write!(f, "builder already finished"),
            Self::NotFinished => write!(f, "builder not yet finished"),
            Self::InvalidOffset => write!(f, "invalid FlatBuffers offset"),
        }
    }
}

impl FlatBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder with a pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        FlatBuilder {
            buf: Vec::with_capacity(cap),
            vtables: vec![],
            finished: false,
        }
    }

    /// Return the current byte length.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Return `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Push a `u8` value.
    pub fn push_u8(&mut self, v: u8) -> Result<usize, FlatError> {
        if self.finished {
            return Err(FlatError::AlreadyFinished);
        }
        let offset = self.buf.len();
        self.buf.push(v);
        Ok(offset)
    }

    /// Push a `u32` value in little-endian format.
    pub fn push_u32(&mut self, v: u32) -> Result<usize, FlatError> {
        if self.finished {
            return Err(FlatError::AlreadyFinished);
        }
        let offset = self.buf.len();
        self.buf.extend_from_slice(&v.to_le_bytes());
        Ok(offset)
    }

    /// Push a byte slice.
    pub fn push_bytes(&mut self, data: &[u8]) -> Result<usize, FlatError> {
        if self.finished {
            return Err(FlatError::AlreadyFinished);
        }
        let offset = self.buf.len();
        self.buf.extend_from_slice(data);
        Ok(offset)
    }

    /// Finish the buffer and return the bytes.
    pub fn finish(mut self) -> Result<Vec<u8>, FlatError> {
        self.finished = true;
        Ok(self.buf)
    }

    /// Return the underlying bytes without consuming the builder.
    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Align the buffer to a given power-of-two size.
    pub fn align(&mut self, alignment: usize) {
        while !self.buf.len().is_multiple_of(alignment) {
            self.buf.push(0);
        }
    }
}

/// Read a `u32` from a FlatBuffers byte slice at a given offset.
pub fn read_u32(data: &[u8], offset: usize) -> Result<u32, FlatError> {
    if offset + 4 > data.len() {
        return Err(FlatError::InvalidOffset);
    }
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap_or_default();
    Ok(u32::from_le_bytes(bytes))
}

/// Compute the padded size for a given alignment.
pub fn padded_size(size: usize, alignment: usize) -> usize {
    let rem = size % alignment;
    if rem == 0 {
        size
    } else {
        size + alignment - rem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_builder_empty() {
        /* new builder starts empty */
        let b = FlatBuilder::new();
        assert!(b.is_empty());
    }

    #[test]
    fn test_push_u8() {
        /* push u8 grows buffer by 1 */
        let mut b = FlatBuilder::new();
        b.push_u8(0xAB).expect("should succeed");
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn test_push_u32() {
        /* push u32 grows buffer by 4 */
        let mut b = FlatBuilder::new();
        b.push_u32(0xDEAD_BEEF).expect("should succeed");
        assert_eq!(b.len(), 4);
    }

    #[test]
    fn test_push_after_finish_fails() {
        /* push after finish returns error */
        let b = FlatBuilder::new();
        b.finish().expect("should succeed");
        /* create new builder to test error */
        let mut b2 = FlatBuilder::new();
        b2.finished = true;
        assert!(b2.push_u8(1).is_err());
    }

    #[test]
    fn test_push_bytes() {
        /* push_bytes copies data */
        let mut b = FlatBuilder::new();
        b.push_bytes(&[1, 2, 3]).expect("should succeed");
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_finish_returns_bytes() {
        /* finish produces the accumulated bytes */
        let mut b = FlatBuilder::new();
        b.push_u8(99).expect("should succeed");
        let data = b.finish().expect("should succeed");
        assert_eq!(data, &[99]);
    }

    #[test]
    fn test_align_pads_to_boundary() {
        /* align pads buffer to next boundary */
        let mut b = FlatBuilder::new();
        b.push_u8(1).expect("should succeed");
        b.align(4);
        assert_eq!(b.len() % 4, 0);
    }

    #[test]
    fn test_read_u32_ok() {
        /* read_u32 decodes little-endian correctly */
        let data = [1u8, 0, 0, 0];
        assert_eq!(read_u32(&data, 0).expect("should succeed"), 1);
    }

    #[test]
    fn test_read_u32_overflow() {
        /* read_u32 with bad offset returns error */
        let data = [0u8; 3];
        assert!(read_u32(&data, 0).is_err());
    }

    #[test]
    fn test_padded_size() {
        /* padded_size rounds up correctly */
        assert_eq!(padded_size(5, 4), 8);
        assert_eq!(padded_size(8, 4), 8);
    }
}
