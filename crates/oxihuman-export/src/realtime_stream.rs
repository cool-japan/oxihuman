// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Real-time mesh streaming session with delta/quantized encoding (WebSocket stub).

// NOTE: `StreamFormat` is also defined in `streaming_export`. We use a distinct
// type here scoped to this module. The pub re-export in lib.rs must alias one of them.

#[allow(dead_code)]
/// One streaming frame of mesh data.
#[derive(Debug, Clone)]
pub struct StreamFrame {
    pub frame_id: u64,
    pub timestamp_ms: u64,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

#[allow(dead_code)]
/// Compression settings.
#[derive(Debug, Clone)]
pub struct StreamCompression {
    /// "none", "delta", or "quantized"
    pub kind: String,
    pub level: u8,
}

#[allow(dead_code)]
/// Wire format settings.
#[derive(Debug, Clone)]
pub struct RtStreamFormat {
    /// "json", "binary", or "msgpack-stub"
    pub kind: String,
}

#[allow(dead_code)]
/// Configuration for a streaming session.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub target_fps: f32,
    pub compression: StreamCompression,
    pub format: RtStreamFormat,
}

#[allow(dead_code)]
/// An active streaming session accumulating frames.
pub struct StreamSession {
    pub config: StreamConfig,
    pub frames: Vec<StreamFrame>,
    pub base_frame: Option<StreamFrame>,
    next_id: u64,
    time_ms: u64,
}

impl StreamSession {
    /// Create a new session.
    pub fn new(config: StreamConfig) -> Self {
        StreamSession {
            config,
            frames: Vec::new(),
            base_frame: None,
            next_id: 0,
            time_ms: 0,
        }
    }

    /// Push a new frame into the session.
    pub fn push_frame(&mut self, positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) {
        let frame = StreamFrame {
            frame_id: self.next_id,
            timestamp_ms: self.time_ms,
            positions,
            normals,
        };
        if self.next_id == 0 {
            self.base_frame = Some(frame.clone());
        }
        self.frames.push(frame);
        self.next_id += 1;
        let dt_ms = (1000.0 / self.config.target_fps.max(1.0)) as u64;
        self.time_ms += dt_ms;
    }

    /// Encode a stored frame by index based on session compression config.
    pub fn encode_frame(&self, idx: usize) -> Vec<u8> {
        let frame = match self.frames.get(idx) {
            Some(f) => f,
            None => return vec![],
        };
        match self.config.compression.kind.as_str() {
            "delta" => {
                let base = self
                    .base_frame
                    .as_ref()
                    .map(|b| b.positions.as_slice())
                    .unwrap_or(&[]);
                let deltas = delta_encode_positions(base, &frame.positions);
                encode_frame_binary(frame.frame_id, frame.timestamp_ms, &deltas, &frame.normals)
            }
            "quantized" => {
                let (bmin, bmax) = positions_bounds(&frame.positions);
                let q = quantize_positions_16bit(&frame.positions, bmin, bmax);
                encode_frame_quantized(frame.frame_id, frame.timestamp_ms, &q, bmin, bmax)
            }
            _ => encode_frame_binary(
                frame.frame_id,
                frame.timestamp_ms,
                &frame.positions,
                &frame.normals,
            ),
        }
    }

    /// Decode a frame from bytes.
    pub fn decode_frame(data: &[u8]) -> Option<StreamFrame> {
        decode_frame_binary(data)
    }

    /// Number of frames in this session.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Average encoded size (bytes) over all frames.
    pub fn avg_frame_size(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        let total: usize = (0..self.frames.len())
            .map(|i| self.encode_frame(i).len())
            .sum();
        total as f32 / self.frames.len() as f32
    }
}

// ---------------------------------------------------------------------------
// Binary frame encoding (simple: header + raw f32 array)
// ---------------------------------------------------------------------------
// Layout: magic(4) | frame_id(8) | timestamp_ms(8) | n_pos(4) | pos(n*12) | n_norm(4) | norm(n*12)

const FRAME_MAGIC: &[u8; 4] = b"OXSF";

fn encode_frame_binary(
    frame_id: u64,
    timestamp_ms: u64,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(FRAME_MAGIC);
    out.extend_from_slice(&frame_id.to_le_bytes());
    out.extend_from_slice(&timestamp_ms.to_le_bytes());
    let np = positions.len() as u32;
    out.extend_from_slice(&np.to_le_bytes());
    for p in positions {
        for &v in p {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    let nn = normals.len() as u32;
    out.extend_from_slice(&nn.to_le_bytes());
    for n in normals {
        for &v in n {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

fn decode_frame_binary(data: &[u8]) -> Option<StreamFrame> {
    if data.len() < 24 {
        return None;
    }
    if &data[..4] != FRAME_MAGIC && &data[..4] != b"OXQF" {
        return None;
    }
    // Support both OXSF and OXQF (quantized) — but quantized decode is different
    if &data[..4] == b"OXQF" {
        return decode_frame_quantized(data);
    }
    let frame_id = u64::from_le_bytes(data[4..12].try_into().ok()?);
    let timestamp_ms = u64::from_le_bytes(data[12..20].try_into().ok()?);
    let np = u32::from_le_bytes(data[20..24].try_into().ok()?) as usize;
    let pos_end = 24 + np * 12;
    if data.len() < pos_end + 4 {
        return None;
    }
    let mut positions = Vec::with_capacity(np);
    for i in 0..np {
        let off = 24 + i * 12;
        let x = f32::from_le_bytes(data[off..off + 4].try_into().ok()?);
        let y = f32::from_le_bytes(data[off + 4..off + 8].try_into().ok()?);
        let z = f32::from_le_bytes(data[off + 8..off + 12].try_into().ok()?);
        positions.push([x, y, z]);
    }
    let nn = u32::from_le_bytes(data[pos_end..pos_end + 4].try_into().ok()?) as usize;
    let norm_end = pos_end + 4 + nn * 12;
    if data.len() < norm_end {
        return None;
    }
    let mut normals = Vec::with_capacity(nn);
    for i in 0..nn {
        let off = pos_end + 4 + i * 12;
        let x = f32::from_le_bytes(data[off..off + 4].try_into().ok()?);
        let y = f32::from_le_bytes(data[off + 4..off + 8].try_into().ok()?);
        let z = f32::from_le_bytes(data[off + 8..off + 12].try_into().ok()?);
        normals.push([x, y, z]);
    }
    Some(StreamFrame {
        frame_id,
        timestamp_ms,
        positions,
        normals,
    })
}

// ---------------------------------------------------------------------------
// Quantized frame encoding
// Layout: magic(4=OXQF) | frame_id(8) | ts(8) | bmin(12) | bmax(12) | n(4) | u16*3*n
// ---------------------------------------------------------------------------

fn encode_frame_quantized(
    frame_id: u64,
    timestamp_ms: u64,
    quantized: &[[u16; 3]],
    bmin: [f32; 3],
    bmax: [f32; 3],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"OXQF");
    out.extend_from_slice(&frame_id.to_le_bytes());
    out.extend_from_slice(&timestamp_ms.to_le_bytes());
    for &v in &bmin {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for &v in &bmax {
        out.extend_from_slice(&v.to_le_bytes());
    }
    let n = quantized.len() as u32;
    out.extend_from_slice(&n.to_le_bytes());
    for q in quantized {
        for &v in q {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

fn decode_frame_quantized(data: &[u8]) -> Option<StreamFrame> {
    // magic(4) + frame_id(8) + ts(8) + bmin(12) + bmax(12) + n(4) = 48 bytes header
    if data.len() < 48 {
        return None;
    }
    let frame_id = u64::from_le_bytes(data[4..12].try_into().ok()?);
    let timestamp_ms = u64::from_le_bytes(data[12..20].try_into().ok()?);
    let bmin = [
        f32::from_le_bytes(data[20..24].try_into().ok()?),
        f32::from_le_bytes(data[24..28].try_into().ok()?),
        f32::from_le_bytes(data[28..32].try_into().ok()?),
    ];
    let bmax = [
        f32::from_le_bytes(data[32..36].try_into().ok()?),
        f32::from_le_bytes(data[36..40].try_into().ok()?),
        f32::from_le_bytes(data[40..44].try_into().ok()?),
    ];
    let n = u32::from_le_bytes(data[44..48].try_into().ok()?) as usize;
    if data.len() < 48 + n * 6 {
        return None;
    }
    let mut quantized = Vec::with_capacity(n);
    for i in 0..n {
        let off = 48 + i * 6;
        let x = u16::from_le_bytes(data[off..off + 2].try_into().ok()?);
        let y = u16::from_le_bytes(data[off + 2..off + 4].try_into().ok()?);
        let z = u16::from_le_bytes(data[off + 4..off + 6].try_into().ok()?);
        quantized.push([x, y, z]);
    }
    let positions = dequantize_positions_16bit(&quantized, bmin, bmax);
    Some(StreamFrame {
        frame_id,
        timestamp_ms,
        positions,
        normals: vec![],
    })
}

// ---------------------------------------------------------------------------
// Delta encoding
// ---------------------------------------------------------------------------

/// Per-vertex position delta: current - base.
pub fn delta_encode_positions(base: &[[f32; 3]], current: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let n = current.len().min(base.len());
    let mut out: Vec<[f32; 3]> = current
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i < n {
                [c[0] - base[i][0], c[1] - base[i][1], c[2] - base[i][2]]
            } else {
                *c
            }
        })
        .collect();
    // If current is shorter than base, stop; if longer, keep extras as-is
    out.truncate(current.len());
    out
}

/// Reconstruct positions from base + delta.
pub fn delta_decode_positions(base: &[[f32; 3]], delta: &[[f32; 3]]) -> Vec<[f32; 3]> {
    delta
        .iter()
        .enumerate()
        .map(|(i, d)| {
            if i < base.len() {
                [base[i][0] + d[0], base[i][1] + d[1], base[i][2] + d[2]]
            } else {
                *d
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 16-bit quantization
// ---------------------------------------------------------------------------

fn positions_bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::INFINITY; 3];
    let mut mx = [f32::NEG_INFINITY; 3];
    for p in positions {
        for d in 0..3 {
            if p[d] < mn[d] {
                mn[d] = p[d];
            }
            if p[d] > mx[d] {
                mx[d] = p[d];
            }
        }
    }
    if mn[0].is_infinite() {
        ([0.0; 3], [1.0; 3])
    } else {
        (mn, mx)
    }
}

/// Quantize positions to 16-bit unsigned integers within given bounds.
pub fn quantize_positions_16bit(
    positions: &[[f32; 3]],
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
) -> Vec<[u16; 3]> {
    positions
        .iter()
        .map(|p| {
            let mut q = [0u16; 3];
            for d in 0..3 {
                let range = (bounds_max[d] - bounds_min[d]).max(1e-9);
                let t = ((p[d] - bounds_min[d]) / range).clamp(0.0, 1.0);
                q[d] = (t * 65535.0) as u16;
            }
            q
        })
        .collect()
}

/// Dequantize 16-bit positions back to f32.
pub fn dequantize_positions_16bit(
    quantized: &[[u16; 3]],
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
) -> Vec<[f32; 3]> {
    quantized
        .iter()
        .map(|q| {
            let mut p = [0.0f32; 3];
            for d in 0..3 {
                let range = bounds_max[d] - bounds_min[d];
                p[d] = bounds_min[d] + (q[d] as f32 / 65535.0) * range;
            }
            p
        })
        .collect()
}

// Re-export the public types under the names the lib.rs re-export expects
pub use RtStreamFormat as StreamFormat;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(compression: &str) -> StreamConfig {
        StreamConfig {
            target_fps: 30.0,
            compression: StreamCompression {
                kind: compression.into(),
                level: 1,
            },
            format: RtStreamFormat {
                kind: "binary".into(),
            },
        }
    }

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ]
    }

    fn sample_normals() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 1.0]; 4]
    }

    #[test]
    fn test_push_frame_increases_count() {
        let mut session = StreamSession::new(make_config("none"));
        assert_eq!(session.frame_count(), 0);
        session.push_frame(sample_positions(), sample_normals());
        assert_eq!(session.frame_count(), 1);
        session.push_frame(sample_positions(), sample_normals());
        assert_eq!(session.frame_count(), 2);
    }

    #[test]
    fn test_encode_decode_round_trip_none() {
        let mut session = StreamSession::new(make_config("none"));
        session.push_frame(sample_positions(), sample_normals());
        let encoded = session.encode_frame(0);
        let decoded = StreamSession::decode_frame(&encoded).expect("decode failed");
        assert_eq!(decoded.positions.len(), 4);
        for (orig, dec) in sample_positions().iter().zip(decoded.positions.iter()) {
            for d in 0..3 {
                assert!((orig[d] - dec[d]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_encode_decode_round_trip_quantized() {
        let mut session = StreamSession::new(make_config("quantized"));
        session.push_frame(sample_positions(), sample_normals());
        let encoded = session.encode_frame(0);
        let decoded = StreamSession::decode_frame(&encoded).expect("decode failed");
        assert_eq!(decoded.positions.len(), 4);
        // quantization error should be < 0.001 for range [0,1]
        for (orig, dec) in sample_positions().iter().zip(decoded.positions.iter()) {
            for d in 0..3 {
                assert!(
                    (orig[d] - dec[d]).abs() < 0.001,
                    "quantization error too large"
                );
            }
        }
    }

    #[test]
    fn test_encode_out_of_range_returns_empty() {
        let session = StreamSession::new(make_config("none"));
        assert!(session.encode_frame(0).is_empty());
    }

    #[test]
    fn test_frame_count_zero_initially() {
        let session = StreamSession::new(make_config("none"));
        assert_eq!(session.frame_count(), 0);
    }

    #[test]
    fn test_avg_frame_size_zero_no_frames() {
        let session = StreamSession::new(make_config("none"));
        assert_eq!(session.avg_frame_size(), 0.0);
    }

    #[test]
    fn test_avg_frame_size_positive() {
        let mut session = StreamSession::new(make_config("none"));
        session.push_frame(sample_positions(), sample_normals());
        assert!(session.avg_frame_size() > 0.0);
    }

    #[test]
    fn test_delta_encode_zero_for_identical() {
        let pos = sample_positions();
        let deltas = delta_encode_positions(&pos, &pos);
        for d in &deltas {
            for &v in d {
                assert!(v.abs() < 1e-9, "expected zero delta");
            }
        }
    }

    #[test]
    fn test_delta_encode_decode_recovers_original() {
        let base = sample_positions();
        let current: Vec<[f32; 3]> = base
            .iter()
            .map(|p| [p[0] + 0.1, p[1] - 0.2, p[2] + 0.05])
            .collect();
        let deltas = delta_encode_positions(&base, &current);
        let recovered = delta_decode_positions(&base, &deltas);
        for (orig, rec) in current.iter().zip(recovered.iter()) {
            for d in 0..3 {
                assert!((orig[d] - rec[d]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_delta_empty_base() {
        let current = sample_positions();
        let deltas = delta_encode_positions(&[], &current);
        assert_eq!(deltas.len(), current.len());
    }

    #[test]
    fn test_quantize_stays_in_u16_range() {
        let pos = vec![[-1.0f32, -1.0, -1.0], [1.0, 1.0, 1.0], [0.0, 0.5, -0.5]];
        let bmin = [-1.0f32; 3];
        let bmax = [1.0f32; 3];
        let q = quantize_positions_16bit(&pos, bmin, bmax);
        // All quantized values must be valid u16; min is 0, max is 65535.
        // Check that the extreme input maps to 0 and 65535 correctly.
        assert_eq!(q[0], [0u16, 0, 0]); // [-1,-1,-1] maps to 0
        assert_eq!(q[1], [65535u16, 65535, 65535]); // [1,1,1] maps to 65535
                                                    // Middle value should be in a reasonable midrange
        assert!(q[2][1] > 0 && q[2][1] < 65535);
    }

    #[test]
    fn test_quantize_dequantize_round_trip() {
        let pos = vec![[0.0f32, 0.5, 1.0], [0.25, 0.75, 0.1]];
        let bmin = [0.0f32; 3];
        let bmax = [1.0f32; 3];
        let q = quantize_positions_16bit(&pos, bmin, bmax);
        let dq = dequantize_positions_16bit(&q, bmin, bmax);
        for (orig, rec) in pos.iter().zip(dq.iter()) {
            for d in 0..3 {
                assert!((orig[d] - rec[d]).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn test_encode_delta_decode_frame_id() {
        let mut session = StreamSession::new(make_config("none"));
        session.push_frame(sample_positions(), sample_normals());
        session.push_frame(sample_positions(), sample_normals());
        let enc1 = session.encode_frame(1);
        let dec = StreamSession::decode_frame(&enc1).unwrap();
        assert_eq!(dec.frame_id, 1);
    }

    #[test]
    fn test_session_base_frame_set_after_first_push() {
        let mut session = StreamSession::new(make_config("delta"));
        assert!(session.base_frame.is_none());
        session.push_frame(sample_positions(), sample_normals());
        assert!(session.base_frame.is_some());
    }
}
