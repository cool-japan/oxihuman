// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Re-exports for backward compatibility.
//!
//! Implementation is split across:
//! - [`super::alembic_ogawa_core`] — data types, encoding, validation, build logic
//! - [`super::alembic_ogawa_io`]   — [`AlembicWriter`] high-level API and file I/O

pub use super::alembic_ogawa_core::{
    identity_matrix, read_data_at, read_group_at, read_root_offset, scale_matrix,
    translation_matrix, unit_cube_polymesh, validate_ogawa_magic, AbcCamera, AbcObject,
    AbcObjectKind, AbcPolyMesh, AbcSubD, AbcXform, AlembicWriter,
};
