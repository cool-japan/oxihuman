//! Fixed-size dense bit array for compact boolean storage (up to 256 bits).
//!
//! Uses four `u64` words for a maximum of 256 bits. All operations are O(1)
//! and branch-free where possible. Useful for compact flag storage, dirty
//! tracking, and selection masks in morph/physics systems.

/// Configuration for the bit array.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitArrayConfig {
    /// Number of usable bits (1–256).
    pub size: usize,
}

#[allow(dead_code)]
impl BitArrayConfig {
    fn new() -> Self {
        Self { size: 64 }
    }
}

/// Returns the default bit array configuration (64-bit).
#[allow(dead_code)]
pub fn default_bit_array_config() -> BitArrayConfig {
    BitArrayConfig::new()
}

/// Fixed-size dense bit array backed by four `u64` words (256 bits max).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitArray {
    words: [u64; 4],
    /// Effective number of bits in use.
    size: usize,
}

/// Creates a new `BitArray` initialised to all zeros.
#[allow(dead_code)]
pub fn new_bit_array(config: BitArrayConfig) -> BitArray {
    BitArray {
        words: [0u64; 4],
        size: config.size.clamp(1, 256),
    }
}

#[inline]
fn word_bit(index: usize) -> (usize, usize) {
    (index / 64, index % 64)
}

/// Sets bit `index` to 1. Silently ignores out-of-range indices.
#[allow(dead_code)]
pub fn bit_set(arr: &mut BitArray, index: usize) {
    if index >= arr.size {
        return;
    }
    let (w, b) = word_bit(index);
    arr.words[w] |= 1u64 << b;
}

/// Clears bit `index` to 0. Silently ignores out-of-range indices.
#[allow(dead_code)]
pub fn bit_clear(arr: &mut BitArray, index: usize) {
    if index >= arr.size {
        return;
    }
    let (w, b) = word_bit(index);
    arr.words[w] &= !(1u64 << b);
}

/// Returns the value of bit `index`. Returns `false` for out-of-range indices.
#[allow(dead_code)]
pub fn bit_get(arr: &BitArray, index: usize) -> bool {
    if index >= arr.size {
        return false;
    }
    let (w, b) = word_bit(index);
    (arr.words[w] >> b) & 1 == 1
}

/// Toggles bit `index`. Silently ignores out-of-range indices.
#[allow(dead_code)]
pub fn bit_toggle(arr: &mut BitArray, index: usize) {
    if index >= arr.size {
        return;
    }
    let (w, b) = word_bit(index);
    arr.words[w] ^= 1u64 << b;
}

/// Returns the number of set bits (population count).
#[allow(dead_code)]
pub fn bit_count_set(arr: &BitArray) -> usize {
    // Count only within the used size.
    let full_words = arr.size / 64;
    let remainder = arr.size % 64;
    let mut count = 0usize;
    for i in 0..full_words {
        count += arr.words[i].count_ones() as usize;
    }
    if remainder > 0 && full_words < 4 {
        let mask = if remainder == 64 { u64::MAX } else { (1u64 << remainder) - 1 };
        count += (arr.words[full_words] & mask).count_ones() as usize;
    }
    count
}

/// Returns the number of clear bits.
#[allow(dead_code)]
pub fn bit_count_clear(arr: &BitArray) -> usize {
    arr.size - bit_count_set(arr)
}

/// Serialises the bit array to a simple JSON string.
#[allow(dead_code)]
pub fn bit_array_to_json(arr: &BitArray) -> String {
    format!(
        "{{\"size\":{},\"set_count\":{},\"words\":[{},{},{},{}]}}",
        arr.size,
        bit_count_set(arr),
        arr.words[0],
        arr.words[1],
        arr.words[2],
        arr.words[3]
    )
}

/// Resets all bits to 0.
#[allow(dead_code)]
pub fn bit_array_reset(arr: &mut BitArray) {
    arr.words = [0u64; 4];
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_arr() -> BitArray {
        new_bit_array(default_bit_array_config())
    }

    #[test]
    fn test_initial_all_clear() {
        let arr = make_arr();
        assert_eq!(bit_count_set(&arr), 0);
        assert_eq!(bit_count_clear(&arr), 64);
    }

    #[test]
    fn test_set_and_get() {
        let mut arr = make_arr();
        bit_set(&mut arr, 7);
        assert!(bit_get(&arr, 7));
        assert!(!bit_get(&arr, 6));
    }

    #[test]
    fn test_clear_bit() {
        let mut arr = make_arr();
        bit_set(&mut arr, 3);
        bit_clear(&mut arr, 3);
        assert!(!bit_get(&arr, 3));
    }

    #[test]
    fn test_toggle() {
        let mut arr = make_arr();
        bit_toggle(&mut arr, 10);
        assert!(bit_get(&arr, 10));
        bit_toggle(&mut arr, 10);
        assert!(!bit_get(&arr, 10));
    }

    #[test]
    fn test_count_set() {
        let mut arr = make_arr();
        bit_set(&mut arr, 0);
        bit_set(&mut arr, 1);
        bit_set(&mut arr, 63);
        assert_eq!(bit_count_set(&arr), 3);
    }

    #[test]
    fn test_count_clear() {
        let mut arr = make_arr();
        bit_set(&mut arr, 5);
        assert_eq!(bit_count_clear(&arr), 63);
    }

    #[test]
    fn test_out_of_range_ignored() {
        let mut arr = make_arr();
        bit_set(&mut arr, 999); // ignored
        assert_eq!(bit_count_set(&arr), 0);
        assert!(!bit_get(&arr, 999));
    }

    #[test]
    fn test_reset() {
        let mut arr = make_arr();
        bit_set(&mut arr, 0);
        bit_set(&mut arr, 32);
        bit_array_reset(&mut arr);
        assert_eq!(bit_count_set(&arr), 0);
    }

    #[test]
    fn test_to_json() {
        let arr = make_arr();
        let json = bit_array_to_json(&arr);
        assert!(json.contains("size"));
        assert!(json.contains("set_count"));
    }

    #[test]
    fn test_256_bit_capacity() {
        let cfg = BitArrayConfig { size: 256 };
        let mut arr = new_bit_array(cfg);
        bit_set(&mut arr, 255);
        assert!(bit_get(&arr, 255));
        assert_eq!(bit_count_set(&arr), 1);
    }

    #[test]
    fn test_cross_word_boundary() {
        let cfg = BitArrayConfig { size: 128 };
        let mut arr = new_bit_array(cfg);
        bit_set(&mut arr, 63);
        bit_set(&mut arr, 64);
        assert!(bit_get(&arr, 63));
        assert!(bit_get(&arr, 64));
        assert_eq!(bit_count_set(&arr), 2);
    }
}
