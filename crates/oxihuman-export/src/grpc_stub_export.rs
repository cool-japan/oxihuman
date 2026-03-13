// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Re-exports for backward compatibility.
//!
//! Implementation is split across:
//! - [`super::grpc_stub_types`]   — protocol types, frames, streams, payloads
//! - [`super::grpc_stub_service`] — convenience builders, gRPC-Web, legacy API

pub use super::grpc_stub_service::{
    add_metadata, build_grpc_request, build_grpc_response, is_ok, GrpcRequest, GrpcResponse,
};
pub use super::grpc_stub_types::{GrpcCompression, GrpcFrame};
