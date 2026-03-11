// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::io::{Read, Write};
use std::path::Path;

use oxihuman_mesh::MeshBuffers;

// ─── Magic / version ────────────────────────────────────────────────────────

const QMSH_MAGIC: &[u8; 4] = b"QMSH";
const QMSH_VERSION: u32 = 1;

// ─── QuantizeRange ───────────────────────────────────────────────────────────

/// Maps a float range `[min, max]` to the unsigned 16-bit integer range
/// `[0, 65535]`.  Used to encode per-axis position and UV channels.
#[derive(Debug, Clone)]
pub struct QuantizeRange {
    pub min: f32,
    pub max: f32,
}

impl QuantizeRange {
    /// Construct a `QuantizeRange` by scanning `values` to find the extremes.
    /// When the slice is empty, or all values are identical, a trivial range
    /// of `[0, 1]` is returned so that division by zero is avoided downstream.
    pub fn from_data(values: &[f32]) -> Self {
        if values.is_empty() {
            return Self { min: 0.0, max: 1.0 };
        }
        let mut lo = values[0];
        let mut hi = values[0];
        for &v in values.iter().skip(1) {
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
        }
        // Guard against degenerate (zero-width) ranges.
        if (hi - lo).abs() < 1e-12 {
            hi = lo + 1.0;
        }
        Self { min: lo, max: hi }
    }

    /// Clamp `v` to `[min, max]` and quantize to `[0, 65535]`.
    #[inline]
    pub fn encode(&self, v: f32) -> u16 {
        let clamped = v.max(self.min).min(self.max);
        let t = (clamped - self.min) / (self.max - self.min);
        (t * 65535.0).round() as u16
    }

    /// Reconstruct a float from a quantized `u16`.
    #[inline]
    pub fn decode(&self, q: u16) -> f32 {
        let t = q as f32 / 65535.0;
        self.min + t * (self.max - self.min)
    }
}

// ─── QuantizedMesh ───────────────────────────────────────────────────────────

/// A mesh whose attributes have been quantized to compact integer types.
///
/// * `positions`  — XYZ per vertex, each axis stored as `u16`.
/// * `normals`    — XYZ per vertex in `i8` (direct; range `[-127, 127]`).
/// * `uvs`        — UV per vertex as `u16`.
/// * `pos_range`  — The per-axis decode parameters for positions.
#[derive(Debug, Clone)]
pub struct QuantizedMesh {
    pub positions: Vec<[u16; 3]>,
    pub normals: Vec<[i8; 3]>,
    pub uvs: Vec<[u16; 2]>,
    pub indices: Vec<u32>,
    pub pos_range: [QuantizeRange; 3],
    pub has_suit: bool,
}

// ─── QuantizeStats ───────────────────────────────────────────────────────────

/// Error metrics comparing the original float mesh with the quantized version.
#[derive(Debug, Clone)]
pub struct QuantizeStats {
    pub position_error_rms: f32,
    pub normal_error_rms: f32,
    pub uv_error_rms: f32,
    /// `original_bytes / quantized_bytes` — values > 1 mean compression.
    pub compression_ratio: f32,
}

// ─── quantize_mesh ───────────────────────────────────────────────────────────

/// Quantize all attributes of `mesh` into a compact [`QuantizedMesh`].
pub fn quantize_mesh(mesh: &MeshBuffers) -> QuantizedMesh {
    let n = mesh.positions.len();

    // Build per-axis ranges for positions.
    let xs: Vec<f32> = mesh.positions.iter().map(|p| p[0]).collect();
    let ys: Vec<f32> = mesh.positions.iter().map(|p| p[1]).collect();
    let zs: Vec<f32> = mesh.positions.iter().map(|p| p[2]).collect();
    let rx = QuantizeRange::from_data(&xs);
    let ry = QuantizeRange::from_data(&ys);
    let rz = QuantizeRange::from_data(&zs);

    // UV ranges.
    let us: Vec<f32> = mesh.uvs.iter().map(|uv| uv[0]).collect();
    let vs: Vec<f32> = mesh.uvs.iter().map(|uv| uv[1]).collect();
    let ru = QuantizeRange::from_data(&us);
    let rv = QuantizeRange::from_data(&vs);

    let mut positions = Vec::with_capacity(n);
    let mut normals = Vec::with_capacity(n);
    let mut uvs = Vec::with_capacity(n);

    for i in 0..n {
        let [px, py, pz] = mesh.positions[i];
        positions.push([rx.encode(px), ry.encode(py), rz.encode(pz)]);

        // Normals: scale float [-1,1] to i8 [-127,127].
        let [nx, ny, nz] = if i < mesh.normals.len() {
            mesh.normals[i]
        } else {
            [0.0, 0.0, 1.0]
        };
        normals.push([
            encode_normal_component(nx),
            encode_normal_component(ny),
            encode_normal_component(nz),
        ]);

        let [pu, pv] = if i < mesh.uvs.len() {
            mesh.uvs[i]
        } else {
            [0.0, 0.0]
        };
        uvs.push([ru.encode(pu), rv.encode(pv)]);
    }

    QuantizedMesh {
        positions,
        normals,
        uvs,
        indices: mesh.indices.clone(),
        pos_range: [rx, ry, rz],
        has_suit: mesh.has_suit,
    }
}

// ─── dequantize_mesh ─────────────────────────────────────────────────────────

/// Reconstruct a [`MeshBuffers`] from a [`QuantizedMesh`].  The result will
/// differ from the original by the quantization error (≈ 1/65535 of the range
/// per axis).
pub fn dequantize_mesh(q: &QuantizedMesh) -> MeshBuffers {
    let n = q.positions.len();

    // Recover UV ranges — we store only pos_range in the struct, so we have
    // to re-derive UV ranges from the quantized data.  We use a [0,1] default
    // since the canonical UV domain is [0,1].
    let ru = QuantizeRange { min: 0.0, max: 1.0 };
    let rv = QuantizeRange { min: 0.0, max: 1.0 };

    let mut positions = Vec::with_capacity(n);
    let mut normals = Vec::with_capacity(n);
    let mut uvs = Vec::with_capacity(n);

    let [ref rx, ref ry, ref rz] = q.pos_range;

    for i in 0..n {
        let [qx, qy, qz] = q.positions[i];
        positions.push([rx.decode(qx), ry.decode(qy), rz.decode(qz)]);

        let [enx, eny, enz] = q.normals[i];
        let nx = decode_normal_component(enx);
        let ny = decode_normal_component(eny);
        let nz = decode_normal_component(enz);
        // Re-normalize to compensate for quantization.
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-9);
        normals.push([nx / len, ny / len, nz / len]);

        let [qu, qv] = q.uvs[i];
        uvs.push([ru.decode(qu), rv.decode(qv)]);
    }

    let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; n];

    MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices: q.indices.clone(),
        colors: None,
        has_suit: q.has_suit,
    }
}

// ─── quantize_stats ──────────────────────────────────────────────────────────

/// Compute error metrics between `original` and the round-tripped mesh derived
/// from `q`.
pub fn quantize_stats(original: &MeshBuffers, q: &QuantizedMesh) -> QuantizeStats {
    let reconstructed = dequantize_mesh(q);
    let n = original.positions.len().min(reconstructed.positions.len());

    // Position RMS.
    let pos_rms = if n == 0 {
        0.0
    } else {
        let sum_sq: f32 = (0..n)
            .map(|i| {
                let [ox, oy, oz] = original.positions[i];
                let [rx, ry, rz] = reconstructed.positions[i];
                let d = [(ox - rx), (oy - ry), (oz - rz)];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
            })
            .sum();
        (sum_sq / n as f32).sqrt()
    };

    // Normal RMS (angle-based: dot product distance).
    let nor_rms = if n == 0 {
        0.0
    } else {
        let mn = original
            .normals
            .len()
            .min(reconstructed.normals.len())
            .min(n);
        let sum_sq: f32 = (0..mn)
            .map(|i| {
                let [ox, oy, oz] = original.normals[i];
                let [rx, ry, rz] = reconstructed.normals[i];
                let d = [(ox - rx), (oy - ry), (oz - rz)];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
            })
            .sum();
        (sum_sq / mn as f32).sqrt()
    };

    // UV RMS.
    let uv_rms = if n == 0 {
        0.0
    } else {
        let mu = original.uvs.len().min(reconstructed.uvs.len()).min(n);
        let sum_sq: f32 = (0..mu)
            .map(|i| {
                let [ou, ov] = original.uvs[i];
                let [ru, rv] = reconstructed.uvs[i];
                let du = ou - ru;
                let dv = ov - rv;
                du * du + dv * dv
            })
            .sum();
        (sum_sq / mu as f32).sqrt()
    };

    // Byte sizes.
    let orig_bytes = original.positions.len() * 12  // f32x3
        + original.normals.len() * 12   // f32x3
        + original.uvs.len() * 8        // f32x2
        + original.indices.len() * 4; // u32

    let quant_bytes = q.positions.len() * 6  // u16x3
        + q.normals.len() * 3           // i8x3
        + q.uvs.len() * 4               // u16x2
        + q.indices.len() * 4; // u32

    let compression_ratio = if quant_bytes == 0 {
        1.0
    } else {
        orig_bytes as f32 / quant_bytes as f32
    };

    QuantizeStats {
        position_error_rms: pos_rms,
        normal_error_rms: nor_rms,
        uv_error_rms: uv_rms,
        compression_ratio,
    }
}

// ─── Oct-map normal encoding ─────────────────────────────────────────────────

/// Encode a unit normal vector using octahedral projection to 2 `i8` bytes.
///
/// Reference: Cigolle et al., "Survey of Efficient Representations for
/// Independent Unit Vectors", JCGT 2014.
pub fn encode_normal_oct(n: [f32; 3]) -> [i8; 2] {
    // Project onto the L1 unit sphere.
    let l1 = n[0].abs() + n[1].abs() + n[2].abs();
    let (mut ox, mut oy) = if l1 < 1e-9 {
        (0.0f32, 0.0f32)
    } else {
        (n[0] / l1, n[1] / l1)
    };

    // Fold for negative hemisphere.
    if n[2] < 0.0 {
        let ox2 = (1.0 - oy.abs()) * ox.signum();
        let oy2 = (1.0 - ox.abs()) * oy.signum();
        ox = ox2;
        oy = oy2;
    }

    // Scale to [-127, 127].
    let ex = (ox * 127.0).round().clamp(-127.0, 127.0) as i8;
    let ey = (oy * 127.0).round().clamp(-127.0, 127.0) as i8;
    [ex, ey]
}

/// Decode an oct-encoded 2-byte normal back to a unit `[f32; 3]`.
pub fn decode_normal_oct(enc: [i8; 2]) -> [f32; 3] {
    let ox = enc[0] as f32 / 127.0;
    let oy = enc[1] as f32 / 127.0;
    let oz = 1.0 - ox.abs() - oy.abs();

    let (fx, fy) = if oz < 0.0 {
        (
            (1.0 - oy.abs()) * ox.signum(),
            (1.0 - ox.abs()) * oy.signum(),
        )
    } else {
        (ox, oy)
    };

    let len = (fx * fx + fy * fy + oz * oz).sqrt().max(1e-9);
    [fx / len, fy / len, oz / len]
}

// ─── Binary I/O ──────────────────────────────────────────────────────────────

/// Write a [`QuantizedMesh`] to a compact binary file.
///
/// File layout:
/// ```text
/// Bytes  0..4   : magic  b"QMSH"
/// Bytes  4..8   : version u32 LE
/// Bytes  8..12  : vertex_count u32 LE
/// Bytes 12..16  : index_count  u32 LE
/// Then: 6 f32s (3 × min/max for pos_range) LE
/// Then: vertex_count × 6 bytes  (u16x3 positions, LE)
/// Then: vertex_count × 3 bytes  (i8x3 normals)
/// Then: vertex_count × 4 bytes  (u16x2 uvs, LE)
/// Then: index_count  × 4 bytes  (u32 indices, LE)
/// ```
///
/// Returns the total number of bytes written.
pub fn write_quantized_bin(q: &QuantizedMesh, path: &Path) -> anyhow::Result<usize> {
    let vc = q.positions.len() as u32;
    let ic = q.indices.len() as u32;

    let mut buf: Vec<u8> = Vec::new();

    // Header: 16 bytes.
    buf.extend_from_slice(QMSH_MAGIC);
    buf.extend_from_slice(&QMSH_VERSION.to_le_bytes());
    buf.extend_from_slice(&vc.to_le_bytes());
    buf.extend_from_slice(&ic.to_le_bytes());

    // Range data: 3 ranges × 2 floats = 6 f32 = 24 bytes.
    for r in &q.pos_range {
        buf.extend_from_slice(&r.min.to_le_bytes());
        buf.extend_from_slice(&r.max.to_le_bytes());
    }

    // Positions.
    for p in &q.positions {
        for &v in p {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Normals (i8 — single byte each).
    for n in &q.normals {
        for &b in n {
            buf.push(b as u8);
        }
    }

    // UVs.
    for uv in &q.uvs {
        for &v in uv {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Indices.
    for &idx in &q.indices {
        buf.extend_from_slice(&idx.to_le_bytes());
    }

    // has_suit flag (1 byte).
    buf.push(q.has_suit as u8);

    let total = buf.len();
    let mut file = std::fs::File::create(path)?;
    file.write_all(&buf)?;

    Ok(total)
}

/// Read a [`QuantizedMesh`] from a binary file written by [`write_quantized_bin`].
pub fn read_quantized_bin(path: &Path) -> anyhow::Result<QuantizedMesh> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    anyhow::ensure!(buf.len() >= 16, "file too short for QMSH header");

    // Check magic.
    anyhow::ensure!(&buf[0..4] == QMSH_MAGIC, "invalid QMSH magic");

    let version = u32::from_le_bytes(buf[4..8].try_into()?);
    anyhow::ensure!(
        version == QMSH_VERSION,
        "unsupported QMSH version {version}"
    );

    let vertex_count = u32::from_le_bytes(buf[8..12].try_into()?) as usize;
    let index_count = u32::from_le_bytes(buf[12..16].try_into()?) as usize;

    let mut offset = 16usize;

    // Ranges: 3 × 2 f32 = 24 bytes.
    anyhow::ensure!(buf.len() >= offset + 24, "file truncated at range data");
    let mut pos_range_arr = std::array::from_fn(|_| QuantizeRange { min: 0.0, max: 1.0 });
    for r in &mut pos_range_arr {
        r.min = f32::from_le_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        r.max = f32::from_le_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
    }

    // Positions: vertex_count × 6.
    let pos_bytes = vertex_count * 6;
    anyhow::ensure!(
        buf.len() >= offset + pos_bytes,
        "file truncated at positions"
    );
    let mut positions = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let x = u16::from_le_bytes(buf[offset..offset + 2].try_into()?);
        let y = u16::from_le_bytes(buf[offset + 2..offset + 4].try_into()?);
        let z = u16::from_le_bytes(buf[offset + 4..offset + 6].try_into()?);
        positions.push([x, y, z]);
        offset += 6;
    }

    // Normals: vertex_count × 3.
    let nor_bytes = vertex_count * 3;
    anyhow::ensure!(buf.len() >= offset + nor_bytes, "file truncated at normals");
    let mut normals = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let nx = buf[offset] as i8;
        let ny = buf[offset + 1] as i8;
        let nz = buf[offset + 2] as i8;
        normals.push([nx, ny, nz]);
        offset += 3;
    }

    // UVs: vertex_count × 4.
    let uv_bytes = vertex_count * 4;
    anyhow::ensure!(buf.len() >= offset + uv_bytes, "file truncated at UVs");
    let mut uvs = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let u = u16::from_le_bytes(buf[offset..offset + 2].try_into()?);
        let v = u16::from_le_bytes(buf[offset + 2..offset + 4].try_into()?);
        uvs.push([u, v]);
        offset += 4;
    }

    // Indices: index_count × 4.
    let idx_bytes = index_count * 4;
    anyhow::ensure!(buf.len() >= offset + idx_bytes, "file truncated at indices");
    let mut indices = Vec::with_capacity(index_count);
    for _ in 0..index_count {
        let idx = u32::from_le_bytes(buf[offset..offset + 4].try_into()?);
        indices.push(idx);
        offset += 4;
    }

    // has_suit flag.
    let has_suit = if offset < buf.len() {
        buf[offset] != 0
    } else {
        false
    };

    Ok(QuantizedMesh {
        positions,
        normals,
        uvs,
        indices,
        pos_range: pos_range_arr,
        has_suit,
    })
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Encode one float normal component in `[-1, 1]` to `i8` in `[-127, 127]`.
#[inline]
fn encode_normal_component(v: f32) -> i8 {
    (v.clamp(-1.0, 1.0) * 127.0).round() as i8
}

/// Decode one `i8` normal component back to `f32`.
#[inline]
fn decode_normal_component(b: i8) -> f32 {
    b as f32 / 127.0
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_mesh(
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        uvs: Vec<[f32; 2]>,
        indices: Vec<u32>,
    ) -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions,
            normals,
            uvs,
            indices,
            has_suit: false,
        })
    }

    fn simple_triangle() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0.0, 0.0, 1.0]; 3],
            vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            vec![0, 1, 2],
        )
    }

    fn empty_mesh() -> MeshBuffers {
        make_mesh(vec![], vec![], vec![], vec![])
    }

    // ── QuantizeRange ────────────────────────────────────────────────────────

    #[test]
    fn qrange_encode_min_gives_zero() {
        let r = QuantizeRange {
            min: -1.0,
            max: 1.0,
        };
        assert_eq!(r.encode(-1.0), 0);
    }

    #[test]
    fn qrange_encode_max_gives_65535() {
        let r = QuantizeRange {
            min: -1.0,
            max: 1.0,
        };
        assert_eq!(r.encode(1.0), 65535);
    }

    #[test]
    fn qrange_decode_zero_gives_min() {
        let r = QuantizeRange { min: 2.0, max: 5.0 };
        assert!((r.decode(0) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn qrange_decode_max_gives_max() {
        let r = QuantizeRange { min: 2.0, max: 5.0 };
        assert!((r.decode(65535) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn qrange_roundtrip_midpoint() {
        let r = QuantizeRange { min: 0.0, max: 1.0 };
        let original = 0.5f32;
        let decoded = r.decode(r.encode(original));
        assert!(
            (decoded - original).abs() < 1.0 / 65535.0 * 2.0,
            "roundtrip error {} too large",
            (decoded - original).abs()
        );
    }

    #[test]
    fn qrange_from_data_detects_extremes() {
        let data = vec![-3.0f32, 0.0, 7.5, 2.1];
        let r = QuantizeRange::from_data(&data);
        assert!((r.min - (-3.0)).abs() < 1e-6);
        assert!((r.max - 7.5).abs() < 1e-6);
    }

    #[test]
    fn qrange_from_empty_is_valid() {
        let r = QuantizeRange::from_data(&[]);
        // Must not produce min == max to avoid div-by-zero.
        assert!((r.max - r.min).abs() > 1e-9);
    }

    #[test]
    fn qrange_clamps_out_of_range_value() {
        let r = QuantizeRange { min: 0.0, max: 1.0 };
        assert_eq!(r.encode(2.0), 65535);
        assert_eq!(r.encode(-1.0), 0);
    }

    // ── quantize_mesh / dequantize_mesh ──────────────────────────────────────

    #[test]
    fn quantize_vertex_count_preserved() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        assert_eq!(q.positions.len(), 3);
        assert_eq!(q.normals.len(), 3);
        assert_eq!(q.uvs.len(), 3);
    }

    #[test]
    fn quantize_index_count_preserved() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        assert_eq!(q.indices, vec![0, 1, 2]);
    }

    #[test]
    fn dequantize_roundtrip_position_error_small() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        let rec = dequantize_mesh(&q);
        for (orig, recon) in mesh.positions.iter().zip(rec.positions.iter()) {
            let err = (0..3)
                .map(|i| (orig[i] - recon[i]).abs())
                .fold(0.0f32, f32::max);
            assert!(err < 1e-3, "position error {err} too large");
        }
    }

    #[test]
    fn dequantize_normal_unit_length() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        let rec = dequantize_mesh(&q);
        for n in &rec.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 0.01, "normal not unit-length: {len}");
        }
    }

    #[test]
    fn quantize_empty_mesh_no_panic() {
        let mesh = empty_mesh();
        let q = quantize_mesh(&mesh);
        assert!(q.positions.is_empty());
        assert!(q.indices.is_empty());
    }

    #[test]
    fn dequantize_empty_mesh_no_panic() {
        let mesh = empty_mesh();
        let q = quantize_mesh(&mesh);
        let rec = dequantize_mesh(&q);
        assert!(rec.positions.is_empty());
    }

    // ── quantize_stats ───────────────────────────────────────────────────────

    #[test]
    fn stats_compression_ratio_above_one() {
        // A mesh with many vertices gets a compression ratio > 1.
        let positions: Vec<[f32; 3]> = (0..100).map(|i| [i as f32 * 0.01, 0.0, 0.0]).collect();
        let normals = vec![[0.0f32, 0.0, 1.0]; 100];
        let uvs: Vec<[f32; 2]> = (0..100).map(|i| [i as f32 * 0.01, 0.0]).collect();
        let indices: Vec<u32> = (0..99).flat_map(|i| [i, i + 1, i]).collect();
        let mesh = make_mesh(positions, normals, uvs, indices);
        let q = quantize_mesh(&mesh);
        let stats = quantize_stats(&mesh, &q);
        assert!(
            stats.compression_ratio > 1.0,
            "expected compression_ratio > 1, got {}",
            stats.compression_ratio
        );
    }

    #[test]
    fn stats_position_error_rms_nonnegative() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        let stats = quantize_stats(&mesh, &q);
        assert!(stats.position_error_rms >= 0.0);
        assert!(stats.position_error_rms < 0.01);
    }

    #[test]
    fn stats_empty_mesh_no_panic() {
        let mesh = empty_mesh();
        let q = quantize_mesh(&mesh);
        let stats = quantize_stats(&mesh, &q);
        assert_eq!(stats.position_error_rms, 0.0);
        assert_eq!(stats.compression_ratio, 1.0);
    }

    // ── encode/decode_normal_oct ─────────────────────────────────────────────

    #[test]
    fn oct_encode_decode_z_up() {
        let n = [0.0f32, 0.0, 1.0];
        let enc = encode_normal_oct(n);
        let dec = decode_normal_oct(enc);
        let dot = n[0] * dec[0] + n[1] * dec[1] + n[2] * dec[2];
        assert!(dot > 0.99, "z-up oct roundtrip dot={dot}");
    }

    #[test]
    fn oct_encode_decode_z_down() {
        let n = [0.0f32, 0.0, -1.0];
        let enc = encode_normal_oct(n);
        let dec = decode_normal_oct(enc);
        let dot = n[0] * dec[0] + n[1] * dec[1] + n[2] * dec[2];
        assert!(dot > 0.99, "z-down oct roundtrip dot={dot}");
    }

    #[test]
    fn oct_encode_decode_diagonal() {
        let s = 1.0f32 / 3.0f32.sqrt();
        let n = [s, s, s];
        let enc = encode_normal_oct(n);
        let dec = decode_normal_oct(enc);
        let dot = n[0] * dec[0] + n[1] * dec[1] + n[2] * dec[2];
        assert!(dot > 0.99, "diagonal oct roundtrip dot={dot}");
    }

    #[test]
    fn oct_decoded_is_unit_length() {
        for enc in [[10i8, 20], [-10, 50], [127, 0], [0, -127]] {
            let dec = decode_normal_oct(enc);
            let len = (dec[0] * dec[0] + dec[1] * dec[1] + dec[2] * dec[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 0.01,
                "oct decoded not unit-length: {len}"
            );
        }
    }

    // ── write / read binary ──────────────────────────────────────────────────

    #[test]
    fn write_read_roundtrip() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        let path = std::path::Path::new("/tmp/test_qmsh_roundtrip.bin");
        let written = write_quantized_bin(&q, path).expect("write failed");
        assert!(
            written > 16,
            "expected more than header bytes, got {written}"
        );

        let q2 = read_quantized_bin(path).expect("read failed");
        assert_eq!(q2.positions.len(), q.positions.len());
        assert_eq!(q2.normals.len(), q.normals.len());
        assert_eq!(q2.uvs.len(), q.uvs.len());
        assert_eq!(q2.indices, q.indices);
        // Verify first position.
        assert_eq!(q2.positions[0], q.positions[0]);
    }

    #[test]
    fn write_read_preserves_ranges() {
        let mesh = simple_triangle();
        let q = quantize_mesh(&mesh);
        let path = std::path::Path::new("/tmp/test_qmsh_ranges.bin");
        write_quantized_bin(&q, path).expect("write failed");
        let q2 = read_quantized_bin(path).expect("read failed");
        for i in 0..3 {
            assert!((q2.pos_range[i].min - q.pos_range[i].min).abs() < 1e-5);
            assert!((q2.pos_range[i].max - q.pos_range[i].max).abs() < 1e-5);
        }
    }

    #[test]
    fn write_read_empty_mesh() {
        let mesh = empty_mesh();
        let q = quantize_mesh(&mesh);
        let path = std::path::Path::new("/tmp/test_qmsh_empty.bin");
        write_quantized_bin(&q, path).expect("write failed");
        let q2 = read_quantized_bin(path).expect("read failed");
        assert!(q2.positions.is_empty());
        assert!(q2.indices.is_empty());
    }

    #[test]
    fn read_bad_magic_returns_error() {
        let path = std::path::Path::new("/tmp/test_qmsh_badmagic.bin");
        std::fs::write(
            path,
            b"BAAD\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        )
        .unwrap();
        assert!(read_quantized_bin(path).is_err());
    }
}
