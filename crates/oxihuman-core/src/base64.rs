// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Full-featured Base64 encoder/decoder.
//!
//! Supports standard and URL-safe alphabets, optional padding,
//! configurable line wrapping, streaming encode/decode, and
//! whitespace-tolerant decoding.

#![allow(dead_code)]

use std::fmt;
use std::io::{self, Read, Write};
use std::mem::ManuallyDrop;

// ---------------------------------------------------------------------------
// Alphabets
// ---------------------------------------------------------------------------

const STANDARD_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const URL_SAFE_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Pre-computed 256-entry decode tables.  `255` means invalid.
const fn build_decode_table(alphabet: &[u8; 64]) -> [u8; 256] {
    let mut table = [255u8; 256];
    let mut i = 0usize;
    while i < 64 {
        table[alphabet[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const STANDARD_DECODE: [u8; 256] = build_decode_table(STANDARD_ALPHABET);
const URL_SAFE_DECODE: [u8; 256] = build_decode_table(URL_SAFE_ALPHABET);

/// Kept for backward compatibility: same as [`STANDARD_ALPHABET`].
const ALPHABET: &[u8; 64] = STANDARD_ALPHABET;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Specific reason a base64 decode failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base64DecodeError {
    /// Encountered a byte that is not part of the chosen alphabet,
    /// not padding, and not whitespace.
    InvalidCharacter {
        /// The offending byte value.
        byte: u8,
        /// Zero-based position inside the (whitespace-stripped) input.
        position: usize,
    },
    /// Padding characters appear at an unexpected position.
    InvalidPadding,
    /// After stripping whitespace, the remaining length is not valid
    /// (i.e. `len % 4 == 1` which can never be produced by encoding).
    InvalidLength {
        /// Length after whitespace removal.
        length: usize,
    },
}

impl fmt::Display for Base64DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter { byte, position } => {
                write!(
                    f,
                    "invalid base64 character 0x{byte:02X} at position {position}"
                )
            }
            Self::InvalidPadding => write!(f, "invalid base64 padding"),
            Self::InvalidLength { length } => {
                write!(f, "invalid base64 length {length} (mod 4 == 1)")
            }
        }
    }
}

impl std::error::Error for Base64DecodeError {}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Selects the 64-character alphabet to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Base64Variant {
    /// Standard alphabet (`+`, `/`).
    #[default]
    Standard,
    /// URL-safe alphabet (`-`, `_`).
    UrlSafe,
}

/// Controls whether `=` padding is emitted / required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Base64Padding {
    /// Always emit padding on encode; tolerate missing padding on decode.
    #[default]
    Pad,
    /// Never emit padding on encode; tolerate present padding on decode.
    NoPad,
}

/// Full configuration for encode / decode operations.
#[derive(Debug, Clone)]
pub struct Base64Config {
    pub variant: Base64Variant,
    pub padding: Base64Padding,
    /// If `Some(n)`, insert `\r\n` every `n` output characters during
    /// encoding (common value: 76 for MIME).  `None` means no wrapping.
    pub line_wrap: Option<usize>,
}

impl Default for Base64Config {
    fn default() -> Self {
        Self {
            variant: Base64Variant::Standard,
            padding: Base64Padding::Pad,
            line_wrap: None,
        }
    }
}

impl Base64Config {
    /// Standard Base64 with padding, no wrapping.
    pub fn standard() -> Self {
        Self::default()
    }

    /// Standard Base64 with MIME line wrapping at 76 chars.
    pub fn mime() -> Self {
        Self {
            line_wrap: Some(76),
            ..Self::default()
        }
    }

    /// URL-safe Base64, no padding, no wrapping.
    pub fn url_safe() -> Self {
        Self {
            variant: Base64Variant::UrlSafe,
            padding: Base64Padding::NoPad,
            line_wrap: None,
        }
    }

    /// URL-safe Base64 with padding.
    pub fn url_safe_padded() -> Self {
        Self {
            variant: Base64Variant::UrlSafe,
            padding: Base64Padding::Pad,
            line_wrap: None,
        }
    }

    fn alphabet(&self) -> &'static [u8; 64] {
        match self.variant {
            Base64Variant::Standard => STANDARD_ALPHABET,
            Base64Variant::UrlSafe => URL_SAFE_ALPHABET,
        }
    }

    fn decode_table(&self) -> &'static [u8; 256] {
        match self.variant {
            Base64Variant::Standard => &STANDARD_DECODE,
            Base64Variant::UrlSafe => &URL_SAFE_DECODE,
        }
    }

    fn emit_padding(&self) -> bool {
        matches!(self.padding, Base64Padding::Pad)
    }
}

// ---------------------------------------------------------------------------
// Length helpers (original public API preserved)
// ---------------------------------------------------------------------------

/// Returns the encoded length **without** line-wrapping overhead.
pub fn base64_encoded_len(input_len: usize) -> usize {
    input_len.div_ceil(3) * 4
}

/// Upper-bound decoded length (may over-estimate by up to 2 bytes
/// because padding/remainder is not accounted for).
pub fn base64_decoded_len(encoded_len: usize) -> usize {
    if encoded_len == 0 {
        return 0;
    }
    encoded_len / 4 * 3
}

// ---------------------------------------------------------------------------
// Core encode (configurable)
// ---------------------------------------------------------------------------

/// Encode `data` using the given configuration.
pub fn base64_encode_config(data: &[u8], config: &Base64Config) -> String {
    let alpha = config.alphabet();
    let pad = config.emit_padding();
    let raw_len = base64_encoded_len(data.len());
    let mut out = Vec::with_capacity(raw_len + raw_len / 38);

    let mut i = 0;
    while i + 2 < data.len() {
        let b0 = data[i] as u32;
        let b1 = data[i + 1] as u32;
        let b2 = data[i + 2] as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(alpha[((n >> 18) & 63) as usize]);
        out.push(alpha[((n >> 12) & 63) as usize]);
        out.push(alpha[((n >> 6) & 63) as usize]);
        out.push(alpha[(n & 63) as usize]);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(alpha[((n >> 18) & 63) as usize]);
        out.push(alpha[((n >> 12) & 63) as usize]);
        if pad {
            out.push(b'=');
            out.push(b'=');
        }
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(alpha[((n >> 18) & 63) as usize]);
        out.push(alpha[((n >> 12) & 63) as usize]);
        out.push(alpha[((n >> 6) & 63) as usize]);
        if pad {
            out.push(b'=');
        }
    }

    // Apply line wrapping if configured
    if let Some(width) = config.line_wrap {
        if width > 0 {
            return apply_line_wrap(&out, width);
        }
    }

    // SAFETY: `out` only contains ASCII alpha, digits, `+/=` or `-_=`
    // so it is always valid UTF-8.
    unsafe { String::from_utf8_unchecked(out) }
}

/// Insert `\r\n` every `width` bytes of raw base64 output.
fn apply_line_wrap(raw: &[u8], width: usize) -> String {
    let num_breaks = if raw.is_empty() {
        0
    } else {
        (raw.len() - 1) / width
    };
    let mut wrapped = Vec::with_capacity(raw.len() + num_breaks * 2);
    for (idx, chunk) in raw.chunks(width).enumerate() {
        if idx > 0 {
            wrapped.push(b'\r');
            wrapped.push(b'\n');
        }
        wrapped.extend_from_slice(chunk);
    }
    // SAFETY: same ASCII guarantee as above.
    unsafe { String::from_utf8_unchecked(wrapped) }
}

// ---------------------------------------------------------------------------
// Core decode (configurable, whitespace-tolerant)
// ---------------------------------------------------------------------------

/// Decode a base64 string using the given configuration.
///
/// Whitespace (spaces, tabs, `\r`, `\n`) is silently stripped before
/// decoding, so MIME-wrapped input is handled transparently.
pub fn base64_decode_config(s: &str, config: &Base64Config) -> Result<Vec<u8>, Base64DecodeError> {
    let table = config.decode_table();

    // Strip whitespace and collect clean bytes
    let clean: Vec<u8> = s
        .bytes()
        .filter(|&b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .collect();

    if clean.is_empty() {
        return Ok(Vec::new());
    }

    // Strip trailing padding and record how many we had
    let pad_count = clean.iter().rev().take_while(|&&b| b == b'=').count();
    if pad_count > 2 {
        return Err(Base64DecodeError::InvalidPadding);
    }
    let body_len = clean.len() - pad_count;

    // Validate: padding must only appear at the end
    for (pos, &b) in clean[..body_len].iter().enumerate() {
        if b == b'=' {
            return Err(Base64DecodeError::InvalidPadding);
        }
        if table[b as usize] == 255 {
            return Err(Base64DecodeError::InvalidCharacter {
                byte: b,
                position: pos,
            });
        }
    }

    // Determine effective length (with virtual padding to make mod-4 == 0)
    let effective_len = body_len + pad_count;
    let remainder = effective_len % 4;
    // If padded, must be multiple of 4.
    if pad_count > 0 && remainder != 0 {
        return Err(Base64DecodeError::InvalidLength {
            length: effective_len,
        });
    }

    // For un-padded input, figure out the virtual padding
    let total_chars = body_len;
    let tail_chars = if pad_count > 0 {
        0 // all quads are complete (padded)
    } else {
        total_chars % 4
    };

    if tail_chars == 1 {
        return Err(Base64DecodeError::InvalidLength { length: body_len });
    }

    let full_quads = if pad_count > 0 {
        (effective_len / 4).saturating_sub(1)
    } else {
        total_chars / 4
    };

    let out_len = full_quads * 3
        + if pad_count > 0 {
            3 - pad_count
        } else {
            match tail_chars {
                2 => 1,
                3 => 2,
                _ => 0,
            }
        };
    let mut out = Vec::with_capacity(out_len);

    // Decode full 4-char groups (no padding)
    let mut pos = 0;
    for _ in 0..full_quads {
        let c0 = table[clean[pos] as usize] as u32;
        let c1 = table[clean[pos + 1] as usize] as u32;
        let c2 = table[clean[pos + 2] as usize] as u32;
        let c3 = table[clean[pos + 3] as usize] as u32;
        let n = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;
        out.push(((n >> 16) & 0xFF) as u8);
        out.push(((n >> 8) & 0xFF) as u8);
        out.push((n & 0xFF) as u8);
        pos += 4;
    }

    // Handle padded last group
    if pad_count > 0 {
        let c0 = table[clean[pos] as usize] as u32;
        let c1 = table[clean[pos + 1] as usize] as u32;
        let n = if pad_count == 2 {
            (c0 << 18) | (c1 << 12)
        } else {
            let c2 = table[clean[pos + 2] as usize] as u32;
            (c0 << 18) | (c1 << 12) | (c2 << 6)
        };
        out.push(((n >> 16) & 0xFF) as u8);
        if pad_count < 2 {
            out.push(((n >> 8) & 0xFF) as u8);
        }
    }

    // Handle un-padded tail (remainder chars without padding)
    if tail_chars == 2 {
        let c0 = table[clean[pos] as usize] as u32;
        let c1 = table[clean[pos + 1] as usize] as u32;
        let n = (c0 << 18) | (c1 << 12);
        out.push(((n >> 16) & 0xFF) as u8);
    } else if tail_chars == 3 {
        let c0 = table[clean[pos] as usize] as u32;
        let c1 = table[clean[pos + 1] as usize] as u32;
        let c2 = table[clean[pos + 2] as usize] as u32;
        let n = (c0 << 18) | (c1 << 12) | (c2 << 6);
        out.push(((n >> 16) & 0xFF) as u8);
        out.push(((n >> 8) & 0xFF) as u8);
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Original public API (backward compatible)
// ---------------------------------------------------------------------------

/// Encode bytes to standard padded Base64 (no line wrapping).
pub fn base64_encode(data: &[u8]) -> String {
    base64_encode_config(data, &Base64Config::standard())
}

/// Decode a standard padded Base64 string.
pub fn base64_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    base64_decode_config(s, &Base64Config::standard()).map_err(|e| match e {
        Base64DecodeError::InvalidCharacter { .. } => "invalid char",
        Base64DecodeError::InvalidPadding => "invalid base64",
        Base64DecodeError::InvalidLength { .. } => "invalid base64",
    })
}

/// Original validation function (backward compatible).
pub fn base64_is_valid(s: &str) -> bool {
    let b = s.as_bytes();
    if !b.len().is_multiple_of(4) {
        return false;
    }
    for (i, &ch) in b.iter().enumerate() {
        if ch == b'=' {
            if i < b.len().saturating_sub(2) {
                return false;
            }
        } else if decode_char(ch).is_none() {
            return false;
        }
    }
    true
}

/// Backward-compatible single-character decode (standard alphabet).
fn decode_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'=' => Some(0),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// URL-safe convenience functions
// ---------------------------------------------------------------------------

/// Encode bytes to URL-safe Base64 without padding.
pub fn base64_encode_url_safe(data: &[u8]) -> String {
    base64_encode_config(data, &Base64Config::url_safe())
}

/// Decode a URL-safe Base64 string (padding optional).
pub fn base64_decode_url_safe(s: &str) -> Result<Vec<u8>, Base64DecodeError> {
    base64_decode_config(s, &Base64Config::url_safe())
}

/// Encode bytes to MIME Base64 (padded, line-wrapped at 76 chars).
pub fn base64_encode_mime(data: &[u8]) -> String {
    base64_encode_config(data, &Base64Config::mime())
}

// ---------------------------------------------------------------------------
// Streaming encoder
// ---------------------------------------------------------------------------

/// A streaming Base64 encoder that wraps an [`io::Write`] sink.
pub struct Base64Encoder<W: Write> {
    inner: ManuallyDrop<W>,
    config: Base64Config,
    buf: [u8; 2],
    buf_len: usize,
    line_pos: usize,
    finished: bool,
}

impl<W: Write> Base64Encoder<W> {
    /// Create a new streaming encoder with the given configuration.
    pub fn new(inner: W, config: Base64Config) -> Self {
        Self {
            inner: ManuallyDrop::new(inner),
            config,
            buf: [0; 2],
            buf_len: 0,
            line_pos: 0,
            finished: false,
        }
    }

    /// Finish encoding, flushing any buffered partial group and
    /// returning the inner writer.
    pub fn finish(mut self) -> io::Result<W> {
        self.flush_final()?;
        self.finished = true;
        // SAFETY: after setting finished=true, Drop will not call flush_final
        // again, and we take ownership of inner before drop runs.
        Ok(unsafe { ManuallyDrop::take(&mut self.inner) })
    }

    fn flush_final(&mut self) -> io::Result<()> {
        if self.buf_len == 0 {
            return Ok(());
        }
        let alpha = self.config.alphabet();
        let pad = self.config.emit_padding();
        let mut quartet = [0u8; 4];
        let written = if self.buf_len == 1 {
            let n = (self.buf[0] as u32) << 16;
            quartet[0] = alpha[((n >> 18) & 63) as usize];
            quartet[1] = alpha[((n >> 12) & 63) as usize];
            if pad {
                quartet[2] = b'=';
                quartet[3] = b'=';
                4
            } else {
                2
            }
        } else {
            let n = ((self.buf[0] as u32) << 16) | ((self.buf[1] as u32) << 8);
            quartet[0] = alpha[((n >> 18) & 63) as usize];
            quartet[1] = alpha[((n >> 12) & 63) as usize];
            quartet[2] = alpha[((n >> 6) & 63) as usize];
            if pad {
                quartet[3] = b'=';
                4
            } else {
                3
            }
        };
        self.write_wrapped(&quartet[..written])?;
        self.buf_len = 0;
        Ok(())
    }

    fn write_wrapped(&mut self, data: &[u8]) -> io::Result<()> {
        let width = match self.config.line_wrap {
            Some(w) if w > 0 => w,
            _ => {
                self.inner.write_all(data)?;
                self.line_pos += data.len();
                return Ok(());
            }
        };

        let mut offset = 0;
        while offset < data.len() {
            let remaining_on_line = width.saturating_sub(self.line_pos);
            if remaining_on_line == 0 {
                self.inner.write_all(b"\r\n")?;
                self.line_pos = 0;
                continue;
            }
            let chunk = std::cmp::min(remaining_on_line, data.len() - offset);
            self.inner.write_all(&data[offset..offset + chunk])?;
            self.line_pos += chunk;
            offset += chunk;
        }
        Ok(())
    }

    fn encode_triple(&mut self, b0: u8, b1: u8, b2: u8) -> io::Result<()> {
        let alpha = self.config.alphabet();
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        let quartet = [
            alpha[((n >> 18) & 63) as usize],
            alpha[((n >> 12) & 63) as usize],
            alpha[((n >> 6) & 63) as usize],
            alpha[(n & 63) as usize],
        ];
        self.write_wrapped(&quartet)
    }
}

impl<W: Write> Write for Base64Encoder<W> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if input.is_empty() {
            return Ok(0);
        }

        let mut pos = 0;

        if self.buf_len == 1 {
            if pos < input.len() {
                let b1 = input[pos];
                pos += 1;
                if pos < input.len() {
                    let b2 = input[pos];
                    pos += 1;
                    self.encode_triple(self.buf[0], b1, b2)?;
                    self.buf_len = 0;
                } else {
                    self.buf[1] = b1;
                    self.buf_len = 2;
                    return Ok(input.len());
                }
            }
        } else if self.buf_len == 2 {
            if pos < input.len() {
                let b2 = input[pos];
                pos += 1;
                self.encode_triple(self.buf[0], self.buf[1], b2)?;
                self.buf_len = 0;
            } else {
                return Ok(input.len());
            }
        }

        let remaining = &input[pos..];
        let full_triples = remaining.len() / 3;
        for i in 0..full_triples {
            let base = i * 3;
            self.encode_triple(remaining[base], remaining[base + 1], remaining[base + 2])?;
        }

        let leftover_start = pos + full_triples * 3;
        let leftover = input.len() - leftover_start;
        if leftover >= 1 {
            self.buf[0] = input[leftover_start];
            self.buf_len = 1;
            if leftover >= 2 {
                self.buf[1] = input[leftover_start + 1];
                self.buf_len = 2;
            }
        }

        Ok(input.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write> Drop for Base64Encoder<W> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.flush_final();
        }
        // SAFETY: drop is called exactly once, and if finish() was called
        // it already took inner out. If not, we drop it here.
        if !self.finished {
            unsafe {
                ManuallyDrop::drop(&mut self.inner);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Streaming decoder
// ---------------------------------------------------------------------------

/// A streaming Base64 decoder that wraps an [`io::Read`] source.
pub struct Base64Decoder<R: Read> {
    inner: R,
    config: Base64Config,
    out_buf: Vec<u8>,
    out_pos: usize,
    quad: [u8; 4],
    quad_len: usize,
    finished: bool,
}

impl<R: Read> Base64Decoder<R> {
    /// Create a new streaming decoder.
    pub fn new(inner: R, config: Base64Config) -> Self {
        Self {
            inner,
            config,
            out_buf: Vec::with_capacity(768),
            out_pos: 0,
            quad: [0; 4],
            quad_len: 0,
            finished: false,
        }
    }

    fn decode_quad_full(&mut self) -> io::Result<()> {
        let table = self.config.decode_table();
        let q = &self.quad;
        let pad_count = q.iter().rev().take_while(|&&b| b == b'=').count();
        let c0 = table[q[0] as usize] as u32;
        let c1 = table[q[1] as usize] as u32;
        let c2 = if q[2] == b'=' {
            0u32
        } else {
            table[q[2] as usize] as u32
        };
        let c3 = if q[3] == b'=' {
            0u32
        } else {
            table[q[3] as usize] as u32
        };
        let n = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;
        self.out_buf.push(((n >> 16) & 0xFF) as u8);
        if pad_count < 2 {
            self.out_buf.push(((n >> 8) & 0xFF) as u8);
        }
        if pad_count == 0 {
            self.out_buf.push((n & 0xFF) as u8);
        }
        if pad_count > 0 {
            self.finished = true;
        }
        self.quad_len = 0;
        Ok(())
    }

    fn decode_quad_partial(&mut self) -> io::Result<()> {
        let table = self.config.decode_table();
        let q = &self.quad;
        let qlen = self.quad_len;
        match qlen {
            0 => {}
            2 => {
                let c0 = table[q[0] as usize] as u32;
                let c1 = table[q[1] as usize] as u32;
                let n = (c0 << 18) | (c1 << 12);
                self.out_buf.push(((n >> 16) & 0xFF) as u8);
            }
            3 => {
                let c0 = table[q[0] as usize] as u32;
                let c1 = table[q[1] as usize] as u32;
                let c2 = table[q[2] as usize] as u32;
                let n = (c0 << 18) | (c1 << 12) | (c2 << 6);
                self.out_buf.push(((n >> 16) & 0xFF) as u8);
                self.out_buf.push(((n >> 8) & 0xFF) as u8);
            }
            1 => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid base64 stream: incomplete quad (1 char)",
                ));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid base64 stream: unexpected quad length",
                ));
            }
        }
        self.quad_len = 0;
        self.finished = true;
        Ok(())
    }

    fn drain_output(&mut self, buf: &mut [u8]) -> usize {
        let avail = self.out_buf.len() - self.out_pos;
        if avail == 0 {
            return 0;
        }
        let n = std::cmp::min(avail, buf.len());
        buf[..n].copy_from_slice(&self.out_buf[self.out_pos..self.out_pos + n]);
        self.out_pos += n;
        if self.out_pos == self.out_buf.len() {
            self.out_buf.clear();
            self.out_pos = 0;
        }
        n
    }
}

impl<R: Read> Read for Base64Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let drained = self.drain_output(buf);
        if drained > 0 {
            return Ok(drained);
        }

        if self.finished {
            return Ok(0);
        }

        let table = self.config.decode_table();

        let mut read_buf = [0u8; 1024];
        let bytes_read = self.inner.read(&mut read_buf)?;
        if bytes_read == 0 {
            if self.quad_len > 0 {
                self.decode_quad_partial()?;
            } else {
                self.finished = true;
            }
            let n = self.drain_output(buf);
            if n == 0 {
                self.finished = true;
            }
            return Ok(n);
        }

        for &b in &read_buf[..bytes_read] {
            if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
                continue;
            }
            if b != b'=' && table[b as usize] == 255 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid base64 character: 0x{b:02X}"),
                ));
            }
            self.quad[self.quad_len] = b;
            self.quad_len += 1;
            if self.quad_len == 4 {
                self.decode_quad_full()?;
            }
        }

        let n = self.drain_output(buf);
        if n == 0 && !self.finished {
            return self.read(buf);
        }
        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Check if a string is valid base64 for the given configuration.
pub fn base64_is_valid_config(s: &str, config: &Base64Config) -> bool {
    base64_decode_config(s, config).is_ok()
}

/// Validate that a string is valid URL-safe base64.
pub fn base64_is_valid_url_safe(s: &str) -> bool {
    base64_decode_config(s, &Base64Config::url_safe()).is_ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn encode_decode_roundtrip() {
        let data = b"Hello, World!";
        let enc = base64_encode(data);
        let dec = base64_decode(&enc).expect("should succeed");
        assert_eq!(dec, data);
    }

    #[test]
    fn encode_one_byte() {
        let enc = base64_encode(b"M");
        assert_eq!(enc, "TQ==");
    }

    #[test]
    fn encode_two_bytes() {
        let enc = base64_encode(b"Ma");
        assert_eq!(enc, "TWE=");
    }

    #[test]
    fn is_valid_true() {
        assert!(base64_is_valid("SGVsbG8="));
    }

    #[test]
    fn is_valid_false_bad_char() {
        assert!(!base64_is_valid("SG?s"));
    }

    #[test]
    fn decoded_len_estimate() {
        assert_eq!(base64_decoded_len(8), 6);
    }

    #[test]
    fn url_safe_encode_decode() {
        let data = b"\xfb\xff\xfe";
        let standard = base64_encode(data);
        assert!(standard.contains('+') || standard.contains('/'));

        let url = base64_encode_url_safe(data);
        assert!(!url.contains('+'));
        assert!(!url.contains('/'));

        let decoded = base64_decode_url_safe(&url).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn url_safe_roundtrip_various() {
        for input in &[b"" as &[u8], b"a", b"ab", b"abc", b"abcd", b"Hello, World!"] {
            let enc = base64_encode_url_safe(input);
            let dec = base64_decode_url_safe(&enc).expect("should succeed");
            assert_eq!(&dec, input, "failed for input len={}", input.len());
        }
    }

    #[test]
    fn no_padding_encode() {
        let config = Base64Config {
            variant: Base64Variant::Standard,
            padding: Base64Padding::NoPad,
            line_wrap: None,
        };
        let enc = base64_encode_config(b"M", &config);
        assert_eq!(enc, "TQ");
        let enc2 = base64_encode_config(b"Ma", &config);
        assert_eq!(enc2, "TWE");

        let dec = base64_decode_config("TQ", &config).expect("should succeed");
        assert_eq!(dec, b"M");
        let dec2 = base64_decode_config("TWE", &config).expect("should succeed");
        assert_eq!(dec2, b"Ma");
    }

    #[test]
    fn no_padding_accepts_padding_on_decode() {
        let config = Base64Config {
            variant: Base64Variant::Standard,
            padding: Base64Padding::NoPad,
            line_wrap: None,
        };
        let dec = base64_decode_config("TQ==", &config).expect("should succeed");
        assert_eq!(dec, b"M");
    }

    #[test]
    fn line_wrapping_mime() {
        let data = vec![0xAA; 57];
        let enc = base64_encode_mime(&data);
        assert!(
            !enc.contains("\r\n"),
            "57 bytes should fit in one 76-char line"
        );
        assert_eq!(enc.len(), 76);

        let data2 = vec![0xBB; 58];
        let enc2 = base64_encode_mime(&data2);
        assert!(enc2.contains("\r\n"), "58 bytes should cause wrapping");

        let dec = base64_decode_config(&enc2, &Base64Config::standard()).expect("should succeed");
        assert_eq!(dec, data2);
    }

    #[test]
    fn whitespace_tolerance() {
        let encoded = "SGVs\r\n bG8s\tIFdv\ncmxkIQ==";
        let dec = base64_decode_config(encoded, &Base64Config::standard()).expect("should succeed");
        assert_eq!(dec, b"Hello, World!");
    }

    #[test]
    fn error_invalid_char() {
        let result = base64_decode_config("SG?s", &Base64Config::standard());
        match result {
            Err(Base64DecodeError::InvalidCharacter { byte, .. }) => {
                assert_eq!(byte, b'?');
            }
            other => panic!("expected InvalidCharacter, got {other:?}"),
        }
    }

    #[test]
    fn error_invalid_padding() {
        let result = base64_decode_config("S=Gs", &Base64Config::standard());
        assert!(matches!(result, Err(Base64DecodeError::InvalidPadding)));
    }

    #[test]
    fn error_invalid_length() {
        let result = base64_decode_config("A", &Base64Config::standard());
        assert!(matches!(
            result,
            Err(Base64DecodeError::InvalidLength { .. })
        ));
    }

    #[test]
    fn error_display() {
        let e1 = Base64DecodeError::InvalidCharacter {
            byte: 0x3F,
            position: 2,
        };
        assert_eq!(
            format!("{e1}"),
            "invalid base64 character 0x3F at position 2"
        );
        let e2 = Base64DecodeError::InvalidPadding;
        assert_eq!(format!("{e2}"), "invalid base64 padding");
        let e3 = Base64DecodeError::InvalidLength { length: 5 };
        assert_eq!(format!("{e3}"), "invalid base64 length 5 (mod 4 == 1)");
    }

    #[test]
    fn streaming_encode_matches_oneshot() {
        let data = b"Hello, World! This is a streaming test with enough data.";
        let expected = base64_encode(data);

        let mut output = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut output, Base64Config::standard());
            for chunk in data.chunks(7) {
                encoder.write_all(chunk).expect("should succeed");
            }
            let _ = encoder.finish().expect("should succeed");
        }
        let result = String::from_utf8(output).expect("should succeed");
        assert_eq!(result, expected);
    }

    #[test]
    fn streaming_encode_single_byte_writes() {
        let data = b"ABC";
        let expected = base64_encode(data);

        let mut output = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut output, Base64Config::standard());
            for &b in data.iter() {
                encoder.write_all(&[b]).expect("should succeed");
            }
            let _ = encoder.finish().expect("should succeed");
        }
        let result = String::from_utf8(output).expect("should succeed");
        assert_eq!(result, expected);
    }

    #[test]
    fn streaming_encode_with_wrapping() {
        let data = vec![0xCC; 120];
        let expected = base64_encode_mime(&data);

        let mut output = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut output, Base64Config::mime());
            encoder.write_all(&data).expect("should succeed");
            let _ = encoder.finish().expect("should succeed");
        }
        let result = String::from_utf8(output).expect("should succeed");
        assert_eq!(result, expected);
    }

    #[test]
    fn streaming_decode_matches_oneshot() {
        let original = b"Streaming decode test data!";
        let encoded = base64_encode(original);

        let cursor = Cursor::new(encoded.as_bytes());
        let mut decoder = Base64Decoder::new(cursor, Base64Config::standard());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).expect("should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn streaming_decode_with_whitespace() {
        let original = b"whitespace tolerant streaming";
        let encoded = base64_encode(original);
        let with_ws: String = encoded
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 10 == 0 {
                    vec!['\n', c]
                } else {
                    vec![c]
                }
            })
            .collect();

        let cursor = Cursor::new(with_ws.as_bytes());
        let mut decoder = Base64Decoder::new(cursor, Base64Config::standard());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).expect("should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn streaming_encode_url_safe_no_pad() {
        let data = b"url safe streaming!";
        let expected = base64_encode_url_safe(data);

        let mut output = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut output, Base64Config::url_safe());
            encoder.write_all(data).expect("should succeed");
            let _ = encoder.finish().expect("should succeed");
        }
        let result = String::from_utf8(output).expect("should succeed");
        assert_eq!(result, expected);
    }

    #[test]
    fn streaming_decode_url_safe() {
        let original = b"url safe decode test";
        let encoded = base64_encode_url_safe(original);

        let cursor = Cursor::new(encoded.as_bytes());
        let mut decoder = Base64Decoder::new(cursor, Base64Config::url_safe());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).expect("should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn roundtrip_all_byte_values() {
        let data: Vec<u8> = (0..=255).collect();
        let enc = base64_encode(&data);
        let dec = base64_decode(&enc).expect("should succeed");
        assert_eq!(dec, data);

        let enc_url = base64_encode_url_safe(&data);
        let dec_url = base64_decode_url_safe(&enc_url).expect("should succeed");
        assert_eq!(dec_url, data);
    }

    #[test]
    fn config_presets() {
        let s = Base64Config::standard();
        assert_eq!(s.variant, Base64Variant::Standard);
        assert_eq!(s.padding, Base64Padding::Pad);
        assert!(s.line_wrap.is_none());

        let m = Base64Config::mime();
        assert_eq!(m.line_wrap, Some(76));

        let u = Base64Config::url_safe();
        assert_eq!(u.variant, Base64Variant::UrlSafe);
        assert_eq!(u.padding, Base64Padding::NoPad);

        let up = Base64Config::url_safe_padded();
        assert_eq!(up.variant, Base64Variant::UrlSafe);
        assert_eq!(up.padding, Base64Padding::Pad);
    }

    #[test]
    fn is_valid_url_safe() {
        let enc = base64_encode_url_safe(b"test");
        assert!(base64_is_valid_url_safe(&enc));
        assert!(!base64_is_valid_url_safe("not+valid/url=safe=="));
    }

    #[test]
    fn decode_empty() {
        let dec = base64_decode("").expect("should succeed");
        assert!(dec.is_empty());

        let dec2 = base64_decode_config("", &Base64Config::url_safe()).expect("should succeed");
        assert!(dec2.is_empty());
    }

    #[test]
    fn encoded_len_correctness() {
        assert_eq!(base64_encoded_len(0), 0);
        assert_eq!(base64_encoded_len(1), 4);
        assert_eq!(base64_encoded_len(2), 4);
        assert_eq!(base64_encoded_len(3), 4);
        assert_eq!(base64_encoded_len(4), 8);
    }

    #[test]
    fn base64_error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(Base64DecodeError::InvalidPadding);
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn streaming_roundtrip_large() {
        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
        let encoded = base64_encode(&data);

        let mut enc_out = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut enc_out, Base64Config::standard());
            for chunk in data.chunks(100) {
                encoder.write_all(chunk).expect("should succeed");
            }
            let _ = encoder.finish().expect("should succeed");
        }
        assert_eq!(String::from_utf8(enc_out).expect("should succeed"), encoded);

        let cursor = Cursor::new(encoded.as_bytes());
        let mut decoder = Base64Decoder::new(cursor, Base64Config::standard());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).expect("should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn backward_compat_decode_error() {
        let result = base64_decode("????");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(!err_msg.is_empty());
    }
}
