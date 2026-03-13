// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Uuid {
    pub bytes: [u8; 16],
}

pub fn uuid_from_bytes(b: [u8; 16]) -> Uuid {
    Uuid { bytes: b }
}

pub fn uuid_nil() -> Uuid {
    Uuid { bytes: [0u8; 16] }
}

pub fn uuid_from_u128(v: u128) -> Uuid {
    Uuid {
        bytes: v.to_be_bytes(),
    }
}

pub fn uuid_version(u: &Uuid) -> u8 {
    (u.bytes[6] >> 4) & 0x0F
}

pub fn uuid_to_string(u: &Uuid) -> String {
    let b = &u.bytes;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3],
        b[4], b[5],
        b[6], b[7],
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

pub fn uuid_is_valid_string(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 36 {
        return false;
    }
    let hyphens = [8, 13, 18, 23];
    for (i, &ch) in b.iter().enumerate() {
        if hyphens.contains(&i) {
            if ch != b'-' {
                return false;
            }
        } else if !ch.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nil_uuid_all_zeros() {
        /* nil UUID bytes are all zero */
        let u = uuid_nil();
        assert_eq!(u.bytes, [0u8; 16]);
    }

    #[test]
    fn from_u128_roundtrip() {
        /* converting u128 to UUID and back */
        let v: u128 = 0x550e8400_e29b_41d4_a716_446655440000;
        let u = uuid_from_u128(v);
        let back = u128::from_be_bytes(u.bytes);
        assert_eq!(back, v);
    }

    #[test]
    fn to_string_format() {
        /* string representation has correct format */
        let u = uuid_nil();
        let s = uuid_to_string(&u);
        assert_eq!(s.len(), 36);
        assert_eq!(&s[8..9], "-");
        assert_eq!(&s[13..14], "-");
    }

    #[test]
    fn is_valid_string_true() {
        /* valid UUID string passes */
        let u = uuid_nil();
        assert!(uuid_is_valid_string(&uuid_to_string(&u)));
    }

    #[test]
    fn is_valid_string_false() {
        /* invalid string fails */
        assert!(!uuid_is_valid_string("not-a-uuid"));
    }

    #[test]
    fn version_bits() {
        /* byte 6 upper nibble encodes version */
        let mut bytes = [0u8; 16];
        bytes[6] = 0x40; /* version 4 */
        let u = uuid_from_bytes(bytes);
        assert_eq!(uuid_version(&u), 4);
    }

    #[test]
    fn from_bytes_identity() {
        /* from_bytes stores exactly those bytes */
        let b = [1u8; 16];
        let u = uuid_from_bytes(b);
        assert_eq!(u.bytes, b);
    }
}
