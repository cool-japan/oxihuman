// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/* Hamming(7,4): 4 data bits → 7-bit codeword.
Bit positions (1-indexed): p1=1, p2=2, d1=3, p3=4, d2=5, d3=6, d4=7.
In 0-indexed bit array: bits[0..6] = [p1,p2,d1,p4,d2,d3,d4]. */

pub fn hamming_encode_nibble(nibble: u8) -> u8 {
    let d = nibble & 0x0F;
    let d1 = d & 1;
    let d2 = (d >> 1) & 1;
    let d3 = (d >> 2) & 1;
    let d4 = (d >> 3) & 1;
    let p1 = d1 ^ d2 ^ d4;
    let p2 = d1 ^ d3 ^ d4;
    let p3 = d2 ^ d3 ^ d4;
    /* codeword bits[0..6] = p1 p2 d1 p3 d2 d3 d4 */
    p1 | (p2 << 1) | (d1 << 2) | (p3 << 3) | (d2 << 4) | (d3 << 5) | (d4 << 6)
}

pub fn hamming_syndrome(codeword: u8) -> u8 {
    let c = codeword & 0x7F;
    let bit = |pos: u8| -> u8 { (c >> pos) & 1 };
    let s1 = bit(0) ^ bit(2) ^ bit(4) ^ bit(6);
    let s2 = bit(1) ^ bit(2) ^ bit(5) ^ bit(6);
    let s3 = bit(3) ^ bit(4) ^ bit(5) ^ bit(6);
    s1 | (s2 << 1) | (s3 << 2)
}

pub fn hamming_is_valid(codeword: u8) -> bool {
    hamming_syndrome(codeword) == 0
}

pub fn hamming_introduce_error(codeword: u8, bit_pos: u8) -> u8 {
    codeword ^ (1 << (bit_pos % 7))
}

pub fn hamming_decode_nibble(codeword: u8) -> (u8, bool) {
    let syndrome = hamming_syndrome(codeword);
    let corrected = if syndrome == 0 {
        codeword
    } else {
        /* syndrome points to 1-indexed error bit; 0-indexed = syndrome-1 */
        codeword ^ (1 << (syndrome - 1))
    };
    let was_corrected = syndrome != 0;
    /* extract data bits: d1=bit2, d2=bit4, d3=bit5, d4=bit6 */
    let c = corrected & 0x7F;
    let d1 = (c >> 2) & 1;
    let d2 = (c >> 4) & 1;
    let d3 = (c >> 5) & 1;
    let d4 = (c >> 6) & 1;
    let data = d1 | (d2 << 1) | (d3 << 2) | (d4 << 3);
    (data, was_corrected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_valid_codeword() {
        /* encoded codeword has zero syndrome */
        for nibble in 0u8..=15 {
            let cw = hamming_encode_nibble(nibble);
            assert!(
                hamming_is_valid(cw),
                "nibble {nibble} → cw {cw:#010b} invalid"
            );
        }
    }

    #[test]
    fn decode_no_error() {
        /* decoding a valid codeword returns original nibble, not corrected */
        for nibble in 0u8..=15 {
            let cw = hamming_encode_nibble(nibble);
            let (data, corrected) = hamming_decode_nibble(cw);
            assert_eq!(data, nibble);
            assert!(!corrected);
        }
    }

    #[test]
    fn single_bit_correction() {
        /* a single flipped bit is corrected */
        let cw = hamming_encode_nibble(0b1010);
        let err_cw = hamming_introduce_error(cw, 2);
        let (data, corrected) = hamming_decode_nibble(err_cw);
        assert_eq!(data, 0b1010);
        assert!(corrected);
    }

    #[test]
    fn syndrome_zero_for_valid() {
        /* syndrome of valid codeword is 0 */
        assert_eq!(hamming_syndrome(hamming_encode_nibble(7)), 0);
    }

    #[test]
    fn introduce_error_flips_bit() {
        /* introducing error changes codeword */
        let cw = hamming_encode_nibble(5);
        let err = hamming_introduce_error(cw, 0);
        assert_ne!(cw, err);
    }

    #[test]
    fn is_valid_false_for_error() {
        /* codeword with error has non-zero syndrome */
        let cw = hamming_encode_nibble(3);
        let err = hamming_introduce_error(cw, 1);
        assert!(!hamming_is_valid(err));
    }

    #[test]
    fn encode_zero_nibble() {
        /* nibble 0 encodes to all-zero codeword */
        assert_eq!(hamming_encode_nibble(0), 0);
    }
}
