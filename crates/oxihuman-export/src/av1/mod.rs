// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pure-Rust AV1 intra still-image codec.
//!
//! Codec profile: Profile 0, reduced_still_picture_header=1, single KEY_FRAME,
//! single tile, 4:4:4, MC_IDENTITY (GBR), lossless backbone (base_q_idx=0,
//! WHT4×4 exact integer), optional lossy DCT/ADST path via AvifPreset.

pub mod bitio;
pub mod cdf_tables;
pub mod color;
pub mod levels;
pub mod leb128;
pub mod obu;
pub mod predict;
pub mod quant;
pub mod scan;
pub mod transform;

pub mod ec;

pub mod frame_header;
pub mod seq_header;
pub mod tile;

pub mod block;
pub mod coeffs;
pub mod partition;

pub mod decoder;
pub mod encoder;

pub mod avif_writer;
pub mod isobmff;
