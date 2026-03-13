// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Network I/O stub for future streaming and collaboration features.
//!
//! Provides a simulated network layer with configurable latency, packet loss,
//! send/receive buffers, and connection-state management. No actual I/O is
//! performed; all communication is simulated in-process.

#![allow(dead_code)]

use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Current state of the simulated network connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Configuration for the network stub.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Simulated round-trip latency in milliseconds.
    pub latency_ms: u32,
    /// Packet loss probability [0.0 = no loss, 1.0 = always lost].
    pub packet_loss_prob: f32,
    /// Maximum receive buffer size (packets).
    pub recv_buffer_size: usize,
    /// Endpoint identifier (e.g. IP:port string).
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
    /// Simulated timestamp (milliseconds).
    pub timestamp_ms: u64,
}

/// Simulated network stub.
pub struct NetworkStub {
    config: NetworkConfig,
    state: ConnectionState,
    send_count: u64,
    recv_count: u64,
    next_id: u64,
    recv_buffer: VecDeque<NetworkPacket>,
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

/// Return a sensible default `NetworkConfig`.
pub fn default_network_config() -> NetworkConfig {
    NetworkConfig {
        latency_ms: 20,
        packet_loss_prob: 0.0,
        recv_buffer_size: 256,
        endpoint: "127.0.0.1:7878".to_string(),
    }
}

/// Create a new `NetworkStub` using the given config.
pub fn new_network_stub(config: NetworkConfig) -> NetworkStub {
    NetworkStub {
        config,
        state: ConnectionState::Disconnected,
        send_count: 0,
        recv_count: 0,
        next_id: 1,
        recv_buffer: VecDeque::new(),
        lcg_state: 0xDEAD_BEEF_CAFE_1234,
    }
}

// ---------------------------------------------------------------------------
// Connection control
// ---------------------------------------------------------------------------

/// Simulate connecting to the configured endpoint.
///
/// Transitions: `Disconnected → Connecting → Connected`.
/// Returns `true` if the connection succeeds.
pub fn connect_stub(stub: &mut NetworkStub) -> bool {
    if stub.state == ConnectionState::Connected {
        return true;
    }
    stub.state = ConnectionState::Connecting;
    // In a stub we immediately succeed.
    stub.state = ConnectionState::Connected;
    true
}

/// Disconnect the stub.
pub fn disconnect_stub(stub: &mut NetworkStub) {
    stub.state = ConnectionState::Disconnected;
    stub.recv_buffer.clear();
}

// ---------------------------------------------------------------------------
// Send / receive
// ---------------------------------------------------------------------------

/// Send a packet. Returns `Some(packet_id)` if accepted, `None` if not connected
/// or if the packet was simulated-lost.
pub fn send_packet(stub: &mut NetworkStub, _channel: &str, _payload: Vec<u8>) -> Option<u64> {
    if stub.state != ConnectionState::Connected {
        return None;
    }
    // Simulate packet loss
    if simulate_packet_loss(stub) {
        return None;
    }
    let id = stub.next_id;
    stub.next_id += 1;
    stub.send_count += 1;
    Some(id)
}

/// Receive the next packet from the receive buffer.
///
/// Returns `None` if the buffer is empty or the stub is not connected.
pub fn receive_packet(stub: &mut NetworkStub) -> Option<NetworkPacket> {
    if stub.state != ConnectionState::Connected {
        return None;
    }
    let pkt = stub.recv_buffer.pop_front()?;
    stub.recv_count += 1;
    Some(pkt)
}

/// Push a packet directly into the receive buffer (used for testing / simulation).
///
/// Returns `false` if the buffer is full.
pub fn push_to_recv_buffer(stub: &mut NetworkStub, channel: &str, payload: Vec<u8>) -> bool {
    if stub.recv_buffer.len() >= stub.config.recv_buffer_size {
        return false;
    }
    let id = stub.next_id;
    stub.next_id += 1;
    stub.recv_buffer.push_back(NetworkPacket {
        id,
        payload,
        channel: channel.to_string(),
        timestamp_ms: stub.send_count * stub.config.latency_ms as u64,
    });
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

/// Total packets successfully sent (not simulated-lost).
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

    fn connected_stub() -> NetworkStub {
        let cfg = default_network_config();
        let mut s = new_network_stub(cfg);
        connect_stub(&mut s);
        s
    }

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

    // 3. connect_stub transitions to Connected
    #[test]
    fn connect_transitions_to_connected() {
        let mut s = new_network_stub(default_network_config());
        assert!(connect_stub(&mut s));
        assert_eq!(connection_state(&s), ConnectionState::Connected);
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

    // 7. receive_packet returns None when buffer empty
    #[test]
    fn recv_empty_buffer_returns_none() {
        let mut s = connected_stub();
        assert!(receive_packet(&mut s).is_none());
    }

    // 8. push then receive round-trip
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
        connect_stub(&mut s);
        disconnect_stub(&mut s);
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
        let mut cfg = default_network_config();
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
        let mut s = connected_stub();
        push_to_recv_buffer(&mut s, "z", vec![0xFF]);
        disconnect_stub(&mut s);
        // Reconnect and check buffer is gone
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
}
