// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Fuzz target: OXP pack reader with arbitrary byte streams.
//
// `load_pack_from_bytes` performs an integrity check, manifest parsing, and
// per-entry deserialization.  Feeding it random bytes verifies that none of
// those stages panic — they should only return `Err(...)`.

#![no_main]
use libfuzzer_sys::fuzz_target;
use oxihuman_core::load_pack_from_bytes;

fuzz_target!(|data: &[u8]| {
    // `load_pack_from_bytes` should never panic on arbitrary input —
    // integrity failures and parse errors must be returned as `Err`.
    let _ = load_pack_from_bytes(data);
});
