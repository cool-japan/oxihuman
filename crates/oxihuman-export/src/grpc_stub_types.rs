// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! gRPC type definitions: status codes, errors, compression, metadata,
//! framing, stream model, error details, and mesh/morph serialization
//! payloads.
//!
//! High-level service functions and tests live in
//! [`super::grpc_stub_service`].

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

// ---------------------------------------------------------------------------
// Message serialization helpers for mesh/morph data
// ---------------------------------------------------------------------------

/// Helper to read a u32 LE from a byte slice at a given offset.
pub(super) fn read_u32_le(data: &[u8], offset: &mut usize) -> Result<u32, GrpcError> {
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
pub(super) fn read_f32_vec(
    data: &[u8],
    offset: &mut usize,
    count: usize,
) -> Result<Vec<f32>, GrpcError> {
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
pub(super) fn read_u32_vec(
    data: &[u8],
    offset: &mut usize,
    count: usize,
) -> Result<Vec<u32>, GrpcError> {
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
