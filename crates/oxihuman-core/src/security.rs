// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Security validation utilities for OxiHuman.
//!
//! Provides path sanitization, file size validation, safe stride/offset
//! arithmetic, and content-type detection for 3D asset files.
//!
//! # Usage
//!
//! ```rust
//! use oxihuman_core::security::{sanitize_path, validate_file_size, SecurityError};
//!
//! // Reject path traversal attempts
//! assert!(sanitize_path("../etc/passwd").is_err());
//! assert!(sanitize_path("models/human.glb").is_ok());
//!
//! // Reject oversized uploads
//! assert!(validate_file_size(200 * 1024 * 1024, 100).is_err());
//! ```

use std::fmt;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// SecurityError
// ---------------------------------------------------------------------------

/// Errors produced by OxiHuman security validation functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    /// Path contains a `..` component that could escape the intended root.
    PathTraversal,
    /// Path is absolute (starts with `/`, `\`, or a Windows drive letter like `C:\`).
    AbsolutePath,
    /// Path contains a null byte (`\0`).
    NullByte,
    /// Path exceeds the maximum allowed length (512 characters).
    TooLong,
    /// Path component matches a Windows reserved device name (e.g. `CON`, `NUL`).
    ReservedName(String),
    /// Path bytes are not valid UTF-8.
    InvalidUtf8,
    /// File size exceeds the configured maximum.
    FileTooLarge {
        /// Actual file size in bytes.
        size_bytes: usize,
        /// Maximum allowed size in bytes.
        max_bytes: usize,
    },
    /// An arithmetic overflow occurred during stride/offset computation.
    OverflowError,
    /// A computed index is out of bounds for the given buffer.
    OutOfBounds {
        /// The computed byte index.
        index: usize,
        /// The total buffer size in bytes.
        total: usize,
    },
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::PathTraversal => {
                write!(
                    f,
                    "security: path traversal detected (contains '..' component)"
                )
            }
            SecurityError::AbsolutePath => {
                write!(f, "security: absolute paths are not allowed")
            }
            SecurityError::NullByte => {
                write!(f, "security: path contains a null byte")
            }
            SecurityError::TooLong => {
                write!(
                    f,
                    "security: path exceeds maximum allowed length of 512 characters"
                )
            }
            SecurityError::ReservedName(name) => {
                write!(
                    f,
                    "security: '{}' is a reserved OS name and cannot be used as a path component",
                    name
                )
            }
            SecurityError::InvalidUtf8 => {
                write!(f, "security: path contains non-UTF-8 bytes")
            }
            SecurityError::FileTooLarge {
                size_bytes,
                max_bytes,
            } => {
                write!(
                    f,
                    "security: file size {} bytes exceeds maximum of {} bytes",
                    size_bytes, max_bytes
                )
            }
            SecurityError::OverflowError => {
                write!(
                    f,
                    "security: arithmetic overflow in stride/offset calculation"
                )
            }
            SecurityError::OutOfBounds { index, total } => {
                write!(
                    f,
                    "security: computed index {} is out of bounds for buffer of size {}",
                    index, total
                )
            }
        }
    }
}

impl std::error::Error for SecurityError {}

// ---------------------------------------------------------------------------
// Windows reserved device names
// ---------------------------------------------------------------------------

/// Returns `true` if `name` (without extension, case-insensitive) is a
/// Windows-reserved device name that must not appear as a path component.
fn is_reserved_name(component: &str) -> bool {
    // Strip any extension to get the bare stem.
    let stem = match component.rfind('.') {
        Some(dot) if dot > 0 => &component[..dot],
        _ => component,
    };

    let upper = stem.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM0"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT0"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

// ---------------------------------------------------------------------------
// sanitize_path
// ---------------------------------------------------------------------------

/// Validates a path string supplied from untrusted input and returns a safe
/// [`PathBuf`] if all checks pass.
///
/// # Checks performed (in order)
///
/// 1. Null bytes → [`SecurityError::NullByte`]
/// 2. Length > 512 → [`SecurityError::TooLong`]
/// 3. Starts with `/`, `\`, or `X:` (Windows drive) → [`SecurityError::AbsolutePath`]
/// 4. Any path component equals `..` → [`SecurityError::PathTraversal`]
/// 5. Any path component (stem, case-insensitive) is a reserved OS name →
///    [`SecurityError::ReservedName`]
///
/// # Example
///
/// ```rust
/// use oxihuman_core::security::sanitize_path;
///
/// assert!(sanitize_path("models/body.glb").is_ok());
/// assert!(sanitize_path("../secret").is_err());
/// assert!(sanitize_path("/etc/passwd").is_err());
/// ```
pub fn sanitize_path(input: &str) -> Result<PathBuf, SecurityError> {
    // Check 1: null bytes
    if input.contains('\0') {
        return Err(SecurityError::NullByte);
    }

    // Check 2: length
    if input.len() > 512 {
        return Err(SecurityError::TooLong);
    }

    // Check 3: absolute paths
    // Unix-style absolute
    if input.starts_with('/') {
        return Err(SecurityError::AbsolutePath);
    }
    // Windows UNC or absolute backslash
    if input.starts_with('\\') {
        return Err(SecurityError::AbsolutePath);
    }
    // Windows drive letter (e.g. "C:" or "C:\")
    {
        let mut chars = input.chars();
        let first = chars.next();
        let second = chars.next();
        if let (Some(c), Some(':')) = (first, second) {
            if c.is_ascii_alphabetic() {
                return Err(SecurityError::AbsolutePath);
            }
        }
    }

    // Check 4 & 5: iterate components split on '/' and '\'
    for component in input.split(['/', '\\']) {
        // Skip empty segments (e.g. trailing slash or double slash)
        if component.is_empty() || component == "." {
            continue;
        }

        // Path traversal
        if component == ".." {
            return Err(SecurityError::PathTraversal);
        }

        // Reserved OS names
        if is_reserved_name(component) {
            return Err(SecurityError::ReservedName(component.to_string()));
        }
    }

    Ok(PathBuf::from(input))
}

// ---------------------------------------------------------------------------
// validate_file_size
// ---------------------------------------------------------------------------

/// Validates that `bytes` does not exceed `max_mb` megabytes.
///
/// # Example
///
/// ```rust
/// use oxihuman_core::security::validate_file_size;
///
/// assert!(validate_file_size(1024 * 1024, 10).is_ok());   // 1 MB, limit 10 MB
/// assert!(validate_file_size(200 * 1024 * 1024, 100).is_err()); // 200 MB > 100 MB
/// ```
pub fn validate_file_size(bytes: usize, max_mb: u32) -> Result<(), SecurityError> {
    let max_bytes = (max_mb as usize).saturating_mul(1024).saturating_mul(1024);
    if bytes > max_bytes {
        Err(SecurityError::FileTooLarge {
            size_bytes: bytes,
            max_bytes,
        })
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// checked_stride_offset
// ---------------------------------------------------------------------------

/// Computes `index * stride` with overflow detection and bounds checking.
///
/// Returns the byte offset if it is strictly less than `total`, otherwise
/// returns an error.
///
/// # Example
///
/// ```rust
/// use oxihuman_core::security::checked_stride_offset;
///
/// assert_eq!(checked_stride_offset(3, 4, 20).unwrap(), 12);
/// assert!(checked_stride_offset(usize::MAX, 2, 100).is_err()); // overflow
/// assert!(checked_stride_offset(5, 4, 16).is_err());           // out of bounds
/// ```
pub fn checked_stride_offset(
    index: usize,
    stride: usize,
    total: usize,
) -> Result<usize, SecurityError> {
    let offset = index
        .checked_mul(stride)
        .ok_or(SecurityError::OverflowError)?;
    if offset >= total {
        return Err(SecurityError::OutOfBounds {
            index: offset,
            total,
        });
    }
    Ok(offset)
}

// ---------------------------------------------------------------------------
// is_safe_content_type
// ---------------------------------------------------------------------------

/// Validates the magic bytes of a 3D asset file against known-good signatures.
///
/// Supported formats:
/// - **GLB** (binary glTF): magic `glTF` at offset 0
/// - **OBJ** (Wavefront): starts with `v `, `# `, `mtllib`, `usemtl`, `o `, or `g `
/// - **STL binary**: at least 84 bytes (80-byte header + 4-byte count)
/// - **STL ASCII**: starts with `solid` (case-insensitive)
/// - **PLY**: starts with `ply\n` or `ply\r`
/// - **FBX binary**: starts with `Kaydara FBX Binary  \x00`
///
/// Returns `false` for empty slices and unrecognized formats.
///
/// # Example
///
/// ```rust
/// use oxihuman_core::security::is_safe_content_type;
///
/// assert!(is_safe_content_type(b"glTF\x02\x00\x00\x00"));
/// assert!(is_safe_content_type(b"ply\nformat ascii 1.0\n"));
/// assert!(!is_safe_content_type(b"\x00\x01\x02\x03"));
/// ```
pub fn is_safe_content_type(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    // GLB: binary glTF magic "glTF"
    if bytes.starts_with(b"glTF") {
        return true;
    }

    // PLY
    if bytes.starts_with(b"ply\n") || bytes.starts_with(b"ply\r") {
        return true;
    }

    // FBX binary
    if bytes.starts_with(b"Kaydara FBX Binary  \x00") {
        return true;
    }

    // STL ASCII: starts with "solid" (case-insensitive, first 5 bytes)
    if bytes.len() >= 5 {
        let prefix: Vec<u8> = bytes[..5].iter().map(|b| b.to_ascii_lowercase()).collect();
        if prefix == b"solid" {
            return true;
        }
    }

    // OBJ: check first 16 bytes for known OBJ line starters
    {
        let probe = if bytes.len() >= 16 {
            &bytes[..16]
        } else {
            bytes
        };
        let obj_prefixes: &[&[u8]] = &[b"v ", b"# ", b"mtllib", b"usemtl", b"o ", b"g "];
        for prefix in obj_prefixes {
            if probe.starts_with(prefix) {
                return true;
            }
        }
    }

    // STL binary: minimum 84 bytes (80-byte header + 4-byte triangle count)
    // Only matched when the above ASCII checks have already failed.
    // A binary STL cannot start with "solid" (handled above), so if we reach
    // here with ≥ 84 bytes and no other match, treat as potentially valid
    // binary STL only if the header does NOT start with "solid".
    if bytes.len() >= 84 {
        // Already excluded ASCII STL above. Accept as binary STL candidate.
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- sanitize_path ---

    #[test]
    fn sanitize_path_valid_relative() {
        let result = sanitize_path("models/human.glb").expect("should succeed");
        assert_eq!(result, PathBuf::from("models/human.glb"));
    }

    #[test]
    fn sanitize_path_valid_nested() {
        let result = sanitize_path("assets/v1/mesh.obj").expect("should succeed");
        assert_eq!(result, PathBuf::from("assets/v1/mesh.obj"));
    }

    #[test]
    fn sanitize_path_rejects_null_byte() {
        let err = sanitize_path("foo\0bar").unwrap_err();
        assert_eq!(err, SecurityError::NullByte);
    }

    #[test]
    fn sanitize_path_rejects_too_long() {
        let long = "a".repeat(513);
        let err = sanitize_path(&long).unwrap_err();
        assert_eq!(err, SecurityError::TooLong);
    }

    #[test]
    fn sanitize_path_512_chars_ok() {
        // Exactly 512 characters must pass
        let ok = "a".repeat(512);
        assert!(sanitize_path(&ok).is_ok());
    }

    #[test]
    fn sanitize_path_rejects_dotdot() {
        let err = sanitize_path("../etc/passwd").unwrap_err();
        assert_eq!(err, SecurityError::PathTraversal);
    }

    #[test]
    fn sanitize_path_rejects_dotdot_middle() {
        let err = sanitize_path("assets/../../../etc/shadow").unwrap_err();
        assert_eq!(err, SecurityError::PathTraversal);
    }

    #[test]
    fn sanitize_path_rejects_absolute_unix() {
        let err = sanitize_path("/etc/passwd").unwrap_err();
        assert_eq!(err, SecurityError::AbsolutePath);
    }

    #[test]
    fn sanitize_path_rejects_absolute_backslash() {
        let err = sanitize_path("\\\\server\\share").unwrap_err();
        assert_eq!(err, SecurityError::AbsolutePath);
    }

    #[test]
    fn sanitize_path_rejects_windows_drive() {
        let err = sanitize_path("C:\\Windows\\System32").unwrap_err();
        assert_eq!(err, SecurityError::AbsolutePath);
    }

    #[test]
    fn sanitize_path_rejects_windows_drive_lowercase() {
        let err = sanitize_path("c:/Users/Admin").unwrap_err();
        assert_eq!(err, SecurityError::AbsolutePath);
    }

    #[test]
    fn sanitize_path_rejects_reserved_con() {
        let err = sanitize_path("CON").unwrap_err();
        assert!(matches!(err, SecurityError::ReservedName(_)));
    }

    #[test]
    fn sanitize_path_rejects_reserved_nul_with_ext() {
        // NUL.txt — strip extension before checking
        let err = sanitize_path("NUL.txt").unwrap_err();
        assert!(matches!(err, SecurityError::ReservedName(_)));
    }

    #[test]
    fn sanitize_path_rejects_reserved_com1() {
        let err = sanitize_path("COM1").unwrap_err();
        assert!(matches!(err, SecurityError::ReservedName(_)));
    }

    #[test]
    fn sanitize_path_rejects_reserved_lpt9_lowercase() {
        let err = sanitize_path("lpt9").unwrap_err();
        assert!(matches!(err, SecurityError::ReservedName(_)));
    }

    #[test]
    fn sanitize_path_rejects_reserved_in_subdir() {
        // Reserved name as a subdirectory component
        let err = sanitize_path("assets/NUL/mesh.obj").unwrap_err();
        assert!(matches!(err, SecurityError::ReservedName(_)));
    }

    // --- validate_file_size ---

    #[test]
    fn validate_file_size_ok() {
        assert!(validate_file_size(1024 * 1024, 10).is_ok());
    }

    #[test]
    fn validate_file_size_exact_boundary() {
        // Exactly at the limit must be accepted
        let max_mb = 10u32;
        let exact = max_mb as usize * 1024 * 1024;
        assert!(validate_file_size(exact, max_mb).is_ok());
    }

    #[test]
    fn validate_file_size_too_large() {
        let max_mb = 10u32;
        let over = max_mb as usize * 1024 * 1024 + 1;
        let err = validate_file_size(over, max_mb).unwrap_err();
        assert!(matches!(err, SecurityError::FileTooLarge { .. }));
    }

    #[test]
    fn validate_file_size_zero_ok() {
        assert!(validate_file_size(0, 1).is_ok());
    }

    // --- checked_stride_offset ---

    #[test]
    fn checked_stride_offset_ok() {
        let result = checked_stride_offset(3, 4, 20).expect("should succeed");
        assert_eq!(result, 12);
    }

    #[test]
    fn checked_stride_offset_zero_index() {
        let result = checked_stride_offset(0, 8, 100).expect("should succeed");
        assert_eq!(result, 0);
    }

    #[test]
    fn checked_stride_offset_out_of_bounds() {
        // 5 * 4 = 20, total = 16 → out of bounds
        let err = checked_stride_offset(5, 4, 16).unwrap_err();
        assert!(matches!(err, SecurityError::OutOfBounds { .. }));
    }

    #[test]
    fn checked_stride_offset_overflow() {
        let err = checked_stride_offset(usize::MAX, 2, 100).unwrap_err();
        assert_eq!(err, SecurityError::OverflowError);
    }

    #[test]
    fn checked_stride_offset_exact_boundary_ok() {
        // index=4, stride=4, total=20 → 16 < 20 → ok
        let result = checked_stride_offset(4, 4, 20).expect("should succeed");
        assert_eq!(result, 16);
    }

    // --- is_safe_content_type ---

    #[test]
    fn is_safe_content_type_glb() {
        assert!(is_safe_content_type(
            b"glTF\x02\x00\x00\x00\x00\x00\x00\x00"
        ));
    }

    #[test]
    fn is_safe_content_type_ply_lf() {
        assert!(is_safe_content_type(b"ply\nformat ascii 1.0\n"));
    }

    #[test]
    fn is_safe_content_type_ply_cr() {
        assert!(is_safe_content_type(b"ply\rformat ascii 1.0\r"));
    }

    #[test]
    fn is_safe_content_type_fbx() {
        assert!(is_safe_content_type(
            b"Kaydara FBX Binary  \x00more_data_here"
        ));
    }

    #[test]
    fn is_safe_content_type_stl_ascii() {
        assert!(is_safe_content_type(b"solid MyModel\nfacet normal 0 0 1\n"));
    }

    #[test]
    fn is_safe_content_type_stl_ascii_uppercase() {
        assert!(is_safe_content_type(b"SOLID mymodel\n"));
    }

    #[test]
    fn is_safe_content_type_obj_vertex() {
        assert!(is_safe_content_type(b"v 0.0 0.0 0.0\nv 1.0 0.0 0.0\n"));
    }

    #[test]
    fn is_safe_content_type_obj_comment() {
        assert!(is_safe_content_type(
            b"# Exported by Blender\nmtllib mat.mtl\n"
        ));
    }

    #[test]
    fn is_safe_content_type_unknown_magic() {
        assert!(!is_safe_content_type(b"\x00\x01\x02\x03"));
    }

    #[test]
    fn is_safe_content_type_empty() {
        assert!(!is_safe_content_type(b""));
    }

    #[test]
    fn is_safe_content_type_random_text() {
        assert!(!is_safe_content_type(b"Hello, world!"));
    }

    // --- SecurityError Display ---

    #[test]
    fn security_error_display_path_traversal() {
        let msg = format!("{}", SecurityError::PathTraversal);
        assert!(msg.contains("path traversal"));
    }

    #[test]
    fn security_error_display_file_too_large() {
        let msg = format!(
            "{}",
            SecurityError::FileTooLarge {
                size_bytes: 200,
                max_bytes: 100
            }
        );
        assert!(msg.contains("200"));
        assert!(msg.contains("100"));
    }
}
