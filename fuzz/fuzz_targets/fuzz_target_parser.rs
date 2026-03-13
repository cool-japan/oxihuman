// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Fuzz target: AssetPackBuilder round-trip with arbitrary name strings.
//
// Exercises the name-sanitisation and ZIP-creation path inside
// `AssetPackBuilder::build` — any input that triggers a panic (instead of
// returning an `Err`) would be a bug.

#![no_main]
use libfuzzer_sys::fuzz_target;
use oxihuman_core::AssetPackBuilder;

fuzz_target!(|data: &[u8]| {
    // Only proceed when the data is valid UTF-8; the interesting surface is the
    // pack-name parsing / sanitisation logic, not raw-bytes handling.
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let s = s.trim();

    // Guard against extremely long names that would just exercise allocator
    // behaviour rather than business logic.
    if s.is_empty() || s.len() > 255 {
        return;
    }

    let mut builder = AssetPackBuilder::new(s);
    builder.set_author("fuzz");
    builder.set_version("0.0.0");
    builder.set_license("MIT");

    // `build()` should never panic — only return Ok or Err.
    let _ = builder.build();
});
