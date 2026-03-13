// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Binary animated geometry cache format (Alembic-inspired, custom binary format).
//!
//! Format: `.oxgc` (OXiHuman Geometry Cache)
//!
//! # File Layout
//!
//! ```text
//! [GeoCacheHeader – fixed 96 bytes]
//!   magic:        [u8; 4]   = b"OXGC"
//!   version:      u16 LE
//!   _pad:         u16       (alignment padding)
//!   vertex_count: u32 LE
//!   frame_count:  u32 LE
//!   fps:          f32 LE
//!   has_normals:  u8  (0 or 1)
//!   _pad:         [u8; 3]
//!   name:         [u8; 64]  (null-padded ASCII)
//!   reserved:     [u8; 4]
//!
//! [Per-frame blocks, repeated frame_count times]
//!   frame_index:  u32 LE
//!   time_seconds: f32 LE
//!   positions:    vertex_count × [f32 LE; 3]
//!   normals:      vertex_count × [f32 LE; 3]  (only when has_normals = 1)
//! ```

#![allow(dead_code)]

use std::io::{Read, Write};
use std::path::Path;

use anyhow::{bail, Context};
use oxihuman_mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Magic bytes identifying an OXGC file.
pub const OXGC_MAGIC: [u8; 4] = *b"OXGC";

/// Current format version.
pub const OXGC_VERSION: u16 = 1;

/// Total size of the fixed binary header in bytes.
const HEADER_SIZE: usize = 96;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Per-frame data stored in a geometry cache.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoCacheFrame {
    /// Zero-based frame index.
    pub frame_index: u32,
    /// Time stamp of this frame in seconds.
    pub time_seconds: f32,
    /// Per-vertex positions (`[x, y, z]`).
    pub positions: Vec<[f32; 3]>,
    /// Optional per-vertex normals (`[nx, ny, nz]`).
    pub normals: Option<Vec<[f32; 3]>>,
}

/// Fixed-size binary header written at the start of every OXGC file.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoCacheHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub vertex_count: u32,
    pub frame_count: u32,
    pub fps: f32,
    pub has_normals: bool,
    /// Null-padded ASCII cache name (64 bytes).
    pub name: [u8; 64],
}

/// In-memory geometry cache containing all frames.
#[derive(Debug, Clone)]
pub struct GeoCache {
    /// Human-readable name (stored in the file header).
    pub name: String,
    /// Frames per second.
    pub fps: f32,
    /// Number of vertices per frame (must be constant across all frames).
    pub vertex_count: usize,
    /// All frames in chronological order.
    pub frames: Vec<GeoCacheFrame>,
}

// ---------------------------------------------------------------------------
// GeoCache implementation
// ---------------------------------------------------------------------------

impl GeoCache {
    /// Create a new empty cache.
    pub fn new(name: &str, fps: f32, vertex_count: usize) -> Self {
        Self {
            name: name.to_owned(),
            fps,
            vertex_count,
            frames: Vec::new(),
        }
    }

    /// Append a frame. Returns an error when vertex counts mismatch, or when
    /// the frame has normals but the existing frames do not (or vice-versa).
    pub fn add_frame(&mut self, frame: GeoCacheFrame) -> Result<(), String> {
        if frame.positions.len() != self.vertex_count {
            return Err(format!(
                "frame {} has {} positions; cache expects {}",
                frame.frame_index,
                frame.positions.len(),
                self.vertex_count
            ));
        }
        if let Some(ref n) = frame.normals {
            if n.len() != self.vertex_count {
                return Err(format!(
                    "frame {} has {} normals; cache expects {}",
                    frame.frame_index,
                    n.len(),
                    self.vertex_count
                ));
            }
        }
        // Consistency check: all frames must agree on has_normals
        if !self.frames.is_empty() {
            let first_has = self.frames[0].normals.is_some();
            let this_has = frame.normals.is_some();
            if first_has != this_has {
                return Err(format!(
                    "frame {} normals presence ({}) differs from earlier frames ({})",
                    frame.frame_index, this_has, first_has
                ));
            }
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Number of frames stored.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Total animation duration in seconds (last frame time, or 0 if empty).
    pub fn duration_seconds(&self) -> f32 {
        self.frames.last().map(|f| f.time_seconds).unwrap_or(0.0)
    }

    /// Access a frame by index.
    pub fn get_frame(&self, index: usize) -> Option<&GeoCacheFrame> {
        self.frames.get(index)
    }

    /// Linearly interpolate vertex positions at `time_seconds`.
    ///
    /// Clamps to the first or last frame when `time_seconds` is out of the
    /// recorded range.  Returns `None` only when the cache is empty.
    pub fn sample(&self, time_seconds: f32) -> Option<Vec<[f32; 3]>> {
        let n = self.frames.len();
        if n == 0 {
            return None;
        }
        if n == 1 {
            return Some(self.frames[0].positions.clone());
        }

        // Clamp to bounds
        let t = time_seconds.clamp(self.frames[0].time_seconds, self.frames[n - 1].time_seconds);

        // Find the two surrounding frames
        let idx = self
            .frames
            .partition_point(|f| f.time_seconds <= t)
            .saturating_sub(1)
            .min(n - 2);

        let fa = &self.frames[idx];
        let fb = &self.frames[idx + 1];

        let dt = fb.time_seconds - fa.time_seconds;
        let alpha = if dt.abs() < f32::EPSILON {
            0.0
        } else {
            ((t - fa.time_seconds) / dt).clamp(0.0, 1.0)
        };

        let result = fa
            .positions
            .iter()
            .zip(fb.positions.iter())
            .map(|(a, b)| {
                [
                    a[0] + alpha * (b[0] - a[0]),
                    a[1] + alpha * (b[1] - a[1]),
                    a[2] + alpha * (b[2] - a[2]),
                ]
            })
            .collect();
        Some(result)
    }

    /// Write the cache to a binary file.
    pub fn write(&self, path: &Path) -> anyhow::Result<()> {
        export_geo_cache(self, path)
    }

    /// Read a cache from a binary file.
    pub fn read(path: &Path) -> anyhow::Result<Self> {
        load_geo_cache(path)
    }

    /// Validate a cache file (check magic, version, sizes).
    pub fn validate(path: &Path) -> anyhow::Result<()> {
        validate_geo_cache_file(path)
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

/// Convert a slice of [`MeshBuffers`] frames into a [`GeoCache`].
///
/// Normals from each mesh are included automatically.
pub fn mesh_sequence_to_geo_cache(name: &str, fps: f32, frames: &[MeshBuffers]) -> GeoCache {
    let vertex_count = frames.first().map(|m| m.positions.len()).unwrap_or(0);
    let mut cache = GeoCache::new(name, fps, vertex_count);

    for (i, mesh) in frames.iter().enumerate() {
        let time_seconds = i as f32 / fps.max(f32::EPSILON);
        let has_normals = !mesh.normals.is_empty() && mesh.normals.len() == mesh.positions.len();
        let normals = if has_normals {
            Some(mesh.normals.clone())
        } else {
            None
        };
        let frame = GeoCacheFrame {
            frame_index: i as u32,
            time_seconds,
            positions: mesh.positions.clone(),
            normals,
        };
        // Ignore vertex-count mismatches gracefully (skip bad frames)
        let _ = cache.add_frame(frame);
    }

    cache
}

// ---------------------------------------------------------------------------
// Public convenience wrappers
// ---------------------------------------------------------------------------

/// Write a [`GeoCache`] to the given path.
pub fn export_geo_cache(cache: &GeoCache, path: &Path) -> anyhow::Result<()> {
    let mut file =
        std::fs::File::create(path).with_context(|| format!("cannot create {}", path.display()))?;

    let has_normals = cache
        .frames
        .first()
        .map(|f| f.normals.is_some())
        .unwrap_or(false);

    write_header(&mut file, cache, has_normals)?;

    for frame in &cache.frames {
        write_frame(&mut file, frame, has_normals, cache.vertex_count)?;
    }

    Ok(())
}

/// Load a [`GeoCache`] from a binary file.
pub fn load_geo_cache(path: &Path) -> anyhow::Result<GeoCache> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;

    let header = read_header(&mut file)?;

    let name = bytes_to_name(&header.name);
    let vertex_count = header.vertex_count as usize;
    let frame_count = header.frame_count as usize;
    let has_normals = header.has_normals;

    let mut cache = GeoCache::new(&name, header.fps, vertex_count);

    for _ in 0..frame_count {
        let frame = read_frame(&mut file, vertex_count, has_normals)?;
        cache.frames.push(frame);
    }

    Ok(cache)
}

/// Validate an OXGC file without loading all frame data.
pub fn validate_geo_cache_file(path: &Path) -> anyhow::Result<()> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;

    let header = read_header(&mut file)?;

    // Validate version
    if header.version != OXGC_VERSION {
        bail!(
            "unsupported OXGC version {} (expected {})",
            header.version,
            OXGC_VERSION
        );
    }

    // Validate name is null-terminated ASCII
    let name = bytes_to_name(&header.name);
    if name.len() > 64 {
        bail!("name field exceeds 64 bytes");
    }

    // Spot-check: verify we can read at least the first frame header
    if header.frame_count > 0 {
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)
            .with_context(|| "could not read first frame_index")?;
        let _frame_index = u32::from_le_bytes(buf4);
        file.read_exact(&mut buf4)
            .with_context(|| "could not read first frame time")?;
        let _time = f32::from_le_bytes(buf4);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal I/O helpers
// ---------------------------------------------------------------------------

fn name_to_bytes(name: &str) -> [u8; 64] {
    let mut buf = [0u8; 64];
    let bytes = name.as_bytes();
    let len = bytes.len().min(63);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

fn bytes_to_name(buf: &[u8; 64]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(64);
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

fn write_header<W: Write>(
    writer: &mut W,
    cache: &GeoCache,
    has_normals: bool,
) -> anyhow::Result<()> {
    // magic (4 bytes)
    writer.write_all(&OXGC_MAGIC)?;
    // version (2 bytes) + alignment pad (2 bytes)
    writer.write_all(&OXGC_VERSION.to_le_bytes())?;
    writer.write_all(&[0u8; 2])?;
    // vertex_count (4 bytes)
    writer.write_all(&(cache.vertex_count as u32).to_le_bytes())?;
    // frame_count (4 bytes)
    writer.write_all(&(cache.frames.len() as u32).to_le_bytes())?;
    // fps (4 bytes)
    writer.write_all(&cache.fps.to_le_bytes())?;
    // has_normals (1 byte) + pad (3 bytes)
    writer.write_all(&[u8::from(has_normals)])?;
    writer.write_all(&[0u8; 3])?;
    // name (64 bytes)
    writer.write_all(&name_to_bytes(&cache.name))?;
    // reserved (4 bytes)
    writer.write_all(&[0u8; 4])?;

    // Total so far: 4+2+2+4+4+4+1+3+64+4 = 92 bytes — need 96
    writer.write_all(&[0u8; 4])?; // extra reserved

    Ok(())
}

fn read_header<R: Read>(reader: &mut R) -> anyhow::Result<GeoCacheHeader> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != OXGC_MAGIC {
        bail!(
            "invalid magic bytes: expected OXGC, got {:?}",
            std::str::from_utf8(&magic).unwrap_or("???")
        );
    }

    let mut buf2 = [0u8; 2];
    let mut buf4 = [0u8; 4];

    // version
    reader.read_exact(&mut buf2)?;
    let version = u16::from_le_bytes(buf2);
    // alignment pad
    reader.read_exact(&mut buf2)?;

    // vertex_count
    reader.read_exact(&mut buf4)?;
    let vertex_count = u32::from_le_bytes(buf4);

    // frame_count
    reader.read_exact(&mut buf4)?;
    let frame_count = u32::from_le_bytes(buf4);

    // fps
    reader.read_exact(&mut buf4)?;
    let fps = f32::from_le_bytes(buf4);

    // has_normals (1 byte) + pad (3 bytes)
    let mut buf1 = [0u8; 1];
    reader.read_exact(&mut buf1)?;
    let has_normals = buf1[0] != 0;
    let mut pad3 = [0u8; 3];
    reader.read_exact(&mut pad3)?;

    // name (64 bytes)
    let mut name = [0u8; 64];
    reader.read_exact(&mut name)?;

    // reserved (4 + 4 = 8 bytes)
    let mut reserved = [0u8; 8];
    reader.read_exact(&mut reserved)?;

    Ok(GeoCacheHeader {
        magic,
        version,
        vertex_count,
        frame_count,
        fps,
        has_normals,
        name,
    })
}

fn write_frame<W: Write>(
    writer: &mut W,
    frame: &GeoCacheFrame,
    has_normals: bool,
    vertex_count: usize,
) -> anyhow::Result<()> {
    writer.write_all(&frame.frame_index.to_le_bytes())?;
    writer.write_all(&frame.time_seconds.to_le_bytes())?;

    for &[x, y, z] in &frame.positions {
        writer.write_all(&x.to_le_bytes())?;
        writer.write_all(&y.to_le_bytes())?;
        writer.write_all(&z.to_le_bytes())?;
    }

    if has_normals {
        if let Some(ref normals) = frame.normals {
            for &[nx, ny, nz] in normals {
                writer.write_all(&nx.to_le_bytes())?;
                writer.write_all(&ny.to_le_bytes())?;
                writer.write_all(&nz.to_le_bytes())?;
            }
        } else {
            // Write zero normals as placeholder
            for _ in 0..vertex_count {
                writer.write_all(&0f32.to_le_bytes())?;
                writer.write_all(&1f32.to_le_bytes())?;
                writer.write_all(&0f32.to_le_bytes())?;
            }
        }
    }

    Ok(())
}

fn read_frame<R: Read>(
    reader: &mut R,
    vertex_count: usize,
    has_normals: bool,
) -> anyhow::Result<GeoCacheFrame> {
    let mut buf4 = [0u8; 4];

    reader.read_exact(&mut buf4)?;
    let frame_index = u32::from_le_bytes(buf4);

    reader.read_exact(&mut buf4)?;
    let time_seconds = f32::from_le_bytes(buf4);

    let mut positions = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        reader.read_exact(&mut buf4)?;
        let x = f32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let y = f32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let z = f32::from_le_bytes(buf4);
        positions.push([x, y, z]);
    }

    let normals = if has_normals {
        let mut nrm = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            reader.read_exact(&mut buf4)?;
            let nx = f32::from_le_bytes(buf4);
            reader.read_exact(&mut buf4)?;
            let ny = f32::from_le_bytes(buf4);
            reader.read_exact(&mut buf4)?;
            let nz = f32::from_le_bytes(buf4);
            nrm.push([nx, ny, nz]);
        }
        Some(nrm)
    } else {
        None
    };

    Ok(GeoCacheFrame {
        frame_index,
        time_seconds,
        positions,
        normals,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_frame(
        index: u32,
        time: f32,
        positions: Vec<[f32; 3]>,
        normals: Option<Vec<[f32; 3]>>,
    ) -> GeoCacheFrame {
        GeoCacheFrame {
            frame_index: index,
            time_seconds: time,
            positions,
            normals,
        }
    }

    fn make_mesh_buffers(positions: Vec<[f32; 3]>) -> MeshBuffers {
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

    // -----------------------------------------------------------------------
    // 1. Constants
    // -----------------------------------------------------------------------

    #[test]
    fn test_constants() {
        assert_eq!(&OXGC_MAGIC, b"OXGC");
        assert_eq!(OXGC_VERSION, 1);
    }

    // -----------------------------------------------------------------------
    // 2. GeoCache::new
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_cache() {
        let cache = GeoCache::new("TestCache", 30.0, 8);
        assert_eq!(cache.name, "TestCache");
        assert_eq!(cache.fps, 30.0);
        assert_eq!(cache.vertex_count, 8);
        assert_eq!(cache.frame_count(), 0);
    }

    // -----------------------------------------------------------------------
    // 3. add_frame – success
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_frame_success() {
        let mut cache = GeoCache::new("A", 24.0, 2);
        let f = make_frame(0, 0.0, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], None);
        cache.add_frame(f).expect("should succeed");
        assert_eq!(cache.frame_count(), 1);
    }

    // -----------------------------------------------------------------------
    // 4. add_frame – wrong vertex count
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_frame_wrong_vertex_count() {
        let mut cache = GeoCache::new("A", 24.0, 3);
        let f = make_frame(0, 0.0, vec![[0.0, 0.0, 0.0]], None); // only 1 vertex
        assert!(cache.add_frame(f).is_err());
    }

    // -----------------------------------------------------------------------
    // 5. add_frame – normals mismatch consistency
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_frame_normals_consistency() {
        let mut cache = GeoCache::new("B", 24.0, 2);
        let f0 = make_frame(0, 0.0, vec![[0.0; 3], [1.0; 3]], None);
        let f1 = make_frame(
            1,
            1.0 / 24.0,
            vec![[0.0; 3], [1.0; 3]],
            Some(vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]]),
        );
        cache.add_frame(f0).expect("should succeed");
        // Adding a frame with normals when the first had none should fail
        assert!(cache.add_frame(f1).is_err());
    }

    // -----------------------------------------------------------------------
    // 6. duration_seconds
    // -----------------------------------------------------------------------

    #[test]
    fn test_duration_seconds() {
        let mut cache = GeoCache::new("D", 25.0, 1);
        assert_eq!(cache.duration_seconds(), 0.0);
        cache
            .add_frame(make_frame(0, 0.0, vec![[0.0; 3]], None))
            .expect("should succeed");
        cache
            .add_frame(make_frame(1, 1.0 / 25.0, vec![[1.0; 3]], None))
            .expect("should succeed");
        let expected = 1.0f32 / 25.0;
        assert!((cache.duration_seconds() - expected).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 7. get_frame
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_frame() {
        let mut cache = GeoCache::new("G", 24.0, 1);
        cache
            .add_frame(make_frame(0, 0.0, vec![[1.0, 2.0, 3.0]], None))
            .expect("should succeed");
        let f = cache.get_frame(0).expect("should succeed");
        assert_eq!(f.positions[0], [1.0, 2.0, 3.0]);
        assert!(cache.get_frame(1).is_none());
    }

    // -----------------------------------------------------------------------
    // 8. sample – interpolation
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_interpolation() {
        let mut cache = GeoCache::new("S", 24.0, 1);
        cache
            .add_frame(make_frame(0, 0.0, vec![[0.0, 0.0, 0.0]], None))
            .expect("should succeed");
        cache
            .add_frame(make_frame(1, 1.0, vec![[10.0, 20.0, 30.0]], None))
            .expect("should succeed");

        let mid = cache.sample(0.5).expect("should succeed");
        let eps = 1e-4;
        assert!((mid[0][0] - 5.0).abs() < eps);
        assert!((mid[0][1] - 10.0).abs() < eps);
        assert!((mid[0][2] - 15.0).abs() < eps);
    }

    // -----------------------------------------------------------------------
    // 9. sample – clamping
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_clamping() {
        let mut cache = GeoCache::new("S", 24.0, 1);
        cache
            .add_frame(make_frame(0, 0.0, vec![[1.0, 2.0, 3.0]], None))
            .expect("should succeed");
        cache
            .add_frame(make_frame(1, 1.0, vec![[4.0, 5.0, 6.0]], None))
            .expect("should succeed");

        // Before start -> clamp to first frame
        let before = cache.sample(-5.0).expect("should succeed");
        assert_eq!(before[0], [1.0, 2.0, 3.0]);

        // After end -> clamp to last frame
        let after = cache.sample(100.0).expect("should succeed");
        assert_eq!(after[0], [4.0, 5.0, 6.0]);
    }

    // -----------------------------------------------------------------------
    // 10. sample – empty cache returns None
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_empty() {
        let cache = GeoCache::new("E", 24.0, 4);
        assert!(cache.sample(0.0).is_none());
    }

    // -----------------------------------------------------------------------
    // 11. export / load round-trip (no normals)
    // -----------------------------------------------------------------------

    #[test]
    fn test_export_load_no_normals() {
        let path = std::path::Path::new("/tmp/test_oxgc_no_normals.oxgc");
        let mut cache = GeoCache::new("RoundTrip", 30.0, 2);
        cache
            .add_frame(make_frame(
                0,
                0.0,
                vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
                None,
            ))
            .expect("should succeed");
        cache
            .add_frame(make_frame(
                1,
                1.0 / 30.0,
                vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]],
                None,
            ))
            .expect("should succeed");

        export_geo_cache(&cache, path).expect("should succeed");
        let loaded = load_geo_cache(path).expect("should succeed");

        assert_eq!(loaded.name, "RoundTrip");
        assert_eq!(loaded.fps, 30.0);
        assert_eq!(loaded.vertex_count, 2);
        assert_eq!(loaded.frame_count(), 2);
        assert_eq!(
            loaded.frames[0].positions,
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
        );
        assert_eq!(
            loaded.frames[1].positions,
            vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]
        );
        assert!(loaded.frames[0].normals.is_none());
    }

    // -----------------------------------------------------------------------
    // 12. export / load round-trip (with normals)
    // -----------------------------------------------------------------------

    #[test]
    fn test_export_load_with_normals() {
        let path = std::path::Path::new("/tmp/test_oxgc_with_normals.oxgc");
        let mut cache = GeoCache::new("WithNormals", 24.0, 2);
        let nrm0 = vec![[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let nrm1 = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        cache
            .add_frame(make_frame(
                0,
                0.0,
                vec![[0.0; 3], [1.0; 3]],
                Some(nrm0.clone()),
            ))
            .expect("should succeed");
        cache
            .add_frame(make_frame(
                1,
                1.0 / 24.0,
                vec![[2.0; 3], [3.0; 3]],
                Some(nrm1.clone()),
            ))
            .expect("should succeed");

        export_geo_cache(&cache, path).expect("should succeed");
        let loaded = load_geo_cache(path).expect("should succeed");

        assert_eq!(loaded.frame_count(), 2);
        assert_eq!(loaded.frames[0].normals.as_ref().expect("should succeed"), &nrm0);
        assert_eq!(loaded.frames[1].normals.as_ref().expect("should succeed"), &nrm1);
    }

    // -----------------------------------------------------------------------
    // 13. validate – good file
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_good_file() {
        let path = std::path::Path::new("/tmp/test_oxgc_validate_ok.oxgc");
        let mut cache = GeoCache::new("Valid", 25.0, 3);
        cache
            .add_frame(make_frame(0, 0.0, vec![[0.0; 3], [1.0; 3], [2.0; 3]], None))
            .expect("should succeed");
        export_geo_cache(&cache, path).expect("should succeed");
        assert!(GeoCache::validate(path).is_ok());
    }

    // -----------------------------------------------------------------------
    // 14. validate – bad magic
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_bad_magic() {
        let path = std::path::Path::new("/tmp/test_oxgc_bad_magic.oxgc");
        let mut data = vec![0u8; HEADER_SIZE];
        data[..4].copy_from_slice(b"BAAD");
        std::fs::write(path, &data).expect("should succeed");
        let result = GeoCache::validate(path);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("invalid magic") || msg.contains("OXGC"));
    }

    // -----------------------------------------------------------------------
    // 15. mesh_sequence_to_geo_cache
    // -----------------------------------------------------------------------

    #[test]
    fn test_mesh_sequence_to_geo_cache() {
        let m0 = make_mesh_buffers(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let m1 = make_mesh_buffers(vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]]);
        let cache = mesh_sequence_to_geo_cache("Seq", 24.0, &[m0, m1]);
        assert_eq!(cache.vertex_count, 2);
        assert_eq!(cache.frame_count(), 2);
        assert_eq!(cache.fps, 24.0);
        assert_eq!(cache.name, "Seq");
        // Normals included from MeshBuffers
        assert!(cache.frames[0].normals.is_some());
    }

    // -----------------------------------------------------------------------
    // 16. GeoCache::write / read method aliases
    // -----------------------------------------------------------------------

    #[test]
    fn test_write_read_methods() {
        let path = std::path::Path::new("/tmp/test_oxgc_methods.oxgc");
        let mut cache = GeoCache::new("Methods", 60.0, 1);
        cache
            .add_frame(make_frame(0, 0.0, vec![[9.0, 8.0, 7.0]], None))
            .expect("should succeed");

        cache.write(path).expect("should succeed");
        let loaded = GeoCache::read(path).expect("should succeed");
        assert_eq!(loaded.name, "Methods");
        assert_eq!(loaded.frames[0].positions[0], [9.0, 8.0, 7.0]);
    }

    // -----------------------------------------------------------------------
    // 17. load_geo_cache convenience wrapper
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_geo_cache_wrapper() {
        let path = std::path::Path::new("/tmp/test_oxgc_load_wrapper.oxgc");
        let mut cache = GeoCache::new("Wrap", 12.0, 1);
        cache
            .add_frame(make_frame(0, 0.0, vec![[3.0, 2.71, 1.41]], None))
            .expect("should succeed");
        export_geo_cache(&cache, path).expect("should succeed");

        let loaded = load_geo_cache(path).expect("should succeed");
        let eps = 1e-5;
        assert!((loaded.frames[0].positions[0][0] - 3.0).abs() < eps);
    }

    // -----------------------------------------------------------------------
    // 18. Name padding / truncation
    // -----------------------------------------------------------------------

    #[test]
    fn test_name_padding() {
        let path = std::path::Path::new("/tmp/test_oxgc_name.oxgc");
        let cache = GeoCache::new("Short", 1.0, 0);
        export_geo_cache(&cache, path).expect("should succeed");
        let loaded = load_geo_cache(path).expect("should succeed");
        assert_eq!(loaded.name, "Short");
    }

    // -----------------------------------------------------------------------
    // 19. Long name truncated to 63 chars
    // -----------------------------------------------------------------------

    #[test]
    fn test_name_truncation() {
        let long_name = "A".repeat(200);
        let name_bytes = name_to_bytes(&long_name);
        // Must fit in 64 bytes with null terminator
        assert_eq!(name_bytes.len(), 64);
        assert_eq!(name_bytes[63], 0); // last byte must be null
        let recovered = bytes_to_name(&name_bytes);
        assert_eq!(recovered.len(), 63);
    }

    // -----------------------------------------------------------------------
    // 20. frame_index stored and recovered correctly
    // -----------------------------------------------------------------------

    #[test]
    fn test_frame_index_round_trip() {
        let path = std::path::Path::new("/tmp/test_oxgc_frame_index.oxgc");
        let mut cache = GeoCache::new("Idx", 24.0, 1);
        cache
            .add_frame(make_frame(42, 0.0, vec![[0.0; 3]], None))
            .expect("should succeed");
        export_geo_cache(&cache, path).expect("should succeed");
        let loaded = load_geo_cache(path).expect("should succeed");
        assert_eq!(loaded.frames[0].frame_index, 42);
    }

    // -----------------------------------------------------------------------
    // 21. Empty cache writes and loads cleanly
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_cache_round_trip() {
        let path = std::path::Path::new("/tmp/test_oxgc_empty.oxgc");
        let cache = GeoCache::new("Empty", 24.0, 100);
        export_geo_cache(&cache, path).expect("should succeed");
        let loaded = load_geo_cache(path).expect("should succeed");
        assert_eq!(loaded.frame_count(), 0);
        assert_eq!(loaded.vertex_count, 100);
    }

    // -----------------------------------------------------------------------
    // 22. Sample on single-frame cache returns that frame
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_single_frame() {
        let mut cache = GeoCache::new("One", 24.0, 2);
        cache
            .add_frame(make_frame(
                0,
                0.0,
                vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
                None,
            ))
            .expect("should succeed");
        let result = cache.sample(99.0).expect("should succeed");
        assert_eq!(result[0], [1.0, 2.0, 3.0]);
        assert_eq!(result[1], [4.0, 5.0, 6.0]);
    }

    // -----------------------------------------------------------------------
    // 23. Header constants match HEADER_SIZE
    // -----------------------------------------------------------------------

    #[test]
    fn test_header_binary_size() {
        // Write a minimal cache and check offset of first frame data
        let path = std::path::Path::new("/tmp/test_oxgc_header_size.oxgc");
        let mut cache = GeoCache::new("Sz", 1.0, 1);
        cache
            .add_frame(make_frame(0, 0.0, vec![[1.0, 2.0, 3.0]], None))
            .expect("should succeed");
        export_geo_cache(&cache, path).expect("should succeed");

        let data = std::fs::read(path).expect("should succeed");
        // Header = 96 bytes, frame = 4+4+12 = 20 bytes
        assert_eq!(data.len(), HEADER_SIZE + 20);
    }

    // -----------------------------------------------------------------------
    // 24. GeoCacheHeader magic field
    // -----------------------------------------------------------------------

    #[test]
    fn test_header_struct_fields() {
        let path = std::path::Path::new("/tmp/test_oxgc_hdr_fields.oxgc");
        let cache = GeoCache::new("Hdr", 48.0, 5);
        export_geo_cache(&cache, path).expect("should succeed");
        let mut file = std::fs::File::open(path).expect("should succeed");
        let hdr = read_header(&mut file).expect("should succeed");
        assert_eq!(hdr.magic, OXGC_MAGIC);
        assert_eq!(hdr.version, OXGC_VERSION);
        assert_eq!(hdr.vertex_count, 5);
        assert_eq!(hdr.frame_count, 0);
        assert_eq!(hdr.fps, 48.0);
        assert!(!hdr.has_normals);
    }
}
