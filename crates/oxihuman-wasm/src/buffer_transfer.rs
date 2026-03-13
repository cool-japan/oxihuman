// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Zero-copy buffer management between Rust/WASM and JavaScript.
//!
//! [`WasmBuffer`] provides an efficient way to share typed array data across
//! the WASM boundary without redundant copies.  [`BufferPool`] keeps a set of
//! reusable buffers so that repeated operations (e.g. animation frames) do not
//! hit the allocator on every tick.

use anyhow::{ensure, Context, Result};

// ---------------------------------------------------------------------------
// WasmBuffer
// ---------------------------------------------------------------------------

/// Manages shared memory buffers for efficient WASM<->JS data transfer.
///
/// The underlying `Vec<u8>` lives on the WASM linear-memory heap.  JavaScript
/// can obtain a view into it via `as_ptr()` + `len()`, avoiding any extra
/// copies when working with `Uint8Array` / `Float32Array` wrappers.
#[derive(Debug, Clone)]
pub struct WasmBuffer {
    data: Vec<u8>,
    capacity: usize,
}

impl WasmBuffer {
    /// Create a new buffer pre-allocated with the given *byte* capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Raw pointer to the beginning of the buffer (for JS interop).
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Current length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the buffer contains no data.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all data while keeping the allocated capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    // -- f32 helpers --------------------------------------------------------

    /// Write an `f32` slice into the buffer (little-endian), replacing any
    /// previous content.
    pub fn write_f32_slice(&mut self, data: &[f32]) -> Result<()> {
        let byte_len = data
            .len()
            .checked_mul(4)
            .context("f32 slice byte-length overflow")?;
        ensure!(
            byte_len <= self.capacity,
            "f32 slice ({byte_len} bytes) exceeds buffer capacity ({})",
            self.capacity
        );
        self.data.clear();
        self.data.reserve(byte_len);
        for &val in data {
            self.data.extend_from_slice(&val.to_le_bytes());
        }
        Ok(())
    }

    /// Write an `f64` slice into the buffer (little-endian), replacing any
    /// previous content.
    pub fn write_f64_slice(&mut self, data: &[f64]) -> Result<()> {
        let byte_len = data
            .len()
            .checked_mul(8)
            .context("f64 slice byte-length overflow")?;
        ensure!(
            byte_len <= self.capacity,
            "f64 slice ({byte_len} bytes) exceeds buffer capacity ({})",
            self.capacity
        );
        self.data.clear();
        self.data.reserve(byte_len);
        for &val in data {
            self.data.extend_from_slice(&val.to_le_bytes());
        }
        Ok(())
    }

    /// Interpret the current buffer contents as a packed `f32` array.
    pub fn read_f32_slice(&self) -> Result<Vec<f32>> {
        ensure!(
            self.data.len().is_multiple_of(4),
            "buffer length ({}) is not a multiple of 4",
            self.data.len()
        );
        let count = self.data.len() / 4;
        let mut out = Vec::with_capacity(count);
        for chunk in self.data.chunks_exact(4) {
            let bytes: [u8; 4] = chunk
                .try_into()
                .context("unexpected chunk size reading f32")?;
            out.push(f32::from_le_bytes(bytes));
        }
        Ok(out)
    }

    /// Interpret the current buffer contents as a packed `f64` array.
    pub fn read_f64_slice(&self) -> Result<Vec<f64>> {
        ensure!(
            self.data.len().is_multiple_of(8),
            "buffer length ({}) is not a multiple of 8",
            self.data.len()
        );
        let count = self.data.len() / 8;
        let mut out = Vec::with_capacity(count);
        for chunk in self.data.chunks_exact(8) {
            let bytes: [u8; 8] = chunk
                .try_into()
                .context("unexpected chunk size reading f64")?;
            out.push(f64::from_le_bytes(bytes));
        }
        Ok(out)
    }

    // -- mesh helpers -------------------------------------------------------

    /// Write mesh vertex positions (`[f64; 3]` per vertex) into the buffer.
    ///
    /// Layout: `[x0 y0 z0 x1 y1 z1 …]` packed little-endian f64.
    pub fn write_mesh_positions(&mut self, positions: &[[f64; 3]]) -> Result<()> {
        let byte_len = positions
            .len()
            .checked_mul(3 * 8)
            .context("position byte-length overflow")?;
        ensure!(
            byte_len <= self.capacity,
            "mesh positions ({byte_len} bytes) exceed buffer capacity ({})",
            self.capacity
        );
        self.data.clear();
        self.data.reserve(byte_len);
        for pos in positions {
            for &v in pos {
                self.data.extend_from_slice(&v.to_le_bytes());
            }
        }
        Ok(())
    }

    /// Write triangle indices (`[usize; 3]` per face) as packed `u32` LE.
    pub fn write_mesh_indices(&mut self, indices: &[[usize; 3]]) -> Result<()> {
        let byte_len = indices
            .len()
            .checked_mul(3 * 4)
            .context("index byte-length overflow")?;
        ensure!(
            byte_len <= self.capacity,
            "mesh indices ({byte_len} bytes) exceed buffer capacity ({})",
            self.capacity
        );
        self.data.clear();
        self.data.reserve(byte_len);
        for tri in indices {
            for &idx in tri {
                let idx32: u32 = idx
                    .try_into()
                    .with_context(|| format!("index {idx} does not fit in u32"))?;
                self.data.extend_from_slice(&idx32.to_le_bytes());
            }
        }
        Ok(())
    }

    /// Write per-vertex normals (`[f64; 3]` per vertex) as packed f64 LE.
    pub fn write_mesh_normals(&mut self, normals: &[[f64; 3]]) -> Result<()> {
        let byte_len = normals
            .len()
            .checked_mul(3 * 8)
            .context("normal byte-length overflow")?;
        ensure!(
            byte_len <= self.capacity,
            "mesh normals ({byte_len} bytes) exceed buffer capacity ({})",
            self.capacity
        );
        self.data.clear();
        self.data.reserve(byte_len);
        for n in normals {
            for &v in n {
                self.data.extend_from_slice(&v.to_le_bytes());
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// BufferPool
// ---------------------------------------------------------------------------

/// Pool of reusable [`WasmBuffer`]s to avoid repeated allocation.
///
/// When a caller is done with a buffer it can return it to the pool; the pool
/// will clear it and keep it for re-use up to `max_buffers` entries.
#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<WasmBuffer>,
    max_buffers: usize,
}

impl BufferPool {
    /// Create a pool that holds at most `max_buffers` idle buffers.
    pub fn new(max_buffers: usize) -> Self {
        Self {
            buffers: Vec::new(),
            max_buffers,
        }
    }

    /// Obtain a buffer from the pool (or allocate a new one with the given
    /// capacity if none is available).
    pub fn acquire(&mut self, capacity: usize) -> WasmBuffer {
        // Try to find one whose capacity is >= requested.
        if let Some(idx) = self.buffers.iter().position(|b| b.capacity >= capacity) {
            let mut buf = self.buffers.swap_remove(idx);
            buf.clear();
            return buf;
        }
        WasmBuffer::new(capacity)
    }

    /// Return a buffer to the pool for later reuse.
    ///
    /// If the pool is already at capacity the buffer is simply dropped.
    pub fn release(&mut self, mut buf: WasmBuffer) {
        if self.buffers.len() < self.max_buffers {
            buf.clear();
            self.buffers.push(buf);
        }
        // else: drop it
    }

    /// Number of idle buffers currently in the pool.
    #[inline]
    pub fn idle_count(&self) -> usize {
        self.buffers.len()
    }

    /// Drain all idle buffers, freeing their memory.
    pub fn drain(&mut self) {
        self.buffers.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_f32() {
        let mut buf = WasmBuffer::new(1024);
        let src = vec![1.0f32, -2.5, std::f32::consts::PI, 0.0];
        buf.write_f32_slice(&src).expect("should succeed");
        let out = buf.read_f32_slice().expect("should succeed");
        assert_eq!(src, out);
    }

    #[test]
    fn round_trip_f64() {
        let mut buf = WasmBuffer::new(1024);
        let src = vec![1.0f64, -2.5, std::f64::consts::PI];
        buf.write_f64_slice(&src).expect("should succeed");
        let out = buf.read_f64_slice().expect("should succeed");
        assert_eq!(src, out);
    }

    #[test]
    fn capacity_guard() {
        let mut buf = WasmBuffer::new(8); // room for 2 f32s
        let too_big = vec![1.0f32; 10];
        assert!(buf.write_f32_slice(&too_big).is_err());
    }

    #[test]
    fn mesh_positions_round_trip() {
        let positions: Vec<[f64; 3]> = vec![[1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]];
        let mut buf = WasmBuffer::new(positions.len() * 3 * 8);
        buf.write_mesh_positions(&positions)
            .expect("should succeed");
        let out = buf.read_f64_slice().expect("should succeed");
        assert_eq!(out.len(), 6);
        assert!((out[0] - 1.0).abs() < f64::EPSILON);
        assert!((out[5] - (-3.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn mesh_indices_round_trip() {
        let indices: Vec<[usize; 3]> = vec![[0, 1, 2], [3, 4, 5]];
        let mut buf = WasmBuffer::new(indices.len() * 3 * 4);
        buf.write_mesh_indices(&indices).expect("should succeed");
        assert_eq!(buf.len(), 24);
    }

    #[test]
    fn pool_acquire_release() {
        let mut pool = BufferPool::new(2);
        let buf = pool.acquire(256);
        assert_eq!(pool.idle_count(), 0);
        pool.release(buf);
        assert_eq!(pool.idle_count(), 1);
        let _b2 = pool.acquire(128);
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn pool_max_limit() {
        let mut pool = BufferPool::new(1);
        pool.release(WasmBuffer::new(64));
        pool.release(WasmBuffer::new(64));
        assert_eq!(pool.idle_count(), 1);
    }
}
