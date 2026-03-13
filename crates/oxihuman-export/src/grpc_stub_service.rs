// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! gRPC service-layer helpers: stream convenience builders, gRPC-Web text
//! encoding/decoding, error-detail metadata encoding, legacy request/response
//! stubs, and all tests.
//!
//! Type definitions and frame/stream primitives live in
//! [`super::grpc_stub_types`].

use super::grpc_stub_types::{
    GrpcError, GrpcFrame, GrpcStatus, GrpcStatusCode, GrpcStream, Metadata, StreamKind,
};

// ---------------------------------------------------------------------------
// Unary / streaming convenience builders
// ---------------------------------------------------------------------------

/// Encode a unary RPC response: single message frame + trailers.
pub fn encode_unary_response(
    body: Vec<u8>,
    status: GrpcStatusCode,
    trailing: &Metadata,
) -> Vec<u8> {
    let mut stream = GrpcStream::new(StreamKind::Unary);
    stream.push_message(body);
    stream.trailing_metadata = trailing.clone();
    stream.status = status;
    stream.encode()
}

/// Encode a server-streaming response from a list of message bodies.
pub fn encode_server_stream(
    bodies: &[Vec<u8>],
    status: GrpcStatusCode,
    trailing: &Metadata,
) -> Vec<u8> {
    let mut stream = GrpcStream::new(StreamKind::ServerStreaming);
    for body in bodies {
        stream.push_message(body.clone());
    }
    stream.trailing_metadata = trailing.clone();
    stream.status = status;
    stream.encode()
}

/// Encode a client-streaming request from a list of message bodies.
pub fn encode_client_stream(bodies: &[Vec<u8>]) -> Vec<u8> {
    let mut buf = Vec::new();
    for body in bodies {
        buf.extend_from_slice(&GrpcFrame::new(body.clone()).encode());
    }
    buf
}

/// Encode a bidi-streaming sequence of frames.
pub fn encode_bidi_frames(bodies: &[Vec<u8>]) -> Vec<u8> {
    let mut buf = Vec::new();
    for body in bodies {
        buf.extend_from_slice(&GrpcFrame::new(body.clone()).encode());
    }
    buf
}

// ---------------------------------------------------------------------------
// gRPC-Web text format (base64-wrapped frames)
// ---------------------------------------------------------------------------

/// Simple Base64 encoding (standard alphabet, with padding).
mod base64_codec {
    const ENCODE_TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub(crate) fn encode(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(ENCODE_TABLE[((triple >> 18) & 0x3F) as usize] as char);
            out.push(ENCODE_TABLE[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                out.push(ENCODE_TABLE[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(ENCODE_TABLE[(triple & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }

    fn decode_char(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => Some(0),
            _ => None,
        }
    }

    pub(crate) fn decode(input: &str) -> Result<Vec<u8>, String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(Vec::new());
        }
        if !input.len().is_multiple_of(4) {
            return Err(format!(
                "invalid base64 length: {} (not a multiple of 4)",
                input.len()
            ));
        }
        let mut out = Vec::with_capacity(input.len() / 4 * 3);
        for chunk in input.as_bytes().chunks(4) {
            let a = decode_char(chunk[0])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[0] as char))?;
            let b = decode_char(chunk[1])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[1] as char))?;
            let c = decode_char(chunk[2])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[2] as char))?;
            let d = decode_char(chunk[3])
                .ok_or_else(|| format!("invalid base64 char: {}", chunk[3] as char))?;
            let triple = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
            out.push(((triple >> 16) & 0xFF) as u8);
            if chunk[2] != b'=' {
                out.push(((triple >> 8) & 0xFF) as u8);
            }
            if chunk[3] != b'=' {
                out.push((triple & 0xFF) as u8);
            }
        }
        Ok(out)
    }
}

/// Encode raw gRPC frame bytes into gRPC-Web text format (base64).
pub fn grpc_web_text_encode(data: &[u8]) -> String {
    base64_codec::encode(data)
}

/// Decode gRPC-Web text format (base64) back to raw frame bytes.
pub fn grpc_web_text_decode(text: &str) -> Result<Vec<u8>, GrpcError> {
    base64_codec::decode(text).map_err(GrpcError::Base64Decode)
}

/// Encode a full gRPC stream as a gRPC-Web text response.
pub fn grpc_web_text_encode_stream(stream: &GrpcStream) -> String {
    let raw = stream.encode();
    grpc_web_text_encode(&raw)
}

/// Decode a gRPC-Web text response back into a stream.
pub fn grpc_web_text_decode_stream(text: &str, kind: StreamKind) -> Result<GrpcStream, GrpcError> {
    let raw = grpc_web_text_decode(text)?;
    GrpcStream::decode(&raw, kind)
}

// ---------------------------------------------------------------------------
// Error detail metadata encoding
// ---------------------------------------------------------------------------

/// Encode a [`GrpcStatus`] with details into trailing metadata.
pub fn encode_error_details_metadata(status: &GrpcStatus) -> Result<Metadata, GrpcError> {
    let mut md = Metadata::new();
    md.insert("grpc-status", format!("{}", status.code));
    if !status.message.is_empty() {
        md.insert("grpc-message", &status.message);
    }
    if !status.details.is_empty() {
        let json_bytes = status.encode_json()?;
        let encoded = base64_codec::encode(&json_bytes);
        md.insert("grpc-status-details-bin", &encoded);
    }
    Ok(md)
}

/// Decode error details from trailing metadata.
pub fn decode_error_details_metadata(md: &Metadata) -> Result<GrpcStatus, GrpcError> {
    let code = md
        .get("grpc-status")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let message = md.get("grpc-message").unwrap_or("").to_owned();

    let details = if let Some(bin) = md.get("grpc-status-details-bin") {
        let json_bytes = base64_codec::decode(bin).map_err(GrpcError::Base64Decode)?;
        let full: GrpcStatus = serde_json::from_slice(&json_bytes)
            .map_err(|e| GrpcError::Serialization(e.to_string()))?;
        full.details
    } else {
        Vec::new()
    };

    Ok(GrpcStatus {
        code,
        message,
        details,
    })
}

// ---------------------------------------------------------------------------
// Legacy public API — kept for backwards compatibility
// ---------------------------------------------------------------------------

/// A gRPC request stub.
#[derive(Debug, Clone)]
pub struct GrpcRequest {
    pub method: String,
    pub metadata: Vec<(String, String)>,
    pub frame: GrpcFrame,
}

/// A gRPC response stub.
#[derive(Debug, Clone)]
pub struct GrpcResponse {
    pub status_code: u32,
    pub message: String,
    pub frame: GrpcFrame,
}

/// Build a gRPC request stub with raw body.
pub fn build_grpc_request(method: &str, body: Vec<u8>) -> GrpcRequest {
    GrpcRequest {
        method: method.to_string(),
        metadata: Vec::new(),
        frame: GrpcFrame::new(body),
    }
}

/// Build a gRPC response stub.
pub fn build_grpc_response(status: u32, body: Vec<u8>) -> GrpcResponse {
    GrpcResponse {
        status_code: status,
        message: if status == 0 {
            "OK".to_string()
        } else {
            "Error".to_string()
        },
        frame: GrpcFrame::new(body),
    }
}

/// Add metadata to a request.
pub fn add_metadata(req: &mut GrpcRequest, key: &str, val: &str) {
    req.metadata.push((key.to_string(), val.to_string()));
}

/// Check if a response indicates success.
pub fn is_ok(resp: &GrpcResponse) -> bool {
    resp.status_code == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::grpc_stub_types::{
        decode_trailers_frame, deserialize_morph_targets, encode_trailers_frame, is_trailers_frame,
        serialize_morph_targets, GrpcCompression, GrpcErrorDetail, MeshDataPayload, MetadataEntry,
        MorphTargetPayload,
    };

    // -- Original tests (preserved) ----------------------------------------

    #[test]
    fn frame_encode_length() {
        let frame = GrpcFrame::new(vec![1, 2, 3]);
        let enc = frame.encode();
        assert_eq!(enc.len(), 8);
    }

    #[test]
    fn frame_header_no_compression() {
        let frame = GrpcFrame::new(vec![]);
        let enc = frame.encode();
        assert_eq!(enc[0], 0);
    }

    #[test]
    fn frame_body_length_in_header() {
        let frame = GrpcFrame::new(vec![0xAA, 0xBB]);
        let enc = frame.encode();
        let len = u32::from_be_bytes([enc[1], enc[2], enc[3], enc[4]]);
        assert_eq!(len, 2);
    }

    #[test]
    fn request_method_stored() {
        let req = build_grpc_request("/pkg.Svc/Method", vec![]);
        assert_eq!(req.method, "/pkg.Svc/Method");
    }

    #[test]
    fn response_ok_status() {
        let resp = build_grpc_response(0, vec![]);
        assert!(is_ok(&resp));
    }

    #[test]
    fn response_error_status() {
        let resp = build_grpc_response(1, vec![]);
        assert!(!is_ok(&resp));
    }

    #[test]
    fn add_metadata_count() {
        let mut req = build_grpc_request("/x", vec![]);
        add_metadata(&mut req, "auth", "Bearer token");
        assert_eq!(req.metadata.len(), 1);
    }

    #[test]
    fn encoded_len_helper() {
        let frame = GrpcFrame::new(vec![0u8; 10]);
        assert_eq!(frame.encoded_len(), 15);
    }

    #[test]
    fn empty_body_frame() {
        let frame = GrpcFrame::new(vec![]);
        let enc = frame.encode();
        assert_eq!(enc.len(), 5);
    }

    #[test]
    fn response_message_ok() {
        let resp = build_grpc_response(0, vec![]);
        assert_eq!(resp.message, "OK");
    }

    // -- Status code tests -------------------------------------------------

    #[test]
    fn status_code_all_17() {
        let codes: Vec<u32> = (0..=16).collect();
        for &c in &codes {
            let sc = GrpcStatusCode::from_u32(c);
            assert_eq!(sc as u32, c);
        }
    }

    #[test]
    fn status_code_unknown_for_invalid() {
        assert_eq!(GrpcStatusCode::from_u32(999), GrpcStatusCode::Unknown);
    }

    #[test]
    fn status_code_display() {
        let sc = GrpcStatusCode::NotFound;
        let s = format!("{}", sc);
        assert!(s.contains("NOT_FOUND"));
        assert!(s.contains("5"));
    }

    #[test]
    fn status_code_is_ok() {
        assert!(GrpcStatusCode::Ok.is_ok());
        assert!(!GrpcStatusCode::Internal.is_ok());
    }

    // -- Frame decode tests ------------------------------------------------

    #[test]
    fn frame_decode_roundtrip() {
        let frame = GrpcFrame::new(vec![10, 20, 30, 40]);
        let encoded = frame.encode();
        let (decoded, consumed) = GrpcFrame::decode(&encoded).expect("decode failed");
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.body, vec![10, 20, 30, 40]);
        assert_eq!(decoded.compression, GrpcCompression::None);
    }

    #[test]
    fn frame_decode_insufficient() {
        let result = GrpcFrame::decode(&[0, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn frame_decode_body_truncated() {
        let data = [0u8, 0, 0, 0, 10, 1, 2, 3];
        let result = GrpcFrame::decode(&data);
        assert!(result.is_err());
    }

    #[test]
    fn frame_decode_all_multiple() {
        let f1 = GrpcFrame::new(vec![1, 2]);
        let f2 = GrpcFrame::new(vec![3, 4, 5]);
        let mut data = f1.encode();
        data.extend_from_slice(&f2.encode());
        let frames = GrpcFrame::decode_all(&data).expect("decode_all failed");
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].body, vec![1, 2]);
        assert_eq!(frames[1].body, vec![3, 4, 5]);
    }

    #[test]
    fn frame_with_compression() {
        let frame = GrpcFrame::with_compression(vec![99], GrpcCompression::Gzip);
        let enc = frame.encode();
        assert_eq!(enc[0], 1);
        let (dec, _) = GrpcFrame::decode(&enc).expect("decode failed");
        assert_eq!(dec.compression, GrpcCompression::Gzip);
    }

    // -- Metadata tests ----------------------------------------------------

    #[test]
    fn metadata_insert_get() {
        let mut md = Metadata::new();
        md.insert("content-type", "application/grpc");
        assert_eq!(md.get("content-type"), Some("application/grpc"));
        assert_eq!(md.len(), 1);
    }

    #[test]
    fn metadata_get_all_duplicates() {
        let mut md = Metadata::new();
        md.insert("x-custom", "a");
        md.insert("x-custom", "b");
        let vals = md.get_all("x-custom");
        assert_eq!(vals, vec!["a", "b"]);
    }

    #[test]
    fn metadata_trailers_roundtrip() {
        let mut md = Metadata::new();
        md.insert("grpc-status", "0");
        md.insert("grpc-message", "success");
        let encoded = md.encode_trailers();
        let decoded = Metadata::decode_trailers(&encoded);
        assert_eq!(decoded.get("grpc-status"), Some("0"));
        assert_eq!(decoded.get("grpc-message"), Some("success"));
    }

    #[test]
    fn metadata_binary_key() {
        let entry = MetadataEntry::new("x-data-bin", "base64stuff");
        assert!(entry.is_binary());
        let entry2 = MetadataEntry::new("x-data", "plain");
        assert!(!entry2.is_binary());
    }

    // -- Trailers frame tests ----------------------------------------------

    #[test]
    fn trailers_frame_roundtrip() {
        let mut md = Metadata::new();
        md.insert("grpc-status", "13");
        md.insert("grpc-message", "internal error");
        let frame_bytes = encode_trailers_frame(&md);
        assert!(is_trailers_frame(&frame_bytes));
        let (decoded, consumed) = decode_trailers_frame(&frame_bytes).expect("decode failed");
        assert_eq!(consumed, frame_bytes.len());
        assert_eq!(decoded.get("grpc-status"), Some("13"));
        assert_eq!(decoded.get("grpc-message"), Some("internal error"));
    }

    #[test]
    fn normal_frame_not_trailers() {
        let frame = GrpcFrame::new(vec![1, 2, 3]);
        let encoded = frame.encode();
        assert!(!is_trailers_frame(&encoded));
    }

    // -- Stream framing tests ----------------------------------------------

    #[test]
    fn unary_stream_encode_decode() {
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(vec![42, 43, 44]);
        stream.set_status(GrpcStatusCode::Ok, "");
        let encoded = stream.encode();
        let decoded = GrpcStream::decode(&encoded, StreamKind::Unary).expect("decode failed");
        assert_eq!(decoded.message_count(), 1);
        assert_eq!(decoded.messages[0].body, vec![42, 43, 44]);
        assert_eq!(decoded.status, GrpcStatusCode::Ok);
    }

    #[test]
    fn server_streaming_multiple_messages() {
        let bodies: Vec<Vec<u8>> = vec![vec![1], vec![2, 3], vec![4, 5, 6]];
        let encoded = encode_server_stream(&bodies, GrpcStatusCode::Ok, &Metadata::new());
        let decoded =
            GrpcStream::decode(&encoded, StreamKind::ServerStreaming).expect("decode failed");
        assert_eq!(decoded.message_count(), 3);
        assert_eq!(decoded.messages[2].body, vec![4, 5, 6]);
    }

    #[test]
    fn client_streaming_encode() {
        let bodies = vec![vec![10], vec![20]];
        let encoded = encode_client_stream(&bodies);
        let frames = GrpcFrame::decode_all(&encoded).expect("decode_all failed");
        assert_eq!(frames.len(), 2);
    }

    #[test]
    fn bidi_streaming_encode() {
        let bodies = vec![vec![7], vec![8], vec![9]];
        let encoded = encode_bidi_frames(&bodies);
        let frames = GrpcFrame::decode_all(&encoded).expect("decode_all failed");
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn stream_with_error_status() {
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(vec![]);
        stream.set_status(GrpcStatusCode::NotFound, "resource missing");
        let encoded = stream.encode();
        let decoded = GrpcStream::decode(&encoded, StreamKind::Unary).expect("decode failed");
        assert_eq!(decoded.status, GrpcStatusCode::NotFound);
        assert_eq!(decoded.status_message, "resource missing");
    }

    #[test]
    fn stream_trailing_metadata_preserved() {
        let mut stream = GrpcStream::new(StreamKind::ServerStreaming);
        stream.push_message(vec![1]);
        stream.trailing_metadata.insert("x-request-id", "abc123");
        stream.status = GrpcStatusCode::Ok;
        let encoded = stream.encode();
        let decoded =
            GrpcStream::decode(&encoded, StreamKind::ServerStreaming).expect("decode failed");
        assert_eq!(
            decoded.trailing_metadata.get("x-request-id"),
            Some("abc123")
        );
    }

    #[test]
    fn stream_kind_display() {
        assert_eq!(format!("{}", StreamKind::Unary), "unary");
        assert_eq!(format!("{}", StreamKind::BidiStreaming), "bidi-streaming");
    }

    // -- gRPC-Web text format tests ----------------------------------------

    #[test]
    fn grpc_web_text_roundtrip() {
        let original = vec![0u8, 0, 0, 0, 3, 10, 20, 30];
        let text = grpc_web_text_encode(&original);
        let decoded = grpc_web_text_decode(&text).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn grpc_web_text_stream_roundtrip() {
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(vec![100, 200]);
        stream.status = GrpcStatusCode::Ok;
        let text = grpc_web_text_encode_stream(&stream);
        let decoded =
            grpc_web_text_decode_stream(&text, StreamKind::Unary).expect("decode failed");
        assert_eq!(decoded.message_count(), 1);
        assert_eq!(decoded.messages[0].body, vec![100, 200]);
    }

    #[test]
    fn grpc_web_text_empty() {
        let text = grpc_web_text_encode(&[]);
        assert!(text.is_empty());
        let decoded = grpc_web_text_decode(&text).expect("decode failed");
        assert!(decoded.is_empty());
    }

    #[test]
    fn grpc_web_text_invalid_base64() {
        let result = grpc_web_text_decode("!!!!");
        assert!(result.is_err());
    }

    // -- Error detail tests ------------------------------------------------

    #[test]
    fn error_detail_encoding_roundtrip() {
        let mut status = GrpcStatus::new(GrpcStatusCode::InvalidArgument, "bad field");
        status.add_detail(
            GrpcErrorDetail::new(
                "type.googleapis.com/google.rpc.BadRequest",
                "field 'name' is empty",
            )
            .with_metadata("field", "name"),
        );
        let md = encode_error_details_metadata(&status).expect("encode failed");
        assert_eq!(md.get("grpc-status"), Some("3"));
        assert_eq!(md.get("grpc-message"), Some("bad field"));
        assert!(md.get("grpc-status-details-bin").is_some());
        let decoded = decode_error_details_metadata(&md).expect("decode failed");
        assert_eq!(decoded.code, 3);
        assert_eq!(decoded.message, "bad field");
        assert_eq!(decoded.details.len(), 1);
        assert_eq!(decoded.details[0].metadata[0].0, "field");
    }

    #[test]
    fn grpc_status_ok_no_details() {
        let status = GrpcStatus::ok();
        assert!(status.is_ok());
        assert!(status.details.is_empty());
    }

    #[test]
    fn grpc_status_json_roundtrip() {
        let status = GrpcStatus::new(GrpcStatusCode::Internal, "crash");
        let json = status.encode_json().expect("encode failed");
        let decoded = GrpcStatus::decode_json(&json).expect("decode failed");
        assert_eq!(decoded.code, 13);
        assert_eq!(decoded.message, "crash");
    }

    // -- Mesh data payload tests -------------------------------------------

    #[test]
    fn mesh_payload_roundtrip() {
        let mut payload = MeshDataPayload::new();
        payload.vertex_count = 3;
        payload.positions = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        payload.normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        payload.indices = vec![0, 1, 2];
        let bytes = payload.serialize();
        let decoded = MeshDataPayload::deserialize(&bytes).expect("deserialize failed");
        assert_eq!(decoded.vertex_count, 3);
        assert_eq!(decoded.positions.len(), 9);
        assert_eq!(decoded.indices, vec![0, 1, 2]);
    }

    #[test]
    fn mesh_payload_to_frame() {
        let mut payload = MeshDataPayload::new();
        payload.vertex_count = 1;
        payload.positions = vec![1.0, 2.0, 3.0];
        let frame = payload.to_frame();
        assert!(!frame.body.is_empty());
        let enc = frame.encode();
        let (dec, _) = GrpcFrame::decode(&enc).expect("decode failed");
        let restored = MeshDataPayload::deserialize(&dec.body).expect("deserialize failed");
        assert_eq!(restored.vertex_count, 1);
    }

    #[test]
    fn mesh_payload_empty() {
        let payload = MeshDataPayload::new();
        let bytes = payload.serialize();
        let decoded = MeshDataPayload::deserialize(&bytes).expect("deserialize failed");
        assert_eq!(decoded.vertex_count, 0);
        assert!(decoded.positions.is_empty());
    }

    // -- Morph target payload tests ----------------------------------------

    #[test]
    fn morph_payload_roundtrip() {
        let mut morph = MorphTargetPayload::new("smile", 0.75);
        morph.deltas = vec![0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
        let bytes = morph.serialize();
        let decoded = MorphTargetPayload::deserialize(&bytes).expect("deserialize failed");
        assert_eq!(decoded.name, "smile");
        assert!((decoded.weight - 0.75).abs() < f32::EPSILON);
        assert_eq!(decoded.deltas.len(), 6);
    }

    #[test]
    fn morph_payload_to_frame() {
        let morph = MorphTargetPayload::new("blink_L", 1.0);
        let frame = morph.to_frame();
        let enc = frame.encode();
        let (dec, _) = GrpcFrame::decode(&enc).expect("decode failed");
        let restored = MorphTargetPayload::deserialize(&dec.body).expect("deserialize failed");
        assert_eq!(restored.name, "blink_L");
    }

    #[test]
    fn multiple_morph_targets_roundtrip() {
        let targets = vec![
            {
                let mut m = MorphTargetPayload::new("jawOpen", 0.5);
                m.deltas = vec![0.1, 0.2];
                m
            },
            {
                let mut m = MorphTargetPayload::new("eyeClose", 1.0);
                m.deltas = vec![0.3, 0.4, 0.5];
                m
            },
        ];
        let bytes = serialize_morph_targets(&targets);
        let decoded = deserialize_morph_targets(&bytes).expect("deserialize failed");
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].name, "jawOpen");
        assert_eq!(decoded[1].name, "eyeClose");
        assert_eq!(decoded[1].deltas.len(), 3);
    }

    #[test]
    fn morph_targets_empty_list() {
        let targets: Vec<MorphTargetPayload> = vec![];
        let bytes = serialize_morph_targets(&targets);
        let decoded = deserialize_morph_targets(&bytes).expect("deserialize failed");
        assert!(decoded.is_empty());
    }

    // -- Base64 codec edge cases -------------------------------------------

    #[test]
    fn base64_padding_1() {
        let data = vec![1, 2, 3, 4, 5];
        let encoded = base64_codec::encode(&data);
        let decoded = base64_codec::decode(&encoded).expect("decode failed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_padding_2() {
        let data = vec![255];
        let encoded = base64_codec::encode(&data);
        assert!(encoded.ends_with("=="));
        let decoded = base64_codec::decode(&encoded).expect("decode failed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_no_padding() {
        let data = vec![1, 2, 3];
        let encoded = base64_codec::encode(&data);
        assert!(!encoded.contains('='));
        let decoded = base64_codec::decode(&encoded).expect("decode failed");
        assert_eq!(decoded, data);
    }

    // -- Integration tests -------------------------------------------------

    #[test]
    fn mesh_over_grpc_web_text() {
        let mut payload = MeshDataPayload::new();
        payload.vertex_count = 2;
        payload.positions = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        payload.normals = vec![0.0, 1.0, 0.0, 0.0, 1.0, 0.0];
        payload.indices = vec![0, 1];
        let mut stream = GrpcStream::new(StreamKind::Unary);
        stream.push_message(payload.serialize());
        stream.status = GrpcStatusCode::Ok;
        let text = grpc_web_text_encode_stream(&stream);
        let decoded =
            grpc_web_text_decode_stream(&text, StreamKind::Unary).expect("decode failed");
        let restored =
            MeshDataPayload::deserialize(&decoded.messages[0].body).expect("deserialize failed");
        assert_eq!(restored.vertex_count, 2);
        assert_eq!(restored.indices, vec![0, 1]);
    }

    #[test]
    fn morph_stream_server_streaming() {
        let targets = [
            MorphTargetPayload::new("a", 0.1),
            MorphTargetPayload::new("b", 0.2),
        ];
        let bodies: Vec<Vec<u8>> = targets.iter().map(|t| t.serialize()).collect();
        let encoded = encode_server_stream(&bodies, GrpcStatusCode::Ok, &Metadata::new());
        let decoded =
            GrpcStream::decode(&encoded, StreamKind::ServerStreaming).expect("decode failed");
        assert_eq!(decoded.message_count(), 2);
        let t0 = MorphTargetPayload::deserialize(&decoded.messages[0].body)
            .expect("deserialize failed");
        assert_eq!(t0.name, "a");
    }

    #[test]
    fn unsupported_compression_flag() {
        let data = [42u8, 0, 0, 0, 1, 0xFF];
        let result = GrpcFrame::decode(&data);
        assert!(result.is_err());
        let err_msg = format!("{}", result.expect_err("should fail"));
        assert!(err_msg.contains("unsupported compression"));
    }
}
