#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A simple UUID representation (128-bit).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleUuid {
    bytes: [u8; 16],
}

#[allow(dead_code)]
pub fn new_simple_uuid(bytes: [u8; 16]) -> SimpleUuid {
    SimpleUuid { bytes }
}

#[allow(dead_code)]
pub fn uuid_to_string(uuid: &SimpleUuid) -> String {
    let b = &uuid.bytes;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

#[allow(dead_code)]
pub fn uuid_from_string(s: &str) -> Option<SimpleUuid> {
    let hex: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(SimpleUuid { bytes })
}

#[allow(dead_code)]
pub fn uuids_equal(a: &SimpleUuid, b: &SimpleUuid) -> bool {
    a.bytes == b.bytes
}

#[allow(dead_code)]
pub fn uuid_is_nil(uuid: &SimpleUuid) -> bool {
    uuid.bytes == [0u8; 16]
}

#[allow(dead_code)]
pub fn uuid_version(uuid: &SimpleUuid) -> u8 {
    (uuid.bytes[6] >> 4) & 0x0F
}

#[allow(dead_code)]
pub fn uuid_to_bytes(uuid: &SimpleUuid) -> [u8; 16] {
    uuid.bytes
}

#[allow(dead_code)]
pub fn uuid_from_bytes(bytes: [u8; 16]) -> SimpleUuid {
    SimpleUuid { bytes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simple_uuid() {
        let uuid = new_simple_uuid([0; 16]);
        assert!(uuid_is_nil(&uuid));
    }

    #[test]
    fn test_uuid_to_string() {
        let uuid = new_simple_uuid([0; 16]);
        let s = uuid_to_string(&uuid);
        assert_eq!(s, "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_uuid_from_string() {
        let s = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = uuid_from_string(s).expect("should succeed");
        assert_eq!(uuid_to_string(&uuid), s);
    }

    #[test]
    fn test_uuid_from_string_invalid() {
        assert!(uuid_from_string("not-a-uuid").is_none());
    }

    #[test]
    fn test_uuids_equal() {
        let a = new_simple_uuid([1; 16]);
        let b = new_simple_uuid([1; 16]);
        assert!(uuids_equal(&a, &b));
    }

    #[test]
    fn test_uuids_not_equal() {
        let a = new_simple_uuid([1; 16]);
        let b = new_simple_uuid([2; 16]);
        assert!(!uuids_equal(&a, &b));
    }

    #[test]
    fn test_uuid_is_nil() {
        let uuid = new_simple_uuid([0; 16]);
        assert!(uuid_is_nil(&uuid));
    }

    #[test]
    fn test_uuid_version() {
        let mut bytes = [0u8; 16];
        bytes[6] = 0x40; // version 4
        let uuid = new_simple_uuid(bytes);
        assert_eq!(uuid_version(&uuid), 4);
    }

    #[test]
    fn test_uuid_to_bytes() {
        let bytes = [1u8; 16];
        let uuid = new_simple_uuid(bytes);
        assert_eq!(uuid_to_bytes(&uuid), bytes);
    }

    #[test]
    fn test_uuid_from_bytes() {
        let bytes = [42u8; 16];
        let uuid = uuid_from_bytes(bytes);
        assert_eq!(uuid.bytes, bytes);
    }
}
