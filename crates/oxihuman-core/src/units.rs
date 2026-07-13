// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Engine-level unit and safety constants.
//!
//! These constants make policy decisions that are otherwise scattered across
//! the pipeline explicit and queryable by third-party embedders.

/// Minimum modelled age, in years, for any human body OxiHuman produces.
///
/// # Policy rationale
///
/// OxiHuman is an adult-body generator. The shipped core asset pack contains
/// **no** child or adolescent morph data at the data level, and its manifest
/// declares `age_floor_years = 18.0`. This constant is the engine/core-level
/// mirror of that data-level guarantee: any age parameter that maps below
/// this floor must be clamped up to it before a mesh is built.
///
/// Keeping the floor here (rather than only in pack metadata) gives the WASM
/// engine, native embedders, and the policy layer a single authoritative
/// value to enforce and to surface to users. See `SAFETY.md` at the repository
/// root for the full safety model.
pub const AGE_ADULT_FLOOR_YR: f32 = 18.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_floor_is_eighteen() {
        assert!((AGE_ADULT_FLOOR_YR - 18.0).abs() < f32::EPSILON);
    }
}
