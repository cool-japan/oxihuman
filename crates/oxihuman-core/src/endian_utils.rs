// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn u16_to_le(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}
pub fn u16_from_le(b: [u8; 2]) -> u16 {
    u16::from_le_bytes(b)
}
pub fn u32_to_le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}
pub fn u32_from_le(b: [u8; 4]) -> u32 {
    u32::from_le_bytes(b)
}
pub fn u32_to_be(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}
pub fn u32_from_be(b: [u8; 4]) -> u32 {
    u32::from_be_bytes(b)
}
pub fn f32_to_le_bytes(v: f32) -> [u8; 4] {
    v.to_le_bytes()
}
pub fn f32_from_le_bytes(b: [u8; 4]) -> f32 {
    f32::from_le_bytes(b)
}

pub fn is_little_endian() -> bool {
    let v: u16 = 1;
    v.to_ne_bytes()[0] == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_roundtrip_le() {
        /* u16 little-endian roundtrip */
        assert_eq!(u16_from_le(u16_to_le(0xABCD)), 0xABCD);
    }

    #[test]
    fn u32_roundtrip_le() {
        /* u32 little-endian roundtrip */
        assert_eq!(u32_from_le(u32_to_le(0xDEAD_BEEF)), 0xDEAD_BEEF);
    }

    #[test]
    fn u32_roundtrip_be() {
        /* u32 big-endian roundtrip */
        assert_eq!(u32_from_be(u32_to_be(0x12345678)), 0x12345678);
    }

    #[test]
    fn le_and_be_differ() {
        /* little-endian and big-endian byte order differ for non-palindrome values */
        assert_ne!(u32_to_le(0x01020304), u32_to_be(0x01020304));
    }

    #[test]
    fn f32_roundtrip_le() {
        /* f32 little-endian roundtrip */
        let v = std::f32::consts::PI;
        assert_eq!(f32_from_le_bytes(f32_to_le_bytes(v)), v);
    }

    #[test]
    fn u16_le_byte_order() {
        /* little-endian stores LSB first */
        let b = u16_to_le(0x0102);
        assert_eq!(b[0], 0x02);
        assert_eq!(b[1], 0x01);
    }

    #[test]
    fn is_little_endian_consistent() {
        /* is_little_endian result is consistent with to_ne_bytes */
        let v: u32 = 1;
        let ne = v.to_ne_bytes();
        assert_eq!(is_little_endian(), ne[0] == 1);
    }

    #[test]
    fn u32_be_byte_order() {
        /* big-endian stores MSB first */
        let b = u32_to_be(0x01020304);
        assert_eq!(b[0], 0x01);
    }
}
