// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! PC2 (Point Cache 2) binary point cache format — used by Blender.
//!
//! Binary layout (little-endian):
//!   - Magic: b"POINTCACHE2\0"  (12 bytes)
//!   - Version: i32 = 1
//!   - Point count: i32
//!   - Start time: f32
//!   - Sample rate: f32
//!   - Sample count: i32
//!   - Data: for each frame, (point_count × 3) f32 values (xyz)

use std::path::Path;

/// Magic bytes for PC2 format (12 bytes, null-terminated).
pub const PC2_MAGIC: &[u8; 12] = b"POINTCACHE2\0";

/// Header metadata for a PC2 point cache file.
#[allow(dead_code)]
pub struct Pc2Header {
    pub point_count: u32,
    pub start_time: f32,
    pub sample_rate: f32,
    pub sample_count: u32,
}

/// Complete PC2 point cache (header + all frame data).
#[allow(dead_code)]
pub struct Pc2Cache {
    pub header: Pc2Header,
    /// One entry per frame; each entry contains `point_count` XYZ positions.
    pub frames: Vec<Vec<[f32; 3]>>,
}

impl Pc2Cache {
    /// Create a new empty cache with the given metadata.
    #[allow(dead_code)]
    pub fn new(point_count: u32, start_time: f32, sample_rate: f32) -> Self {
        Self {
            header: Pc2Header {
                point_count,
                start_time,
                sample_rate,
                sample_count: 0,
            },
            frames: Vec::new(),
        }
    }

    /// Append a frame of positions.  Panics if `positions.len() != point_count`.
    #[allow(dead_code)]
    pub fn add_frame(&mut self, positions: Vec<[f32; 3]>) {
        assert_eq!(
            positions.len(),
            self.header.point_count as usize,
            "add_frame: expected {} points, got {}",
            self.header.point_count,
            positions.len()
        );
        self.frames.push(positions);
        self.header.sample_count += 1;
    }
}

// ── serialisation ─────────────────────────────────────────────────────────────

/// Serialise a [`Pc2Cache`] to a `Vec<u8>` (little-endian binary).
#[allow(dead_code)]
pub fn write_pc2(cache: &Pc2Cache) -> Vec<u8> {
    let h = &cache.header;
    let data_bytes = (h.point_count as usize) * 3 * 4 * cache.frames.len();
    let mut out = Vec::with_capacity(12 + 4 + 4 + 4 + 4 + 4 + data_bytes);

    // Magic
    out.extend_from_slice(PC2_MAGIC);
    // Version = 1 (i32 LE)
    out.extend_from_slice(&1_i32.to_le_bytes());
    // Header fields
    out.extend_from_slice(&(h.point_count as i32).to_le_bytes());
    out.extend_from_slice(&h.start_time.to_le_bytes());
    out.extend_from_slice(&h.sample_rate.to_le_bytes());
    out.extend_from_slice(&(h.sample_count as i32).to_le_bytes());

    // Frame data
    for frame in &cache.frames {
        for pos in frame {
            out.extend_from_slice(&pos[0].to_le_bytes());
            out.extend_from_slice(&pos[1].to_le_bytes());
            out.extend_from_slice(&pos[2].to_le_bytes());
        }
    }
    out
}

/// Deserialise a PC2 binary blob into a [`Pc2Cache`].
#[allow(dead_code)]
pub fn read_pc2(data: &[u8]) -> anyhow::Result<Pc2Cache> {
    use anyhow::bail;

    if data.len() < 28 {
        bail!("PC2 data too short: {} bytes", data.len());
    }

    // Magic
    if &data[..12] != PC2_MAGIC {
        bail!("PC2 magic mismatch");
    }

    let version = i32::from_le_bytes(
        data[12..16]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    );
    if version != 1 {
        bail!("unsupported PC2 version: {}", version);
    }

    let point_count = i32::from_le_bytes(
        data[16..20]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    ) as u32;
    let start_time = f32::from_le_bytes(
        data[20..24]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    );
    let sample_rate = f32::from_le_bytes(
        data[24..28]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    );
    let sample_count = i32::from_le_bytes(
        data[28..32]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    ) as u32;

    let frame_stride = (point_count as usize) * 3 * 4;
    let expected_total = 32 + frame_stride * (sample_count as usize);
    if data.len() < expected_total {
        bail!(
            "PC2 data truncated: need {} bytes, have {}",
            expected_total,
            data.len()
        );
    }

    let mut frames = Vec::with_capacity(sample_count as usize);
    let mut offset = 32_usize;
    for _ in 0..sample_count {
        let mut frame = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            let x = f32::from_le_bytes(
                data[offset..offset + 4]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
            );
            let y = f32::from_le_bytes(
                data[offset + 4..offset + 8]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
            );
            let z = f32::from_le_bytes(
                data[offset + 8..offset + 12]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
            );
            frame.push([x, y, z]);
            offset += 12;
        }
        frames.push(frame);
    }

    Ok(Pc2Cache {
        header: Pc2Header {
            point_count,
            start_time,
            sample_rate,
            sample_count,
        },
        frames,
    })
}

/// Write a [`Pc2Cache`] to a file on disk.
#[allow(dead_code)]
pub fn export_pc2(cache: &Pc2Cache, path: &Path) -> anyhow::Result<()> {
    let bytes = write_pc2(cache);
    std::fs::write(path, &bytes)
        .map_err(|e| anyhow::anyhow!("writing PC2 to {}: {}", path.display(), e))
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Convert a slice of position frames into a [`Pc2Cache`].
///
/// All frames must have the same number of points; the first frame's length
/// is used as `point_count`.  Panics if `frames` is empty.
#[allow(dead_code)]
pub fn mesh_sequence_to_pc2(
    frames: &[Vec<[f32; 3]>],
    start_time: f32,
    sample_rate: f32,
) -> Pc2Cache {
    assert!(
        !frames.is_empty(),
        "mesh_sequence_to_pc2: frames must not be empty"
    );
    let point_count = frames[0].len() as u32;
    let mut cache = Pc2Cache::new(point_count, start_time, sample_rate);
    for frame in frames {
        cache.add_frame(frame.clone());
    }
    cache
}

/// Return a human-readable summary string for a [`Pc2Cache`].
#[allow(dead_code)]
pub fn pc2_stats(cache: &Pc2Cache) -> String {
    let h = &cache.header;
    format!(
        "PC2 | points={} | frames={} | start={:.3} | rate={:.2} fps",
        h.point_count, h.sample_count, h.start_time, h.sample_rate
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_point_cache() -> Pc2Cache {
        let mut c = Pc2Cache::new(2, 0.0, 24.0);
        c.add_frame(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        c.add_frame(vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]);
        c
    }

    // 1. round-trip: write then read gives back identical data
    #[test]
    fn roundtrip_basic() {
        let cache = two_point_cache();
        let bytes = write_pc2(&cache);
        let back = read_pc2(&bytes).expect("should succeed");
        assert_eq!(back.header.point_count, 2);
        assert_eq!(back.header.sample_count, 2);
        assert_eq!(back.frames[0], vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        assert_eq!(back.frames[1], vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]);
    }

    // 2. round-trip preserves start_time and sample_rate
    #[test]
    fn roundtrip_metadata() {
        let mut c = Pc2Cache::new(1, 1.5, 30.0);
        c.add_frame(vec![[0.0, 0.0, 0.0]]);
        let back = read_pc2(&write_pc2(&c)).expect("should succeed");
        assert!((back.header.start_time - 1.5).abs() < 1e-6);
        assert!((back.header.sample_rate - 30.0).abs() < 1e-6);
    }

    // 3. magic bytes are correct
    #[test]
    fn magic_bytes() {
        let cache = two_point_cache();
        let bytes = write_pc2(&cache);
        assert_eq!(&bytes[..12], PC2_MAGIC);
    }

    // 4. version field is 1
    #[test]
    fn version_field_is_one() {
        let cache = two_point_cache();
        let bytes = write_pc2(&cache);
        let ver = i32::from_le_bytes(bytes[12..16].try_into().expect("should succeed"));
        assert_eq!(ver, 1);
    }

    // 5. wrong point count panics
    #[test]
    #[should_panic]
    fn add_frame_wrong_count_panics() {
        let mut c = Pc2Cache::new(3, 0.0, 24.0);
        c.add_frame(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]); // only 2 points
    }

    // 6. empty frames (zero frames added)
    #[test]
    fn empty_frames() {
        let c = Pc2Cache::new(5, 0.0, 24.0);
        let bytes = write_pc2(&c);
        let back = read_pc2(&bytes).expect("should succeed");
        assert_eq!(back.header.sample_count, 0);
        assert!(back.frames.is_empty());
    }

    // 7. pc2_stats contains key fields
    #[test]
    fn pc2_stats_contains_points_and_frames() {
        let cache = two_point_cache();
        let s = pc2_stats(&cache);
        assert!(s.contains("points=2"));
        assert!(s.contains("frames=2"));
    }

    // 8. pc2_stats contains fps info
    #[test]
    fn pc2_stats_contains_rate() {
        let cache = two_point_cache();
        let s = pc2_stats(&cache);
        assert!(s.contains("24"));
    }

    // 9. mesh_sequence_to_pc2 produces correct frame count
    #[test]
    fn mesh_sequence_frame_count() {
        let frames: Vec<Vec<[f32; 3]>> = (0..5)
            .map(|i| vec![[i as f32, 0.0, 0.0], [0.0, i as f32, 0.0]])
            .collect();
        let cache = mesh_sequence_to_pc2(&frames, 0.0, 24.0);
        assert_eq!(cache.header.sample_count, 5);
        assert_eq!(cache.header.point_count, 2);
    }

    // 10. mesh_sequence_to_pc2 first frame positions are preserved
    #[test]
    fn mesh_sequence_positions_preserved() {
        let frames = vec![vec![[1.0_f32, 2.0, 3.0]], vec![[4.0_f32, 5.0, 6.0]]];
        let cache = mesh_sequence_to_pc2(&frames, 0.0, 24.0);
        let back = read_pc2(&write_pc2(&cache)).expect("should succeed");
        assert_eq!(back.frames[0][0], [1.0, 2.0, 3.0]);
        assert_eq!(back.frames[1][0], [4.0, 5.0, 6.0]);
    }

    // 11. read_pc2 errors on truncated data
    #[test]
    fn read_pc2_truncated_error() {
        let cache = two_point_cache();
        let bytes = write_pc2(&cache);
        let result = read_pc2(&bytes[..20]);
        assert!(result.is_err());
    }

    // 12. read_pc2 errors on bad magic
    #[test]
    fn read_pc2_bad_magic() {
        let cache = two_point_cache();
        let mut bytes = write_pc2(&cache);
        bytes[0] = 0xFF;
        assert!(read_pc2(&bytes).is_err());
    }
}
