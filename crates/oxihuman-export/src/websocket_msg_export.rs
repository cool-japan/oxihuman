// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! WebSocket message export stub — packages mesh/animation data as WebSocket frames.

/// WebSocket opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsOpcode {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

/// A WebSocket frame stub.
#[derive(Debug, Clone)]
pub struct WsFrame {
    pub opcode: WsOpcode,
    pub payload: Vec<u8>,
    pub is_final: bool,
    pub masked: bool,
}

/// A WebSocket message export session.
#[derive(Debug, Default)]
pub struct WsMessageExport {
    pub frames: Vec<WsFrame>,
    pub url: String,
}

/// Create a new WebSocket message export session.
pub fn new_ws_export(url: &str) -> WsMessageExport {
    WsMessageExport {
        frames: Vec::new(),
        url: url.to_owned(),
    }
}

/// Add a text frame.
pub fn ws_send_text(export: &mut WsMessageExport, text: &str) {
    export.frames.push(WsFrame {
        opcode: WsOpcode::Text,
        payload: text.as_bytes().to_vec(),
        is_final: true,
        masked: true,
    });
}

/// Add a binary frame.
pub fn ws_send_binary(export: &mut WsMessageExport, data: Vec<u8>) {
    export.frames.push(WsFrame {
        opcode: WsOpcode::Binary,
        payload: data,
        is_final: true,
        masked: true,
    });
}

/// Add a ping frame.
pub fn ws_send_ping(export: &mut WsMessageExport, data: Vec<u8>) {
    export.frames.push(WsFrame {
        opcode: WsOpcode::Ping,
        payload: data,
        is_final: true,
        masked: true,
    });
}

/// Number of frames.
pub fn ws_frame_count(export: &WsMessageExport) -> usize {
    export.frames.len()
}

/// Total payload bytes.
pub fn total_ws_bytes(export: &WsMessageExport) -> usize {
    export.frames.iter().map(|f| f.payload.len()).sum()
}

/// Count frames of a given opcode.
pub fn frames_of_opcode(export: &WsMessageExport, opcode: WsOpcode) -> usize {
    export.frames.iter().filter(|f| f.opcode == opcode).count()
}

/// Opcode name string.
pub fn opcode_name(op: WsOpcode) -> &'static str {
    match op {
        WsOpcode::Text => "text",
        WsOpcode::Binary => "binary",
        WsOpcode::Ping => "ping",
        WsOpcode::Pong => "pong",
        WsOpcode::Close => "close",
    }
}

/// Serialize metadata to JSON-style string.
pub fn ws_export_to_json(export: &WsMessageExport) -> String {
    format!(
        r#"{{"url":"{}", "frame_count":{}, "total_bytes":{}}}"#,
        export.url,
        ws_frame_count(export),
        total_ws_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no frames */
        let e = new_ws_export("wss://localhost:8080");
        assert_eq!(ws_frame_count(&e), 0);
    }

    #[test]
    fn send_text_increments_count() {
        /* sending text increments frame count */
        let mut e = new_ws_export("wss://localhost:8080");
        ws_send_text(&mut e, "hello");
        assert_eq!(ws_frame_count(&e), 1);
    }

    #[test]
    fn send_binary_increments_count() {
        /* sending binary increments frame count */
        let mut e = new_ws_export("wss://localhost:8080");
        ws_send_binary(&mut e, vec![1, 2, 3]);
        assert_eq!(ws_frame_count(&e), 1);
    }

    #[test]
    fn total_bytes_counted() {
        /* 5-byte text payload counted */
        let mut e = new_ws_export("wss://localhost:8080");
        ws_send_text(&mut e, "hello");
        assert_eq!(total_ws_bytes(&e), 5);
    }

    #[test]
    fn frames_of_opcode_text() {
        /* count of Text frames is 1 */
        let mut e = new_ws_export("wss://localhost");
        ws_send_text(&mut e, "x");
        ws_send_binary(&mut e, vec![]);
        assert_eq!(frames_of_opcode(&e, WsOpcode::Text), 1);
    }

    #[test]
    fn ping_frame_opcode_correct() {
        /* ping frame has Ping opcode */
        let mut e = new_ws_export("wss://localhost");
        ws_send_ping(&mut e, vec![]);
        assert_eq!(e.frames[0].opcode, WsOpcode::Ping);
    }

    #[test]
    fn opcode_name_text() {
        /* Text opcode name is "text" */
        assert_eq!(opcode_name(WsOpcode::Text), "text");
    }

    #[test]
    fn opcode_name_binary() {
        /* Binary opcode name is "binary" */
        assert_eq!(opcode_name(WsOpcode::Binary), "binary");
    }

    #[test]
    fn json_contains_url() {
        /* JSON includes URL */
        let e = new_ws_export("wss://ws.example.com");
        assert!(ws_export_to_json(&e).contains("ws.example.com"));
    }
}
