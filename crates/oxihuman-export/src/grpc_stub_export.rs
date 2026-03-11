// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! gRPC request/response serialization with full protocol support.
//!
//! Implements gRPC framing (length-prefixed messages), status codes per the
//! gRPC spec, trailing metadata, stream framing for all four RPC patterns,
//! gRPC-Web text (base64) encoding, error detail encoding, and serialization
//! helpers for mesh/morph data payloads.

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// gRPC Status Codes (all 17 per gRPC spec)
// ---------------------------------------------------------------------------

/// All 17 gRPC status codes as defined in the gRPC protocol specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
#[derive(Default)]
pub enum GrpcStatusCode {
    /// Not an error; returned on success.
    #[default]
    Ok = 0,
    /// The operation was cancelled, typically by the caller.
    Cancelled = 1,
    /// Unknown error.
    Unknown = 2,
    /// The client specified an invalid argument.
    InvalidArgument = 3,
    /// The deadline expired before the operation could complete.
    DeadlineExceeded = 4,
    /// Some requested entity was not found.
    NotFound = 5,
    /// The entity that a client attempted to create already exists.
    AlreadyExists = 6,
    /// The caller does not have permission.
    PermissionDenied = 7,
    /// Some resource has been exhausted.
    ResourceExhausted = 8,
    /// The system is not in a state required for the operation.
    FailedPrecondition = 9,
    /// The operation was aborted.
    Aborted = 10,
    /// Operation was attempted past the valid range.
    OutOfRange = 11,
    /// The operation is not implemented.
    Unimplemented = 12,
    /// Internal errors — a serious fault in the system.
    Internal = 13,
    /// The service is currently unavailable.
    Unavailable = 14,
    /// Unrecoverable data loss or corruption.
    DataLoss = 15,
    /// The request does not have valid authentication credentials.
    Unauthenticated = 16,
}

impl GrpcStatusCode {
    /// Convert a raw u32 value to a status code, returning `Unknown` for
    /// unrecognised values.
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::Ok,
            1 => Self::Cancelled,
            2 => Self::Unknown,
            3 => Self::InvalidArgument,
            4 => Self::DeadlineExceeded,
            5 => Self::NotFound,
            6 => Self::AlreadyExists,
            7 => Self::PermissionDenied,
            8 => Self::ResourceExhausted,
            9 => Self::FailedPrecondition,
            10 => Self::Aborted,
            11 => Self::OutOfRange,
            12 => Self::Unimplemented,
            13 => Self::Internal,
            14 => Self::Unavailable,
            15 => Self::DataLoss,
            16 => Self::Unauthenticated,
            _ => Self::Unknown,
        }
    }

    /// Canonical string name of the status code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Cancelled => "CANCELLED",
            Self::Unknown => "UNKNOWN",
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::DeadlineExceeded => "DEADLINE_EXCEEDED",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::Aborted => "ABORTED",
            Self::OutOfRange => "OUT_OF_RANGE",
            Self::Unimplemented => "UNIMPLEMENTED",
            Self::Internal => "INTERNAL",
            Self::Unavailable => "UNAVAILABLE",
            Self::DataLoss => "DATA_LOSS",
            Self::Unauthenticated => "UNAUTHENTICATED",
        }
    }

    /// Whether this code represents a successful outcome.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

impl fmt::Display for GrpcStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.as_str(), *self as u32)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by gRPC frame decoding and serialization.
#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("insufficient data: need {needed} bytes, got {available}")]
    InsufficientData { needed: usize, available: usize },

    #[error("unsupported compression flag: {0}")]
    UnsupportedCompression(u8),

    #[error("body length mismatch: header says {expected}, buffer has {available}")]
    BodyLengthMismatch { expected: usize, available: usize },

    #[error("base64 decode error: {0}")]
    Base64Decode(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("invalid metadata key: {0}")]
    InvalidMetadataKey(String),
}

// ---------------------------------------------------------------------------
// Compression flag
// ---------------------------------------------------------------------------

/// gRPC compression flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcCompression {
    None = 0,
    Gzip = 1,
}

impl GrpcCompression {
    /// Parse from the first byte of a gRPC frame.
    pub fn from_byte(b: u8) -> Result<Self, GrpcError> {
        match b {
            0 => Ok(Self::None),
            1 => Ok(Self::Gzip),
            other => Err(GrpcError::UnsupportedCompression(other)),
        }
    }
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

/// A single metadata entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

impl MetadataEntry {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Whether this is a binary metadata entry (key ends with `-bin`).
    pub fn is_binary(&self) -> bool {
        self.key.ends_with("-bin")
    }
}

/// Ordered collection of metadata entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    entries: Vec<MetadataEntry>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an entry. Duplicate keys are allowed per gRPC spec.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.push(MetadataEntry::new(key, value));
    }

    /// Get the first value for a given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.key == key)
            .map(|e| e.value.as_str())
    }

    /// Get all values for a given key.
    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| e.key == key)
            .map(|e| e.value.as_str())
            .collect()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = &MetadataEntry> {
        self.entries.iter()
    }

    /// Encode trailing metadata into the gRPC wire format.
    /// Each entry is `key: value\r\n`.
    pub fn encode_trailers(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for e in &self.entries {
            buf.extend_from_slice(e.key.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(e.value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }
        buf
    }

    /// Parse trailing metadata from the wire format.
    pub fn decode_trailers(data: &[u8]) -> Self {
        let text = String::from_utf8_lossy(data);
        let mut md = Metadata::new();
        for line in text.lines() {
            if let Some((k, v)) = line.split_once(':') {
                md.insert(k.trim(), v.trim());
            }
        }
        md
    }
}

// ---------------------------------------------------------------------------
// gRPC Frame
// ---------------------------------------------------------------------------

/// A gRPC message frame (5-byte header + body).
#[derive(Debug, Clone)]
pub struct GrpcFrame {
    pub compression: GrpcCompression,
    pub body: Vec<u8>,
}

impl GrpcFrame {
    /// Create an uncompressed frame.
    pub fn new(body: Vec<u8>) -> Self {
        GrpcFrame {
            compression: GrpcCompression::None,
            body,
        }
    }

    /// Create a frame with an explicit compression setting.
    pub fn with_compression(body: Vec<u8>, compression: GrpcCompression) -> Self {
        GrpcFrame { compression, body }
    }

    /// Encode the frame to bytes (5-byte header + body).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.body.len());
        out.push(self.compression as u8);
        out.extend_from_slice(&(self.body.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.body);
        out
    }

    /// Decode a single frame from the front of `data`.
    /// Returns the frame and the number of bytes consumed.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), GrpcError> {
        if data.len() < 5 {
            return Err(GrpcError::InsufficientData {
                needed: 5,
                available: data.len(),
            });
        }
        let compression = GrpcCompression::from_byte(data[0])?;
        let body_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let total = 5 + body_len;
        if data.len() < total {
            return Err(GrpcError::BodyLengthMismatch {
                expected: body_len,
                available: data.len() - 5,
            });
        }
        let body = data[5..total].to_vec();
        Ok((GrpcFrame { compression, body }, total))
    }

    /// Decode all frames from a contiguous byte buffer.
    pub fn decode_all(data: &[u8]) -> Result<Vec<Self>, GrpcError> {
        let mut frames = Vec::new();
        let mut offset = 0;
        while offset < data.len() {
            let (frame, consumed) = Self::decode(&data[offset..])?;
            frames.push(frame);
            offset += consumed;
        }
        Ok(frames)
    }

    /// Byte length of the encoded frame.
    pub fn encoded_len(&self) -> usize {
        5 + self.body.len()
    }
}

// ---------------------------------------------------------------------------
// Trailing metadata as a gRPC trailers frame
// ---------------------------------------------------------------------------

/// Encode a [`Metadata`] as a gRPC *trailers* frame (compression flag byte
/// is `0x80` to signal trailers-only, body is the encoded header block).
pub fn encode_trailers_frame(md: &Metadata) -> Vec<u8> {
    let body = md.encode_trailers();
    let mut out = Vec::with_capacity(5 + body.len());
    out.push(0x80);
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

/// Decode a trailers frame, returning the metadata.
pub fn decode_trailers_frame(data: &[u8]) -> Result<(Metadata, usize), GrpcError> {
    if data.len() < 5 {
        return Err(GrpcError::InsufficientData {
            needed: 5,
            available: data.len(),
        });
    }
    let body_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    let total = 5 + body_len;
    if data.len() < total {
        return Err(GrpcError::BodyLengthMismatch {
            expected: body_len,
            available: data.len() - 5,
        });
    }
    let md = Metadata::decode_trailers(&data[5..total]);
    Ok((md, total))
}

/// Check whether a raw frame byte-slice is a trailers frame (bit 7 set).
pub fn is_trailers_frame(data: &[u8]) -> bool {
    data.first().is_some_and(|b| b & 0x80 != 0)
}

// ---------------------------------------------------------------------------
// Stream framing helpers
// ---------------------------------------------------------------------------

/// Streaming pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Unary,
    ServerStreaming,
    ClientStreaming,
    BidiStreaming,
}

impl fmt::Display for StreamKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unary => write!(f, "unary"),
            Self::ServerStreaming => write!(f, "server-streaming"),
            Self::ClientStreaming => write!(f, "client-streaming"),
            Self::BidiStreaming => write!(f, "bidi-streaming"),
        }
    }
}

/// Framed gRPC stream — a sequence of message frames followed by an optional
/// trailers frame.
#[derive(Debug, Clone)]
pub struct GrpcStream {
    pub kind: StreamKind,
    pub messages: Vec<GrpcFrame>,
    pub trailing_metadata: Metadata,
    pub status: GrpcStatusCode,
    pub status_message: String,
}

impl GrpcStream {
    /// Create a new empty stream.
    pub fn new(kind: StreamKind) -> Self {
        Self {
            kind,
            messages: Vec::new(),
            trailing_metadata: Metadata::new(),
            status: GrpcStatusCode::Ok,
            status_message: String::new(),
        }
    }

    /// Append a message body to the stream.
    pub fn push_message(&mut self, body: Vec<u8>) {
        self.messages.push(GrpcFrame::new(body));
    }

    /// Append a pre-built frame.
    pub fn push_frame(&mut self, frame: GrpcFrame) {
        self.messages.push(frame);
    }

    /// Set the stream status.
    pub fn set_status(&mut self, code: GrpcStatusCode, message: impl Into<String>) {
        self.status = code;
        self.status_message = message.into();
    }

    /// Encode the entire stream to bytes: message frames followed by the
    /// trailers frame that carries `grpc-status` and `grpc-message`.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for frame in &self.messages {
            buf.extend_from_slice(&frame.encode());
        }
        let mut trailers = self.trailing_metadata.clone();
        trailers.insert("grpc-status", format!("{}", self.status as u32));
        if !self.status_message.is_empty() {
            trailers.insert("grpc-message", &self.status_message);
        }
        buf.extend_from_slice(&encode_trailers_frame(&trailers));
        buf
    }

    /// Decode a stream from raw bytes.
    pub fn decode(data: &[u8], kind: StreamKind) -> Result<Self, GrpcError> {
        let mut stream = Self::new(kind);
        let mut offset = 0;
        while offset < data.len() {
            if is_trailers_frame(&data[offset..]) {
                let (md, consumed) = decode_trailers_frame(&data[offset..])?;
                if let Some(status_str) = md.get("grpc-status") {
                    if let Ok(code) = status_str.parse::<u32>() {
                        stream.status = GrpcStatusCode::from_u32(code);
                    }
                }
                if let Some(msg) = md.get("grpc-message") {
                    stream.status_message = msg.to_owned();
                }
                for e in md.iter() {
                    if e.key != "grpc-status" && e.key != "grpc-message" {
                        stream.trailing_metadata.insert(&e.key, &e.value);
                    }
                }
                offset += consumed;
            } else {
                let (frame, consumed) = GrpcFrame::decode(&data[offset..])?;
                stream.messages.push(frame);
                offset += consumed;
            }
        }
        Ok(stream)
    }

    /// Number of data messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

// ---------------------------------------------------------------------------
// Unary / streaming convenience builders
// ---------------------------------------------------------------------------

/// Encode a unary RPC response: single message frame + trailers.
pub fn encode_unary_response(
    body: Vec<u8>,
    status: GrpcStatusCode,
    trailing: &Metadata,
) -> Vec<u8> {
    let mut stream = GrpcStream::new(StreamKind::Unary);
    stream.push_message(body);
    stream.trailing_metadata = trailing.clone();
    stream.status = status;
    stream.encode()
}

/// Encode a server-streaming response from a list of message bodies.
pub fn encode_server_stream(
    bodies: &[Vec<u8>],
    status: GrpcStatusCode,
    trailing: &Metadata,
) -> Vec<u8> {
    let mut stream = GrpcStream::new(StreamKind::ServerStreaming);
    for body in bodies {
        stream.push_message(body.clone());
    }
    stream.trailing_metadata = trailing.clone();
    stream.status = status;
    stream.encode()
}

/// Encode a client-streaming request from a list of message bodies.
pub fn encode_client_stream(bodies: &[Vec<u8>]) -> Vec<u8> {
    let mut buf = Vec::new();
    for body in bodies {
        buf.extend_from_slice(&GrpcFrame::new(body.clone()).encode());
    }
    buf
}

/// Encode a bidi-streaming sequence of frames.
pub fn encode_bidi_frames(bodies: &[Vec<u8>]) -> Vec<u8> {
    let mut buf = Vec::new();
    for body in bodies {
        buf.extend_from_slice(&GrpcFrame::new(body.clone()).encode());
    }
    buf
}

// ---------------------------------------------------------------------------
// gRPC-Web text format (base64-wrapped frames)
// ---------------------------------------------------------------------------

/// Simple Base64 encoding (standard alphabet, with padding).
mod base64_codec {
    const ENCODE_TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub(crate) fn encode(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(ENCODE_TABLE[((triple >> 18) & 0x3F) as usize] as char);
            out.push(ENCODE_TABLE[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                out.push(ENCODE_TABLE[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(ENCODE_TABLE[(triple & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }

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

    pub(crate) fn decode(input: &str) -> Result<Vec<u8>, String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(Vec::new());
        }
        if !input.len().is_multiple_of(4) {
            return Err(format!(
                "invalid base64 length: {} (not a multiple of 4)",
                input.len()
            ));
        }
        let mut out = Vec::with_capacity(input.len() / 4 * 3);
        for chunk in input.as_bytes().chunks(4) {
            let a = decode_char(chunk[0])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[0] as char))?;
            let b = decode_char(chunk[1])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[1] as char))?;
            let c = decode_char(chunk[2])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[2] as char))?;
            let d = decode_char(chunk[3])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[3] as char))?;
            let triple = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
            out.push(((triple >> 16) & 0xFF) as u8);
            if chunk[2] != b'=' {
                out.push(((triple >> 8) & 0xFF) as u8);
            }
            if chunk[3] != b'=' {
                out.push((triple & 0xFF) as u8);
            }
        }
        Ok(out)
    }
}

/// Encode raw gRPC frame bytes into gRPC-Web text format (base64).
pub fn grpc_web_text_encode(data: &[u8]) -> String {
    base64_codec::encode(data)
}

/// Decode gRPC-Web text format (base64) back to raw frame bytes.
pub fn grpc_web_text_decode(text: &str) -> Result<Vec<u8>, GrpcError> {
    base64_codec::decode(text).map_err(GrpcError::Base64Decode)
}

/// Encode a full gRPC stream as a gRPC-Web text response.
pub fn grpc_web_text_encode_stream(stream: &GrpcStream) -> String {
    let raw = stream.encode();
    grpc_web_text_encode(&raw)
}

/// Decode a gRPC-Web text response back into a stream.
pub fn grpc_web_text_decode_stream(text: &str, kind: StreamKind) -> Result<GrpcStream, GrpcError> {
    let raw = grpc_web_text_decode(text)?;
    GrpcStream::decode(&raw, kind)
}

// ---------------------------------------------------------------------------
// Error detail encoding
// ---------------------------------------------------------------------------

/// A structured error detail attached to a gRPC status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcErrorDetail {
    /// Error type URI.
    pub type_url: String,
    /// Human-readable description.
    pub description: String,
    /// Machine-readable metadata.
    pub metadata: Vec<(String, String)>,
}

impl GrpcErrorDetail {
    pub fn new(type_url: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            type_url: type_url.into(),
            description: description.into(),
            metadata: Vec::new(),
        }
    }

    /// Add a metadata key-value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// Structured gRPC error status with optional details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcStatus {
    pub code: u32,
    pub message: String,
    pub details: Vec<GrpcErrorDetail>,
}

impl GrpcStatus {
    pub fn new(code: GrpcStatusCode, message: impl Into<String>) -> Self {
        Self {
            code: code as u32,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(GrpcStatusCode::Ok, "")
    }

    pub fn add_detail(&mut self, detail: GrpcErrorDetail) {
        self.details.push(detail);
    }

    /// Encode the status as a JSON byte vector.
    pub fn encode_json(&self) -> Result<Vec<u8>, GrpcError> {
        serde_json::to_vec(self).map_err(|e| GrpcError::Serialization(e.to_string()))
    }

    /// Decode a status from JSON bytes.
    pub fn decode_json(data: &[u8]) -> Result<Self, GrpcError> {
        serde_json::from_slice(data).map_err(|e| GrpcError::Serialization(e.to_string()))
    }

    /// Status code as the enum variant.
    pub fn status_code(&self) -> GrpcStatusCode {
        GrpcStatusCode::from_u32(self.code)
    }

    /// Whether the status represents success.
    pub fn is_ok(&self) -> bool {
        self.code == 0
    }
}

/// Encode a [`GrpcStatus`] with details into trailing metadata.
pub fn encode_error_details_metadata(status: &GrpcStatus) -> Result<Metadata, GrpcError> {
    let mut md = Metadata::new();
    md.insert("grpc-status", format!("{}", status.code));
    if !status.message.is_empty() {
        md.insert("grpc-message", &status.message);
    }
    if !status.details.is_empty() {
        let json_bytes = status.encode_json()?;
        let encoded = base64_codec::encode(&json_bytes);
        md.insert("grpc-status-details-bin", &encoded);
    }
    Ok(md)
}

/// Decode error details from trailing metadata.
pub fn decode_error_details_metadata(md: &Metadata) -> Result<GrpcStatus, GrpcError> {
    let code = md
        .get("grpc-status")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let message = md.get("grpc-message").unwrap_or("").to_owned();

    let details = if let Some(bin) = md.get("grpc-status-details-bin") {
        let json_bytes = base64_codec::decode(bin).map_err(GrpcError::Base64Decode)?;
        let full: GrpcStatus = serde_json::from_slice(&json_bytes)
            .map_err(|e| GrpcError::Serialization(e.to_string()))?;
        full.details
    } else {
        Vec::new()
    };

    Ok(GrpcStatus {
        code,
        message,
        details,
    })
}

// ---------------------------------------------------------------------------
// Message serialization helpers for mesh/morph data
// ---------------------------------------------------------------------------

/// Helper to read a u32 LE from a byte slice at a given offset.
fn read_u32_le(data: &[u8], offset: &mut usize) -> Result<u32, GrpcError> {
    if *offset + 4 > data.len() {
        return Err(GrpcError::InsufficientData {
            needed: *offset + 4,
            available: data.len(),
        });
    }
    let val = u32::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    Ok(val)
}

/// Helper to read a vec of f32 LE values from a byte slice.
fn read_f32_vec(data: &[u8], offset: &mut usize, count: usize) -> Result<Vec<f32>, GrpcError> {
    let byte_count = count * 4;
    if *offset + byte_count > data.len() {
        return Err(GrpcError::InsufficientData {
            needed: *offset + byte_count,
            available: data.len(),
        });
    }
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        let val = f32::from_le_bytes([
            data[*offset],
            data[*offset + 1],
            data[*offset + 2],
            data[*offset + 3],
        ]);
        *offset += 4;
        vec.push(val);
    }
    Ok(vec)
}

/// Helper to read a vec of u32 LE values from a byte slice.
fn read_u32_vec(data: &[u8], offset: &mut usize, count: usize) -> Result<Vec<u32>, GrpcError> {
    let byte_count = count * 4;
    if *offset + byte_count > data.len() {
        return Err(GrpcError::InsufficientData {
            needed: *offset + byte_count,
            available: data.len(),
        });
    }
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        let val = u32::from_le_bytes([
            data[*offset],
            data[*offset + 1],
            data[*offset + 2],
            data[*offset + 3],
        ]);
        *offset += 4;
        vec.push(val);
    }
    Ok(vec)
}

/// Lightweight serialization envelope for mesh vertex data.
#[derive(Debug, Clone)]
pub struct MeshDataPayload {
    pub vertex_count: u32,
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}

impl MeshDataPayload {
    pub fn new() -> Self {
        Self {
            vertex_count: 0,
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Serialize to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let cap = 4
            + 4
            + self.positions.len() * 4
            + 4
            + self.normals.len() * 4
            + 4
            + self.indices.len() * 4;
        let mut buf = Vec::with_capacity(cap);
        buf.extend_from_slice(&self.vertex_count.to_le_bytes());
        buf.extend_from_slice(&(self.positions.len() as u32).to_le_bytes());
        for &v in &self.positions {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.extend_from_slice(&(self.normals.len() as u32).to_le_bytes());
        for &v in &self.normals {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.extend_from_slice(&(self.indices.len() as u32).to_le_bytes());
        for &v in &self.indices {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf
    }

    /// Deserialize from bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, GrpcError> {
        let mut offset = 0usize;
        let vertex_count = read_u32_le(data, &mut offset)?;
        let pos_count = read_u32_le(data, &mut offset)? as usize;
        let positions = read_f32_vec(data, &mut offset, pos_count)?;
        let norm_count = read_u32_le(data, &mut offset)? as usize;
        let normals = read_f32_vec(data, &mut offset, norm_count)?;
        let idx_count = read_u32_le(data, &mut offset)? as usize;
        let indices = read_u32_vec(data, &mut offset, idx_count)?;
        Ok(Self {
            vertex_count,
            positions,
            normals,
            indices,
        })
    }

    /// Wrap the serialized payload in a gRPC frame.
    pub fn to_frame(&self) -> GrpcFrame {
        GrpcFrame::new(self.serialize())
    }
}

impl Default for MeshDataPayload {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightweight serialization envelope for morph target data.
#[derive(Debug, Clone)]
pub struct MorphTargetPayload {
    pub name: String,
    pub weight: f32,
    pub deltas: Vec<f32>,
}

impl MorphTargetPayload {
    pub fn new(name: impl Into<String>, weight: f32) -> Self {
        Self {
            name: name.into(),
            weight,
            deltas: Vec::new(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let name_bytes = self.name.as_bytes();
        let cap = 4 + name_bytes.len() + 4 + 4 + self.deltas.len() * 4;
        let mut buf = Vec::with_capacity(cap);
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&self.weight.to_le_bytes());
        buf.extend_from_slice(&(self.deltas.len() as u32).to_le_bytes());
        for &d in &self.deltas {
            buf.extend_from_slice(&d.to_le_bytes());
        }
        buf
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, GrpcError> {
        let mut offset = 0usize;
        let name_len = read_u32_le(data, &mut offset)? as usize;
        if offset + name_len > data.len() {
            return Err(GrpcError::InsufficientData {
                needed: offset + name_len,
                available: data.len(),
            });
        }
        let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
        offset += name_len;
        if offset + 4 > data.len() {
            return Err(GrpcError::InsufficientData {
                needed: offset + 4,
                available: data.len(),
            });
        }
        let weight = f32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        let delta_count = read_u32_le(data, &mut offset)? as usize;
        let deltas = read_f32_vec(data, &mut offset, delta_count)?;
        Ok(Self {
            name,
            weight,
            deltas,
        })
    }

    /// Wrap the serialized payload in a gRPC frame.
    pub fn to_frame(&self) -> GrpcFrame {
        GrpcFrame::new(self.serialize())
    }
}

/// Serialize multiple morph targets into a single buffer.
pub fn serialize_morph_targets(targets: &[MorphTargetPayload]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(targets.len() as u32).to_le_bytes());
    for t in targets {
        let serialized = t.serialize();
        buf.extend_from_slice(&(serialized.len() as u32).to_le_bytes());
        buf.extend_from_slice(&serialized);
    }
    buf
}

/// Deserialize multiple morph targets from a buffer.
pub fn deserialize_morph_targets(data: &[u8]) -> Result<Vec<MorphTargetPayload>, GrpcError> {
    let mut offset = 0usize;
    let count = read_u32_le(data, &mut offset)? as usize;
    let mut targets = Vec::with_capacity(count);
    for _ in 0..count {
        let seg_len = read_u32_le(data, &mut offset)? as usize;
        if offset + seg_len > data.len() {
            return Err(GrpcError::InsufficientData {
                needed: offset + seg_len,
                available: data.len(),
            });
        }
        let target = MorphTargetPayload::deserialize(&data[offset..offset + seg_len])?;
        offset += seg_len;
        targets.push(target);
    }
    Ok(targets)
}

// ---------------------------------------------------------------------------
// Legacy public API — kept for backwards compatibility
// ---------------------------------------------------------------------------

/// A gRPC request stub.
#[derive(Debug, Clone)]
pub struct GrpcRequest {
    pub method: String,
    pub metadata: Vec<(String, String)>,
    pub frame: GrpcFrame,
}

/// A gRPC response stub.
#[derive(Debug, Clone)]
pub struct GrpcResponse {
    pub status_code: u32,
    pub message: String,
    pub frame: GrpcFrame,
}

/// Build a gRPC request stub with raw body.
pub fn build_grpc_request(method: &str, body: Vec<u8>) -> GrpcRequest {
    GrpcRequest {
        method: method.to_string(),
        metadata: Vec::new(),
        frame: GrpcFrame::new(body),
    }
}

/// Build a gRPC response stub.
pub fn build_grpc_response(status: u32, body: Vec<u8>) -> GrpcResponse {
    GrpcResponse {
        status_code: status,
        message: if status == 0 {
            "OK".to_string()
        } else {
            "Error".to_string()
        },
        frame: GrpcFrame::new(body),
    }
}

/// Add metadata to a request.
pub fn add_metadata(req: &mut GrpcRequest, key: &str, val: &str) {
    req.metadata.push((key.to_string(), val.to_string()));
}

/// Check if a response indicates success.
pub fn is_ok(resp: &GrpcResponse) -> bool {
    resp.status_code == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Original tests (preserved) ----------------------------------------

    #[test]
    fn frame_encode_length() {
        let frame = GrpcFrame::new(vec![1, 2, 3]);
        let enc = frame.encode();
        assert_eq!(enc.len(), 8);
    }

    #[test]
    fn frame_header_no_compression() {
        let frame = GrpcFrame::new(vec![]);
        let enc = frame.encode();
        assert_eq!(enc[0], 0);
    }

    #[test]
    fn frame_body_length_in_header() {
        let frame = GrpcFrame::new(vec![0xAA, 0xBB]);
        let enc = frame.encode();
        let len = u32::from_be_bytes([enc[1], enc[2], enc[3], enc[4]]);
        assert_eq!(len, 2);
    }

    #[test]
    fn request_method_stored() {
        let req = build_grpc_request("/pkg.Svc/Method", vec![]);
        assert_eq!(req.method, "/pkg.Svc/Method");
    }

    #[test]
    fn response_ok_status() {
        let resp = build_grpc_response(0, vec![]);
        assert!(is_ok(&resp));
    }

    #[test]
    fn response_error_status() {
        let resp = build_grpc_response(1, vec![]);
        assert!(!is_ok(&resp));
    }

    #[test]
    fn add_metadata_count() {
        let mut req = build_grpc_request("/x", vec![]);
        add_metadata(&mut req, "auth", "Bearer token");
        assert_eq!(req.metadata.len(), 1);
    }

    #[test]
    fn encoded_len_helper() {
        let frame = GrpcFrame::new(vec![0u8; 10]);
        assert_eq!(frame.encoded_len(), 15);
    }

    #[test]
    fn empty_body_frame() {
        let frame = GrpcFrame::new(vec![]);
        let enc = frame.encode();
        assert_eq!(enc.len(), 5);
    }

    #[test]
    fn response_message_ok() {
        let resp = build_grpc_response(0, vec![]);
        assert_eq!(resp.message, "OK");
    }

    // -- Status code tests -------------------------------------------------

    #[test]
    fn status_code_all_17() {
        let codes: Vec<u32> = (0..=16).collect();
        for &c in &codes {
            let sc = GrpcStatusCode::from_u32(c);
            assert_eq!(sc as u32, c);
        }
    }

    #[test]
    fn status_code_unknown_for_invalid() {
        assert_eq!(GrpcStatusCode::from_u32(999), GrpcStatusCode::Unknown);
    }

    #[test]
    fn status_code_display() {
        let sc = GrpcStatusCode::NotFound;
        let s = format!("{}", sc);
        assert!(s.contains("NOT_FOUND"));
        assert!(s.contains("5"));
    }

    #[test]
    fn status_code_is_ok() {
        assert!(GrpcStatusCode::Ok.is_ok());
        assert!(!GrpcStatusCode::Internal.is_ok());
    }

    // -- Frame decode tests ------------------------------------------------

    #[test]
    fn frame_decode_roundtrip() {
        let frame = GrpcFrame::new(vec![10, 20, 30, 40]);
        let encoded = frame.encode();
        let (decoded, consumed) = GrpcFrame::decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.body, vec![10, 20, 30, 40]);
        assert_eq!(decoded.compression, GrpcCompression::None);
    }

    #[test]
    fn frame_decode_insufficient() {
        let result = GrpcFrame::decode(&[0, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn frame_decode_body_truncated() {
        let data = [0u8, 0, 0, 0, 10, 1, 2, 3];
        let result = GrpcFrame::decode(&data);
        assert!(result.is_err());
    }

    #[test]
    fn frame_decode_all_multiple() {
        let f1 = GrpcFrame::new(vec![1, 2]);
        let f2 = GrpcFrame::new(vec![3, 4, 5]);
        let mut data = f1.encode();
        data.extend_from_slice(&f2.encode());
        let frames = GrpcFrame::decode_all(&data).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].body, vec![1, 2]);
        assert_eq!(frames[1].body, vec![3, 4, 5]);
    }

    #[test]
    fn frame_with_compression() {
        let frame = GrpcFrame::with_compression(vec![99], GrpcCompression::Gzip);
        let enc = frame.encode();
        assert_eq!(enc[0], 1);
        let (dec, _) = GrpcFrame::decode(&enc).unwrap();
        assert_eq!(dec.compression, GrpcCompression::Gzip);
    }

    // -- Metadata tests ----------------------------------------------------

    #[test]
    fn metadata_insert_get() {
        let mut md = Metadata::new();
        md.insert("content-type", "application/grpc");
        assert_eq!(md.get("content-type"), Some("application/grpc"));
        assert_eq!(md.len(), 1);
    }

    #[test]
    fn metadata_get_all_duplicates() {
        let mut md = Metadata::new();
        md.insert("x-custom", "a");
        md.insert("x-custom", "b");
        let vals = md.get_all("x-custom");
        assert_eq!(vals, vec!["a", "b"]);
    }

    #[test]
    fn metadata_trailers_roundtrip() {
        let mut md = Metadata::new();
        md.insert("grpc-status", "0");
        md.insert("grpc-message", "success");
        let encoded = md.encode_trailers();
        let decoded = Metadata::decode_trailers(&encoded);
        assert_eq!(decoded.get("grpc-status"), Some("0"));
        assert_eq!(decoded.get("grpc-message"), Some("success"));
    }

    #[test]
    fn metadata_binary_key() {
        let entry = MetadataEntry::new("x-data-bin", "base64stuff");
        assert!(entry.is_binary());
        let entry2 = MetadataEntry::new("x-data", "plain");
        assert!(!entry2.is_binary());
    }

    // -- Trailers frame tests ----------------------------------------------

    #[test]
    fn trailers_frame_roundtrip() {
        let mut md = Metadata::new();
        md.insert("grpc-status", "13");
        md.insert("grpc-message", "internal error");
        let frame_bytes = encode_trailers_frame(&md);
        assert!(is_trailers_frame(&frame_bytes));
        let (decoded, consumed) = decode_trailers_frame(&frame_bytes).unwrap();
        assert_eq!(consumed, frame_bytes.len());
        assert_eq!(decoded.get("grpc-status"), Some("13"));
        assert_eq!(decoded.get("grpc-message"), Some("internal error"));
    }

    #[test]
    fn normal_frame_not_trailers() {
        let frame = GrpcFrame::new(vec![1, 2, 3]);
        let encoded = frame.encode();
        assert!(!is_trailers_frame(&encoded));
    }

    // -- Stream framing tests ----------------------------------------------

    #[test]
    fn unary_stream_encode_decode() {
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(vec![42, 43, 44]);
        stream.set_status(GrpcStatusCode::Ok, "");
        let encoded = stream.encode();
        let decoded = GrpcStream::decode(&encoded, StreamKind::Unary).unwrap();
        assert_eq!(decoded.message_count(), 1);
        assert_eq!(decoded.messages[0].body, vec![42, 43, 44]);
        assert_eq!(decoded.status, GrpcStatusCode::Ok);
    }

    #[test]
    fn server_streaming_multiple_messages() {
        let bodies: Vec<Vec<u8>> = vec![vec![1], vec![2, 3], vec![4, 5, 6]];
        let encoded = encode_server_stream(&bodies, GrpcStatusCode::Ok, &Metadata::new());
        let decoded = GrpcStream::decode(&encoded, StreamKind::ServerStreaming).unwrap();
        assert_eq!(decoded.message_count(), 3);
        assert_eq!(decoded.messages[2].body, vec![4, 5, 6]);
    }

    #[test]
    fn client_streaming_encode() {
        let bodies = vec![vec![10], vec![20]];
        let encoded = encode_client_stream(&bodies);
        let frames = GrpcFrame::decode_all(&encoded).unwrap();
        assert_eq!(frames.len(), 2);
    }

    #[test]
    fn bidi_streaming_encode() {
        let bodies = vec![vec![7], vec![8], vec![9]];
        let encoded = encode_bidi_frames(&bodies);
        let frames = GrpcFrame::decode_all(&encoded).unwrap();
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn stream_with_error_status() {
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(vec![]);
        stream.set_status(GrpcStatusCode::NotFound, "resource missing");
        let encoded = stream.encode();
        let decoded = GrpcStream::decode(&encoded, StreamKind::Unary).unwrap();
        assert_eq!(decoded.status, GrpcStatusCode::NotFound);
        assert_eq!(decoded.status_message, "resource missing");
    }

    #[test]
    fn stream_trailing_metadata_preserved() {
        let mut stream = GrpcStream::new(StreamKind::ServerStreaming);
        stream.push_message(vec![1]);
        stream.trailing_metadata.insert("x-request-id", "abc123");
        stream.status = GrpcStatusCode::Ok;
        let encoded = stream.encode();
        let decoded = GrpcStream::decode(&encoded, StreamKind::ServerStreaming).unwrap();
        assert_eq!(
            decoded.trailing_metadata.get("x-request-id"),
            Some("abc123")
        );
    }

    #[test]
    fn stream_kind_display() {
        assert_eq!(format!("{}", StreamKind::Unary), "unary");
        assert_eq!(format!("{}", StreamKind::BidiStreaming), "bidi-streaming");
    }

    // -- gRPC-Web text format tests ----------------------------------------

    #[test]
    fn grpc_web_text_roundtrip() {
        let original = vec![0u8, 0, 0, 0, 3, 10, 20, 30];
        let text = grpc_web_text_encode(&original);
        let decoded = grpc_web_text_decode(&text).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn grpc_web_text_stream_roundtrip() {
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(vec![100, 200]);
        stream.status = GrpcStatusCode::Ok;
        let text = grpc_web_text_encode_stream(&stream);
        let decoded = grpc_web_text_decode_stream(&text, StreamKind::Unary).unwrap();
        assert_eq!(decoded.message_count(), 1);
        assert_eq!(decoded.messages[0].body, vec![100, 200]);
    }

    #[test]
    fn grpc_web_text_empty() {
        let text = grpc_web_text_encode(&[]);
        assert!(text.is_empty());
        let decoded = grpc_web_text_decode(&text).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn grpc_web_text_invalid_base64() {
        let result = grpc_web_text_decode("!!!!");
        assert!(result.is_err());
    }

    // -- Error detail tests ------------------------------------------------

    #[test]
    fn error_detail_encoding_roundtrip() {
        let mut status = GrpcStatus::new(GrpcStatusCode::InvalidArgument, "bad field");
        status.add_detail(
            GrpcErrorDetail::new(
                "type.googleapis.com/google.rpc.BadRequest",
                "field 'name' is empty",
            )
            .with_metadata("field", "name"),
        );
        let md = encode_error_details_metadata(&status).unwrap();
        assert_eq!(md.get("grpc-status"), Some("3"));
        assert_eq!(md.get("grpc-message"), Some("bad field"));
        assert!(md.get("grpc-status-details-bin").is_some());
        let decoded = decode_error_details_metadata(&md).unwrap();
        assert_eq!(decoded.code, 3);
        assert_eq!(decoded.message, "bad field");
        assert_eq!(decoded.details.len(), 1);
        assert_eq!(decoded.details[0].metadata[0].0, "field");
    }

    #[test]
    fn grpc_status_ok_no_details() {
        let status = GrpcStatus::ok();
        assert!(status.is_ok());
        assert!(status.details.is_empty());
    }

    #[test]
    fn grpc_status_json_roundtrip() {
        let status = GrpcStatus::new(GrpcStatusCode::Internal, "crash");
        let json = status.encode_json().unwrap();
        let decoded = GrpcStatus::decode_json(&json).unwrap();
        assert_eq!(decoded.code, 13);
        assert_eq!(decoded.message, "crash");
    }

    // -- Mesh data payload tests -------------------------------------------

    #[test]
    fn mesh_payload_roundtrip() {
        let mut payload = MeshDataPayload::new();
        payload.vertex_count = 3;
        payload.positions = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        payload.normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        payload.indices = vec![0, 1, 2];
        let bytes = payload.serialize();
        let decoded = MeshDataPayload::deserialize(&bytes).unwrap();
        assert_eq!(decoded.vertex_count, 3);
        assert_eq!(decoded.positions.len(), 9);
        assert_eq!(decoded.indices, vec![0, 1, 2]);
    }

    #[test]
    fn mesh_payload_to_frame() {
        let mut payload = MeshDataPayload::new();
        payload.vertex_count = 1;
        payload.positions = vec![1.0, 2.0, 3.0];
        let frame = payload.to_frame();
        assert!(!frame.body.is_empty());
        let enc = frame.encode();
        let (dec, _) = GrpcFrame::decode(&enc).unwrap();
        let restored = MeshDataPayload::deserialize(&dec.body).unwrap();
        assert_eq!(restored.vertex_count, 1);
    }

    #[test]
    fn mesh_payload_empty() {
        let payload = MeshDataPayload::new();
        let bytes = payload.serialize();
        let decoded = MeshDataPayload::deserialize(&bytes).unwrap();
        assert_eq!(decoded.vertex_count, 0);
        assert!(decoded.positions.is_empty());
    }

    // -- Morph target payload tests ----------------------------------------

    #[test]
    fn morph_payload_roundtrip() {
        let mut morph = MorphTargetPayload::new("smile", 0.75);
        morph.deltas = vec![0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
        let bytes = morph.serialize();
        let decoded = MorphTargetPayload::deserialize(&bytes).unwrap();
        assert_eq!(decoded.name, "smile");
        assert!((decoded.weight - 0.75).abs() < f32::EPSILON);
        assert_eq!(decoded.deltas.len(), 6);
    }

    #[test]
    fn morph_payload_to_frame() {
        let morph = MorphTargetPayload::new("blink_L", 1.0);
        let frame = morph.to_frame();
        let enc = frame.encode();
        let (dec, _) = GrpcFrame::decode(&enc).unwrap();
        let restored = MorphTargetPayload::deserialize(&dec.body).unwrap();
        assert_eq!(restored.name, "blink_L");
    }

    #[test]
    fn multiple_morph_targets_roundtrip() {
        let targets = vec![
            {
                let mut m = MorphTargetPayload::new("jawOpen", 0.5);
                m.deltas = vec![0.1, 0.2];
                m
            },
            {
                let mut m = MorphTargetPayload::new("eyeClose", 1.0);
                m.deltas = vec![0.3, 0.4, 0.5];
                m
            },
        ];
        let bytes = serialize_morph_targets(&targets);
        let decoded = deserialize_morph_targets(&bytes).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].name, "jawOpen");
        assert_eq!(decoded[1].name, "eyeClose");
        assert_eq!(decoded[1].deltas.len(), 3);
    }

    #[test]
    fn morph_targets_empty_list() {
        let targets: Vec<MorphTargetPayload> = vec![];
        let bytes = serialize_morph_targets(&targets);
        let decoded = deserialize_morph_targets(&bytes).unwrap();
        assert!(decoded.is_empty());
    }

    // -- Base64 codec edge cases -------------------------------------------

    #[test]
    fn base64_padding_1() {
        let data = vec![1, 2, 3, 4, 5];
        let encoded = base64_codec::encode(&data);
        let decoded = base64_codec::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_padding_2() {
        let data = vec![255];
        let encoded = base64_codec::encode(&data);
        assert!(encoded.ends_with("=="));
        let decoded = base64_codec::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_no_padding() {
        let data = vec![1, 2, 3];
        let encoded = base64_codec::encode(&data);
        assert!(!encoded.contains('='));
        let decoded = base64_codec::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    // -- Integration tests -------------------------------------------------

    #[test]
    fn mesh_over_grpc_web_text() {
        let mut payload = MeshDataPayload::new();
        payload.vertex_count = 2;
        payload.positions = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        payload.normals = vec![0.0, 1.0, 0.0, 0.0, 1.0, 0.0];
        payload.indices = vec![0, 1];
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(payload.serialize());
        stream.status = GrpcStatusCode::Ok;
        let text = grpc_web_text_encode_stream(&stream);
        let decoded = grpc_web_text_decode_stream(&text, StreamKind::Unary).unwrap();
        let restored = MeshDataPayload::deserialize(&decoded.messages[0].body).unwrap();
        assert_eq!(restored.vertex_count, 2);
        assert_eq!(restored.indices, vec![0, 1]);
    }

    #[test]
    fn morph_stream_server_streaming() {
        let targets = [
            MorphTargetPayload::new("a", 0.1),
            MorphTargetPayload::new("b", 0.2),
        ];
        let bodies: Vec<Vec<u8>> = targets.iter().map(|t| t.serialize()).collect();
        let encoded = encode_server_stream(&bodies, GrpcStatusCode::Ok, &Metadata::new());
        let decoded = GrpcStream::decode(&encoded, StreamKind::ServerStreaming).unwrap();
        assert_eq!(decoded.message_count(), 2);
        let t0 = MorphTargetPayload::deserialize(&decoded.messages[0].body).unwrap();
        assert_eq!(t0.name, "a");
    }

    #[test]
    fn unsupported_compression_flag() {
        let data = [42u8, 0, 0, 0, 1, 0xFF];
        let result = GrpcFrame::decode(&data);
        assert!(result.is_err());
        let err_msg = format!("{}", result.expect_err("should fail"));
        assert!(err_msg.contains("unsupported compression"));
    }
}
