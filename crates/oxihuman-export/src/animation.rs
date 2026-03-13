// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GLTF 2.0 morph animation keyframe export.

use anyhow::{bail, Result};
use serde_json::{json, Value};

/// One keyframe: a time stamp + a set of morph target weights.
pub struct AnimKeyframe {
    /// Time in seconds.
    pub time_s: f32,
    /// One weight per morph target (0..1).
    pub weights: Vec<f32>,
}

/// A named animation clip composed of keyframes.
pub struct AnimClip {
    pub name: String,
    pub keyframes: Vec<AnimKeyframe>,
}

// ── base64 helper (std only, no external crate) ──────────────────────────────

fn to_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let combined = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((combined >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((combined >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARS[((combined >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[(combined & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

// ── byte helpers ─────────────────────────────────────────────────────────────

/// Append a slice of f32 values as little-endian bytes.
fn push_f32_slice(buf: &mut Vec<u8>, values: &[f32]) {
    for &v in values {
        buf.extend_from_slice(&v.to_le_bytes());
    }
}

/// Flatten `[[f32;3]]` into a byte vec.
fn positions_to_bytes(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(positions.len() * 12);
    for p in positions {
        for &c in p {
            buf.extend_from_slice(&c.to_le_bytes());
        }
    }
    buf
}

/// Compute per-vertex position deltas: `morph[i] - base[i]`.
fn compute_deltas(base: &[[f32; 3]], morph: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(morph.iter())
        .map(|(b, m)| [m[0] - b[0], m[1] - b[1], m[2] - b[2]])
        .collect()
}

// ── main export ──────────────────────────────────────────────────────────────

/// Export a GLTF animation as a standalone JSON string.
///
/// * `base_positions` — base mesh vertex positions.
/// * `morph_target_positions` — per-target position delta arrays; each inner
///   vec must have the same length as `base_positions`.
/// * `clip` — the animation keyframes.
///
/// Returns a GLTF 2.0 JSON string with mesh + animation.
pub fn export_animation_gltf(
    base_positions: &[[f32; 3]],
    morph_target_positions: &[Vec<[f32; 3]>],
    clip: &AnimClip,
) -> Result<String> {
    let n_verts = base_positions.len();
    let n_targets = morph_target_positions.len();

    // Validate morph target vertex counts
    for (i, target) in morph_target_positions.iter().enumerate() {
        if target.len() != n_verts {
            bail!(
                "morph_target_positions[{}] has {} verts, expected {}",
                i,
                target.len(),
                n_verts
            );
        }
    }

    // Validate keyframe weight counts
    for (ki, kf) in clip.keyframes.iter().enumerate() {
        if kf.weights.len() != n_targets {
            bail!(
                "keyframe[{}].weights has {} entries, expected {} (n_targets)",
                ki,
                kf.weights.len(),
                n_targets
            );
        }
    }

    // ── Assemble binary buffer ────────────────────────────────────────────────
    //
    // Layout (all sections 4-byte aligned):
    //   [0] BASE POSITION  (n_verts × 3 × f32)
    //   [1..n_targets] DELTA POSITION per morph target
    //   [last-2] times accessor  (n_keyframes × f32)      — only if keyframes > 0
    //   [last-1] weights accessor (n_keyframes × n_targets × f32) — only if keyframes > 0

    let mut buffer: Vec<u8> = Vec::new();

    // Helper to record a section: returns (byte_offset, byte_length).
    // Pads to 4-byte alignment after appending.
    let append_section = |buffer: &mut Vec<u8>, data: &[u8]| -> (usize, usize) {
        let offset = buffer.len();
        let length = data.len();
        buffer.extend_from_slice(data);
        // pad to 4-byte alignment
        while !buffer.len().is_multiple_of(4) {
            buffer.push(0x00);
        }
        (offset, length)
    };

    // BASE POSITION
    let base_bytes = positions_to_bytes(base_positions);
    let (base_pos_offset, base_pos_len) = append_section(&mut buffer, &base_bytes);

    // MORPH TARGET DELTAS
    let mut delta_sections: Vec<(usize, usize)> = Vec::with_capacity(n_targets);
    for target in morph_target_positions {
        let deltas = compute_deltas(base_positions, target);
        let delta_bytes = positions_to_bytes(&deltas);
        let sec = append_section(&mut buffer, &delta_bytes);
        delta_sections.push(sec);
    }

    // TIMES + WEIGHTS (only when clip has keyframes)
    let has_animation = !clip.keyframes.is_empty();
    let n_keyframes = clip.keyframes.len();

    let times_section: (usize, usize);
    let weights_section: (usize, usize);

    if has_animation {
        // times
        let mut times_bytes: Vec<u8> = Vec::with_capacity(n_keyframes * 4);
        for kf in &clip.keyframes {
            push_f32_slice(&mut times_bytes, &[kf.time_s]);
        }
        times_section = append_section(&mut buffer, &times_bytes);

        // weights — flattened: for each keyframe, all target weights in order
        let total_weights = n_keyframes * n_targets;
        let mut weights_bytes: Vec<u8> = Vec::with_capacity(total_weights * 4);
        for kf in &clip.keyframes {
            push_f32_slice(&mut weights_bytes, &kf.weights);
        }
        weights_section = append_section(&mut buffer, &weights_bytes);
    } else {
        times_section = (0, 0);
        weights_section = (0, 0);
    }

    // ── Build accessor / bufferView index accounting ──────────────────────────
    //
    // Accessor indices:
    //   0            → BASE POSITION
    //   1..n_targets → DELTA POSITION per target
    //   (if animation)
    //   n_targets+1  → times
    //   n_targets+2  → weights

    let times_accessor_idx = (n_targets + 1) as u64;
    let weights_accessor_idx = (n_targets + 2) as u64;

    // bufferViews mirror accessors 1-to-1
    let mut buffer_views: Vec<Value> = Vec::new();
    let mut accessors: Vec<Value> = Vec::new();

    // BASE POSITION accessor
    buffer_views.push(json!({
        "buffer": 0,
        "byteOffset": base_pos_offset,
        "byteLength": base_pos_len
    }));
    accessors.push(json!({
        "bufferView": 0,
        "componentType": 5126,   // FLOAT
        "count": n_verts,
        "type": "VEC3"
    }));

    // DELTA POSITION accessors
    for (i, &(offset, length)) in delta_sections.iter().enumerate() {
        let bv_idx = (i + 1) as u64;
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": length
        }));
        accessors.push(json!({
            "bufferView": bv_idx,
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC3"
        }));
    }

    // TIMES accessor
    if has_animation {
        let bv_times = (n_targets + 1) as u64;
        let (t_offset, t_length) = times_section;
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": t_offset,
            "byteLength": t_length
        }));
        accessors.push(json!({
            "bufferView": bv_times,
            "componentType": 5126,
            "count": n_keyframes,
            "type": "SCALAR"
        }));

        // WEIGHTS accessor
        let bv_weights = (n_targets + 2) as u64;
        let (w_offset, w_length) = weights_section;
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": w_offset,
            "byteLength": w_length
        }));
        accessors.push(json!({
            "bufferView": bv_weights,
            "componentType": 5126,
            "count": n_keyframes * n_targets,
            "type": "SCALAR"
        }));
    }

    // ── Mesh primitives targets ───────────────────────────────────────────────
    let targets: Vec<Value> = (0..n_targets)
        .map(|i| json!({ "POSITION": i + 1 }))
        .collect();

    let initial_weights: Vec<f32> = vec![0.0_f32; n_targets];

    // ── Animation ────────────────────────────────────────────────────────────
    let animations: Value = if has_animation {
        json!([{
            "name": clip.name,
            "samplers": [{
                "input": times_accessor_idx,
                "output": weights_accessor_idx,
                "interpolation": "LINEAR"
            }],
            "channels": [{
                "sampler": 0,
                "target": { "node": 0, "path": "weights" }
            }]
        }])
    } else {
        json!([])
    };

    // ── Buffer URI ────────────────────────────────────────────────────────────
    let b64 = to_base64(&buffer);
    let data_uri = format!("data:application/octet-stream;base64,{}", b64);

    // ── Assemble GLTF ─────────────────────────────────────────────────────────
    let gltf = json!({
        "asset": { "version": "2.0", "generator": "OxiHuman 0.1.0" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{
            "name": clip.name,
            "primitives": [{
                "attributes": { "POSITION": 0 },
                "mode": 4,
                "targets": targets
            }],
            "weights": initial_weights
        }],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{
            "uri": data_uri,
            "byteLength": buffer.len()
        }],
        "animations": animations
    });

    Ok(serde_json::to_string_pretty(&gltf)?)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// Build a minimal base mesh and morph targets for testing.
    fn make_base(n_verts: usize) -> Vec<[f32; 3]> {
        (0..n_verts).map(|i| [i as f32, 0.0, 0.0]).collect()
    }

    fn make_target(base: &[[f32; 3]], offset: f32) -> Vec<[f32; 3]> {
        base.iter().map(|&[x, y, z]| [x + offset, y, z]).collect()
    }

    fn two_keyframe_clip(n_targets: usize) -> AnimClip {
        AnimClip {
            name: "Test".to_string(),
            keyframes: vec![
                AnimKeyframe {
                    time_s: 0.0,
                    weights: vec![0.0; n_targets],
                },
                AnimKeyframe {
                    time_s: 1.0,
                    weights: vec![1.0; n_targets],
                },
            ],
        }
    }

    // ── Test 1 ────────────────────────────────────────────────────────────────
    /// Exported JSON must parse and have asset.version == "2.0".
    #[test]
    fn animation_json_parses() {
        let base = make_base(4);
        let targets = vec![make_target(&base, 0.1)];
        let clip = two_keyframe_clip(1);
        let json_str = export_animation_gltf(&base, &targets, &clip).expect("should succeed");
        let val: Value = serde_json::from_str(&json_str).expect("must parse as JSON");
        assert_eq!(val["asset"]["version"].as_str().expect("should succeed"), "2.0");
    }

    // ── Test 2 ────────────────────────────────────────────────────────────────
    /// 3 morph targets → meshes[0].primitives[0].targets has 3 entries.
    #[test]
    fn animation_has_correct_target_count() {
        let base = make_base(5);
        let targets: Vec<Vec<[f32; 3]>> =
            (0..3).map(|i| make_target(&base, i as f32 * 0.1)).collect();
        let clip = two_keyframe_clip(3);
        let json_str = export_animation_gltf(&base, &targets, &clip).expect("should succeed");
        let val: Value = serde_json::from_str(&json_str).expect("should succeed");
        let tgt_arr = val["meshes"][0]["primitives"][0]["targets"]
            .as_array()
            .expect("should succeed");
        assert_eq!(tgt_arr.len(), 3);
    }

    // ── Test 3 ────────────────────────────────────────────────────────────────
    /// 4 keyframes, 2 targets → weights accessor count = 4 * 2 = 8.
    #[test]
    fn animation_keyframe_count_matches() {
        let n_targets = 2usize;
        let n_keyframes = 4usize;
        let base = make_base(3);
        let targets: Vec<Vec<[f32; 3]>> = (0..n_targets)
            .map(|i| make_target(&base, i as f32 * 0.05))
            .collect();
        let clip = AnimClip {
            name: "Walk".to_string(),
            keyframes: (0..n_keyframes)
                .map(|k| AnimKeyframe {
                    time_s: k as f32 * 0.25,
                    weights: vec![k as f32 / n_keyframes as f32; n_targets],
                })
                .collect(),
        };
        let json_str = export_animation_gltf(&base, &targets, &clip).expect("should succeed");
        let val: Value = serde_json::from_str(&json_str).expect("should succeed");

        // The weights accessor is the last one.
        let accessors = val["accessors"].as_array().expect("should succeed");
        let weights_acc = accessors.last().expect("should succeed");
        assert_eq!(
            weights_acc["count"].as_u64().expect("should succeed"),
            (n_keyframes * n_targets) as u64
        );
    }

    // ── Test 4 ────────────────────────────────────────────────────────────────
    /// Empty clip → animations array is empty.
    #[test]
    fn empty_clip_no_animation() {
        let base = make_base(4);
        let targets = vec![make_target(&base, 0.1)];
        let clip = AnimClip {
            name: "Empty".to_string(),
            keyframes: vec![],
        };
        let json_str = export_animation_gltf(&base, &targets, &clip).expect("should succeed");
        let val: Value = serde_json::from_str(&json_str).expect("should succeed");
        let anim = val["animations"].as_array().expect("should succeed");
        assert!(anim.is_empty(), "animations should be empty for empty clip");
    }

    // ── Test 5 ────────────────────────────────────────────────────────────────
    /// to_base64 on known input matches expected base64 string.
    #[test]
    fn base64_roundtrip() {
        // "Man" → base64 "TWFu"
        assert_eq!(to_base64(b"Man"), "TWFu");
        // "Ma" → "TWE="
        assert_eq!(to_base64(b"Ma"), "TWE=");
        // "M" → "TQ=="
        assert_eq!(to_base64(b"M"), "TQ==");
        // empty
        assert_eq!(to_base64(b""), "");
        // longer known value: "Hello" → "SGVsbG8="
        assert_eq!(to_base64(b"Hello"), "SGVsbG8=");
    }

    // ── Test 6 ────────────────────────────────────────────────────────────────
    /// Weights accessor binary data matches expected byte layout.
    ///
    /// We construct a 1-target, 2-keyframe clip with weights [0.25] and [0.75],
    /// decode the base64 buffer from the JSON, and check the bytes at the
    /// weights accessor offset match the expected f32 little-endian layout.
    #[test]
    fn weights_flattened_order() {
        let base = make_base(2);
        let targets = vec![make_target(&base, 0.1)];
        let clip = AnimClip {
            name: "Weights".to_string(),
            keyframes: vec![
                AnimKeyframe {
                    time_s: 0.0,
                    weights: vec![0.25_f32],
                },
                AnimKeyframe {
                    time_s: 1.0,
                    weights: vec![0.75_f32],
                },
            ],
        };

        let json_str = export_animation_gltf(&base, &targets, &clip).expect("should succeed");
        let val: Value = serde_json::from_str(&json_str).expect("should succeed");

        // Extract buffer URI and decode base64
        let uri = val["buffers"][0]["uri"].as_str().expect("should succeed");
        let b64_data = uri
            .strip_prefix("data:application/octet-stream;base64,")
            .expect("should succeed");
        let raw = decode_base64(b64_data);

        // Find weights bufferView (last bufferView)
        let bvs = val["bufferViews"].as_array().expect("should succeed");
        let weights_bv = bvs.last().expect("should succeed");
        let offset = weights_bv["byteOffset"].as_u64().expect("should succeed") as usize;
        let length = weights_bv["byteLength"].as_u64().expect("should succeed") as usize;

        let weights_bytes = &raw[offset..offset + length];

        // Expected: [0.25f32, 0.75f32] in LE
        let mut expected = Vec::new();
        expected.extend_from_slice(&0.25_f32.to_le_bytes());
        expected.extend_from_slice(&0.75_f32.to_le_bytes());

        assert_eq!(weights_bytes, expected.as_slice());
    }

    /// Minimal base64 decoder for test verification only.
    fn decode_base64(s: &str) -> Vec<u8> {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = Vec::new();
        let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'=').collect();
        let lookup = |c: u8| {
            CHARS
                .iter()
                .position(|&x| x == c)
                .expect("invalid base64 character") as u32
        };
        let mut i = 0;
        // Count actual padding
        let pad = s.bytes().filter(|&b| b == b'=').count();
        let chunks_len = bytes.len().div_ceil(4);
        while i < chunks_len {
            let start = i * 4;
            let get = |j: usize| {
                if start + j < bytes.len() {
                    lookup(bytes[start + j])
                } else {
                    0
                }
            };
            let combined = (get(0) << 18) | (get(1) << 12) | (get(2) << 6) | get(3);
            out.push(((combined >> 16) & 0xFF) as u8);
            if !(i == chunks_len - 1 && pad >= 2) {
                out.push(((combined >> 8) & 0xFF) as u8);
            }
            if !(i == chunks_len - 1 && pad >= 1) {
                out.push((combined & 0xFF) as u8);
            }
            i += 1;
        }
        out
    }
}
