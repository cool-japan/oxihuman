// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Byte manipulation utilities.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BytePackConfig {
    pub order: ByteOrder,
}

#[allow(dead_code)]
pub fn default_byte_pack_config() -> BytePackConfig {
    BytePackConfig { order: ByteOrder::LittleEndian }
}

#[allow(dead_code)]
pub fn u16_to_bytes_le(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

#[allow(dead_code)]
pub fn u16_from_bytes_le(b: [u8; 2]) -> u16 {
    u16::from_le_bytes(b)
}

#[allow(dead_code)]
pub fn u32_to_bytes_le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

#[allow(dead_code)]
pub fn u32_from_bytes_le(b: [u8; 4]) -> u32 {
    u32::from_le_bytes(b)
}

#[allow(dead_code)]
pub fn swap_bytes_u32(v: u32) -> u32 {
    v.swap_bytes()
}

#[allow(dead_code)]
pub fn align_up(value: usize, align: usize) -> usize {
    if align == 0 {
        return value;
    }
    (value + align - 1) & !(align - 1)
}

#[allow(dead_code)]
pub fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

#[allow(dead_code)]
pub fn next_power_of_two(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut p = 1usize;
    while p < n {
        p <<= 1;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_byte_pack_config();
        assert_eq!(cfg.order, ByteOrder::LittleEndian);
    }

    #[test]
    fn test_u16_roundtrip() {
        let v: u16 = 0xABCD;
        let b = u16_to_bytes_le(v);
        assert_eq!(u16_from_bytes_le(b), v);
    }

    #[test]
    fn test_u32_roundtrip() {
        let v: u32 = 0xDEADBEEF;
        let b = u32_to_bytes_le(v);
        assert_eq!(u32_from_bytes_le(b), v);
    }

    #[test]
    fn test_swap_bytes() {
        assert_eq!(swap_bytes_u32(0x12345678), 0x78563412);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(16));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(8), 8);
    }

    #[test]
    fn test_u16_little_endian_bytes() {
        let b = u16_to_bytes_le(0x0102);
        assert_eq!(b[0], 0x02);
        assert_eq!(b[1], 0x01);
    }

    #[test]
    fn test_u32_little_endian_bytes() {
        let b = u32_to_bytes_le(0x01020304);
        assert_eq!(b[0], 0x04);
        assert_eq!(b[3], 0x01);
    }

    #[test]
    fn test_align_up_zero_align() {
        assert_eq!(align_up(10, 0), 10);
    }
}
