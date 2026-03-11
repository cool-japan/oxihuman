// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! gRPC framing codec stub.

/// A gRPC frame header (5 bytes: 1 compression flag + 4-byte length).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrpcFrameHeader {
    /// 0 = not compressed, 1 = compressed.
    pub compressed: bool,
    /// Message length in bytes.
    pub message_len: u32,
}

/// gRPC framing error.
#[derive(Debug, Clone, PartialEq)]
pub enum GrpcError {
    InsufficientData,
    InvalidCompressionFlag(u8),
    MessageTruncated { expected: u32, got: usize },
}

impl std::fmt::Display for GrpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientData => write!(f, "insufficient data for gRPC frame"),
            Self::InvalidCompressionFlag(b) => write!(f, "invalid gRPC compression flag: {b}"),
            Self::MessageTruncated { expected, got } => {
                write!(
                    f,
                    "gRPC message truncated: expected {expected} bytes, got {got}"
                )
            }
        }
    }
}

/// Encode a gRPC frame: 5-byte header + message bytes.
pub fn encode_frame(data: &[u8], compressed: bool, buf: &mut Vec<u8>) {
    buf.push(if compressed { 1 } else { 0 });
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(data);
}

/// Decode a gRPC frame header from the first 5 bytes.
pub fn decode_frame_header(buf: &[u8]) -> Result<GrpcFrameHeader, GrpcError> {
    if buf.len() < 5 {
        return Err(GrpcError::InsufficientData);
    }
    let flag = buf[0];
    if flag > 1 {
        return Err(GrpcError::InvalidCompressionFlag(flag));
    }
    let len_bytes: [u8; 4] = buf[1..5].try_into().unwrap_or_default();
    Ok(GrpcFrameHeader {
        compressed: flag == 1,
        message_len: u32::from_be_bytes(len_bytes),
    })
}

/// Decode a complete gRPC frame, returning the message bytes.
pub fn decode_frame(buf: &[u8]) -> Result<(GrpcFrameHeader, &[u8]), GrpcError> {
    let header = decode_frame_header(buf)?;
    let msg_len = header.message_len as usize;
    if buf.len() < 5 + msg_len {
        return Err(GrpcError::MessageTruncated {
            expected: header.message_len,
            got: buf.len().saturating_sub(5),
        });
    }
    Ok((header, &buf[5..5 + msg_len]))
}

/// Return the total framed length of a message.
pub fn framed_length(msg_len: usize) -> usize {
    5 + msg_len
}

/// Return `true` if the buffer contains a complete gRPC frame.
pub fn is_complete_frame(buf: &[u8]) -> bool {
    if buf.len() < 5 {
        return false;
    }
    let len_bytes: [u8; 4] = buf[1..5].try_into().unwrap_or([0; 4]);
    let msg_len = u32::from_be_bytes(len_bytes) as usize;
    buf.len() >= 5 + msg_len
}

/// Split a byte slice into multiple gRPC frames.
pub fn split_frames(mut buf: &[u8]) -> Vec<Vec<u8>> {
    let mut frames = vec![];
    while buf.len() >= 5 {
        if let Ok((header, msg)) = decode_frame(buf) {
            frames.push(msg.to_vec());
            let consumed = 5 + header.message_len as usize;
            if consumed > buf.len() {
                break;
            }
            buf = &buf[consumed..];
        } else {
            break;
        }
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_frame_length() {
        /* encoded frame is header (5) + data */
        let mut buf = vec![];
        encode_frame(b"hello", false, &mut buf);
        assert_eq!(buf.len(), 10);
    }

    #[test]
    fn test_decode_header() {
        /* header decoded correctly */
        let mut buf = vec![];
        encode_frame(b"test", false, &mut buf);
        let h = decode_frame_header(&buf).unwrap();
        assert!(!h.compressed);
        assert_eq!(h.message_len, 4);
    }

    #[test]
    fn test_decode_complete_frame() {
        /* full frame decodes message bytes */
        let mut buf = vec![];
        encode_frame(b"abc", false, &mut buf);
        let (_, msg) = decode_frame(&buf).unwrap();
        assert_eq!(msg, b"abc");
    }

    #[test]
    fn test_insufficient_data() {
        /* short buffer returns error */
        assert!(decode_frame_header(&[0, 0, 0]).is_err());
    }

    #[test]
    fn test_is_complete_frame_true() {
        /* complete frame detected */
        let mut buf = vec![];
        encode_frame(&[1, 2], false, &mut buf);
        assert!(is_complete_frame(&buf));
    }

    #[test]
    fn test_is_complete_frame_false() {
        /* truncated frame not complete */
        assert!(!is_complete_frame(&[0, 0, 0, 0, 5]));
    }

    #[test]
    fn test_framed_length() {
        /* framed_length adds 5 bytes */
        assert_eq!(framed_length(10), 15);
    }

    #[test]
    fn test_split_frames() {
        /* two back-to-back frames split correctly */
        let mut buf = vec![];
        encode_frame(b"A", false, &mut buf);
        encode_frame(b"BB", false, &mut buf);
        let frames = split_frames(&buf);
        assert_eq!(frames.len(), 2);
    }

    #[test]
    fn test_compressed_flag() {
        /* compressed flag set correctly */
        let mut buf = vec![];
        encode_frame(b"x", true, &mut buf);
        let h = decode_frame_header(&buf).unwrap();
        assert!(h.compressed);
    }

    #[test]
    fn test_invalid_compression_flag() {
        /* flag > 1 is invalid */
        let buf = [2u8, 0, 0, 0, 0];
        assert!(decode_frame_header(&buf).is_err());
    }
}
