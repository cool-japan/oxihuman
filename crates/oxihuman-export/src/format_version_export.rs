#![allow(dead_code)]
//! Export format version data.

/// Format version.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// Create a new format version.
#[allow(dead_code)]
pub fn new_format_version(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion { major, minor, patch }
}

/// Get major version.
#[allow(dead_code)]
pub fn version_major(v: &FormatVersion) -> u16 {
    v.major
}

/// Get minor version.
#[allow(dead_code)]
pub fn version_minor(v: &FormatVersion) -> u16 {
    v.minor
}

/// Get patch version.
#[allow(dead_code)]
pub fn version_patch(v: &FormatVersion) -> u16 {
    v.patch
}

/// Convert version to string.
#[allow(dead_code)]
pub fn version_to_string(v: &FormatVersion) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

/// Check if two versions are compatible (same major version).
#[allow(dead_code)]
pub fn version_compatible(a: &FormatVersion, b: &FormatVersion) -> bool {
    a.major == b.major
}

/// Convert version to bytes (6 bytes: 2 per component).
#[allow(dead_code)]
pub fn version_to_bytes(v: &FormatVersion) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(6);
    bytes.extend_from_slice(&v.major.to_le_bytes());
    bytes.extend_from_slice(&v.minor.to_le_bytes());
    bytes.extend_from_slice(&v.patch.to_le_bytes());
    bytes
}

/// Parse version from bytes.
#[allow(dead_code)]
pub fn version_from_bytes(bytes: &[u8]) -> Option<FormatVersion> {
    if bytes.len() < 6 {
        return None;
    }
    let major = u16::from_le_bytes([bytes[0], bytes[1]]);
    let minor = u16::from_le_bytes([bytes[2], bytes[3]]);
    let patch = u16::from_le_bytes([bytes[4], bytes[5]]);
    Some(FormatVersion { major, minor, patch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_format_version() {
        let v = new_format_version(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_major() {
        let v = new_format_version(5, 0, 0);
        assert_eq!(version_major(&v), 5);
    }

    #[test]
    fn test_version_minor() {
        let v = new_format_version(1, 7, 0);
        assert_eq!(version_minor(&v), 7);
    }

    #[test]
    fn test_version_patch() {
        let v = new_format_version(1, 0, 9);
        assert_eq!(version_patch(&v), 9);
    }

    #[test]
    fn test_version_to_string() {
        let v = new_format_version(1, 2, 3);
        assert_eq!(version_to_string(&v), "1.2.3");
    }

    #[test]
    fn test_version_compatible() {
        let a = new_format_version(1, 0, 0);
        let b = new_format_version(1, 5, 3);
        assert!(version_compatible(&a, &b));
    }

    #[test]
    fn test_version_not_compatible() {
        let a = new_format_version(1, 0, 0);
        let b = new_format_version(2, 0, 0);
        assert!(!version_compatible(&a, &b));
    }

    #[test]
    fn test_version_to_bytes() {
        let v = new_format_version(1, 2, 3);
        let bytes = version_to_bytes(&v);
        assert_eq!(bytes.len(), 6);
    }

    #[test]
    fn test_version_from_bytes() {
        let v = new_format_version(10, 20, 30);
        let bytes = version_to_bytes(&v);
        let parsed = version_from_bytes(&bytes);
        assert_eq!(parsed, Some(v));
    }

    #[test]
    fn test_version_from_bytes_too_short() {
        assert!(version_from_bytes(&[0, 1]).is_none());
    }

    #[test]
    fn test_version_roundtrip() {
        let v = new_format_version(255, 128, 64);
        let bytes = version_to_bytes(&v);
        let restored = version_from_bytes(&bytes).unwrap();
        assert_eq!(v, restored);
    }
}
