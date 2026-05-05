// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FlatBuffers builder with vtable machinery.
//!
//! Implements a subset of the FlatBuffers wire format:
//! <https://flatbuffers.dev/internals/>
//!
//! Layout in buffer (append order):
//! ```text
//! [object: soffset(4) | field_data...] [vtable: vtable_size(2) | object_size(2) | field_offsets(2 each)...]
//! ```
//! The `soffset` at the start of every object is a signed i32 pointing from the object start
//! forward to its vtable: `vtable_address = object_start + soffset`.

/// A FlatBuffers builder that accumulates bytes and manages vtable deduplication.
#[derive(Debug, Default)]
pub struct FlatBuilder {
    buf: Vec<u8>,
    /// Byte offsets within `buf` where each vtable begins (for deduplication).
    pub(crate) vtables: Vec<usize>,
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

impl std::error::Error for FlatError {}

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

    /// Begin building a table with `num_slots` field slots.
    ///
    /// Writes a 4-byte placeholder for the vtable pointer (soffset) and returns
    /// a [`TableBuilder`] that tracks slot assignments.  Call [`TableBuilder::end_table`]
    /// when all fields have been added; it writes the vtable and back-patches the soffset.
    pub fn start_table(&mut self, num_slots: usize) -> Result<TableBuilder<'_>, FlatError> {
        if self.finished {
            return Err(FlatError::AlreadyFinished);
        }
        let object_start = self.buf.len();
        // Write placeholder for soffset (4 bytes, will be back-patched in end_table)
        self.buf.extend_from_slice(&[0u8; 4]);
        Ok(TableBuilder {
            builder: self,
            object_start,
            slots: vec![None; num_slots],
        })
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

/// A transient builder for a single FlatBuffers table.
///
/// Created by [`FlatBuilder::start_table`] and consumed by [`TableBuilder::end_table`].
#[derive(Debug)]
pub struct TableBuilder<'a> {
    builder: &'a mut FlatBuilder,
    /// Byte offset in `builder.buf` where this object (soffset placeholder) begins.
    object_start: usize,
    /// Per-slot recorded byte-offset within the object (relative to `object_start`).
    /// `None` means the slot was not set (field absent; defaults apply at read time).
    slots: Vec<Option<usize>>,
}

impl<'a> TableBuilder<'a> {
    /// Add a `u32` field at the given `slot` index.
    ///
    /// If `value == default` the field is elided from the object (FlatBuffers default elision).
    pub fn add_slot_u32(&mut self, slot: usize, value: u32, default: u32) -> Result<(), FlatError> {
        if value == default {
            return Ok(()); // elide default values per FlatBuffers spec
        }
        let offset_in_object = self.builder.buf.len() - self.object_start;
        self.builder.push_u32(value)?;
        if slot < self.slots.len() {
            self.slots[slot] = Some(offset_in_object);
        }
        Ok(())
    }

    /// Add a raw byte-slice field at the given `slot` index.
    pub fn add_slot_bytes(&mut self, slot: usize, bytes: &[u8]) -> Result<(), FlatError> {
        let offset_in_object = self.builder.buf.len() - self.object_start;
        self.builder.push_bytes(bytes)?;
        if slot < self.slots.len() {
            self.slots[slot] = Some(offset_in_object);
        }
        Ok(())
    }

    /// Finalise the table: write the vtable (with deduplication), back-patch the soffset,
    /// and return the byte offset of the object start within the buffer.
    pub fn end_table(self) -> Result<usize, FlatError> {
        let TableBuilder {
            builder,
            object_start,
            slots,
        } = self;

        let object_size = builder.buf.len() - object_start;
        let num_slots = slots.len();

        // Compute vtable bytes
        // vtable layout: [vtable_size: u16][object_size: u16][field_offset_0: u16]...
        let vtable_size_bytes: u16 = (2 + 2 + num_slots * 2) as u16;
        let object_size_u16: u16 = object_size as u16;

        let mut vtable_bytes: Vec<u8> = Vec::with_capacity(vtable_size_bytes as usize);
        vtable_bytes.extend_from_slice(&vtable_size_bytes.to_le_bytes());
        vtable_bytes.extend_from_slice(&object_size_u16.to_le_bytes());
        for slot_opt in &slots {
            let field_offset: u16 = slot_opt.map_or(0, |off| off as u16);
            vtable_bytes.extend_from_slice(&field_offset.to_le_bytes());
        }

        // Vtable deduplication: reuse an existing vtable with identical content
        let vtable_start = builder
            .vtables
            .iter()
            .find(|&&off| {
                let end = off.saturating_add(vtable_bytes.len());
                end <= builder.buf.len() && builder.buf[off..end] == vtable_bytes[..]
            })
            .copied()
            .unwrap_or_else(|| {
                // Append new vtable after object data
                let start = builder.buf.len();
                builder.vtables.push(start);
                builder.buf.extend_from_slice(&vtable_bytes);
                start
            });

        // Back-patch the soffset: signed distance from object_start to vtable_start.
        // Positive because we append the vtable *after* the object data.
        let soffset: i32 = vtable_start as i32 - object_start as i32;
        let soffset_bytes = soffset.to_le_bytes();
        builder.buf[object_start..object_start + 4].copy_from_slice(&soffset_bytes);

        Ok(object_start)
    }
}

// ── Read helpers ──────────────────────────────────────────────────────────────

/// Read a raw `u32` from a FlatBuffers byte slice at a given offset.
pub fn read_u32(data: &[u8], offset: usize) -> Result<u32, FlatError> {
    let bytes: [u8; 4] = data
        .get(offset..offset + 4)
        .and_then(|s| s.try_into().ok())
        .ok_or(FlatError::InvalidOffset)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Read a table field of type `u32` from `buf`, using the vtable pointer at `table_off`.
///
/// Returns `default` when the field is absent or the offset is out of range.
pub fn read_table_u32(buf: &[u8], table_off: usize, slot: usize, default: u32) -> u32 {
    read_table_u32_inner(buf, table_off, slot).unwrap_or(default)
}

/// Internal fallible implementation of [`read_table_u32`].
fn read_table_u32_inner(buf: &[u8], table_off: usize, slot: usize) -> Option<u32> {
    // Read soffset (i32) at the start of the object
    let soffset_bytes: [u8; 4] = buf
        .get(table_off..table_off + 4)
        .and_then(|s| s.try_into().ok())?;
    let soffset = i32::from_le_bytes(soffset_bytes);

    // Compute vtable start (may be after the object in our append-order builder)
    let vtable_off = (table_off as i64 + soffset as i64) as usize;

    // Read vtable header: vtable_size (u16) | object_size (u16)
    let vtable_size_bytes: [u8; 2] = buf
        .get(vtable_off..vtable_off + 2)
        .and_then(|s| s.try_into().ok())?;
    let vtable_size = u16::from_le_bytes(vtable_size_bytes) as usize;

    // Slot entry lives at vtable_off + 4 + slot * 2
    let slot_entry_off = vtable_off + 4 + slot * 2;
    if slot_entry_off + 2 > vtable_off + vtable_size {
        return None;
    }
    let field_offset_bytes: [u8; 2] = buf
        .get(slot_entry_off..slot_entry_off + 2)
        .and_then(|s| s.try_into().ok())?;
    let field_off = u16::from_le_bytes(field_offset_bytes) as usize;
    if field_off == 0 {
        return None; // field absent, caller uses default
    }

    let abs_off = table_off + field_off;
    let value_bytes: [u8; 4] = buf
        .get(abs_off..abs_off + 4)
        .and_then(|s| s.try_into().ok())?;
    Some(u32::from_le_bytes(value_bytes))
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

    // ── vtable tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_empty_table_round_trip() {
        /* empty table (0 slots) can be built and does not panic */
        let mut builder = FlatBuilder::new();
        let table = builder.start_table(0).expect("start_table");
        let obj_off = table.end_table().expect("end_table");
        let buf = builder.finish().expect("finish");
        assert!(!buf.is_empty());
        // object lives at obj_off; vtable at (obj_off + soffset)
        assert_eq!(obj_off, 0);
    }

    #[test]
    fn test_single_u32_field() {
        /* a table with one u32 slot round-trips the value */
        let mut builder = FlatBuilder::new();
        let mut table = builder.start_table(1).expect("start_table");
        table.add_slot_u32(0, 42, 0).expect("add_slot_u32");
        let obj_off = table.end_table().expect("end_table");
        let buf = builder.finish().expect("finish");
        assert_eq!(read_table_u32(&buf, obj_off, 0, 0), 42);
    }

    #[test]
    fn test_default_value_unset() {
        /* slot not set → read returns the provided default */
        let mut builder = FlatBuilder::new();
        let table = builder.start_table(2).expect("start_table");
        let obj_off = table.end_table().expect("end_table");
        let buf = builder.finish().expect("finish");
        assert_eq!(read_table_u32(&buf, obj_off, 0, 5), 5);
        assert_eq!(read_table_u32(&buf, obj_off, 1, 99), 99);
    }

    #[test]
    fn test_two_field_round_trip() {
        /* two u32 fields in the same table both round-trip correctly */
        let mut builder = FlatBuilder::new();
        let mut table = builder.start_table(2).expect("start_table");
        table.add_slot_u32(0, 100, 0).expect("slot 0");
        table.add_slot_u32(1, 200, 0).expect("slot 1");
        let obj_off = table.end_table().expect("end_table");
        let buf = builder.finish().expect("finish");
        assert_eq!(read_table_u32(&buf, obj_off, 0, 0), 100);
        assert_eq!(read_table_u32(&buf, obj_off, 1, 0), 200);
    }

    #[test]
    fn test_vtable_dedup() {
        /* two tables with identical vtable shape share one vtable entry */
        let mut builder = FlatBuilder::new();

        let mut t1 = builder.start_table(1).expect("start t1");
        t1.add_slot_u32(0, 10, 0).expect("t1 slot");
        let off1 = t1.end_table().expect("t1 end");

        let mut t2 = builder.start_table(1).expect("start t2");
        t2.add_slot_u32(0, 20, 0).expect("t2 slot");
        let off2 = t2.end_table().expect("t2 end");

        // Both vtables had the same shape → deduplication keeps only 1 vtable
        assert_eq!(builder.vtables.len(), 1);

        // Values still read back correctly even though vtable is shared
        let buf = builder.finish().expect("finish");
        assert_eq!(read_table_u32(&buf, off1, 0, 0), 10);
        assert_eq!(read_table_u32(&buf, off2, 0, 0), 20);
    }

    #[test]
    fn test_bytes_field_round_trip() {
        /* add_slot_bytes stores raw bytes accessible via read_u32 when 4 bytes long */
        let mut builder = FlatBuilder::new();
        let mut table = builder.start_table(1).expect("start_table");
        table
            .add_slot_bytes(0, &[0x01, 0x00, 0x00, 0x00])
            .expect("add_slot_bytes");
        let obj_off = table.end_table().expect("end_table");
        let buf = builder.finish().expect("finish");
        // The bytes spell out 1u32 in little-endian
        assert_eq!(read_table_u32(&buf, obj_off, 0, 0), 1);
    }

    #[test]
    fn test_start_table_after_finish_fails() {
        /* start_table on a finished builder returns AlreadyFinished */
        let mut builder = FlatBuilder::new();
        builder.finished = true;
        assert_eq!(
            builder.start_table(1).unwrap_err(),
            FlatError::AlreadyFinished
        );
    }
}
