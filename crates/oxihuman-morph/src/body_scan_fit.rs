// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Fit body parameters to a 3-D point cloud from a body scanner.
//!
//! This module re-exports all public items from the two sub-modules:
//! - `body_scan_fit_core` — scan fitting types and core fitting logic.
//! - `body_scan_fit_icp` — ICP aligner, SVD math, and multi-stage pipeline.

pub(crate) mod body_scan_fit_core;
pub(crate) mod body_scan_fit_icp;

pub use body_scan_fit_core::*;
pub use body_scan_fit_icp::*;
