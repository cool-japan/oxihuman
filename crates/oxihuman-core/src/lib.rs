// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Core utilities, data structures, and algorithms for the OxiHuman engine.
//!
//! This crate is the foundational layer of the OxiHuman workspace. It provides
//! everything that other crates depend on: parsers for `.obj` and `.target`
//! files, the [`Policy`] / [`PolicyProfile`] system for content filtering,
//! spatial indexing with an octree, asset hashing, event buses, undo/redo
//! stacks, plugin registries, and dozens of supporting subsystems.
//!
//! # Quick start
//!
//! ```rust
//! use oxihuman_core::policy::{Policy, PolicyProfile};
//! use oxihuman_core::parser::obj::parse_obj;
//!
//! let obj_src = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nvt 0 0\nf 1/1/1 2/1/1 3/1/1\n";
//! let mesh = parse_obj(obj_src).expect("OBJ parse failed");
//! assert_eq!(mesh.positions.len(), 3);
//!
//! let policy = Policy::new(PolicyProfile::Standard);
//! assert!(policy.is_target_allowed("height", &[]));
//! ```
//!
//! # Content policy
//!
//! All morph targets are filtered through a [`Policy`] before they can affect
//! a mesh. [`PolicyProfile::Standard`] blocks targets whose names or tags
//! contain explicit-content keywords. [`PolicyProfile::Strict`] additionally
//! requires targets to appear in an explicit allowlist.

// ── Part 1: core infrastructure (policy, manifest, parser, asset pack, …)
mod _core_part1;
pub use _core_part1::*;

// ── Part 2: data structures (tree/heap variants, graph algorithms, codecs, …)
mod _core_part2;
pub use _core_part2::*;

// ── Part 3: utilities (locale, statistics, observability, DDD, math, …)
mod _core_part3;
pub use _core_part3::*;
