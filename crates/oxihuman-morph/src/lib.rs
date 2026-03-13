// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Morphology engine for parametric human body generation.
//!
//! Provides target-based morphing, blendshape interpolation, age and body
//! composition models, FACS facial expressions, pose graphs, and GPU-ready
//! skin deformation — all in pure Rust.
//!
//! # Module Organisation
//!
//! The public API is split across six internal sub-modules to keep individual
//! source files manageable. All items are re-exported at the crate root so
//! callers import everything directly from `oxihuman_morph::`.
//!
//! | Part | Approx. content |
//! |------|-----------------|
//! | `_morph_part1` | Core engine, animation, basic morphing |
//! | `_morph_part2` | Facial rigs, pose systems, body scan |
//! | `_morph_part3` | Fine facial controls (brow/cheek/chin/ear) |
//! | `_morph_part4` | Fine facial controls (eye/face/body segments) |
//! | `_morph_part5` | Morph targets, skinning, ML morphs |
//! | `_morph_part6` | Skeletal morphs, body-shape archetypes, anatomy |

mod _morph_part1;
mod _morph_part2;
mod _morph_part3;
mod _morph_part4;
mod _morph_part5;
mod _morph_part6;

pub mod fabrik_ik;
pub use fabrik_ik::IkChain;

pub use _morph_part1::*;
pub use _morph_part2::*;
pub use _morph_part3::*;
pub use _morph_part4::*;
pub use _morph_part5::*;
pub use _morph_part6::*;
