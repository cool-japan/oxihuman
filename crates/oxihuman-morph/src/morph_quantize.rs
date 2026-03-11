//! Morph delta compression via quantization.
//! Compresses morph delta data to fewer bits for efficient storage and transfer.

#[allow(dead_code)]
pub struct QuantizedDelta {
    pub vertex_id: u32,
    pub dx: i16,
    pub dy: i16,
    pub dz: i16,
}

#[allow(dead_code)]
pub struct MorphQuantizeConfig {
    pub bits: u8,
    pub scale: f32,
    pub threshold: f32,
}

#[allow(dead_code)]
pub struct QuantizedMorph {
    pub name: String,
    pub deltas: Vec<QuantizedDelta>,
    pub scale: f32,
    pub original_count: usize,
}

#[allow(dead_code)]
pub fn default_morph_quantize_config() -> MorphQuantizeConfig {
    MorphQuantizeConfig {
        bits: 16,
        scale: 0.01,
        threshold: 0.0001,
    }
}

#[allow(dead_code)]
pub fn quantize_delta(dx: f32, dy: f32, dz: f32, scale: f32) -> (i16, i16, i16) {
    let clamp = |v: f32| -> i16 {
        let q = (v / scale).round();
        q.clamp(i16::MIN as f32, i16::MAX as f32) as i16
    };
    (clamp(dx), clamp(dy), clamp(dz))
}

#[allow(dead_code)]
pub fn dequantize_delta(qx: i16, qy: i16, qz: i16, scale: f32) -> [f32; 3] {
    [qx as f32 * scale, qy as f32 * scale, qz as f32 * scale]
}

#[allow(dead_code)]
pub fn quantize_morph(
    name: &str,
    deltas: &[(u32, [f32; 3])],
    cfg: &MorphQuantizeConfig,
) -> QuantizedMorph {
    let original_count = deltas.len();
    let mut quantized_deltas = Vec::new();

    for (vertex_id, delta) in deltas {
        let [dx, dy, dz] = *delta;
        let magnitude = (dx * dx + dy * dy + dz * dz).sqrt();
        if magnitude < cfg.threshold {
            continue;
        }
        let (qx, qy, qz) = quantize_delta(dx, dy, dz, cfg.scale);
        quantized_deltas.push(QuantizedDelta {
            vertex_id: *vertex_id,
            dx: qx,
            dy: qy,
            dz: qz,
        });
    }

    QuantizedMorph {
        name: name.to_string(),
        deltas: quantized_deltas,
        scale: cfg.scale,
        original_count,
    }
}

#[allow(dead_code)]
pub fn dequantize_morph(qm: &QuantizedMorph) -> Vec<(u32, [f32; 3])> {
    qm.deltas
        .iter()
        .map(|d| {
            let xyz = dequantize_delta(d.dx, d.dy, d.dz, qm.scale);
            (d.vertex_id, xyz)
        })
        .collect()
}

#[allow(dead_code)]
pub fn morph_compression_ratio(original: &[(u32, [f32; 3])], quantized: &QuantizedMorph) -> f32 {
    // original: u32 + 3xf32 = 4 + 12 = 16 bytes per entry
    // quantized: u32 + 3xi16 = 4 + 6 = 10 bytes per entry
    let orig_bytes = original.len() * 16;
    let quant_bytes = quantized.deltas.len() * 10;
    if quant_bytes == 0 {
        return f32::INFINITY;
    }
    orig_bytes as f32 / quant_bytes as f32
}

#[allow(dead_code)]
pub fn max_quantization_error(original: &[(u32, [f32; 3])], quantized: &QuantizedMorph) -> f32 {
    let deq = dequantize_morph(quantized);
    let mut max_err = 0.0f32;

    for (orig_vid, orig_delta) in original {
        let found = deq.iter().find(|(vid, _)| vid == orig_vid);
        if let Some((_, deq_delta)) = found {
            let err = (0..3)
                .map(|i| (orig_delta[i] - deq_delta[i]).abs())
                .fold(0.0f32, f32::max);
            max_err = max_err.max(err);
        } else {
            // dropped delta — error is the magnitude of original
            let err = orig_delta.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
            max_err = max_err.max(err);
        }
    }
    max_err
}

/// Pack to bytes: `[name_len: u32][name bytes][scale: f32][original_count: u32][delta_count: u32][deltas...]`
#[allow(dead_code)]
pub fn pack_quantized_morph(qm: &QuantizedMorph) -> Vec<u8> {
    let name_bytes = qm.name.as_bytes();
    let name_len = name_bytes.len() as u32;
    let delta_count = qm.deltas.len() as u32;
    let original_count = qm.original_count as u32;

    let mut out = Vec::new();
    out.extend_from_slice(&name_len.to_le_bytes());
    out.extend_from_slice(name_bytes);
    out.extend_from_slice(&qm.scale.to_bits().to_le_bytes());
    out.extend_from_slice(&original_count.to_le_bytes());
    out.extend_from_slice(&delta_count.to_le_bytes());

    for d in &qm.deltas {
        out.extend_from_slice(&d.vertex_id.to_le_bytes());
        out.extend_from_slice(&d.dx.to_le_bytes());
        out.extend_from_slice(&d.dy.to_le_bytes());
        out.extend_from_slice(&d.dz.to_le_bytes());
    }
    out
}

/// Parse from bytes: returns None if malformed
#[allow(dead_code)]
pub fn unpack_quantized_morph(data: &[u8]) -> Option<QuantizedMorph> {
    if data.len() < 4 {
        return None;
    }
    let mut pos = 0usize;

    let name_len = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
    pos += 4;
    if data.len() < pos + name_len {
        return None;
    }
    let name = std::str::from_utf8(&data[pos..pos + name_len])
        .ok()?
        .to_string();
    pos += name_len;

    if data.len() < pos + 4 {
        return None;
    }
    let scale_bits = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
    let scale = f32::from_bits(scale_bits);
    pos += 4;

    if data.len() < pos + 8 {
        return None;
    }
    let original_count = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
    pos += 4;
    let delta_count = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
    pos += 4;

    // each delta: u32 + i16 + i16 + i16 = 10 bytes
    if data.len() < pos + delta_count * 10 {
        return None;
    }

    let mut deltas = Vec::with_capacity(delta_count);
    for _ in 0..delta_count {
        let vertex_id = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let dx = i16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
        pos += 2;
        let dy = i16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
        pos += 2;
        let dz = i16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
        pos += 2;
        deltas.push(QuantizedDelta {
            vertex_id,
            dx,
            dy,
            dz,
        });
    }

    Some(QuantizedMorph {
        name,
        deltas,
        scale,
        original_count,
    })
}

/// Merge two quantized morphs (same scale assumed). If vertex appears in both, sum the deltas.
#[allow(dead_code)]
pub fn merge_quantized_morphs(a: &QuantizedMorph, b: &QuantizedMorph) -> QuantizedMorph {
    use std::collections::HashMap;
    let mut map: HashMap<u32, (i16, i16, i16)> = HashMap::new();

    for d in &a.deltas {
        map.insert(d.vertex_id, (d.dx, d.dy, d.dz));
    }
    for d in &b.deltas {
        let entry = map.entry(d.vertex_id).or_insert((0, 0, 0));
        entry.0 = entry.0.saturating_add(d.dx);
        entry.1 = entry.1.saturating_add(d.dy);
        entry.2 = entry.2.saturating_add(d.dz);
    }

    let mut deltas: Vec<QuantizedDelta> = map
        .into_iter()
        .map(|(vertex_id, (dx, dy, dz))| QuantizedDelta {
            vertex_id,
            dx,
            dy,
            dz,
        })
        .collect();
    deltas.sort_by_key(|d| d.vertex_id);

    QuantizedMorph {
        name: format!("{}+{}", a.name, b.name),
        deltas,
        scale: a.scale,
        original_count: a.original_count + b.original_count,
    }
}

/// Remove entries where dx=dy=dz=0
#[allow(dead_code)]
pub fn filter_zero_deltas(qm: &mut QuantizedMorph) {
    qm.deltas.retain(|d| d.dx != 0 || d.dy != 0 || d.dz != 0);
}

#[allow(dead_code)]
pub fn quantized_delta_count(qm: &QuantizedMorph) -> usize {
    qm.deltas.len()
}

#[allow(dead_code)]
pub fn apply_quantized_morph(positions: &mut [[f32; 3]], qm: &QuantizedMorph, weight: f32) {
    for d in &qm.deltas {
        let vid = d.vertex_id as usize;
        if vid < positions.len() {
            let [dx, dy, dz] = dequantize_delta(d.dx, d.dy, d.dz, qm.scale);
            positions[vid][0] += dx * weight;
            positions[vid][1] += dy * weight;
            positions[vid][2] += dz * weight;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_deltas() -> Vec<(u32, [f32; 3])> {
        vec![
            (0, [0.05, 0.0, 0.0]),
            (1, [0.0, 0.1, 0.0]),
            (2, [0.0, 0.0, 0.15]),
            (3, [0.02, 0.03, 0.04]),
            (4, [0.0, 0.0, 0.0]), // zero — should be filtered
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_morph_quantize_config();
        assert_eq!(cfg.bits, 16);
        assert!(cfg.scale > 0.0);
        assert!(cfg.threshold >= 0.0);
    }

    #[test]
    fn test_quantize_dequantize_roundtrip() {
        let scale = 0.001;
        let (dx, dy, dz) = (0.123f32, -0.456f32, 0.789f32);
        let (qx, qy, qz) = quantize_delta(dx, dy, dz, scale);
        let [rx, ry, rz] = dequantize_delta(qx, qy, qz, scale);
        assert!((rx - dx).abs() < scale);
        assert!((ry - dy).abs() < scale);
        assert!((rz - dz).abs() < scale);
    }

    #[test]
    fn test_quantize_zero() {
        let (qx, qy, qz) = quantize_delta(0.0, 0.0, 0.0, 0.01);
        assert_eq!(qx, 0);
        assert_eq!(qy, 0);
        assert_eq!(qz, 0);
    }

    #[test]
    fn test_dequantize_zero() {
        let [rx, ry, rz] = dequantize_delta(0, 0, 0, 0.01);
        assert_eq!(rx, 0.0);
        assert_eq!(ry, 0.0);
        assert_eq!(rz, 0.0);
    }

    #[test]
    fn test_quantize_morph_basic() {
        let cfg = default_morph_quantize_config();
        let deltas = sample_deltas();
        let qm = quantize_morph("test", &deltas, &cfg);
        assert_eq!(qm.name, "test");
        assert_eq!(qm.original_count, deltas.len());
        // zero delta should be filtered
        assert!(qm.deltas.len() < deltas.len());
    }

    #[test]
    fn test_quantize_morph_threshold() {
        let cfg = MorphQuantizeConfig {
            bits: 16,
            scale: 0.01,
            threshold: 0.5,
        };
        let deltas = vec![(0, [0.01f32, 0.0, 0.0]), (1, [1.0, 0.0, 0.0])];
        let qm = quantize_morph("th", &deltas, &cfg);
        // only vertex 1 should pass threshold
        assert_eq!(qm.deltas.len(), 1);
        assert_eq!(qm.deltas[0].vertex_id, 1);
    }

    #[test]
    fn test_dequantize_morph() {
        let cfg = default_morph_quantize_config();
        let deltas = vec![(0, [0.05f32, 0.1, 0.15])];
        let qm = quantize_morph("deq", &deltas, &cfg);
        let result = dequantize_morph(&qm);
        assert_eq!(result.len(), qm.deltas.len());
        if !result.is_empty() {
            let [rx, ry, rz] = result[0].1;
            assert!((rx - 0.05).abs() < cfg.scale * 2.0);
            assert!((ry - 0.1).abs() < cfg.scale * 2.0);
            assert!((rz - 0.15).abs() < cfg.scale * 2.0);
        }
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let cfg = default_morph_quantize_config();
        let deltas = sample_deltas();
        let qm = quantize_morph("roundtrip", &deltas, &cfg);
        let packed = pack_quantized_morph(&qm);
        let unpacked = unpack_quantized_morph(&packed).expect("unpack failed");
        assert_eq!(unpacked.name, qm.name);
        assert_eq!(unpacked.deltas.len(), qm.deltas.len());
        assert_eq!(unpacked.original_count, qm.original_count);
        assert!((unpacked.scale - qm.scale).abs() < 1e-9);
    }

    #[test]
    fn test_unpack_empty_returns_none() {
        assert!(unpack_quantized_morph(&[]).is_none());
    }

    #[test]
    fn test_compression_ratio_geq_one() {
        let cfg = default_morph_quantize_config();
        let deltas = sample_deltas();
        let qm = quantize_morph("cr", &deltas, &cfg);
        let ratio = morph_compression_ratio(&deltas, &qm);
        assert!(ratio >= 1.0);
    }

    #[test]
    fn test_max_quantization_error() {
        let cfg = MorphQuantizeConfig {
            bits: 16,
            scale: 0.001,
            threshold: 0.0001,
        };
        let deltas = vec![(0, [0.123f32, 0.456, 0.789])];
        let qm = quantize_morph("err", &deltas, &cfg);
        let err = max_quantization_error(&deltas, &qm);
        assert!(err < cfg.scale * 2.0);
    }

    #[test]
    fn test_filter_zero_deltas() {
        let mut qm = QuantizedMorph {
            name: "fz".to_string(),
            deltas: vec![
                QuantizedDelta {
                    vertex_id: 0,
                    dx: 0,
                    dy: 0,
                    dz: 0,
                },
                QuantizedDelta {
                    vertex_id: 1,
                    dx: 1,
                    dy: 0,
                    dz: 0,
                },
                QuantizedDelta {
                    vertex_id: 2,
                    dx: 0,
                    dy: 0,
                    dz: 0,
                },
            ],
            scale: 0.01,
            original_count: 3,
        };
        filter_zero_deltas(&mut qm);
        assert_eq!(qm.deltas.len(), 1);
        assert_eq!(qm.deltas[0].vertex_id, 1);
    }

    #[test]
    fn test_quantized_delta_count() {
        let qm = QuantizedMorph {
            name: "cnt".to_string(),
            deltas: vec![
                QuantizedDelta {
                    vertex_id: 0,
                    dx: 1,
                    dy: 0,
                    dz: 0,
                },
                QuantizedDelta {
                    vertex_id: 1,
                    dx: 0,
                    dy: 1,
                    dz: 0,
                },
            ],
            scale: 0.01,
            original_count: 2,
        };
        assert_eq!(quantized_delta_count(&qm), 2);
    }

    #[test]
    fn test_apply_quantized_morph() {
        let qm = QuantizedMorph {
            name: "apply".to_string(),
            deltas: vec![QuantizedDelta {
                vertex_id: 0,
                dx: 100,
                dy: 0,
                dz: 0,
            }],
            scale: 0.01,
            original_count: 1,
        };
        let mut positions = [[0.0f32; 3]; 3];
        apply_quantized_morph(&mut positions, &qm, 1.0);
        assert!((positions[0][0] - 1.0).abs() < 1e-5);
        assert_eq!(positions[0][1], 0.0);
        assert_eq!(positions[1][0], 0.0);
    }

    #[test]
    fn test_apply_quantized_morph_weight() {
        let qm = QuantizedMorph {
            name: "w".to_string(),
            deltas: vec![QuantizedDelta {
                vertex_id: 0,
                dx: 100,
                dy: 0,
                dz: 0,
            }],
            scale: 0.01,
            original_count: 1,
        };
        let mut positions = [[0.0f32; 3]; 1];
        apply_quantized_morph(&mut positions, &qm, 0.5);
        assert!((positions[0][0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_merge_quantized_morphs() {
        let a = QuantizedMorph {
            name: "a".to_string(),
            deltas: vec![QuantizedDelta {
                vertex_id: 0,
                dx: 10,
                dy: 0,
                dz: 0,
            }],
            scale: 0.01,
            original_count: 1,
        };
        let b = QuantizedMorph {
            name: "b".to_string(),
            deltas: vec![QuantizedDelta {
                vertex_id: 0,
                dx: 5,
                dy: 0,
                dz: 0,
            }],
            scale: 0.01,
            original_count: 1,
        };
        let merged = merge_quantized_morphs(&a, &b);
        assert_eq!(merged.deltas.len(), 1);
        assert_eq!(merged.deltas[0].dx, 15);
    }
}
