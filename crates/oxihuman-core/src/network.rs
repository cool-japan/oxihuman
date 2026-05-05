// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Real TCP networking layer for streaming and collaboration features.
//!
//! Provides a TCP-backed network connection with configurable latency simulation,
//! packet loss, send/receive buffers, and connection-state management. Actual I/O
//! is performed via Tokio; the public API remains synchronous for callers that do
//! not need async.
//!
//! # Packet framing
//!
//! Each frame on the wire is:
//! ```text
//! [4 bytes LE: total_len][1 byte: channel_len][channel bytes][payload bytes]
//! ```
//! where `total_len = 1 + channel_len + payload_len`.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::io::{self};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can arise from network operations.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("not connected")]
    NotConnected,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("connection refused: {0}")]
    ConnectionRefused(String),
    #[error("frame too large: channel name length {0} exceeds 255 bytes")]
    ChannelTooLong(usize),
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Current state of the network connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Configuration for the network connection.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Hostname or IP address to connect to.
    pub host: String,
    /// TCP port to connect to.
    pub port: u16,
    /// Simulated round-trip latency in milliseconds (0 = no simulation).
    pub latency_ms: u32,
    /// Packet loss probability [0.0 = no loss, 1.0 = always lost].
    pub packet_loss_prob: f32,
    /// Maximum receive buffer size (packets).
    pub recv_buffer_size: usize,
    /// Human-readable endpoint string (derived from host:port by default).
    pub endpoint: String,
}

/// A single network packet with a typed payload.
#[derive(Debug, Clone)]
pub struct NetworkPacket {
    /// Monotonically increasing packet identifier.
    pub id: u64,
    /// Raw payload bytes.
    pub payload: Vec<u8>,
    /// Packet channel / topic tag.
    pub channel: String,
    /// Approximate timestamp in milliseconds (wall-clock at receipt).
    pub timestamp_ms: u64,
}

/// TCP-backed network stub with a synchronous public API.
pub struct NetworkStub {
    /// Public configuration (host, port, latency, loss, …).
    pub config: NetworkConfig,
    runtime: tokio::runtime::Runtime,
    stream: Option<TcpStream>,
    recv_buffer: VecDeque<NetworkPacket>,
    /// Partial-frame accumulator for the TCP read path.
    partial_buf: Vec<u8>,
    state: ConnectionState,
    send_count: u64,
    recv_count: u64,
    next_id: u64,
    /// LCG state for deterministic packet-loss simulation.
    lcg_state: u64,
}

// ---------------------------------------------------------------------------
// LCG helper
// ---------------------------------------------------------------------------

fn lcg_next(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*state >> 33) as f32) / (u32::MAX as f32 + 1.0)
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Return a sensible default `NetworkConfig` (connects to 127.0.0.1:7878).
pub fn default_network_config() -> NetworkConfig {
    NetworkConfig {
        host: "127.0.0.1".to_string(),
        port: 7878,
        latency_ms: 20,
        packet_loss_prob: 0.0,
        recv_buffer_size: 256,
        endpoint: "127.0.0.1:7878".to_string(),
    }
}

/// Create a new `NetworkStub` using the given config.
///
/// Spawns a single-threaded Tokio runtime for synchronous blocking I/O.
pub fn new_network_stub(config: NetworkConfig) -> NetworkStub {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio current-thread runtime is infallible");

    NetworkStub {
        config,
        runtime,
        stream: None,
        recv_buffer: VecDeque::new(),
        partial_buf: Vec::new(),
        state: ConnectionState::Disconnected,
        send_count: 0,
        recv_count: 0,
        next_id: 1,
        lcg_state: 0xDEAD_BEEF_CAFE_1234,
    }
}

// ---------------------------------------------------------------------------
// Internal framing helpers
// ---------------------------------------------------------------------------

/// Encode a packet into a length-prefixed frame.
///
/// Frame layout:
/// ```text
/// [4 bytes LE: total_len][1 byte: channel_len][channel bytes][payload bytes]
/// ```
fn encode_frame(channel: &str, payload: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let ch_bytes = channel.as_bytes();
    let ch_len = ch_bytes.len();
    if ch_len > 255 {
        return Err(NetworkError::ChannelTooLong(ch_len));
    }
    let total_len: u32 = (1 + ch_len + payload.len()) as u32;
    let mut frame = Vec::with_capacity(4 + 1 + ch_len + payload.len());
    frame.extend_from_slice(&total_len.to_le_bytes());
    frame.push(ch_len as u8);
    frame.extend_from_slice(ch_bytes);
    frame.extend_from_slice(payload);
    Ok(frame)
}

/// Try to parse one complete packet out of `buf`, advancing `pos`.
///
/// Returns `None` if the buffer contains fewer bytes than the next frame
/// requires (partial frame — caller should wait for more data).
fn try_parse_frame(buf: &[u8]) -> Option<(NetworkPacket, usize)> {
    if buf.len() < 4 {
        return None;
    }
    let total_len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let frame_end = 4 + total_len;
    if buf.len() < frame_end {
        return None;
    }
    if total_len < 1 {
        // Malformed: no channel-length byte.
        return None;
    }
    let ch_len = buf[4] as usize;
    if total_len < 1 + ch_len {
        return None;
    }
    let ch_start = 5;
    let ch_end = ch_start + ch_len;
    let payload_start = ch_end;
    let payload_end = frame_end;

    let channel = String::from_utf8_lossy(&buf[ch_start..ch_end]).into_owned();
    let payload = buf[payload_start..payload_end].to_vec();
    let pkt = NetworkPacket {
        id: 0, // filled in by caller
        payload,
        channel,
        timestamp_ms: current_millis(),
    };
    Some((pkt, frame_end))
}

fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Connection control
// ---------------------------------------------------------------------------

/// Connect to the configured endpoint.
///
/// Transitions: `Disconnected → Connecting → Connected` (or `Error` on failure).
/// Returns `true` if the connection succeeds.
pub fn connect_stub(stub: &mut NetworkStub) -> bool {
    if stub.state == ConnectionState::Connected {
        return true;
    }
    stub.state = ConnectionState::Connecting;

    let addr = format!("{}:{}", stub.config.host, stub.config.port);
    let latency_ms = stub.config.latency_ms;

    let result = stub.runtime.block_on(async move {
        if latency_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(latency_ms as u64)).await;
        }
        TcpStream::connect(&addr).await
    });

    match result {
        Ok(stream) => {
            stub.stream = Some(stream);
            stub.state = ConnectionState::Connected;
            true
        }
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            stub.state = ConnectionState::Error;
            false
        }
        Err(_) => {
            stub.state = ConnectionState::Error;
            false
        }
    }
}

/// Disconnect from the endpoint and clear the receive buffer.
pub fn disconnect_stub(stub: &mut NetworkStub) {
    // Drop the stream inside the runtime so Tokio's async drop logic runs.
    if let Some(stream) = stub.stream.take() {
        stub.runtime.block_on(async move {
            drop(stream);
        });
    }
    stub.state = ConnectionState::Disconnected;
    stub.recv_buffer.clear();
    stub.partial_buf.clear();
}

// ---------------------------------------------------------------------------
// Send / receive
// ---------------------------------------------------------------------------

/// Send a packet over the TCP connection.
///
/// Returns `Some(send_count)` if sent, `None` if not connected, if
/// `simulate_packet_loss` decides the packet is lost, or on I/O error.
pub fn send_packet(stub: &mut NetworkStub, channel: &str, payload: Vec<u8>) -> Option<u64> {
    if stub.state != ConnectionState::Connected {
        return None;
    }
    if simulate_packet_loss(stub) {
        return None;
    }

    let frame = encode_frame(channel, &payload).ok()?;

    // Move stream out, write, put back.
    let stream = stub.stream.take()?;
    let write_result = stub.runtime.block_on(async move {
        let mut s = stream;
        match s.write_all(&frame).await {
            Ok(()) => Ok(s),
            Err(e) => Err((e, s)),
        }
    });

    match write_result {
        Ok(s) => {
            stub.stream = Some(s);
            stub.send_count += 1;
            Some(stub.send_count)
        }
        Err((_, s)) => {
            stub.stream = Some(s);
            stub.state = ConnectionState::Error;
            None
        }
    }
}

/// Receive the next packet from the receive buffer or from the TCP stream.
///
/// First drains any already-buffered packets. If none are available and the
/// stub is connected, attempts a non-blocking 10 ms read from the stream to
/// fill the buffer. Returns `None` on timeout, empty stream, or error.
pub fn receive_packet(stub: &mut NetworkStub) -> Option<NetworkPacket> {
    if stub.state != ConnectionState::Connected {
        return None;
    }

    // 1. Serve from in-memory push buffer first.
    if let Some(pkt) = stub.recv_buffer.pop_front() {
        // Packets from push_to_recv_buffer already have an id assigned.
        // Only stream-decoded packets arrive with id == 0 (placeholder).
        let pkt = if pkt.id == 0 {
            let mut p = pkt;
            p.id = stub.next_id;
            stub.next_id += 1;
            p
        } else {
            pkt
        };
        stub.recv_count += 1;
        return Some(pkt);
    }

    // 2. No buffered packet — try to read from the TCP stream (10 ms window).
    let stream = stub.stream.take()?;
    let partial = std::mem::take(&mut stub.partial_buf);

    let (returned_stream, returned_partial, new_packets) = stub
        .runtime
        .block_on(async move { try_read_packets(stream, partial).await });

    stub.stream = Some(returned_stream);
    stub.partial_buf = returned_partial;

    // Push newly decoded packets into the recv_buffer, respecting the cap.
    for pkt in new_packets {
        if stub.recv_buffer.len() < stub.config.recv_buffer_size {
            stub.recv_buffer.push_back(pkt);
        }
    }

    // Serve one (stream-decoded packets have id == 0 — assign a real id).
    if let Some(pkt) = stub.recv_buffer.pop_front() {
        let pkt = if pkt.id == 0 {
            let mut p = pkt;
            p.id = stub.next_id;
            stub.next_id += 1;
            p
        } else {
            pkt
        };
        stub.recv_count += 1;
        return Some(pkt);
    }

    None
}

/// Internal async helper: try to read available bytes from the stream within
/// a short timeout, then parse as many complete frames as possible.
///
/// Returns `(stream, remaining_partial_buf, decoded_packets)`.
async fn try_read_packets(
    mut stream: TcpStream,
    mut partial: Vec<u8>,
) -> (TcpStream, Vec<u8>, Vec<NetworkPacket>) {
    let mut tmp = [0u8; 4096];

    // Non-blocking peek: try with a 10 ms timeout.
    let read_result =
        tokio::time::timeout(std::time::Duration::from_millis(10), stream.read(&mut tmp)).await;

    match read_result {
        Ok(Ok(0)) => {
            // Connection closed by peer — return what we have.
            (stream, partial, vec![])
        }
        Ok(Ok(n)) => {
            partial.extend_from_slice(&tmp[..n]);
            let packets = parse_all_frames(&mut partial);
            (stream, partial, packets)
        }
        // Timeout or I/O error — return stream unchanged.
        Ok(Err(_)) | Err(_) => (stream, partial, vec![]),
    }
}

/// Parse and remove all complete frames from `buf`, returning decoded packets.
/// Any trailing incomplete frame bytes remain in `buf`.
fn parse_all_frames(buf: &mut Vec<u8>) -> Vec<NetworkPacket> {
    let mut packets = Vec::new();
    let mut cursor = 0usize;

    while let Some((pkt, consumed)) = try_parse_frame(&buf[cursor..]) {
        packets.push(pkt);
        cursor += consumed;
    }

    if cursor > 0 {
        buf.drain(..cursor);
    }

    packets
}

/// Push a packet directly into the receive buffer (used for testing / loopback).
///
/// Returns `false` if the buffer is full.
pub fn push_to_recv_buffer(stub: &mut NetworkStub, channel: &str, payload: Vec<u8>) -> bool {
    if stub.recv_buffer.len() >= stub.config.recv_buffer_size {
        return false;
    }
    stub.recv_buffer.push_back(NetworkPacket {
        id: stub.next_id,
        payload,
        channel: channel.to_string(),
        timestamp_ms: stub.send_count * stub.config.latency_ms as u64,
    });
    stub.next_id += 1;
    true
}

/// Flush (discard) all packets in the receive buffer.
pub fn flush_receive_buffer(stub: &mut NetworkStub) {
    stub.recv_buffer.clear();
}

// ---------------------------------------------------------------------------
// Configuration / metrics
// ---------------------------------------------------------------------------

/// Current connection state.
pub fn connection_state(stub: &NetworkStub) -> ConnectionState {
    stub.state
}

/// Total packets successfully sent.
pub fn packet_count_sent(stub: &NetworkStub) -> u64 {
    stub.send_count
}

/// Total packets received (popped from the receive buffer).
pub fn packet_count_received(stub: &NetworkStub) -> u64 {
    stub.recv_count
}

/// Update the simulated latency.
pub fn set_latency_ms(stub: &mut NetworkStub, latency_ms: u32) {
    stub.config.latency_ms = latency_ms;
}

/// Check whether the current packet should be simulated-lost.
///
/// Returns `true` if the packet is lost. Advances the internal LCG state.
pub fn simulate_packet_loss(stub: &mut NetworkStub) -> bool {
    if stub.config.packet_loss_prob <= 0.0 {
        return false;
    }
    lcg_next(&mut stub.lcg_state) < stub.config.packet_loss_prob
}

/// Serialize stub metadata to a simple JSON-like string.
pub fn network_stub_to_json(stub: &NetworkStub) -> String {
    let state_str = match stub.state {
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Error => "error",
    };
    format!(
        "{{\"state\":\"{}\",\"endpoint\":\"{}\",\"latency_ms\":{},\
         \"packet_loss_prob\":{:.4},\"sent\":{},\"received\":{},\"recv_buffer\":{}}}",
        state_str,
        stub.config.endpoint,
        stub.config.latency_ms,
        stub.config.packet_loss_prob,
        stub.send_count,
        stub.recv_count,
        stub.recv_buffer.len(),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener as StdTcpListener;

    // ---------------------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------------------

    /// Bind a TCP listener on an OS-assigned port and return the port.
    ///
    /// The returned `StdTcpListener` must be kept alive for the duration of
    /// the test; drop it when the test is done.
    fn bind_test_listener() -> (StdTcpListener, u16) {
        let listener = StdTcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let port = listener.local_addr().expect("local_addr").port();
        (listener, port)
    }

    /// Spawn a background thread that accepts one connection and holds it open
    /// until `listener` is dropped.  Returns the assigned port.
    ///
    /// The `StdTcpListener` should be held in the caller for its lifetime.
    fn accept_one_in_background(listener: StdTcpListener) -> u16 {
        let port = listener.local_addr().expect("local_addr").port();
        std::thread::spawn(move || {
            // accept() blocks until a client connects; keep the conn alive
            let _conn = listener.accept();
            // _conn is held until the thread exits (when _conn is dropped at
            // the end of the closure).  The thread will linger until the OS
            // reclaims it, which is fine for test teardown.
        });
        port
    }

    /// Build a `NetworkConfig` pointing at a local test listener.
    fn test_config(port: u16) -> NetworkConfig {
        NetworkConfig {
            host: "127.0.0.1".to_string(),
            port,
            latency_ms: 0, // no simulated delay in unit tests
            packet_loss_prob: 0.0,
            recv_buffer_size: 256,
            endpoint: format!("127.0.0.1:{}", port),
        }
    }

    /// Create a connected stub against a fresh ephemeral listener.
    ///
    /// The listener is started in a background thread (accept loop) so `connect`
    /// succeeds immediately.  The returned stub is already in `Connected` state.
    fn connected_stub() -> NetworkStub {
        let (listener, port) = bind_test_listener();
        accept_one_in_background(listener);
        let cfg = test_config(port);
        let mut s = new_network_stub(cfg);
        assert!(connect_stub(&mut s), "connected_stub: connect failed");
        s
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    // 1. default config has sane values
    #[test]
    fn default_config_sane() {
        let cfg = default_network_config();
        assert_eq!(cfg.latency_ms, 20);
        assert_eq!(cfg.packet_loss_prob, 0.0);
        assert!(cfg.recv_buffer_size > 0);
        assert!(!cfg.endpoint.is_empty());
    }

    // 2. new stub starts disconnected
    #[test]
    fn new_stub_disconnected() {
        let s = new_network_stub(default_network_config());
        assert_eq!(connection_state(&s), ConnectionState::Disconnected);
    }

    // 3. connect_stub transitions to Connected (requires listener)
    #[test]
    fn connect_transitions_to_connected() {
        let s = connected_stub();
        assert_eq!(connection_state(&s), ConnectionState::Connected);
    }

    // 3b. connect to a port with nothing listening → Error state
    #[test]
    fn connect_refused_sets_error() {
        // Bind and immediately drop to free the port.
        let (listener, port) = bind_test_listener();
        drop(listener);
        let cfg = test_config(port);
        let mut s = new_network_stub(cfg);
        let ok = connect_stub(&mut s);
        assert!(!ok);
        assert_eq!(connection_state(&s), ConnectionState::Error);
    }

    // 4. disconnect sets Disconnected
    #[test]
    fn disconnect_sets_disconnected() {
        let mut s = connected_stub();
        disconnect_stub(&mut s);
        assert_eq!(connection_state(&s), ConnectionState::Disconnected);
    }

    // 5. send_packet when not connected returns None
    #[test]
    fn send_when_disconnected_returns_none() {
        let mut s = new_network_stub(default_network_config());
        assert!(send_packet(&mut s, "ch", vec![1, 2, 3]).is_none());
    }

    // 6. send_packet when connected increments counter
    #[test]
    fn send_increments_count() {
        let mut s = connected_stub();
        send_packet(&mut s, "data", vec![42]).expect("should succeed");
        send_packet(&mut s, "data", vec![43]).expect("should succeed");
        assert_eq!(packet_count_sent(&s), 2);
    }

    // 7. receive_packet returns None when buffer empty (no data from peer)
    #[test]
    fn recv_empty_buffer_returns_none() {
        let mut s = connected_stub();
        // Nothing was pushed to recv_buffer and peer sends nothing → None
        assert!(receive_packet(&mut s).is_none());
    }

    // 8. push then receive round-trip (in-memory buffer path)
    #[test]
    fn push_recv_round_trip() {
        let mut s = connected_stub();
        assert!(push_to_recv_buffer(&mut s, "ch", vec![9, 8, 7]));
        let pkt = receive_packet(&mut s).expect("must have a packet");
        assert_eq!(pkt.payload, vec![9, 8, 7]);
        assert_eq!(pkt.channel, "ch");
    }

    // 9. receive_packet when not connected returns None
    #[test]
    fn recv_when_disconnected_returns_none() {
        let mut s = new_network_stub(default_network_config());
        push_to_recv_buffer(&mut s, "ch", vec![1]);
        // No real connection needed for this test — just verify disconnected path.
        // State is Disconnected, so receive_packet returns None.
        assert!(receive_packet(&mut s).is_none());
    }

    // 10. flush_receive_buffer empties buffer
    #[test]
    fn flush_empties_buffer() {
        let mut s = connected_stub();
        push_to_recv_buffer(&mut s, "x", vec![1]);
        push_to_recv_buffer(&mut s, "x", vec![2]);
        flush_receive_buffer(&mut s);
        assert!(receive_packet(&mut s).is_none());
    }

    // 11. set_latency_ms updates config
    #[test]
    fn set_latency_updates() {
        let mut s = new_network_stub(default_network_config());
        set_latency_ms(&mut s, 100);
        assert_eq!(s.config.latency_ms, 100);
    }

    // 12. zero packet loss never loses packets
    #[test]
    fn zero_loss_never_loses() {
        let mut cfg = default_network_config();
        cfg.packet_loss_prob = 0.0;
        let mut s = new_network_stub(cfg);
        for _ in 0..50 {
            assert!(!simulate_packet_loss(&mut s));
        }
    }

    // 13. full packet loss always loses
    #[test]
    fn full_loss_always_loses() {
        let mut cfg = default_network_config();
        cfg.packet_loss_prob = 1.0;
        let mut s = new_network_stub(cfg);
        for _ in 0..20 {
            assert!(simulate_packet_loss(&mut s));
        }
    }

    // 14. recv_buffer_size enforced on push
    #[test]
    fn recv_buffer_size_enforced() {
        let (listener, port) = bind_test_listener();
        accept_one_in_background(listener);
        let mut cfg = test_config(port);
        cfg.recv_buffer_size = 2;
        let mut s = new_network_stub(cfg);
        connect_stub(&mut s);
        assert!(push_to_recv_buffer(&mut s, "a", vec![1]));
        assert!(push_to_recv_buffer(&mut s, "b", vec![2]));
        assert!(!push_to_recv_buffer(&mut s, "c", vec![3]));
    }

    // 15. network_stub_to_json contains endpoint
    #[test]
    fn to_json_contains_endpoint() {
        let s = new_network_stub(default_network_config());
        let json = network_stub_to_json(&s);
        assert!(json.contains("127.0.0.1:7878"));
    }

    // 16. packet_count_received increments on receive
    #[test]
    fn recv_count_increments() {
        let mut s = connected_stub();
        push_to_recv_buffer(&mut s, "x", vec![5]);
        push_to_recv_buffer(&mut s, "x", vec![6]);
        receive_packet(&mut s);
        assert_eq!(packet_count_received(&s), 1);
        receive_packet(&mut s);
        assert_eq!(packet_count_received(&s), 2);
    }

    // 17. ConnectionState::Error is distinct
    #[test]
    fn connection_state_error_distinct() {
        assert_ne!(ConnectionState::Error, ConnectionState::Connected);
        assert_ne!(ConnectionState::Error, ConnectionState::Disconnected);
    }

    // 18. connect when already connected returns true
    #[test]
    fn connect_when_already_connected() {
        let mut s = connected_stub();
        assert!(connect_stub(&mut s));
        assert_eq!(connection_state(&s), ConnectionState::Connected);
    }

    // 19. disconnect clears recv buffer
    #[test]
    fn disconnect_clears_recv() {
        let (listener, port) = bind_test_listener();
        accept_one_in_background(listener);
        let cfg = test_config(port);
        let mut s = new_network_stub(cfg);
        connect_stub(&mut s);
        push_to_recv_buffer(&mut s, "z", vec![0xFF]);
        disconnect_stub(&mut s);

        // Re-connect to a fresh listener to re-enter Connected state.
        let (listener2, port2) = bind_test_listener();
        accept_one_in_background(listener2);
        s.config.host = "127.0.0.1".to_string();
        s.config.port = port2;
        s.config.endpoint = format!("127.0.0.1:{}", port2);
        connect_stub(&mut s);
        assert!(receive_packet(&mut s).is_none());
    }

    // 20. network_stub_to_json contains sent/received counts
    #[test]
    fn to_json_contains_counts() {
        let mut s = connected_stub();
        send_packet(&mut s, "ch", vec![1]);
        let json = network_stub_to_json(&s);
        assert!(json.contains("\"sent\":1"));
        assert!(json.contains("\"received\":0"));
    }

    // 21. encode_frame / parse round-trip
    #[test]
    fn frame_encode_decode_round_trip() {
        let frame = encode_frame("test-channel", b"hello world").expect("encode");
        let (pkt, consumed) = try_parse_frame(&frame).expect("parse");
        assert_eq!(consumed, frame.len());
        assert_eq!(pkt.channel, "test-channel");
        assert_eq!(pkt.payload, b"hello world");
    }

    // 22. parse_all_frames handles two back-to-back frames
    #[test]
    fn parse_all_frames_two_frames() {
        let mut buf = encode_frame("a", b"foo").expect("enc a");
        buf.extend(encode_frame("b", b"bar").expect("enc b"));
        let pkts = parse_all_frames(&mut buf);
        assert_eq!(pkts.len(), 2);
        assert_eq!(pkts[0].channel, "a");
        assert_eq!(pkts[1].channel, "b");
        assert!(buf.is_empty());
    }

    // 23. parse_all_frames leaves partial frame in buf
    #[test]
    fn parse_all_frames_partial_frame() {
        let mut buf = encode_frame("x", b"data").expect("enc");
        // Truncate by 2 bytes → incomplete frame.
        let full_len = buf.len();
        buf.truncate(full_len - 2);
        let pkts = parse_all_frames(&mut buf);
        assert!(pkts.is_empty());
        assert_eq!(buf.len(), full_len - 2); // partial remains
    }

    // 24. encode_frame rejects channel name > 255 bytes
    #[test]
    fn encode_frame_rejects_long_channel() {
        let long_name: String = "x".repeat(256);
        let result = encode_frame(&long_name, b"payload");
        assert!(result.is_err());
    }
}
