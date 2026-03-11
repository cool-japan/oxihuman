#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Bitset {
    bits: Vec<u64>,
    num_bits: usize,
}

#[allow(dead_code)]
pub fn new_bitset(num_bits: usize) -> Bitset {
    let words = num_bits.div_ceil(64);
    Bitset {
        bits: vec![0u64; words],
        num_bits,
    }
}

#[allow(dead_code)]
pub fn bitset_set(bs: &mut Bitset, idx: usize) {
    if idx < bs.num_bits {
        let word = idx / 64;
        let bit = idx % 64;
        bs.bits[word] |= 1u64 << bit;
    }
}

#[allow(dead_code)]
pub fn bitset_get(bs: &Bitset, idx: usize) -> bool {
    if idx < bs.num_bits {
        let word = idx / 64;
        let bit = idx % 64;
        (bs.bits[word] >> bit) & 1 == 1
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn bitset_clear_bit(bs: &mut Bitset, idx: usize) {
    if idx < bs.num_bits {
        let word = idx / 64;
        let bit = idx % 64;
        bs.bits[word] &= !(1u64 << bit);
    }
}

#[allow(dead_code)]
pub fn bitset_and(a: &Bitset, b: &Bitset) -> Bitset {
    let len = a.bits.len().min(b.bits.len());
    let mut result = new_bitset(a.num_bits.min(b.num_bits));
    for i in 0..len {
        result.bits[i] = a.bits[i] & b.bits[i];
    }
    result
}

#[allow(dead_code)]
pub fn bitset_or(a: &Bitset, b: &Bitset) -> Bitset {
    let len = a.bits.len().max(b.bits.len());
    let mut result = new_bitset(a.num_bits.max(b.num_bits));
    for i in 0..len {
        let va = if i < a.bits.len() { a.bits[i] } else { 0 };
        let vb = if i < b.bits.len() { b.bits[i] } else { 0 };
        result.bits[i] = va | vb;
    }
    result
}

#[allow(dead_code)]
pub fn bitset_xor(a: &Bitset, b: &Bitset) -> Bitset {
    let len = a.bits.len().max(b.bits.len());
    let mut result = new_bitset(a.num_bits.max(b.num_bits));
    for i in 0..len {
        let va = if i < a.bits.len() { a.bits[i] } else { 0 };
        let vb = if i < b.bits.len() { b.bits[i] } else { 0 };
        result.bits[i] = va ^ vb;
    }
    result
}

#[allow(dead_code)]
pub fn bitset_count_ones(bs: &Bitset) -> u32 {
    bs.bits.iter().map(|w| w.count_ones()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bitset() {
        let bs = new_bitset(128);
        assert_eq!(bitset_count_ones(&bs), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut bs = new_bitset(64);
        bitset_set(&mut bs, 0);
        assert!(bitset_get(&bs, 0));
        assert!(!bitset_get(&bs, 1));
    }

    #[test]
    fn test_clear_bit() {
        let mut bs = new_bitset(64);
        bitset_set(&mut bs, 5);
        bitset_clear_bit(&mut bs, 5);
        assert!(!bitset_get(&bs, 5));
    }

    #[test]
    fn test_and() {
        let mut a = new_bitset(64);
        let mut b = new_bitset(64);
        bitset_set(&mut a, 0);
        bitset_set(&mut a, 1);
        bitset_set(&mut b, 1);
        bitset_set(&mut b, 2);
        let c = bitset_and(&a, &b);
        assert!(!bitset_get(&c, 0));
        assert!(bitset_get(&c, 1));
        assert!(!bitset_get(&c, 2));
    }

    #[test]
    fn test_or() {
        let mut a = new_bitset(64);
        let mut b = new_bitset(64);
        bitset_set(&mut a, 0);
        bitset_set(&mut b, 1);
        let c = bitset_or(&a, &b);
        assert!(bitset_get(&c, 0));
        assert!(bitset_get(&c, 1));
    }

    #[test]
    fn test_xor() {
        let mut a = new_bitset(64);
        let mut b = new_bitset(64);
        bitset_set(&mut a, 0);
        bitset_set(&mut a, 1);
        bitset_set(&mut b, 1);
        let c = bitset_xor(&a, &b);
        assert!(bitset_get(&c, 0));
        assert!(!bitset_get(&c, 1));
    }

    #[test]
    fn test_count_ones() {
        let mut bs = new_bitset(256);
        bitset_set(&mut bs, 0);
        bitset_set(&mut bs, 100);
        bitset_set(&mut bs, 200);
        assert_eq!(bitset_count_ones(&bs), 3);
    }

    #[test]
    fn test_out_of_bounds_get() {
        let bs = new_bitset(8);
        assert!(!bitset_get(&bs, 100));
    }

    #[test]
    fn test_large_index() {
        let mut bs = new_bitset(1024);
        bitset_set(&mut bs, 1000);
        assert!(bitset_get(&bs, 1000));
    }

    #[test]
    fn test_empty_bitset() {
        let bs = new_bitset(0);
        assert_eq!(bitset_count_ones(&bs), 0);
    }
}
