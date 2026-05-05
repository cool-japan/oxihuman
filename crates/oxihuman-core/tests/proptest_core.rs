// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Property-based tests for oxihuman-core: LZ77, Huffman, Base64, SHA-256.

use proptest::prelude::*;

// ---------------------------------------------------------------------------
// LZ77 compress/decompress roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn lz77_roundtrip(data in proptest::collection::vec(any::<u8>(), 0..512)) {
        let tokens = oxihuman_core::lz77::compress(&data);
        let recovered = oxihuman_core::lz77::decompress(&tokens);
        prop_assert_eq!(&recovered, &data,
            "LZ77 roundtrip failed for input of length {}", data.len());
    }

    #[test]
    fn lz77_roundtrip_with_params(
        data in proptest::collection::vec(any::<u8>(), 0..256),
        window in 64usize..4096,
        max_match in 3usize..258,
    ) {
        let tokens = oxihuman_core::lz77::compress_with_params(&data, window, max_match);
        let recovered = oxihuman_core::lz77::decompress(&tokens);
        prop_assert_eq!(&recovered, &data,
            "LZ77 roundtrip with params (w={}, m={}) failed", window, max_match);
    }
}

// ---------------------------------------------------------------------------
// Huffman encode/decode roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn huffman_roundtrip(data in proptest::collection::vec(any::<u8>(), 1..256)) {
        // Build table from data
        let table = oxihuman_core::huffman::HuffmanCodeTable::from_data(&data);
        let table = match table {
            Some(t) => t,
            None => return Ok(()), // empty data edge case
        };

        // Encode
        let (encoded_bytes, bit_count) = oxihuman_core::huffman::huffman_encode(&data, &table).expect("should succeed");

        // Decode
        let decoded = oxihuman_core::huffman::huffman_decode(
            &encoded_bytes,
            bit_count,
            data.len(),
            &table,
        ).expect("should succeed");

        prop_assert_eq!(&decoded, &data,
            "Huffman roundtrip failed for input of length {}", data.len());
    }
}

// ---------------------------------------------------------------------------
// Base64 encode/decode roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn base64_roundtrip(data in proptest::collection::vec(any::<u8>(), 0..512)) {
        let encoded = oxihuman_core::base64_codec::base64_encode(&data);
        let decoded = oxihuman_core::base64_codec::base64_decode(&encoded).expect("should succeed");
        prop_assert_eq!(&decoded, &data,
            "Base64 roundtrip failed for input of length {}", data.len());
    }
}

// ---------------------------------------------------------------------------
// SHA-256 hash is always 64 hex characters
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn sha256_hex_length(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let digest = oxihuman_core::hashing_sha256::sha256_hash(&data);
        let hex = digest.to_hex();
        prop_assert_eq!(hex.len(), 64,
            "SHA-256 hex digest should always be 64 characters, got {}", hex.len());
        // Every character must be a valid hex digit
        prop_assert!(hex.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 hex digest contains non-hex characters: {}", hex);
    }

    #[test]
    fn sha256_raw_bytes_length(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let digest = oxihuman_core::hashing_sha256::sha256_hash(&data);
        prop_assert_eq!(digest.as_bytes().len(), 32,
            "SHA-256 digest should always be 32 bytes");
    }

    #[test]
    fn sha256_deterministic(data in proptest::collection::vec(any::<u8>(), 0..512)) {
        let d1 = oxihuman_core::hashing_sha256::sha256_hash(&data);
        let d2 = oxihuman_core::hashing_sha256::sha256_hash(&data);
        prop_assert_eq!(d1, d2,
            "SHA-256 should be deterministic for the same input");
    }
}
