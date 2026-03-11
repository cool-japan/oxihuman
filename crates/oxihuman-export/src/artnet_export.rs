// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Art-Net UDP packet builder (DMX over IP stub).

pub const ARTNET_ID: &[u8; 8] = b"Art-Net\0";
pub const ARTNET_PORT: u16 = 6454;
pub const ARTNET_OP_DMX: u16 = 0x5000;
pub const ARTNET_PROTOCOL_VER: u16 = 14;

/// Art-Net DMX packet.
#[allow(dead_code)]
pub struct ArtDmxPacket {
    pub universe: u16,
    pub sequence: u8,
    pub physical: u8,
    pub data: Vec<u8>,
}

impl ArtDmxPacket {
    #[allow(dead_code)]
    pub fn new(universe: u16) -> Self {
        Self {
            universe,
            sequence: 0,
            physical: 0,
            data: vec![0u8; 512],
        }
    }
}

/// Build an Art-Net ArtDMX UDP payload.
#[allow(dead_code)]
pub fn build_artnet_dmx_packet(pkt: &ArtDmxPacket) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(ARTNET_ID);
    out.extend_from_slice(&ARTNET_OP_DMX.to_le_bytes());
    out.extend_from_slice(&ARTNET_PROTOCOL_VER.to_be_bytes());
    out.push(pkt.sequence);
    out.push(pkt.physical);
    out.extend_from_slice(&pkt.universe.to_le_bytes());
    let len = pkt.data.len().min(512) as u16;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&pkt.data[..len as usize]);
    out
}

/// Set a DMX channel in the packet (1-indexed).
#[allow(dead_code)]
pub fn artnet_set_channel(pkt: &mut ArtDmxPacket, channel: usize, value: u8) {
    if channel >= 1 && channel <= pkt.data.len() {
        pkt.data[channel - 1] = value;
    }
}

/// Get a DMX channel value.
#[allow(dead_code)]
pub fn artnet_get_channel(pkt: &ArtDmxPacket, channel: usize) -> u8 {
    if channel >= 1 && channel <= pkt.data.len() {
        pkt.data[channel - 1]
    } else {
        0
    }
}

/// Packet byte length.
#[allow(dead_code)]
pub fn artnet_packet_size(pkt: &ArtDmxPacket) -> usize {
    build_artnet_dmx_packet(pkt).len()
}

/// Number of active (non-zero) channels.
#[allow(dead_code)]
pub fn artnet_active_channels(pkt: &ArtDmxPacket) -> usize {
    pkt.data.iter().filter(|&&v| v != 0).count()
}

/// Clear all channels.
#[allow(dead_code)]
pub fn artnet_clear(pkt: &mut ArtDmxPacket) {
    pkt.data.fill(0);
}

/// Fill all channels with a value.
#[allow(dead_code)]
pub fn artnet_fill(pkt: &mut ArtDmxPacket, value: u8) {
    pkt.data.fill(value);
}

/// Universe as subnet/universe bytes.
#[allow(dead_code)]
pub fn universe_to_subnet_address(universe: u16) -> (u8, u8) {
    let subnet = ((universe >> 4) & 0x0F) as u8;
    let net = (universe & 0x0F) as u8;
    (subnet, net)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_starts_with_art_net_id() {
        let pkt = ArtDmxPacket::new(0);
        let bytes = build_artnet_dmx_packet(&pkt);
        assert_eq!(&bytes[0..8], ARTNET_ID);
    }

    #[test]
    fn packet_op_code_correct() {
        let pkt = ArtDmxPacket::new(0);
        let bytes = build_artnet_dmx_packet(&pkt);
        let op = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(op, ARTNET_OP_DMX);
    }

    #[test]
    fn packet_header_length_minimum() {
        let pkt = ArtDmxPacket::new(0);
        let bytes = build_artnet_dmx_packet(&pkt);
        assert!(bytes.len() >= 18);
    }

    #[test]
    fn set_get_channel() {
        let mut pkt = ArtDmxPacket::new(0);
        artnet_set_channel(&mut pkt, 1, 200);
        assert_eq!(artnet_get_channel(&pkt, 1), 200);
    }

    #[test]
    fn active_channels_after_set() {
        let mut pkt = ArtDmxPacket::new(0);
        artnet_set_channel(&mut pkt, 5, 100);
        assert_eq!(artnet_active_channels(&pkt), 1);
    }

    #[test]
    fn artnet_clear_zeros() {
        let mut pkt = ArtDmxPacket::new(0);
        artnet_fill(&mut pkt, 255);
        artnet_clear(&mut pkt);
        assert_eq!(artnet_active_channels(&pkt), 0);
    }

    #[test]
    fn artnet_fill_all_same() {
        let mut pkt = ArtDmxPacket::new(0);
        artnet_fill(&mut pkt, 42);
        assert!(pkt.data.iter().all(|&v| v == 42));
    }

    #[test]
    fn universe_subnet_address() {
        let (subnet, net) = universe_to_subnet_address(0x23);
        assert_eq!(subnet, 2);
        assert_eq!(net, 3);
    }

    #[test]
    fn packet_size_fixed() {
        let pkt = ArtDmxPacket::new(0);
        let sz = artnet_packet_size(&pkt);
        assert!((18..=600).contains(&sz));
    }

    #[test]
    fn default_port_correct() {
        assert_eq!(ARTNET_PORT, 6454);
    }
}
