// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! MDD (Motion Displacement Data) binary point cache — LightWave format.
//!
//! Binary layout (big-endian):
//!   - Frame count:   i32
//!   - Point count:   i32
//!   - Time values:   frame_count × f32   (seconds)
//!   - Data:          frame_count × point_count × 3 × f32  (xyz)

use std::path::Path;

/// Complete MDD point cache.
#[allow(dead_code)]
pub struct MddCache {
    pub point_count: u32,
    /// Timestamp (seconds) for each frame.
    pub times: Vec<f32>,
    /// Frame data: `frames[i]` contains `point_count` XYZ positions.
    pub frames: Vec<Vec<[f32; 3]>>,
}

impl MddCache {
    /// Create an empty cache for the given point count.
    #[allow(dead_code)]
    pub fn new(point_count: u32) -> Self {
        Self {
            point_count,
            times: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// Append a frame.  Panics if `positions.len() != point_count`.
    #[allow(dead_code)]
    pub fn add_frame(&mut self, time: f32, positions: Vec<[f32; 3]>) {
        assert_eq!(
            positions.len(),
            self.point_count as usize,
            "add_frame: expected {} points, got {}",
            self.point_count,
            positions.len()
        );
        self.times.push(time);
        self.frames.push(positions);
    }
}

// ── serialisation ─────────────────────────────────────────────────────────────

/// Serialise an [`MddCache`] to big-endian binary bytes.
#[allow(dead_code)]
pub fn write_mdd(cache: &MddCache) -> Vec<u8> {
    let frame_count = cache.frames.len() as i32;
    let point_count = cache.point_count as i32;
    let time_bytes = (frame_count as usize) * 4;
    let data_bytes = (frame_count as usize) * (point_count as usize) * 3 * 4;
    let mut out = Vec::with_capacity(8 + time_bytes + data_bytes);

    out.extend_from_slice(&frame_count.to_be_bytes());
    out.extend_from_slice(&point_count.to_be_bytes());

    for &t in &cache.times {
        out.extend_from_slice(&t.to_be_bytes());
    }

    for frame in &cache.frames {
        for pos in frame {
            out.extend_from_slice(&pos[0].to_be_bytes());
            out.extend_from_slice(&pos[1].to_be_bytes());
            out.extend_from_slice(&pos[2].to_be_bytes());
        }
    }
    out
}

/// Deserialise a big-endian MDD binary blob into an [`MddCache`].
#[allow(dead_code)]
pub fn read_mdd(data: &[u8]) -> anyhow::Result<MddCache> {
    use anyhow::bail;

    if data.len() < 8 {
        bail!("MDD data too short: {} bytes", data.len());
    }

    let frame_count = i32::from_be_bytes(
        data[0..4]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    ) as u32;
    let point_count = i32::from_be_bytes(
        data[4..8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
    ) as u32;

    let time_end = 8 + (frame_count as usize) * 4;
    let data_end = time_end + (frame_count as usize) * (point_count as usize) * 12;

    if data.len() < data_end {
        bail!(
            "MDD data truncated: need {} bytes, have {}",
            data_end,
            data.len()
        );
    }

    let mut times = Vec::with_capacity(frame_count as usize);
    let mut offset = 8_usize;
    for _ in 0..frame_count {
        let t = f32::from_be_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
        );
        times.push(t);
        offset += 4;
    }

    let mut frames = Vec::with_capacity(frame_count as usize);
    for _ in 0..frame_count {
        let mut frame = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            let x = f32::from_be_bytes(
                data[offset..offset + 4]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
            );
            let y = f32::from_be_bytes(
                data[offset + 4..offset + 8]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
            );
            let z = f32::from_be_bytes(
                data[offset + 8..offset + 12]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("byte conversion failed"))?,
            );
            frame.push([x, y, z]);
            offset += 12;
        }
        frames.push(frame);
    }

    Ok(MddCache {
        point_count,
        times,
        frames,
    })
}

/// Write an [`MddCache`] to a file on disk.
#[allow(dead_code)]
pub fn export_mdd(cache: &MddCache, path: &Path) -> anyhow::Result<()> {
    let bytes = write_mdd(cache);
    std::fs::write(path, &bytes)
        .map_err(|e| anyhow::anyhow!("writing MDD to {}: {}", path.display(), e))
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Build an [`MddCache`] from position frames, assigning uniform timestamps
/// at intervals of `1.0 / fps` seconds.
#[allow(dead_code)]
pub fn uniform_time_mdd(frames: &[Vec<[f32; 3]>], fps: f32) -> MddCache {
    assert!(
        !frames.is_empty(),
        "uniform_time_mdd: frames must not be empty"
    );
    let point_count = frames[0].len() as u32;
    let mut cache = MddCache::new(point_count);
    for (i, frame) in frames.iter().enumerate() {
        let time = i as f32 / fps;
        cache.add_frame(time, frame.clone());
    }
    cache
}

/// Return the total duration of the cache in seconds (last time value).
/// Returns `0.0` if the cache has no frames.
#[allow(dead_code)]
pub fn mdd_duration(cache: &MddCache) -> f32 {
    cache.times.last().copied().unwrap_or(0.0)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_frame_cache() -> MddCache {
        let mut c = MddCache::new(2);
        c.add_frame(0.0, vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        c.add_frame(1.0 / 24.0, vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]);
        c
    }

    // 1. round-trip preserves positions
    #[test]
    fn roundtrip_positions() {
        let cache = two_frame_cache();
        let back = read_mdd(&write_mdd(&cache)).expect("should succeed");
        assert_eq!(back.frames[0], vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        assert_eq!(back.frames[1], vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]);
    }

    // 2. round-trip preserves metadata
    #[test]
    fn roundtrip_metadata() {
        let cache = two_frame_cache();
        let back = read_mdd(&write_mdd(&cache)).expect("should succeed");
        assert_eq!(back.point_count, 2);
        assert_eq!(back.frames.len(), 2);
    }

    // 3. bytes 0..3 are big-endian frame count
    #[test]
    fn big_endian_frame_count() {
        let cache = two_frame_cache();
        let bytes = write_mdd(&cache);
        let fc = i32::from_be_bytes(bytes[0..4].try_into().expect("should succeed"));
        assert_eq!(fc, 2);
    }

    // 4. bytes 4..7 are big-endian point count
    #[test]
    fn big_endian_point_count() {
        let cache = two_frame_cache();
        let bytes = write_mdd(&cache);
        let pc = i32::from_be_bytes(bytes[4..8].try_into().expect("should succeed"));
        assert_eq!(pc, 2);
    }

    // 5. times are preserved through round-trip
    #[test]
    fn roundtrip_times() {
        let cache = two_frame_cache();
        let back = read_mdd(&write_mdd(&cache)).expect("should succeed");
        assert!((back.times[0] - 0.0).abs() < 1e-6);
        assert!((back.times[1] - 1.0 / 24.0).abs() < 1e-5);
    }

    // 6. uniform_time_mdd generates correct timestamps
    #[test]
    fn uniform_time_timestamps() {
        let frames: Vec<Vec<[f32; 3]>> = (0..4).map(|_| vec![[0.0, 0.0, 0.0]]).collect();
        let cache = uniform_time_mdd(&frames, 10.0);
        assert!((cache.times[0] - 0.0).abs() < 1e-6);
        assert!((cache.times[1] - 0.1).abs() < 1e-5);
        assert!((cache.times[2] - 0.2).abs() < 1e-5);
        assert!((cache.times[3] - 0.3).abs() < 1e-5);
    }

    // 7. uniform_time_mdd frame count
    #[test]
    fn uniform_time_frame_count() {
        let frames: Vec<Vec<[f32; 3]>> = (0..6).map(|_| vec![[1.0, 0.0, 0.0]]).collect();
        let cache = uniform_time_mdd(&frames, 24.0);
        assert_eq!(cache.frames.len(), 6);
    }

    // 8. mdd_duration returns last time
    #[test]
    fn duration_is_last_time() {
        let cache = two_frame_cache();
        let d = mdd_duration(&cache);
        assert!((d - 1.0 / 24.0).abs() < 1e-5);
    }

    // 9. mdd_duration with empty cache
    #[test]
    fn duration_empty_is_zero() {
        let c = MddCache::new(5);
        assert_eq!(mdd_duration(&c), 0.0);
    }

    // 10. add_frame wrong count panics
    #[test]
    #[should_panic]
    fn add_frame_wrong_count_panics() {
        let mut c = MddCache::new(3);
        c.add_frame(0.0, vec![[0.0, 0.0, 0.0]]); // only 1 point
    }

    // 11. truncated data returns error
    #[test]
    fn read_mdd_truncated() {
        let cache = two_frame_cache();
        let bytes = write_mdd(&cache);
        assert!(read_mdd(&bytes[..6]).is_err());
    }

    // 12. write then read with 1 point per frame
    #[test]
    fn single_point_roundtrip() {
        let mut c = MddCache::new(1);
        c.add_frame(0.0, vec![[1.23, 4.56, 7.89]]);
        let back = read_mdd(&write_mdd(&c)).expect("should succeed");
        let p = back.frames[0][0];
        assert!((p[0] - 1.23).abs() < 1e-5);
        assert!((p[1] - 4.56).abs() < 1e-5);
        assert!((p[2] - 7.89).abs() < 1e-5);
    }
}
