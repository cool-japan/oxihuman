#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A tagged pointer storing a value and tag bits in a u64.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaggedPtr {
    bits: u64,
}

const TAG_BITS: u32 = 3;
const TAG_MASK: u64 = (1 << TAG_BITS) - 1;
const PTR_MASK: u64 = !TAG_MASK;

#[allow(dead_code)]
pub fn new_tagged_ptr(value: u64, tag: u8) -> TaggedPtr {
    TaggedPtr {
        bits: (value & PTR_MASK) | (u64::from(tag) & TAG_MASK),
    }
}

#[allow(dead_code)]
pub fn ptr_tag(tp: &TaggedPtr) -> u8 {
    (tp.bits & TAG_MASK) as u8
}

#[allow(dead_code)]
pub fn ptr_value(tp: &TaggedPtr) -> u64 {
    tp.bits & PTR_MASK
}

#[allow(dead_code)]
pub fn set_ptr_tag(tp: &mut TaggedPtr, tag: u8) {
    tp.bits = (tp.bits & PTR_MASK) | (u64::from(tag) & TAG_MASK);
}

#[allow(dead_code)]
pub fn clear_ptr_tag(tp: &mut TaggedPtr) {
    tp.bits &= PTR_MASK;
}

#[allow(dead_code)]
pub fn tagged_to_u64(tp: &TaggedPtr) -> u64 {
    tp.bits
}

#[allow(dead_code)]
pub fn tagged_from_parts(value: u64, tag: u8) -> TaggedPtr {
    new_tagged_ptr(value, tag)
}

#[allow(dead_code)]
pub fn tagged_is_null(tp: &TaggedPtr) -> bool {
    ptr_value(tp) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tagged_ptr() {
        let tp = new_tagged_ptr(0x1000, 3);
        assert_eq!(ptr_tag(&tp), 3);
    }

    #[test]
    fn test_ptr_value() {
        let tp = new_tagged_ptr(0x1000, 0);
        assert_eq!(ptr_value(&tp), 0x1000);
    }

    #[test]
    fn test_ptr_tag() {
        let tp = new_tagged_ptr(0, 5);
        assert_eq!(ptr_tag(&tp), 5);
    }

    #[test]
    fn test_set_ptr_tag() {
        let mut tp = new_tagged_ptr(0x1000, 0);
        set_ptr_tag(&mut tp, 7);
        assert_eq!(ptr_tag(&tp), 7);
        assert_eq!(ptr_value(&tp), 0x1000);
    }

    #[test]
    fn test_clear_ptr_tag() {
        let mut tp = new_tagged_ptr(0x1000, 7);
        clear_ptr_tag(&mut tp);
        assert_eq!(ptr_tag(&tp), 0);
    }

    #[test]
    fn test_tagged_to_u64() {
        let tp = new_tagged_ptr(8, 3);
        let v = tagged_to_u64(&tp);
        assert_eq!(v & TAG_MASK, 3);
    }

    #[test]
    fn test_tagged_from_parts() {
        let tp = tagged_from_parts(0x1000, 2);
        assert_eq!(ptr_tag(&tp), 2);
        assert_eq!(ptr_value(&tp), 0x1000);
    }

    #[test]
    fn test_tagged_is_null() {
        let tp = new_tagged_ptr(0, 3);
        assert!(tagged_is_null(&tp));
    }

    #[test]
    fn test_tagged_not_null() {
        let tp = new_tagged_ptr(0x1000, 0);
        assert!(!tagged_is_null(&tp));
    }

    #[test]
    fn test_tag_mask() {
        let tp = new_tagged_ptr(0, 0xFF);
        assert_eq!(ptr_tag(&tp), 7); // only lowest 3 bits
    }
}
