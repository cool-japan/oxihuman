// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Long-poll response export stub — packages mesh/animation updates as long-poll HTTP responses.

/// A single long-poll response stub.
#[derive(Debug, Clone)]
pub struct LongPollResponse {
    pub sequence_id: u64,
    pub body: Vec<u8>,
    pub content_type: String,
    pub timeout_ms: u32,
    pub is_final: bool,
}

/// A long-poll export session.
#[derive(Debug, Default)]
pub struct LongPollExport {
    pub responses: Vec<LongPollResponse>,
    pub poll_endpoint: String,
    pub next_seq: u64,
}

/// Create a new long-poll export session.
pub fn new_long_poll_export(endpoint: &str) -> LongPollExport {
    LongPollExport {
        responses: Vec::new(),
        poll_endpoint: endpoint.to_owned(),
        next_seq: 0,
    }
}

/// Add a JSON response.
pub fn lp_add_json(export: &mut LongPollExport, json: &str, timeout_ms: u32) {
    let seq = export.next_seq;
    export.next_seq += 1;
    export.responses.push(LongPollResponse {
        sequence_id: seq,
        body: json.as_bytes().to_vec(),
        content_type: "application/json".to_owned(),
        timeout_ms,
        is_final: false,
    });
}

/// Add a binary response.
pub fn lp_add_binary(export: &mut LongPollExport, data: Vec<u8>, timeout_ms: u32, is_final: bool) {
    let seq = export.next_seq;
    export.next_seq += 1;
    export.responses.push(LongPollResponse {
        sequence_id: seq,
        body: data,
        content_type: "application/octet-stream".to_owned(),
        timeout_ms,
        is_final,
    });
}

/// Number of responses.
pub fn lp_response_count(export: &LongPollExport) -> usize {
    export.responses.len()
}

/// Total body bytes.
pub fn total_lp_bytes(export: &LongPollExport) -> usize {
    export.responses.iter().map(|r| r.body.len()).sum()
}

/// Find a response by sequence ID.
pub fn find_lp_response(export: &LongPollExport, seq: u64) -> Option<&LongPollResponse> {
    export.responses.iter().find(|r| r.sequence_id == seq)
}

/// Count final (terminal) responses.
pub fn final_response_count(export: &LongPollExport) -> usize {
    export.responses.iter().filter(|r| r.is_final).count()
}

/// Average timeout across all responses.
pub fn average_timeout_ms(export: &LongPollExport) -> f64 {
    if export.responses.is_empty() {
        return 0.0;
    }
    let sum: u64 = export.responses.iter().map(|r| r.timeout_ms as u64).sum();
    sum as f64 / export.responses.len() as f64
}

/// Serialize metadata to JSON-style string.
pub fn lp_export_to_json(export: &LongPollExport) -> String {
    format!(
        r#"{{"endpoint":"{}", "response_count":{}, "total_bytes":{}}}"#,
        export.poll_endpoint,
        lp_response_count(export),
        total_lp_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no responses */
        let e = new_long_poll_export("/poll");
        assert_eq!(lp_response_count(&e), 0);
    }

    #[test]
    fn add_json_increments_count() {
        /* adding a JSON response increments count */
        let mut e = new_long_poll_export("/poll");
        lp_add_json(&mut e, r#"{"v":1}"#, 30000);
        assert_eq!(lp_response_count(&e), 1);
    }

    #[test]
    fn sequence_ids_auto_increment() {
        /* sequence IDs should be 0, 1, 2 */
        let mut e = new_long_poll_export("/poll");
        lp_add_json(&mut e, "{}", 1000);
        lp_add_json(&mut e, "{}", 1000);
        assert_eq!(e.responses[0].sequence_id, 0);
        assert_eq!(e.responses[1].sequence_id, 1);
    }

    #[test]
    fn find_by_sequence() {
        /* find returns correct response */
        let mut e = new_long_poll_export("/poll");
        lp_add_json(&mut e, "{}", 1000);
        assert!(find_lp_response(&e, 0).is_some());
    }

    #[test]
    fn find_missing_seq_none() {
        /* missing sequence returns None */
        let e = new_long_poll_export("/poll");
        assert!(find_lp_response(&e, 99).is_none());
    }

    #[test]
    fn total_bytes_correct() {
        /* 6-char JSON body counted */
        let mut e = new_long_poll_export("/poll");
        lp_add_json(&mut e, r#"{"a":1}"#, 1000);
        assert_eq!(total_lp_bytes(&e), 7);
    }

    #[test]
    fn final_response_counted() {
        /* is_final responses are counted */
        let mut e = new_long_poll_export("/poll");
        lp_add_binary(&mut e, vec![], 1000, true);
        lp_add_binary(&mut e, vec![], 1000, false);
        assert_eq!(final_response_count(&e), 1);
    }

    #[test]
    fn average_timeout_correct() {
        /* average of 1000 and 3000 is 2000 */
        let mut e = new_long_poll_export("/poll");
        lp_add_json(&mut e, "{}", 1000);
        lp_add_json(&mut e, "{}", 3000);
        assert!((average_timeout_ms(&e) - 2000.0).abs() < 1.0);
    }

    #[test]
    fn json_contains_endpoint() {
        /* JSON includes endpoint */
        let e = new_long_poll_export("/api/poll");
        assert!(lp_export_to_json(&e).contains("/api/poll"));
    }
}
