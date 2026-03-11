// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for the oxihuman-wasm crate.

use std::fmt;

/// Errors that can occur during WASM engine operations.
#[derive(Debug)]
pub enum WasmError {
    /// The input data is not valid UTF-8.
    InvalidUtf8(std::str::Utf8Error),
    /// OBJ parsing failed.
    ObjParse(String),
    /// Target parsing failed.
    TargetParse(String),
    /// ZIP pack error (missing entry, truncated, etc.).
    ZipPack(String),
    /// JSON deserialization error.
    Json(String),
    /// Generic error with a message.
    Other(String),
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmError::InvalidUtf8(e) => write!(f, "invalid UTF-8: {e}"),
            WasmError::ObjParse(msg) => write!(f, "OBJ parse error: {msg}"),
            WasmError::TargetParse(msg) => write!(f, "target parse error: {msg}"),
            WasmError::ZipPack(msg) => write!(f, "ZIP pack error: {msg}"),
            WasmError::Json(msg) => write!(f, "JSON error: {msg}"),
            WasmError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for WasmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WasmError::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::str::Utf8Error> for WasmError {
    fn from(e: std::str::Utf8Error) -> Self {
        WasmError::InvalidUtf8(e)
    }
}

impl From<serde_json::Error> for WasmError {
    fn from(e: serde_json::Error) -> Self {
        WasmError::Json(e.to_string())
    }
}
