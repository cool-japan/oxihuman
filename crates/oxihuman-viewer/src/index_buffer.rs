// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Index buffer descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndexBuffer {
    pub data: Vec<u32>,
    /// 0 = u16, 1 = u32
    pub format: u8,
    pub buffer_id: u32,
}

/// Create a new empty index buffer with the given GPU ID.
#[allow(dead_code)]
pub fn new_index_buffer(id: u32) -> IndexBuffer {
    IndexBuffer { data: Vec::new(), format: 1, buffer_id: id }
}

/// Build an index buffer from a list of triangles.
#[allow(dead_code)]
pub fn index_buffer_from_tris(tris: &[[u32; 3]], id: u32) -> IndexBuffer {
    let mut data = Vec::with_capacity(tris.len() * 3);
    for tri in tris {
        data.push(tri[0]);
        data.push(tri[1]);
        data.push(tri[2]);
    }
    IndexBuffer { data, format: 1, buffer_id: id }
}

/// Total number of indices.
#[allow(dead_code)]
pub fn index_count(ib: &IndexBuffer) -> usize {
    ib.data.len()
}

/// Number of triangles (index_count / 3).
#[allow(dead_code)]
pub fn tri_count(ib: &IndexBuffer) -> usize {
    ib.data.len() / 3
}

/// Maximum index value, or 0 for an empty buffer.
#[allow(dead_code)]
pub fn max_index(ib: &IndexBuffer) -> u32 {
    ib.data.iter().copied().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_index_buffer_empty() {
        let ib = new_index_buffer(1);
        assert_eq!(index_count(&ib), 0);
    }

    #[test]
    fn from_tris_index_count() {
        let tris = [[0u32, 1, 2], [2, 1, 3]];
        let ib = index_buffer_from_tris(&tris, 1);
        assert_eq!(index_count(&ib), 6);
    }

    #[test]
    fn tri_count_correct() {
        let tris = [[0u32, 1, 2]];
        let ib = index_buffer_from_tris(&tris, 1);
        assert_eq!(tri_count(&ib), 1);
    }

    #[test]
    fn max_index_correct() {
        let tris = [[0u32, 1, 2], [3, 4, 5]];
        let ib = index_buffer_from_tris(&tris, 1);
        assert_eq!(max_index(&ib), 5);
    }

    #[test]
    fn max_index_empty() {
        let ib = new_index_buffer(1);
        assert_eq!(max_index(&ib), 0);
    }

    #[test]
    fn tri_count_zero_for_empty() {
        let ib = new_index_buffer(1);
        assert_eq!(tri_count(&ib), 0);
    }

    #[test]
    fn buffer_id_stored() {
        let ib = new_index_buffer(42);
        assert_eq!(ib.buffer_id, 42);
    }

    #[test]
    fn format_u32() {
        let ib = new_index_buffer(1);
        assert_eq!(ib.format, 1);
    }

    #[test]
    fn from_tris_data_correct() {
        let tris = [[10u32, 20, 30]];
        let ib = index_buffer_from_tris(&tris, 1);
        assert_eq!(ib.data, vec![10, 20, 30]);
    }

    #[test]
    fn from_tris_id_stored() {
        let tris: Vec<[u32; 3]> = vec![];
        let ib = index_buffer_from_tris(&tris, 99);
        assert_eq!(ib.buffer_id, 99);
    }
}
