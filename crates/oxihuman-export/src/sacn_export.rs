// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! sACN (E1.31) packet builder stub.

pub const SACN_PREAMBLE_SIZE: u16 = 0x0010;
pub const SACN_POSTAMBLE_SIZE: u16 = 0x0000;
pub const SACN_ACN_PID: [u8; 12] = [
    0x41, 0x53, 0x43, 0x2D, 0x45, 0x31, 0x2E, 0x31, 0x37, 0x00, 0x00, 0x00,
];
pub const SACN_VECTOR_ROOT: u32 = 0x00000004;
pub const SACN_VECTOR_FRAME: u32 = 0x00000002;
pub const SACN_VECTOR_DMP: u8 = 0x02;
pub const SACN_DEFAULT_PRIORITY: u8 = 100;
pub const SACN_DMX_START_CODE: u8 = 0x00;

/// sACN packet configuration.
#[allow(dead_code)]
pub struct SacnConfig {
    pub universe: u16,
    pub priority: u8,
    pub source_name: String,
    pub cid: [u8; 16],
}

impl Default for SacnConfig {
    fn default() -> Self {
        Self {
            universe: 1,
            priority: SACN_DEFAULT_PRIORITY,
            source_name: String::from("OxiHuman"),
            cid: [0u8; 16],
        }
    }
}

/// An sACN data packet (stub).
#[allow(dead_code)]
pub struct SacnPacket {
    pub config: SacnConfig,
    pub sequence: u8,
    pub dmx_data: Vec<u8>,
}

impl SacnPacket {
    #[allow(dead_code)]
    pub fn new(config: SacnConfig) -> Self {
        Self {
            config,
            sequence: 0,
            dmx_data: vec![0u8; 512],
        }
    }
}

/// Build minimal sACN E1.31 UDP payload (stub - simplified header).
#[allow(dead_code)]
pub fn build_sacn_packet(pkt: &SacnPacket) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&SACN_PREAMBLE_SIZE.to_be_bytes());
    out.extend_from_slice(&SACN_POSTAMBLE_SIZE.to_be_bytes());
    out.extend_from_slice(&SACN_ACN_PID);
    out.extend_from_slice(&SACN_VECTOR_ROOT.to_be_bytes());
    out.extend_from_slice(&pkt.config.cid);
    out.extend_from_slice(&SACN_VECTOR_FRAME.to_be_bytes());
    let src = pkt.config.source_name.as_bytes();
    let mut src_buf = [0u8; 64];
    let copy_len = src.len().min(63);
    src_buf[..copy_len].copy_from_slice(&src[..copy_len]);
    out.extend_from_slice(&src_buf);
    out.push(pkt.config.priority);
    out.push(pkt.sequence);
    out.extend_from_slice(&pkt.config.universe.to_be_bytes());
    out.push(SACN_VECTOR_DMP);
    out.push(SACN_DMX_START_CODE);
    out.extend_from_slice(&pkt.dmx_data);
    out
}

/// Set a DMX channel (1-indexed).
#[allow(dead_code)]
pub fn sacn_set_channel(pkt: &mut SacnPacket, channel: usize, value: u8) {
    if channel >= 1 && channel <= pkt.dmx_data.len() {
        pkt.dmx_data[channel - 1] = value;
    }
}

/// Get a DMX channel.
#[allow(dead_code)]
pub fn sacn_get_channel(pkt: &SacnPacket, channel: usize) -> u8 {
    if channel >= 1 && channel <= pkt.dmx_data.len() {
        pkt.dmx_data[channel - 1]
    } else {
        0
    }
}

/// Active (non-zero) channel count.
#[allow(dead_code)]
pub fn sacn_active_channels(pkt: &SacnPacket) -> usize {
    pkt.dmx_data.iter().filter(|&&v| v != 0).count()
}

/// Increment sequence number (wraps at 255).
#[allow(dead_code)]
pub fn sacn_next_sequence(pkt: &mut SacnPacket) {
    pkt.sequence = pkt.sequence.wrapping_add(1);
}

/// Validate packet (universe in 1..63999).
#[allow(dead_code)]
pub fn sacn_validate(pkt: &SacnPacket) -> bool {
    pkt.config.universe >= 1 && pkt.config.universe <= 63999
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_packet() -> SacnPacket {
        SacnPacket::new(SacnConfig::default())
    }

    #[test]
    fn packet_starts_with_preamble() {
        let pkt = default_packet();
        let bytes = build_sacn_packet(&pkt);
        let pre = u16::from_be_bytes([bytes[0], bytes[1]]);
        assert_eq!(pre, SACN_PREAMBLE_SIZE);
    }

    #[test]
    fn packet_nonempty() {
        let pkt = default_packet();
        let bytes = build_sacn_packet(&pkt);
        assert!(bytes.len() > 100);
    }

    #[test]
    fn set_get_channel() {
        let mut pkt = default_packet();
        sacn_set_channel(&mut pkt, 1, 150);
        assert_eq!(sacn_get_channel(&pkt, 1), 150);
    }

    #[test]
    fn active_channels_after_set() {
        let mut pkt = default_packet();
        sacn_set_channel(&mut pkt, 10, 100);
        assert_eq!(sacn_active_channels(&pkt), 1);
    }

    #[test]
    fn sequence_increments() {
        let mut pkt = default_packet();
        sacn_next_sequence(&mut pkt);
        assert_eq!(pkt.sequence, 1);
    }

    #[test]
    fn sequence_wraps() {
        let mut pkt = default_packet();
        pkt.sequence = 255;
        sacn_next_sequence(&mut pkt);
        assert_eq!(pkt.sequence, 0);
    }

    #[test]
    fn validate_default_universe() {
        let pkt = default_packet();
        assert!(sacn_validate(&pkt));
    }

    #[test]
    fn validate_universe_zero_fails() {
        let cfg = SacnConfig {
            universe: 0,
            ..Default::default()
        };
        let pkt = SacnPacket::new(cfg);
        assert!(!sacn_validate(&pkt));
    }

    #[test]
    fn acn_pid_in_packet() {
        let pkt = default_packet();
        let bytes = build_sacn_packet(&pkt);
        assert_eq!(&bytes[4..16], &SACN_ACN_PID);
    }

    #[test]
    fn default_config_priority() {
        let c = SacnConfig::default();
        assert_eq!(c.priority, SACN_DEFAULT_PRIORITY);
    }
}
