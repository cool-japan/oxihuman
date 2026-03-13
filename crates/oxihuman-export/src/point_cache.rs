// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Binary point cache export for vertex animation sequences.
//!
//! Format: `.opc` (OxiHuman Point Cache)
//! Header: 32 bytes, frame data follows as packed f32 LE triples.

#![allow(dead_code)]

use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, bail, Context};
use oxihuman_mesh::MeshBuffers;

/// Magic bytes identifying an OPC file.
pub const OPC_MAGIC: &[u8; 4] = b"OPC1";

/// Header for a point cache file.
#[derive(Debug, Clone, PartialEq)]
pub struct PointCacheHeader {
    pub vertex_count: u32,
    pub frame_count: u32,
    pub fps: f32,
}

/// In-memory point cache: per-frame vertex positions.
#[derive(Debug, Clone)]
pub struct PointCache {
    pub header: PointCacheHeader,
    /// All frames: outer = frame index, inner = per-vertex [x, y, z].
    pub frames: Vec<Vec<[f32; 3]>>,
}

impl PointCache {
    /// Create a new empty cache for the given vertex count and frame rate.
    pub fn new(vertex_count: usize, fps: f32) -> Self {
        Self {
            header: PointCacheHeader {
                vertex_count: vertex_count as u32,
                frame_count: 0,
                fps,
            },
            frames: Vec::new(),
        }
    }

    /// Append a frame. Returns an error if the position count does not match.
    pub fn add_frame(&mut self, positions: Vec<[f32; 3]>) -> anyhow::Result<()> {
        if positions.len() != self.header.vertex_count as usize {
            bail!(
                "frame vertex count {} does not match cache vertex count {}",
                positions.len(),
                self.header.vertex_count
            );
        }
        self.frames.push(positions);
        self.header.frame_count = self.frames.len() as u32;
        Ok(())
    }

    /// Number of frames currently stored.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Number of vertices per frame.
    pub fn vertex_count(&self) -> usize {
        self.header.vertex_count as usize
    }

    /// Total duration in seconds.
    pub fn duration(&self) -> f32 {
        if self.header.fps == 0.0 {
            0.0
        } else {
            self.header.frame_count as f32 / self.header.fps
        }
    }

    /// Get a reference to a frame by index.
    pub fn get_frame(&self, index: usize) -> Option<&Vec<[f32; 3]>> {
        self.frames.get(index)
    }

    /// Interpolate vertex positions at fractional frame time `t` (range: 0 .. frame_count-1).
    ///
    /// Returns `None` if the cache has fewer than two frames or `t` is out of range.
    pub fn sample(&self, t: f32) -> Option<Vec<[f32; 3]>> {
        let n = self.frames.len();
        if n < 2 {
            return None;
        }
        if t < 0.0 || t > (n - 1) as f32 {
            return None;
        }
        let f = t.floor() as usize;
        let frac = t - f as f32;
        let f_next = (f + 1).min(n - 1);

        let a = &self.frames[f];
        let b = &self.frames[f_next];

        let result = a
            .iter()
            .zip(b.iter())
            .map(|(pa, pb)| {
                [
                    pa[0] + frac * (pb[0] - pa[0]),
                    pa[1] + frac * (pb[1] - pa[1]),
                    pa[2] + frac * (pb[2] - pa[2]),
                ]
            })
            .collect();
        Some(result)
    }
}

/// Export a `PointCache` to a `.opc` binary file.
pub fn export_point_cache(cache: &PointCache, path: &Path) -> anyhow::Result<()> {
    let mut file =
        std::fs::File::create(path).with_context(|| format!("cannot create {}", path.display()))?;

    // --- Header (32 bytes) ---
    file.write_all(OPC_MAGIC)?;
    file.write_all(&cache.header.vertex_count.to_le_bytes())?;
    file.write_all(&cache.header.frame_count.to_le_bytes())?;
    file.write_all(&cache.header.fps.to_le_bytes())?;
    file.write_all(&[0u8; 16])?; // reserved

    // --- Frame data ---
    for frame in &cache.frames {
        for &[x, y, z] in frame {
            file.write_all(&x.to_le_bytes())?;
            file.write_all(&y.to_le_bytes())?;
            file.write_all(&z.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Load a `PointCache` from a `.opc` binary file.
pub fn load_point_cache(path: &Path) -> anyhow::Result<PointCache> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;

    let header = read_header(&mut file)?;

    let vertex_count = header.vertex_count as usize;
    let frame_count = header.frame_count as usize;

    let mut frames = Vec::with_capacity(frame_count);
    let mut buf4 = [0u8; 4];

    for _ in 0..frame_count {
        let mut verts = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            file.read_exact(&mut buf4)?;
            let x = f32::from_le_bytes(buf4);
            file.read_exact(&mut buf4)?;
            let y = f32::from_le_bytes(buf4);
            file.read_exact(&mut buf4)?;
            let z = f32::from_le_bytes(buf4);
            verts.push([x, y, z]);
        }
        frames.push(verts);
    }

    Ok(PointCache { header, frames })
}

/// Build a `PointCache` from a slice of `MeshBuffers`. All meshes must share the same vertex count.
pub fn mesh_sequence_to_cache(frames: &[MeshBuffers], fps: f32) -> anyhow::Result<PointCache> {
    if frames.is_empty() {
        bail!("frame sequence is empty");
    }
    let vertex_count = frames[0].positions.len();
    let mut cache = PointCache::new(vertex_count, fps);
    for (i, mesh) in frames.iter().enumerate() {
        if mesh.positions.len() != vertex_count {
            bail!(
                "frame {} has {} vertices; expected {}",
                i,
                mesh.positions.len(),
                vertex_count
            );
        }
        cache.add_frame(mesh.positions.clone())?;
    }
    Ok(cache)
}

/// Extract the vertex positions for a single frame from a cache.
pub fn cache_frame_to_positions(cache: &PointCache, frame: usize) -> anyhow::Result<Vec<[f32; 3]>> {
    cache.get_frame(frame).cloned().ok_or_else(|| {
        anyhow!(
            "frame index {} out of range (cache has {} frames)",
            frame,
            cache.frame_count()
        )
    })
}

/// Validate a `.opc` file and return its header without loading frame data.
pub fn validate_point_cache_file(path: &Path) -> anyhow::Result<PointCacheHeader> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    read_header(&mut file)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn read_header<R: Read>(reader: &mut R) -> anyhow::Result<PointCacheHeader> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != OPC_MAGIC {
        bail!("invalid magic bytes: expected OPC1, got {:?}", magic);
    }

    let mut buf4 = [0u8; 4];

    reader.read_exact(&mut buf4)?;
    let vertex_count = u32::from_le_bytes(buf4);

    reader.read_exact(&mut buf4)?;
    let frame_count = u32::from_le_bytes(buf4);

    reader.read_exact(&mut buf4)?;
    let fps = f32::from_le_bytes(buf4);

    // Skip 16 reserved bytes
    let mut reserved = [0u8; 16];
    reader.read_exact(&mut reserved)?;

    Ok(PointCacheHeader {
        vertex_count,
        frame_count,
        fps,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;

    fn make_mesh(positions: Vec<[f32; 3]>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices: vec![],
            colors: None,
            has_suit: false,
        }
    }

    #[test]
    fn test_point_cache_new() {
        let cache = PointCache::new(4, 24.0);
        assert_eq!(cache.vertex_count(), 4);
        assert_eq!(cache.frame_count(), 0);
        assert_eq!(cache.header.fps, 24.0);
    }

    #[test]
    fn test_add_frame() {
        let mut cache = PointCache::new(2, 30.0);
        let frame = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        cache.add_frame(frame.clone()).expect("should succeed");
        assert_eq!(cache.frame_count(), 1);
        assert_eq!(cache.header.frame_count, 1);
        assert_eq!(cache.get_frame(0).expect("should succeed"), &frame);
    }

    #[test]
    fn test_add_frame_wrong_vertex_count() {
        let mut cache = PointCache::new(2, 30.0);
        let bad = vec![[1.0, 2.0, 3.0]]; // only 1 vertex
        let result = cache.add_frame(bad);
        assert!(result.is_err());
    }

    #[test]
    fn test_duration() {
        let mut cache = PointCache::new(1, 25.0);
        cache.add_frame(vec![[0.0, 0.0, 0.0]]).expect("should succeed");
        cache.add_frame(vec![[1.0, 1.0, 1.0]]).expect("should succeed");
        // 2 frames / 25 fps = 0.08 s
        let expected = 2.0 / 25.0;
        assert!((cache.duration() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_get_frame() {
        let mut cache = PointCache::new(2, 24.0);
        let f0 = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let f1 = vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        cache.add_frame(f0.clone()).expect("should succeed");
        cache.add_frame(f1.clone()).expect("should succeed");
        assert_eq!(cache.get_frame(0).expect("should succeed"), &f0);
        assert_eq!(cache.get_frame(1).expect("should succeed"), &f1);
        assert!(cache.get_frame(2).is_none());
    }

    #[test]
    fn test_sample_exact_frame() {
        let mut cache = PointCache::new(1, 24.0);
        cache.add_frame(vec![[0.0, 0.0, 0.0]]).expect("should succeed");
        cache.add_frame(vec![[2.0, 4.0, 6.0]]).expect("should succeed");
        let s0 = cache.sample(0.0).expect("should succeed");
        assert_eq!(s0[0], [0.0, 0.0, 0.0]);
        let s1 = cache.sample(1.0).expect("should succeed");
        assert_eq!(s1[0], [2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_sample_between_frames() {
        let mut cache = PointCache::new(1, 24.0);
        cache.add_frame(vec![[0.0, 0.0, 0.0]]).expect("should succeed");
        cache.add_frame(vec![[2.0, 4.0, 6.0]]).expect("should succeed");
        let s = cache.sample(0.5).expect("should succeed");
        let eps = 1e-5;
        assert!((s[0][0] - 1.0).abs() < eps);
        assert!((s[0][1] - 2.0).abs() < eps);
        assert!((s[0][2] - 3.0).abs() < eps);
    }

    #[test]
    fn test_sample_out_of_range() {
        let mut cache = PointCache::new(1, 24.0);
        cache.add_frame(vec![[0.0, 0.0, 0.0]]).expect("should succeed");
        cache.add_frame(vec![[1.0, 1.0, 1.0]]).expect("should succeed");
        assert!(cache.sample(-0.1).is_none());
        assert!(cache.sample(1.1).is_none());
    }

    #[test]
    fn test_export_and_load() {
        let path = std::path::Path::new("/tmp/test_export_and_load.opc");
        let mut cache = PointCache::new(2, 30.0);
        cache
            .add_frame(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])
            .expect("should succeed");
        cache
            .add_frame(vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]])
            .expect("should succeed");

        export_point_cache(&cache, path).expect("should succeed");
        let loaded = load_point_cache(path).expect("should succeed");

        assert_eq!(loaded.header.vertex_count, 2);
        assert_eq!(loaded.header.frame_count, 2);
        assert_eq!(loaded.header.fps, 30.0);
        assert_eq!(loaded.frames[0], vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        assert_eq!(loaded.frames[1], vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]);
    }

    #[test]
    fn test_validate_header() {
        let path = std::path::Path::new("/tmp/test_validate_header.opc");
        let mut cache = PointCache::new(3, 60.0);
        cache
            .add_frame(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
            .expect("should succeed");
        export_point_cache(&cache, path).expect("should succeed");

        let hdr = validate_point_cache_file(path).expect("should succeed");
        assert_eq!(hdr.vertex_count, 3);
        assert_eq!(hdr.frame_count, 1);
        assert_eq!(hdr.fps, 60.0);
    }

    #[test]
    fn test_validate_bad_magic() {
        let path = std::path::Path::new("/tmp/test_validate_bad_magic.opc");
        // Write a file with wrong magic
        std::fs::write(path, b"BAAD\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00").expect("should succeed");
        let result = validate_point_cache_file(path);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("invalid magic bytes") || msg.contains("OPC1"));
    }

    #[test]
    fn test_mesh_sequence_to_cache() {
        let m0 = make_mesh(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let m1 = make_mesh(vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]]);
        let cache = mesh_sequence_to_cache(&[m0, m1], 24.0).expect("should succeed");
        assert_eq!(cache.vertex_count(), 2);
        assert_eq!(cache.frame_count(), 2);
        assert_eq!(cache.header.fps, 24.0);
    }

    #[test]
    fn test_mesh_sequence_mismatched_vertex_count() {
        let m0 = make_mesh(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let m1 = make_mesh(vec![[0.0, 1.0, 0.0]]); // 1 vertex
        let result = mesh_sequence_to_cache(&[m0, m1], 24.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_frame_to_positions() {
        let mut cache = PointCache::new(2, 24.0);
        let f0 = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        cache.add_frame(f0.clone()).expect("should succeed");
        let positions = cache_frame_to_positions(&cache, 0).expect("should succeed");
        assert_eq!(positions, f0);
        assert!(cache_frame_to_positions(&cache, 99).is_err());
    }
}
