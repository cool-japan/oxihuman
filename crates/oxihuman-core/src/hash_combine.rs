#![allow(dead_code)]

#[allow(dead_code)]
pub fn hash_combine_u64(a: u64, b: u64) -> u64 {
    let mut h = a;
    h ^= b.wrapping_add(0x9e3779b97f4a7c15).wrapping_add(h << 6).wrapping_add(h >> 2);
    h
}

#[allow(dead_code)]
pub fn hash_combine_u32(a: u32, b: u32) -> u32 {
    let mut h = a;
    h ^= b.wrapping_add(0x9e3779b9).wrapping_add(h << 6).wrapping_add(h >> 2);
    h
}

#[allow(dead_code)]
pub fn hash_bytes_fnv(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[allow(dead_code)]
pub fn hash_string_fnv(s: &str) -> u64 {
    hash_bytes_fnv(s.as_bytes())
}

#[allow(dead_code)]
pub fn hash_f32(v: f32) -> u64 {
    hash_bytes_fnv(&v.to_le_bytes())
}

#[allow(dead_code)]
pub fn hash_pair(a: u64, b: u64) -> u64 {
    hash_combine_u64(a, b)
}

#[allow(dead_code)]
pub fn hash_triple(a: u64, b: u64, c: u64) -> u64 {
    hash_combine_u64(hash_combine_u64(a, b), c)
}

#[allow(dead_code)]
pub fn hash_slice(data: &[u64]) -> u64 {
    let mut h: u64 = 0;
    for &v in data {
        h = hash_combine_u64(h, v);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_u64() {
        let h = hash_combine_u64(1, 2);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_combine_u32() {
        let h = hash_combine_u32(1, 2);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_fnv_bytes() {
        let h1 = hash_bytes_fnv(b"hello");
        let h2 = hash_bytes_fnv(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_fnv_string() {
        let h = hash_string_fnv("test");
        assert_ne!(h, 0);
    }

    #[test]
    fn test_fnv_deterministic() {
        let h1 = hash_string_fnv("abc");
        let h2 = hash_string_fnv("abc");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_f32() {
        let h1 = hash_f32(1.0);
        let h2 = hash_f32(2.0);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_pair() {
        let h = hash_pair(10, 20);
        assert_ne!(h, hash_pair(20, 10));
    }

    #[test]
    fn test_hash_triple() {
        let h = hash_triple(1, 2, 3);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_hash_slice() {
        let h = hash_slice(&[1, 2, 3, 4]);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_hash_empty_slice() {
        let h = hash_slice(&[]);
        assert_eq!(h, 0);
    }
}
