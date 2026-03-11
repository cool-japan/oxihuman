// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! WebSocket frame encode/decode stub (RFC 6455).

/// WebSocket opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WsOpcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl WsOpcode {
    /// Parse a raw opcode byte.
    pub fn from_u8(b: u8) -> Option<Self> {
        match b & 0x0F {
            0x0 => Some(Self::Continuation),
            0x1 => Some(Self::Text),
            0x2 => Some(Self::Binary),
            0x8 => Some(Self::Close),
            0x9 => Some(Self::Ping),
            0xA => Some(Self::Pong),
            _ => None,
        }
    }
}

/// A decoded WebSocket frame.
#[derive(Debug, Clone, PartialEq)]
pub struct WsFrame {
    pub fin: bool,
    pub opcode: WsOpcode,
    pub masked: bool,
    pub masking_key: Option<[u8; 4]>,
    pub payload: Vec<u8>,
}

/// WebSocket frame error.
#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    InsufficientData,
    UnknownOpcode(u8),
    Rsv1Set,
    Rsv2Set,
    Rsv3Set,
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientData => write!(f, "insufficient data for WebSocket frame"),
            Self::UnknownOpcode(b) => write!(f, "unknown WebSocket opcode: 0x{b:x}"),
            Self::Rsv1Set => write!(f, "RSV1 bit set without negotiated extension"),
            Self::Rsv2Set => write!(f, "RSV2 bit set without negotiated extension"),
            Self::Rsv3Set => write!(f, "RSV3 bit set without negotiated extension"),
        }
    }
}

/// Encode a WebSocket frame (server-side: unmasked).
pub fn encode_frame(frame: &WsFrame, buf: &mut Vec<u8>) {
    let b0 = (if frame.fin { 0x80 } else { 0 }) | (frame.opcode as u8);
    buf.push(b0);
    let payload_len = frame.payload.len();
    if payload_len <= 125 {
        buf.push(payload_len as u8);
    } else if payload_len <= 65535 {
        buf.push(126);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
    } else {
        buf.push(127);
        buf.extend_from_slice(&(payload_len as u64).to_be_bytes());
    }
    buf.extend_from_slice(&frame.payload);
}

/// Decode a WebSocket frame from a byte slice (unmasked only).
pub fn decode_frame(buf: &[u8]) -> Result<WsFrame, WsError> {
    if buf.len() < 2 {
        return Err(WsError::InsufficientData);
    }
    let b0 = buf[0];
    let b1 = buf[1];
    let fin = (b0 & 0x80) != 0;
    let opcode_raw = b0 & 0x0F;
    let opcode = WsOpcode::from_u8(opcode_raw).ok_or(WsError::UnknownOpcode(opcode_raw))?;
    let masked = (b1 & 0x80) != 0;
    let len_field = (b1 & 0x7F) as usize;
    let payload_start = 2;
    let payload_len = if len_field <= 125 {
        len_field
    } else if len_field == 126 {
        if buf.len() < 4 {
            return Err(WsError::InsufficientData);
        }
        u16::from_be_bytes([buf[2], buf[3]]) as usize
    } else {
        if buf.len() < 10 {
            return Err(WsError::InsufficientData);
        }
        u64::from_be_bytes(buf[2..10].try_into().unwrap_or_default()) as usize
    };
    let offset = if len_field <= 125 {
        payload_start
    } else if len_field == 126 {
        4
    } else {
        10
    };
    if buf.len() < offset + payload_len {
        return Err(WsError::InsufficientData);
    }
    let payload = buf[offset..offset + payload_len].to_vec();
    Ok(WsFrame {
        fin,
        opcode,
        masked,
        masking_key: None,
        payload,
    })
}

/// Apply (or remove) a WebSocket masking key to a payload in place.
pub fn apply_mask(payload: &mut [u8], key: [u8; 4]) {
    for (i, byte) in payload.iter_mut().enumerate() {
        *byte ^= key[i % 4];
    }
}

/// Return `true` if the frame is a control frame.
pub fn is_control_frame(frame: &WsFrame) -> bool {
    matches!(
        frame.opcode,
        WsOpcode::Close | WsOpcode::Ping | WsOpcode::Pong
    )
}

/// Build a simple text frame.
pub fn text_frame(payload: &str) -> WsFrame {
    WsFrame {
        fin: true,
        opcode: WsOpcode::Text,
        masked: false,
        masking_key: None,
        payload: payload.as_bytes().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_small_frame() {
        /* small payload encodes to 2 + payload bytes */
        let f = text_frame("hi");
        let mut buf = vec![];
        encode_frame(&f, &mut buf);
        assert_eq!(buf.len(), 4); /* 2 header + 2 payload */
    }

    #[test]
    fn test_decode_text_frame() {
        /* round-trip encode/decode */
        let f = text_frame("hello");
        let mut buf = vec![];
        encode_frame(&f, &mut buf);
        let decoded = decode_frame(&buf).unwrap();
        assert_eq!(decoded.opcode, WsOpcode::Text);
        assert_eq!(decoded.payload, b"hello");
    }

    #[test]
    fn test_is_control_ping() {
        /* Ping is a control frame */
        let f = WsFrame {
            fin: true,
            opcode: WsOpcode::Ping,
            masked: false,
            masking_key: None,
            payload: vec![],
        };
        assert!(is_control_frame(&f));
    }

    #[test]
    fn test_is_control_text_false() {
        /* Text is not a control frame */
        let f = text_frame("x");
        assert!(!is_control_frame(&f));
    }

    #[test]
    fn test_apply_mask_roundtrip() {
        /* applying mask twice restores original */
        let key = [0xAB, 0xCD, 0xEF, 0x12];
        let original = vec![1u8, 2, 3, 4];
        let mut data = original.clone();
        apply_mask(&mut data, key);
        apply_mask(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_opcode_from_u8_text() {
        /* opcode 1 is Text */
        assert_eq!(WsOpcode::from_u8(1), Some(WsOpcode::Text));
    }

    #[test]
    fn test_opcode_unknown() {
        /* opcode 3 is unknown */
        assert!(WsOpcode::from_u8(3).is_none());
    }

    #[test]
    fn test_insufficient_data() {
        /* single byte returns error */
        assert!(decode_frame(&[0x81]).is_err());
    }

    #[test]
    fn test_fin_bit() {
        /* FIN bit preserved */
        let f = text_frame("x");
        let mut buf = vec![];
        encode_frame(&f, &mut buf);
        let d = decode_frame(&buf).unwrap();
        assert!(d.fin);
    }

    #[test]
    fn test_binary_opcode() {
        /* binary opcode round-trip */
        let f = WsFrame {
            fin: true,
            opcode: WsOpcode::Binary,
            masked: false,
            masking_key: None,
            payload: vec![0xFF],
        };
        let mut buf = vec![];
        encode_frame(&f, &mut buf);
        let d = decode_frame(&buf).unwrap();
        assert_eq!(d.opcode, WsOpcode::Binary);
    }
}
