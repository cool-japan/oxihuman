// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Morph channel export: named morph targets with weights.

/// A morph channel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphChannel {
    pub name: String,
    pub weight: f32,
    pub vertex_count: usize,
}

/// Morph channel collection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphChannelExport {
    pub channels: Vec<MorphChannel>,
}

#[allow(dead_code)]
pub fn new_morph_channel_export() -> MorphChannelExport {
    MorphChannelExport {
        channels: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn mc_add_channel(e: &mut MorphChannelExport, name: &str, weight: f32, verts: usize) {
    e.channels.push(MorphChannel {
        name: name.to_string(),
        weight,
        vertex_count: verts,
    });
}

#[allow(dead_code)]
pub fn mc_channel_count(e: &MorphChannelExport) -> usize {
    e.channels.len()
}

#[allow(dead_code)]
pub fn mc_get_channel(e: &MorphChannelExport, idx: usize) -> Option<&MorphChannel> {
    e.channels.get(idx)
}

#[allow(dead_code)]
pub fn mc_find_by_name(e: &MorphChannelExport, name: &str) -> Option<usize> {
    e.channels.iter().position(|c| c.name == name)
}

#[allow(dead_code)]
pub fn mc_set_weight(e: &mut MorphChannelExport, idx: usize, w: f32) {
    if let Some(c) = e.channels.get_mut(idx) {
        c.weight = w.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn mc_total_vertices(e: &MorphChannelExport) -> usize {
    e.channels.iter().map(|c| c.vertex_count).sum()
}

#[allow(dead_code)]
pub fn mc_clear(e: &mut MorphChannelExport) {
    e.channels.clear();
}

#[allow(dead_code)]
pub fn morph_channel_to_json(e: &MorphChannelExport) -> String {
    format!("{{\"channels\":{}}}", e.channels.len())
}

#[allow(dead_code)]
pub fn mc_validate(e: &MorphChannelExport) -> bool {
    e.channels
        .iter()
        .all(|c| (0.0..=1.0).contains(&c.weight) && !c.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(mc_channel_count(&new_morph_channel_export()), 0);
    }

    #[test]
    fn test_add_channel() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "smile", 0.5, 100);
        assert_eq!(mc_channel_count(&e), 1);
    }

    #[test]
    fn test_get_channel() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "blink", 0.3, 50);
        let c = mc_get_channel(&e, 0).unwrap();
        assert_eq!(c.name, "blink");
    }

    #[test]
    fn test_find_by_name() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "smile", 0.5, 100);
        mc_add_channel(&mut e, "frown", 0.0, 100);
        assert_eq!(mc_find_by_name(&e, "frown"), Some(1));
    }

    #[test]
    fn test_find_missing() {
        let e = new_morph_channel_export();
        assert!(mc_find_by_name(&e, "x").is_none());
    }

    #[test]
    fn test_set_weight() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "a", 0.0, 10);
        mc_set_weight(&mut e, 0, 0.7);
        assert!((e.channels[0].weight - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_total_vertices() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "a", 0.0, 10);
        mc_add_channel(&mut e, "b", 0.0, 20);
        assert_eq!(mc_total_vertices(&e), 30);
    }

    #[test]
    fn test_clear() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "a", 0.0, 10);
        mc_clear(&mut e);
        assert_eq!(mc_channel_count(&e), 0);
    }

    #[test]
    fn test_to_json() {
        assert!(morph_channel_to_json(&new_morph_channel_export()).contains("\"channels\":0"));
    }

    #[test]
    fn test_validate() {
        let mut e = new_morph_channel_export();
        mc_add_channel(&mut e, "ok", 0.5, 10);
        assert!(mc_validate(&e));
    }
}
