// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Core engine logic: `WasmEngine` struct and its implementation.
//! The implementation is split into focused sub-modules declared in `lib.rs`.

pub use crate::engine_core::{
    age_param_to_years, age_years_to_param, Particle, ParticleSystem, WasmEngine, MODEL_UNIT_CM,
};
