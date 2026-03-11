#![allow(dead_code)]
//! Typed vertex identifiers with generation tracking.

/// A vertex identifier with generation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId {
    pub index: u32,
    pub generation: u32,
}

/// Create a new vertex id.
#[allow(dead_code)]
pub fn new_vertex_id(index: u32, generation: u32) -> VertexId {
    VertexId { index, generation }
}

/// Get the index.
#[allow(dead_code)]
pub fn vertex_id_index(id: VertexId) -> u32 {
    id.index
}

/// Get the generation.
#[allow(dead_code)]
pub fn vertex_id_generation(id: VertexId) -> u32 {
    id.generation
}

/// Check if two vertex ids are equal.
#[allow(dead_code)]
pub fn vertex_ids_equal(a: VertexId, b: VertexId) -> bool {
    a == b
}

/// Pack vertex id into a u64.
#[allow(dead_code)]
pub fn vertex_id_to_u64(id: VertexId) -> u64 {
    ((id.generation as u64) << 32) | (id.index as u64)
}

/// Unpack vertex id from parts.
#[allow(dead_code)]
pub fn vertex_id_from_parts(index: u32, generation: u32) -> VertexId {
    VertexId { index, generation }
}

/// Check if a vertex id is valid (non-max values).
#[allow(dead_code)]
pub fn vertex_id_is_valid(id: VertexId) -> bool {
    id.index != u32::MAX && id.generation != u32::MAX
}

/// Get next generation id for same index.
#[allow(dead_code)]
pub fn vertex_id_next(id: VertexId) -> VertexId {
    VertexId {
        index: id.index,
        generation: id.generation.wrapping_add(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_id() {
        let id = new_vertex_id(5, 1);
        assert_eq!(id.index, 5);
        assert_eq!(id.generation, 1);
    }

    #[test]
    fn test_vertex_id_index() {
        let id = new_vertex_id(42, 0);
        assert_eq!(vertex_id_index(id), 42);
    }

    #[test]
    fn test_vertex_id_generation() {
        let id = new_vertex_id(0, 7);
        assert_eq!(vertex_id_generation(id), 7);
    }

    #[test]
    fn test_vertex_ids_equal() {
        let a = new_vertex_id(1, 1);
        let b = new_vertex_id(1, 1);
        let c = new_vertex_id(1, 2);
        assert!(vertex_ids_equal(a, b));
        assert!(!vertex_ids_equal(a, c));
    }

    #[test]
    fn test_vertex_id_to_u64() {
        let id = new_vertex_id(10, 20);
        let packed = vertex_id_to_u64(id);
        assert_eq!(packed, (20_u64 << 32) | 10);
    }

    #[test]
    fn test_vertex_id_from_parts() {
        let id = vertex_id_from_parts(3, 4);
        assert_eq!(id.index, 3);
        assert_eq!(id.generation, 4);
    }

    #[test]
    fn test_vertex_id_is_valid() {
        assert!(vertex_id_is_valid(new_vertex_id(0, 0)));
        assert!(!vertex_id_is_valid(new_vertex_id(u32::MAX, 0)));
        assert!(!vertex_id_is_valid(new_vertex_id(0, u32::MAX)));
    }

    #[test]
    fn test_vertex_id_next() {
        let id = new_vertex_id(5, 0);
        let next = vertex_id_next(id);
        assert_eq!(next.index, 5);
        assert_eq!(next.generation, 1);
    }

    #[test]
    fn test_vertex_id_next_wrapping() {
        let id = new_vertex_id(0, u32::MAX);
        let next = vertex_id_next(id);
        assert_eq!(next.generation, 0);
    }

    #[test]
    fn test_vertex_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(new_vertex_id(1, 0));
        set.insert(new_vertex_id(1, 0));
        assert_eq!(set.len(), 1);
    }
}
