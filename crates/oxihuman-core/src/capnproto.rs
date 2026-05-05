// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cap'n Proto wire format implementation.
//!
//! Provides:
//! - Segment table / frame header encoding (A1)
//! - Struct pointer and list pointer encoding/decoding (A2)
//! - Legacy stub API (`CapnSegment`, `CapnMessage`) preserved for compatibility

// ============================================================================
// A1: Segment table / frame header (Cap'n Proto wire format)
// ============================================================================

/// Errors that may arise during Cap'n Proto frame encoding/decoding.
#[derive(Debug, thiserror::Error)]
pub enum CapnpError {
    /// Input terminated before the segment-count word could be read.
    #[error("truncated header")]
    TruncatedHeader,
    /// Input terminated before the declared segment bytes were present.
    #[error("truncated segment")]
    TruncatedSegment,
    /// Pointer kind bits (2 LSB) did not match an expected value.
    #[error("invalid pointer kind")]
    InvalidPointerKind,
    /// Offset value would reach outside the segment.
    #[error("offset out of bounds")]
    OffsetOutOfBounds,
    /// Composite list tag (element_size tag 7) is not yet supported.
    #[error("composite list tag not yet supported")]
    ListTagUnsupported,
    /// Traversal limit was exceeded; aborting to prevent DoS.
    #[error("traversal limit exceeded")]
    TraversalLimitExceeded,
}

/// A Cap'n Proto wire-format message: a list of raw byte segments.
///
/// Every segment **must** be a multiple of 8 bytes (word-aligned).
/// `write_frame` / `read_frame` assume this invariant and compute
/// `word_count = segment.len() / 8`.
#[derive(Debug, Clone, Default)]
pub struct Message {
    /// Raw byte slices, one per segment.  Each slice is word-aligned.
    pub segments: Vec<Vec<u8>>,
}

impl Message {
    /// Create an empty [`Message`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Encode a [`Message`] into a Cap'n Proto framing envelope.
///
/// # Frame layout
/// ```text
/// [u32 LE: (segment_count - 1)]
/// [u32 LE: segment_0_word_count]
/// ...one u32 per additional segment...
/// [u32 LE: 0x00000000]  (padding, only when (segment_count + 1) is odd)
/// <segment 0 bytes> <segment 1 bytes> ...
/// ```
pub fn write_frame(msg: &Message) -> Vec<u8> {
    let n = msg.segments.len();

    // Number of u32 header words before optional padding:
    //   1 (segment count) + n (one word per segment size)
    let header_word_count = 1 + n;
    // Pad to make header_word_count even (i.e. 8-byte aligned).
    let padded_header_words = if header_word_count.is_multiple_of(2) {
        header_word_count
    } else {
        header_word_count + 1
    };

    let total_segment_bytes: usize = msg.segments.iter().map(|s| s.len()).sum();
    let mut out = Vec::with_capacity(padded_header_words * 4 + total_segment_bytes);

    // Write (segment_count - 1) as u32 LE.
    let seg_count_minus_one = if n > 0 { (n as u32) - 1 } else { 0 };
    out.extend_from_slice(&seg_count_minus_one.to_le_bytes());

    // Write one word per segment: word count of that segment.
    for seg in &msg.segments {
        let word_count = (seg.len() / 8) as u32;
        out.extend_from_slice(&word_count.to_le_bytes());
    }

    // Padding word if needed.
    if !header_word_count.is_multiple_of(2) {
        out.extend_from_slice(&0u32.to_le_bytes());
    }

    // Segment bodies.
    for seg in &msg.segments {
        out.extend_from_slice(seg);
    }

    out
}

/// Decode a wire-format byte slice back into a [`Message`].
///
/// Returns [`CapnpError::TruncatedHeader`] if the header cannot be parsed,
/// or [`CapnpError::TruncatedSegment`] if the body is shorter than declared.
pub fn read_frame(bytes: &[u8]) -> Result<Message, CapnpError> {
    // Need at least 4 bytes for the segment-count word.
    if bytes.len() < 4 {
        return Err(CapnpError::TruncatedHeader);
    }

    let seg_count_minus_one = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let segment_count = seg_count_minus_one + 1;

    // Header contains 1 + segment_count u32 words, possibly one padding word.
    let header_word_count = 1 + segment_count;
    let padded_header_words = if header_word_count.is_multiple_of(2) {
        header_word_count
    } else {
        header_word_count + 1
    };
    let header_bytes = padded_header_words * 4;

    if bytes.len() < header_bytes {
        return Err(CapnpError::TruncatedHeader);
    }

    // Read per-segment word counts.
    let mut word_counts = Vec::with_capacity(segment_count);
    for i in 0..segment_count {
        let offset = 4 + i * 4;
        let wc = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        word_counts.push(wc);
    }

    // Extract segment bodies.
    let mut segments = Vec::with_capacity(segment_count);
    let mut cursor = header_bytes;
    for wc in &word_counts {
        let seg_bytes = wc * 8;
        if cursor + seg_bytes > bytes.len() {
            return Err(CapnpError::TruncatedSegment);
        }
        segments.push(bytes[cursor..cursor + seg_bytes].to_vec());
        cursor += seg_bytes;
    }

    Ok(Message { segments })
}

// ============================================================================
// A2: Struct pointer and list pointer encoding/decoding
// ============================================================================

/// A Cap'n Proto struct pointer (2 LSB = 0b00).
///
/// Layout (u64 LE):
/// - Bits  \[1:0\]  = 0b00
/// - Bits \[31:2\]  = offset_words (signed 30-bit, sign-extended to i32)
/// - Bits \[47:32\] = data_words (u16)
/// - Bits \[63:48\] = pointer_words (u16)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructPointer {
    /// Signed 30-bit offset in words from the word *after* the pointer to the struct data.
    pub offset_words: i32,
    /// Number of 64-bit words in the struct's data section.
    pub data_words: u16,
    /// Number of 64-bit words in the struct's pointer section.
    pub pointer_words: u16,
}

/// Encode a [`StructPointer`] to a raw `u64`.
pub fn encode_struct_ptr(p: &StructPointer) -> u64 {
    // The offset occupies bits [31:2]; kind bits [1:0] = 0b00.
    // Mask to 30 bits, then shift to position [31:2].
    let offset_30 = (p.offset_words as u32) & 0x3FFF_FFFF;
    let low32 = offset_30 << 2; // bits [1:0] remain 0b00
    let high32 = ((p.pointer_words as u32) << 16) | (p.data_words as u32);
    (low32 as u64) | ((high32 as u64) << 32)
}

/// Decode a raw `u64` as a [`StructPointer`].
///
/// Does NOT validate that bits \[1:0\] are 0b00; callers should check the kind
/// before dispatching.
pub fn decode_struct_ptr(raw: u64) -> StructPointer {
    // Bits [31:2]: extract then sign-extend from bit 29.
    let raw30 = ((raw >> 2) & 0x3FFF_FFFF) as u32;
    // Arithmetic right-shift by 2 to sign-extend the 30-bit value.
    let offset_words = ((raw30 << 2) as i32) >> 2;

    let high32 = (raw >> 32) as u32;
    let data_words = (high32 & 0xFFFF) as u16;
    let pointer_words = (high32 >> 16) as u16;

    StructPointer {
        offset_words,
        data_words,
        pointer_words,
    }
}

/// Element size tags for list pointers (bits \[34:32\]).
///
/// Values 0–6 match the Cap'n Proto specification exactly.
/// Tag 7 (Composite) is not yet supported by this implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ElementSize {
    /// 0 bytes per element (void).
    Void = 0,
    /// 1 bit per element.
    Bit = 1,
    /// 1 byte per element.
    Byte = 2,
    /// 2 bytes per element.
    TwoBytes = 3,
    /// 4 bytes per element.
    FourBytes = 4,
    /// 8 bytes per element (non-pointer).
    EightBytesNonPtr = 5,
    /// 8 bytes per element (pointer).
    EightBytesPtr = 6,
}

/// A Cap'n Proto list pointer (2 LSB = 0b01).
///
/// Layout (u64 LE):
/// - Bits  \[1:0\]  = 0b01
/// - Bits \[31:2\]  = offset_words (signed 30-bit)
/// - Bits \[34:32\] = element_size_tag (3 bits, 0–6)
/// - Bits \[63:35\] = element_count (29 bits)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPointer {
    /// Signed 30-bit offset in words from the word after the pointer to the list data.
    pub offset_words: i32,
    /// Size encoding for each list element.
    pub element_size: ElementSize,
    /// Number of elements in the list (29-bit max).
    pub element_count: u32,
}

/// Encode a [`ListPointer`] to a raw `u64`.
pub fn encode_list_ptr(p: &ListPointer) -> u64 {
    // bits [1:0] = 0b01
    // bits [31:2] = offset (30-bit signed)
    // bits [34:32] = element_size tag (3 bits)
    // bits [63:35] = element_count (29 bits)
    let offset_30 = (p.offset_words as u32) & 0x3FFF_FFFF;
    let low32 = (offset_30 << 2) | 0b01u32;
    let element_size_tag = p.element_size as u64;
    let element_count = (p.element_count as u64) & 0x1FFF_FFFF; // 29 bits
    let high32 = (element_count << 3) | element_size_tag;
    (low32 as u64) | (high32 << 32)
}

/// Decode a raw `u64` as a [`ListPointer`].
///
/// Returns [`CapnpError::ListTagUnsupported`] if element_size tag = 7
/// (composite lists, not yet implemented).
pub fn decode_list_ptr(raw: u64) -> Result<ListPointer, CapnpError> {
    let offset_30 = ((raw >> 2) & 0x3FFF_FFFF) as u32;
    let offset_words = ((offset_30 << 2) as i32) >> 2;

    let high32 = raw >> 32;
    let element_size_tag = (high32 & 0b111) as u8;
    let element_count = ((high32 >> 3) & 0x1FFF_FFFF) as u32;

    let element_size = match element_size_tag {
        0 => ElementSize::Void,
        1 => ElementSize::Bit,
        2 => ElementSize::Byte,
        3 => ElementSize::TwoBytes,
        4 => ElementSize::FourBytes,
        5 => ElementSize::EightBytesNonPtr,
        6 => ElementSize::EightBytesPtr,
        7 => return Err(CapnpError::ListTagUnsupported),
        _ => unreachable!("3-bit tag cannot exceed 7"),
    };

    Ok(ListPointer {
        offset_words,
        element_size,
        element_count,
    })
}

// ============================================================================
// A3: Traversal limiter (Cap'n Proto security model)
// ============================================================================

/// Counts traversal budget remaining (in 64-bit words).
///
/// The Cap'n Proto spec requires that every pointer traversal decrements a
/// counter by the size of the pointed-to object.  When the counter reaches 0
/// further traversal returns [`CapnpError::TraversalLimitExceeded`].
///
/// Default budget: 8 MiB / 8 bytes = 8 * 1024 * 1024 words.
pub struct TraversalLimiter {
    pub remaining_words: u64,
}

impl TraversalLimiter {
    /// Create a limiter with the default Cap'n Proto budget (8M words = 64 MiB).
    pub fn new() -> Self {
        TraversalLimiter {
            remaining_words: 8 * 1024 * 1024,
        }
    }

    /// Create a limiter with a custom budget.
    pub fn with_limit(words: u64) -> Self {
        TraversalLimiter {
            remaining_words: words,
        }
    }

    /// Consume `words` from the budget, or return `TraversalLimitExceeded`.
    pub fn consume(&mut self, words: u64) -> Result<(), CapnpError> {
        if words > self.remaining_words {
            return Err(CapnpError::TraversalLimitExceeded);
        }
        self.remaining_words -= words;
        Ok(())
    }
}

impl Default for TraversalLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the word count consumed by a list pointer's data region.
///
/// For tag-7 (composite) lists the total word count is encoded in bits \[63:35\]
/// of the raw pointer — the caller should supply that directly as
/// `element_count` together with `ElementSize::EightBytesPtr` as a sentinel
/// only when synthesising; the dedicated v2 path handles composite lists.
fn list_body_words(size: ElementSize, element_count: u32) -> u64 {
    let n = element_count as u64;
    match size {
        ElementSize::Void => 0,
        ElementSize::Bit => n.div_ceil(64),
        ElementSize::Byte => n.div_ceil(8),
        ElementSize::TwoBytes => n.div_ceil(4),
        ElementSize::FourBytes => n.div_ceil(2),
        ElementSize::EightBytesNonPtr | ElementSize::EightBytesPtr => n,
    }
}

/// Follows a struct pointer inside `segment_data`, decrementing the limiter.
///
/// Returns `(data_start, data_end, pointers_start)` — byte positions within
/// `segment_data`.
///
/// # Errors
/// - [`CapnpError::InvalidPointerKind`] if bits \[1:0\] ≠ 0b00.
/// - [`CapnpError::OffsetOutOfBounds`] if computed range escapes the segment.
/// - [`CapnpError::TraversalLimitExceeded`] if limiter budget exhausted.
pub fn read_struct_ptr_checked(
    segment_data: &[u8],
    ptr_offset_bytes: usize,
    limiter: &mut TraversalLimiter,
) -> Result<(usize, usize, usize), CapnpError> {
    if ptr_offset_bytes + 8 > segment_data.len() {
        return Err(CapnpError::OffsetOutOfBounds);
    }

    let raw = u64::from_le_bytes(
        segment_data[ptr_offset_bytes..ptr_offset_bytes + 8]
            .try_into()
            .map_err(|_| CapnpError::OffsetOutOfBounds)?,
    );

    if raw & 0b11 != 0b00 {
        return Err(CapnpError::InvalidPointerKind);
    }

    let sp = decode_struct_ptr(raw);

    // The offset is measured in words from the word immediately after the pointer.
    let ptr_word_index = (ptr_offset_bytes / 8) as i64;
    // "word after pointer" = ptr_word_index + 1
    let data_start_word = ptr_word_index + 1 + (sp.offset_words as i64);
    if data_start_word < 0 {
        return Err(CapnpError::OffsetOutOfBounds);
    }

    let data_start = (data_start_word as usize) * 8;
    let data_end = data_start + (sp.data_words as usize) * 8;
    let pointers_start = data_end;
    let object_end = pointers_start + (sp.pointer_words as usize) * 8;

    if object_end > segment_data.len() {
        return Err(CapnpError::OffsetOutOfBounds);
    }

    let pointed_words = (sp.data_words as u64) + (sp.pointer_words as u64);
    limiter.consume(pointed_words)?;

    Ok((data_start, data_end, pointers_start))
}

/// Follows a list pointer inside `segment_data`, decrementing the limiter.
///
/// Returns `(data_start, element_count)` — byte position within `segment_data`
/// and the declared element count.
///
/// # Errors
/// - [`CapnpError::InvalidPointerKind`] if bits \[1:0\] ≠ 0b01.
/// - [`CapnpError::ListTagUnsupported`] if element_size tag = 7 (composite).
/// - [`CapnpError::OffsetOutOfBounds`] if computed range escapes the segment.
/// - [`CapnpError::TraversalLimitExceeded`] if limiter budget exhausted.
pub fn read_list_ptr_checked(
    segment_data: &[u8],
    ptr_offset_bytes: usize,
    limiter: &mut TraversalLimiter,
) -> Result<(usize, u32), CapnpError> {
    if ptr_offset_bytes + 8 > segment_data.len() {
        return Err(CapnpError::OffsetOutOfBounds);
    }

    let raw = u64::from_le_bytes(
        segment_data[ptr_offset_bytes..ptr_offset_bytes + 8]
            .try_into()
            .map_err(|_| CapnpError::OffsetOutOfBounds)?,
    );

    if raw & 0b11 != 0b01 {
        return Err(CapnpError::InvalidPointerKind);
    }

    let lp = decode_list_ptr(raw)?;

    let ptr_word_index = (ptr_offset_bytes / 8) as i64;
    let data_start_word = ptr_word_index + 1 + (lp.offset_words as i64);
    if data_start_word < 0 {
        return Err(CapnpError::OffsetOutOfBounds);
    }

    let data_start = (data_start_word as usize) * 8;
    let body_words = list_body_words(lp.element_size, lp.element_count);
    let data_end = data_start + (body_words as usize) * 8;

    if data_end > segment_data.len() {
        return Err(CapnpError::OffsetOutOfBounds);
    }

    limiter.consume(body_words)?;

    Ok((data_start, lp.element_count))
}

// ============================================================================
// A4: Far pointers + composite list tag (element_size = 7)
// ============================================================================

/// A Cap'n Proto far pointer (2 LSB = 0b10).
///
/// Wire encoding (u64 LE):
/// - Bits  \[1:0\] = 0b10
/// - Bit   \[2\]   = 0 for single-far, 1 for double-far
/// - Bits \[31:3\] = landing pad offset in words within the target segment (29 bits)
/// - Bits \[63:32\]= target segment ID (u32)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FarPointer {
    /// `false` → single-far (one landing pad word); `true` → double-far (two words).
    pub is_double: bool,
    /// Word offset within the target segment to the landing pad.
    pub landing_pad_offset_words: u32,
    /// Index of the segment that holds the landing pad.
    pub target_segment_id: u32,
}

/// Encode a [`FarPointer`] to a raw `u64`.
pub fn encode_far_ptr(p: &FarPointer) -> u64 {
    // bits [1:0] = 0b10
    // bit  [2]   = is_double
    // bits [31:3]= landing_pad_offset (29 bits)
    // bits [63:32]= target_segment_id
    let double_bit = if p.is_double { 1u64 << 2 } else { 0u64 };
    let offset_29 = (p.landing_pad_offset_words as u64) & 0x1FFF_FFFF; // 29 bits
    let low32 = 0b10u64 | double_bit | (offset_29 << 3);
    let high32 = p.target_segment_id as u64;
    low32 | (high32 << 32)
}

/// Decode a raw `u64` as a [`FarPointer`].
///
/// Returns [`CapnpError::InvalidPointerKind`] if bits \[1:0\] ≠ 0b10.
pub fn decode_far_ptr(raw: u64) -> Result<FarPointer, CapnpError> {
    if raw & 0b11 != 0b10 {
        return Err(CapnpError::InvalidPointerKind);
    }
    let is_double = (raw >> 2) & 1 == 1;
    let landing_pad_offset_words = ((raw >> 3) & 0x1FFF_FFFF) as u32;
    let target_segment_id = (raw >> 32) as u32;
    Ok(FarPointer {
        is_double,
        landing_pad_offset_words,
        target_segment_id,
    })
}

/// Read 8 bytes from a segment at a word offset, returning raw u64 LE.
fn read_word_from_segment(seg: &[u8], word_offset: usize) -> Result<u64, CapnpError> {
    let byte_offset = word_offset * 8;
    if byte_offset + 8 > seg.len() {
        return Err(CapnpError::OffsetOutOfBounds);
    }
    let raw = u64::from_le_bytes(
        seg[byte_offset..byte_offset + 8]
            .try_into()
            .map_err(|_| CapnpError::OffsetOutOfBounds)?,
    );
    Ok(raw)
}

/// Resolve a pointer that may be a far pointer, returning the
/// canonical `(segment_id, ptr_word_offset_in_segment, raw_struct_or_list_ptr)`.
///
/// Traversal hops each consume 1 word from the limiter:
/// - single-far: 1 hop → 1 word consumed
/// - double-far: 2 hops → 2 words consumed
///
/// # Errors
/// - [`CapnpError::OffsetOutOfBounds`] if any segment read is out of range.
/// - [`CapnpError::InvalidPointerKind`] if 2 LSB = 0b11 (capability pointer, not supported).
/// - [`CapnpError::TraversalLimitExceeded`] if limiter budget exhausted.
pub fn resolve_pointer(
    msg: &Message,
    segment_id: usize,
    ptr_word_offset: usize,
    limiter: &mut TraversalLimiter,
) -> Result<(usize, usize, u64), CapnpError> {
    let seg = msg
        .segments
        .get(segment_id)
        .ok_or(CapnpError::OffsetOutOfBounds)?;
    let raw = read_word_from_segment(seg, ptr_word_offset)?;

    match raw & 0b11 {
        0b00 | 0b01 => {
            // Already a struct or list pointer — no far hops needed.
            Ok((segment_id, ptr_word_offset, raw))
        }
        0b10 => {
            let far = decode_far_ptr(raw)?;
            let target_seg = msg
                .segments
                .get(far.target_segment_id as usize)
                .ok_or(CapnpError::OffsetOutOfBounds)?;

            if !far.is_double {
                // Single-far: the landing pad in target_seg IS the real pointer.
                // Consume 1 word for the landing pad read.
                limiter.consume(1)?;
                let landing_raw = read_word_from_segment(target_seg, far.landing_pad_offset_words as usize)?;
                Ok((
                    far.target_segment_id as usize,
                    far.landing_pad_offset_words as usize,
                    landing_raw,
                ))
            } else {
                // Double-far: landing pad has two words.
                //   word 0 = another far pointer (into the final segment)
                //   word 1 = real struct/list pointer tag
                // Consume 2 words for both landing pad reads.
                limiter.consume(2)?;
                let lp_offset = far.landing_pad_offset_words as usize;
                let word0 = read_word_from_segment(target_seg, lp_offset)?;
                let word1 = read_word_from_segment(target_seg, lp_offset + 1)?;

                // word0 must be a far pointer pointing to the final segment.
                let inner_far = decode_far_ptr(word0)?;
                Ok((
                    inner_far.target_segment_id as usize,
                    inner_far.landing_pad_offset_words as usize,
                    word1,
                ))
            }
        }
        _ /* 0b11 */ => {
            // Capability pointer — not supported in this implementation.
            Err(CapnpError::InvalidPointerKind)
        }
    }
}

// ============================================================================
// A4: Composite list tag (element_size = 7)
// ============================================================================

/// Decoded composite list tag word.
///
/// In a composite list the data region starts with a one-word tag that looks
/// like a struct pointer (bits \[1:0\] = 0b00) **except** the offset field
/// encodes the total number of elements (as an unsigned 30-bit value, not a
/// signed pointer offset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeTag {
    /// Total number of elements in the list.
    pub element_count: u32,
    /// Number of data words per element.
    pub data_words_per_element: u16,
    /// Number of pointer words per element.
    pub pointers_per_element: u16,
}

/// Decode a composite list tag word.
///
/// The tag looks like a struct pointer (bits \[1:0\] = 0b00) but the
/// `offset_words` field is reinterpreted as an **unsigned** element count.
///
/// # Errors
/// Returns [`CapnpError::InvalidPointerKind`] if bits \[1:0\] ≠ 0b00.
pub fn decode_composite_tag(tag_word: u64) -> Result<CompositeTag, CapnpError> {
    if tag_word & 0b11 != 0b00 {
        return Err(CapnpError::InvalidPointerKind);
    }
    // Extract bits [31:2] as unsigned 30-bit element count.
    let element_count = ((tag_word >> 2) & 0x3FFF_FFFF) as u32;

    let high32 = (tag_word >> 32) as u32;
    let data_words_per_element = (high32 & 0xFFFF) as u16;
    let pointers_per_element = (high32 >> 16) as u16;

    Ok(CompositeTag {
        element_count,
        data_words_per_element,
        pointers_per_element,
    })
}

/// Result of [`decode_list_ptr_v2`]: either a fixed-size list or a composite list.
///
/// For composite lists (`element_size` = 7 in the wire format):
/// - `offset_words` is the signed 30-bit offset to the tag word.
/// - `element_count` is taken from bits \[63:35\] of the raw pointer (total body
///   words excluding the tag).  The actual per-element layout is in the tag
///   word and must be decoded separately with [`decode_composite_tag`].
/// - `data_words` and `ptr_words` are **not** available from the pointer alone
///   and are set to `0` here; decode the tag word at the body start to obtain them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListPointerOrComposite {
    /// A fixed-size list (element_size tags 0–6).
    Fixed(ListPointer),
    /// A composite list (element_size tag 7).
    ///
    /// `element_count` here is the value from bits \[63:35\] of the raw pointer,
    /// which encodes the **total number of body words** (excluding the tag word).
    /// `data_words` and `ptr_words` are always 0 and must be read from the tag.
    Composite {
        offset_words: i32,
        element_count: u32,
        data_words: u16,
        ptr_words: u16,
    },
}

/// Decode a raw list pointer `u64`, handling all 8 element sizes.
///
/// Unlike [`decode_list_ptr`], this function does **not** return
/// `ListTagUnsupported` for element_size = 7; instead it returns
/// `ListPointerOrComposite::Composite`.
///
/// # Errors
/// Returns [`CapnpError::InvalidPointerKind`] if bits \[1:0\] ≠ 0b01.
pub fn decode_list_ptr_v2(raw: u64) -> Result<ListPointerOrComposite, CapnpError> {
    if raw & 0b11 != 0b01 {
        return Err(CapnpError::InvalidPointerKind);
    }

    let offset_30 = ((raw >> 2) & 0x3FFF_FFFF) as u32;
    let offset_words = ((offset_30 << 2) as i32) >> 2;

    let high32 = raw >> 32;
    let element_size_tag = (high32 & 0b111) as u8;
    let element_count = ((high32 >> 3) & 0x1FFF_FFFF) as u32;

    if element_size_tag == 7 {
        return Ok(ListPointerOrComposite::Composite {
            offset_words,
            element_count,
            data_words: 0,
            ptr_words: 0,
        });
    }

    let element_size = match element_size_tag {
        0 => ElementSize::Void,
        1 => ElementSize::Bit,
        2 => ElementSize::Byte,
        3 => ElementSize::TwoBytes,
        4 => ElementSize::FourBytes,
        5 => ElementSize::EightBytesNonPtr,
        6 => ElementSize::EightBytesPtr,
        _ => unreachable!("3-bit tag cannot exceed 7"),
    };

    Ok(ListPointerOrComposite::Fixed(ListPointer {
        offset_words,
        element_size,
        element_count,
    }))
}

// ============================================================================
// Legacy stub API (preserved for backward compatibility)
// ============================================================================

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
        assert_eq!(m.segment(0).expect("should succeed").read_word(0), Some(99));
    }

    // ========================================================================
    // A1 tests: segment table / frame header
    // ========================================================================

    #[test]
    fn test_segment_table_round_trip_single_segment() {
        // 1 segment with 8 words (64 bytes) of data.
        let data: Vec<u8> = (0u8..64).collect();
        let msg = Message {
            segments: vec![data.clone()],
        };

        let frame = write_frame(&msg);
        let recovered = read_frame(&frame).expect("round-trip should succeed");

        assert_eq!(recovered.segments.len(), 1);
        assert_eq!(recovered.segments[0], data);
    }

    #[test]
    fn test_segment_table_round_trip_three_segments() {
        // Three segments: 3+1=4 header u32 words — even, no padding needed.
        let seg0: Vec<u8> = vec![0xAA; 16]; // 2 words
        let seg1: Vec<u8> = vec![0xBB; 8]; // 1 word
        let seg2: Vec<u8> = vec![0xCC; 24]; // 3 words
        let msg = Message {
            segments: vec![seg0.clone(), seg1.clone(), seg2.clone()],
        };

        let frame = write_frame(&msg);

        // 3 segments → header_word_count = 4 (even) → no padding word.
        // Header bytes = 4 * 4 = 16.
        assert_eq!(
            &frame[..16],
            &[
                // segment_count - 1 = 2
                2, 0, 0, 0, // seg0 word count = 2
                2, 0, 0, 0, // seg1 word count = 1
                1, 0, 0, 0, // seg2 word count = 3
                3, 0, 0, 0,
            ]
        );

        let recovered = read_frame(&frame).expect("round-trip should succeed");
        assert_eq!(recovered.segments.len(), 3);
        assert_eq!(recovered.segments[0], seg0);
        assert_eq!(recovered.segments[1], seg1);
        assert_eq!(recovered.segments[2], seg2);

        // Also verify the 2-segment padding path:
        // 2 segments → header_word_count = 3 (odd) → padding word required.
        let seg_a: Vec<u8> = vec![0x11; 8]; // 1 word
        let seg_b: Vec<u8> = vec![0x22; 16]; // 2 words
        let msg2 = Message {
            segments: vec![seg_a.clone(), seg_b.clone()],
        };
        let frame2 = write_frame(&msg2);

        // header_word_count = 3, padded to 4 → header bytes = 16.
        assert_eq!(
            &frame2[..16],
            &[
                // segment_count - 1 = 1
                1, 0, 0, 0, // seg_a word count = 1
                1, 0, 0, 0, // seg_b word count = 2
                2, 0, 0, 0, // padding
                0, 0, 0, 0,
            ]
        );

        let recovered2 = read_frame(&frame2).expect("2-segment round-trip should succeed");
        assert_eq!(recovered2.segments.len(), 2);
        assert_eq!(recovered2.segments[0], seg_a);
        assert_eq!(recovered2.segments[1], seg_b);
    }

    // ========================================================================
    // A2 tests: struct pointer encoding/decoding
    // ========================================================================

    #[test]
    fn test_struct_pointer_encode_decode() {
        let original = StructPointer {
            data_words: 2,
            pointer_words: 3,
            offset_words: 5,
        };
        let encoded = encode_struct_ptr(&original);
        let decoded = decode_struct_ptr(encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_struct_pointer_negative_offset() {
        let original = StructPointer {
            data_words: 1,
            pointer_words: 0,
            offset_words: -3,
        };
        let encoded = encode_struct_ptr(&original);
        let decoded = decode_struct_ptr(encoded);
        assert_eq!(decoded.offset_words, -3);
    }

    // ========================================================================
    // A2 tests: list pointer encoding/decoding
    // ========================================================================

    #[test]
    fn test_list_pointer_encode_decode() {
        let original = ListPointer {
            element_size: ElementSize::TwoBytes,
            element_count: 42,
            offset_words: 10,
        };
        let encoded = encode_list_ptr(&original);
        let decoded = decode_list_ptr(encoded).expect("decode should succeed");
        assert_eq!(decoded, original);
    }

    // ========================================================================
    // Error-path tests
    // ========================================================================

    #[test]
    fn test_read_frame_rejects_truncated_header() {
        // Only 2 bytes — cannot read even the segment-count word.
        let result = read_frame(&[0x00, 0x01]);
        assert!(matches!(result, Err(CapnpError::TruncatedHeader)));
    }

    #[test]
    fn test_read_frame_rejects_truncated_segment() {
        // Header says 1 segment of 8 words (64 bytes), but body has only 6*8=48 bytes.
        let mut frame = Vec::new();
        // segment_count - 1 = 0
        frame.extend_from_slice(&0u32.to_le_bytes());
        // segment 0 word count = 8
        frame.extend_from_slice(&8u32.to_le_bytes());
        // no padding (1+1=2, even)
        // body: only 6 words (48 bytes) instead of 8 words (64 bytes)
        frame.extend(vec![0u8; 48]);

        let result = read_frame(&frame);
        assert!(matches!(result, Err(CapnpError::TruncatedSegment)));
    }

    #[test]
    fn test_decode_unsupported_list_tag_7() {
        // Construct a raw u64 with element_size bits [34:32] = 0b111 (tag 7).
        // bits [1:0] = 0b01 (list pointer kind), bits [34:32] = 0b111 (composite tag).
        let raw: u64 = (7u64 << 32) | 0b01u64;
        let result = decode_list_ptr(raw);
        assert!(matches!(result, Err(CapnpError::ListTagUnsupported)));
    }

    // ========================================================================
    // A3 tests: TraversalLimiter
    // ========================================================================

    #[test]
    fn test_traversal_limiter_basic() {
        let mut lim = TraversalLimiter::new();
        // Consume 100 words — should succeed.
        lim.consume(100).expect("consume 100 words should succeed");
        // Consume the rest — should succeed.
        let remaining = lim.remaining_words;
        lim.consume(remaining)
            .expect("consume all remaining should succeed");
        // Now limiter is at 0; consume one more → error.
        let result = lim.consume(1);
        assert!(matches!(result, Err(CapnpError::TraversalLimitExceeded)));
    }

    #[test]
    fn test_traversal_limiter_exact_limit() {
        let mut lim = TraversalLimiter::with_limit(5);
        lim.consume(3).expect("consume 3 from 5 should succeed");
        lim.consume(2).expect("consume 2 from 2 should succeed");
        // Exactly at 0 — any further consumption must fail.
        let result = lim.consume(1);
        assert!(matches!(result, Err(CapnpError::TraversalLimitExceeded)));
    }

    // ========================================================================
    // A4 tests: FarPointer encode/decode
    // ========================================================================

    #[test]
    fn test_far_pointer_encode_decode_single() {
        let fp = FarPointer {
            is_double: false,
            landing_pad_offset_words: 42,
            target_segment_id: 1,
        };
        let raw = encode_far_ptr(&fp);
        let decoded = decode_far_ptr(raw).expect("decode should succeed");
        assert_eq!(decoded, fp);
    }

    #[test]
    fn test_far_pointer_encode_decode_double() {
        let fp = FarPointer {
            is_double: true,
            landing_pad_offset_words: 0x1234_5678 & 0x1FFF_FFFF,
            target_segment_id: 7,
        };
        let raw = encode_far_ptr(&fp);
        let decoded = decode_far_ptr(raw).expect("decode should succeed");
        assert_eq!(decoded, fp);
    }

    #[test]
    fn test_far_pointer_wrong_kind() {
        // 2 LSB = 0b00 → struct pointer, not a far pointer.
        let raw: u64 = 0x0000_0000_0000_0000u64; // bits [1:0] = 0b00
        let result = decode_far_ptr(raw);
        assert!(matches!(result, Err(CapnpError::InvalidPointerKind)));
    }

    // ========================================================================
    // A4 tests: resolve_pointer
    // ========================================================================

    #[test]
    fn test_resolve_pointer_local() {
        // A struct pointer (2 LSB = 0b00) in segment 0 — no far hop needed.
        // Encode: offset=0, data_words=1, pointer_words=0.
        let sp = StructPointer {
            offset_words: 0,
            data_words: 1,
            pointer_words: 0,
        };
        let raw = encode_struct_ptr(&sp);
        let mut seg0 = vec![0u8; 16]; // 2 words
        seg0[0..8].copy_from_slice(&raw.to_le_bytes());
        let msg = Message {
            segments: vec![seg0],
        };
        let mut lim = TraversalLimiter::new();
        let (seg, offset, returned_raw) =
            resolve_pointer(&msg, 0, 0, &mut lim).expect("resolve should succeed");
        assert_eq!(seg, 0);
        assert_eq!(offset, 0);
        assert_eq!(returned_raw, raw);
        // Limiter should be unchanged (no hops consumed).
        assert_eq!(lim.remaining_words, 8 * 1024 * 1024);
    }

    #[test]
    fn test_resolve_pointer_single_far() {
        // Segment 0: contains a single-far pointer pointing at segment 1, word 0.
        // Segment 1: contains the real struct pointer at word 0.
        let sp = StructPointer {
            offset_words: 0,
            data_words: 2,
            pointer_words: 0,
        };
        let real_raw = encode_struct_ptr(&sp);

        // Build segment 1: real pointer at word 0, data at word 1 (2 words of zeros).
        let mut seg1 = vec![0u8; 24]; // 3 words: pointer + 2 data words
        seg1[0..8].copy_from_slice(&real_raw.to_le_bytes());

        // Build segment 0: far pointer → seg 1, word 0, single-far.
        let fp = FarPointer {
            is_double: false,
            landing_pad_offset_words: 0,
            target_segment_id: 1,
        };
        let far_raw = encode_far_ptr(&fp);
        let mut seg0 = vec![0u8; 8]; // 1 word
        seg0[0..8].copy_from_slice(&far_raw.to_le_bytes());

        let msg = Message {
            segments: vec![seg0, seg1],
        };
        let mut lim = TraversalLimiter::new();
        let (seg, offset, returned_raw) =
            resolve_pointer(&msg, 0, 0, &mut lim).expect("resolve should succeed");

        assert_eq!(seg, 1);
        assert_eq!(offset, 0);
        assert_eq!(returned_raw, real_raw);
        // 1 hop → 1 word consumed.
        assert_eq!(lim.remaining_words, 8 * 1024 * 1024 - 1);
    }

    #[test]
    fn test_resolve_pointer_double_far() {
        // Layout:
        //   seg0 word 0: double-far pointer → seg1 word 0
        //   seg1 word 0: another far pointer (single-far) → seg2 word 0  [word0 of landing pad]
        //   seg1 word 1: real struct pointer                               [word1 of landing pad]
        //   seg2 word 0: (struct data — not read by resolve_pointer)

        let sp = StructPointer {
            offset_words: 0,
            data_words: 1,
            pointer_words: 0,
        };
        let real_raw = encode_struct_ptr(&sp);

        // seg1 landing pad: word0 = far ptr → seg2 word 0, word1 = real ptr
        let fp_inner = FarPointer {
            is_double: false,
            landing_pad_offset_words: 0,
            target_segment_id: 2,
        };
        let word0_raw = encode_far_ptr(&fp_inner);

        let mut seg1 = vec![0u8; 16]; // 2 words
        seg1[0..8].copy_from_slice(&word0_raw.to_le_bytes());
        seg1[8..16].copy_from_slice(&real_raw.to_le_bytes());

        // seg0: double-far pointer → seg1 word 0
        let fp_outer = FarPointer {
            is_double: true,
            landing_pad_offset_words: 0,
            target_segment_id: 1,
        };
        let outer_raw = encode_far_ptr(&fp_outer);
        let mut seg0 = vec![0u8; 8];
        seg0[0..8].copy_from_slice(&outer_raw.to_le_bytes());

        // seg2: just data
        let seg2 = vec![0u8; 8];

        let msg = Message {
            segments: vec![seg0, seg1, seg2],
        };
        let mut lim = TraversalLimiter::new();
        let (seg, offset, returned_raw) =
            resolve_pointer(&msg, 0, 0, &mut lim).expect("resolve should succeed");

        assert_eq!(seg, 2);
        assert_eq!(offset, 0);
        assert_eq!(returned_raw, real_raw);
        // 2 hops → 2 words consumed.
        assert_eq!(lim.remaining_words, 8 * 1024 * 1024 - 2);
    }

    // ========================================================================
    // A4 tests: CompositeTag and decode_list_ptr_v2
    // ========================================================================

    #[test]
    fn test_composite_tag_decode() {
        // Build a tag word: element_count=10, data_words=2, ptr_words=1.
        // Tag word looks like a struct pointer: bits [1:0]=0b00, bits[31:2]=10 (unsigned),
        // bits[47:32]=2 (data_words), bits[63:48]=1 (ptr_words).
        let element_count: u64 = 10;
        let data_words: u64 = 2;
        let ptr_words: u64 = 1;
        let tag_word = (element_count << 2) | (data_words << 32) | (ptr_words << 48);
        let tag = decode_composite_tag(tag_word).expect("decode should succeed");
        assert_eq!(tag.element_count, 10);
        assert_eq!(tag.data_words_per_element, 2);
        assert_eq!(tag.pointers_per_element, 1);
    }

    #[test]
    fn test_decode_list_ptr_v2_fixed() {
        // A regular TwoBytes list — should come back as Fixed.
        let original = ListPointer {
            element_size: ElementSize::TwoBytes,
            element_count: 100,
            offset_words: 5,
        };
        let raw = encode_list_ptr(&original);
        let result = decode_list_ptr_v2(raw).expect("decode should succeed");
        assert!(matches!(result, ListPointerOrComposite::Fixed(_)));
        if let ListPointerOrComposite::Fixed(lp) = result {
            assert_eq!(lp, original);
        }
    }

    #[test]
    fn test_decode_list_ptr_v2_composite() {
        // Construct a raw list pointer with element_size_tag = 7 and
        // bits [63:35] = 50 (total body words), offset_words = 3.
        // bits [1:0] = 0b01, bits [31:2] = offset (30-bit), bits [34:32] = 7, bits [63:35] = 50.
        let offset_words: i32 = 3;
        let offset_30 = (offset_words as u32) & 0x3FFF_FFFF;
        let low32 = (offset_30 << 2) | 0b01u32;
        let element_size_tag: u64 = 7;
        let body_words: u64 = 50;
        let high32: u64 = (body_words << 3) | element_size_tag;
        let raw = (low32 as u64) | (high32 << 32);

        let result = decode_list_ptr_v2(raw).expect("decode should succeed");
        match result {
            ListPointerOrComposite::Composite {
                offset_words: ow,
                element_count,
                data_words,
                ptr_words,
            } => {
                assert_eq!(ow, 3);
                assert_eq!(element_count, 50);
                assert_eq!(data_words, 0);
                assert_eq!(ptr_words, 0);
            }
            other => panic!("expected Composite, got {:?}", other),
        }
    }
}
