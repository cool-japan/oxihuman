#![allow(dead_code)]
//! Mesh content hashing.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A mesh hash result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshHash {
    pub hash_value: u64,
}

fn hash_f32_slice(data: &[[f32; 3]]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for v in data {
        for &f in v {
            f.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Hash vertex positions.
#[allow(dead_code)]
pub fn hash_positions(positions: &[[f32; 3]]) -> u64 {
    hash_f32_slice(positions)
}

/// Hash vertex normals.
#[allow(dead_code)]
pub fn hash_normals(normals: &[[f32; 3]]) -> u64 {
    hash_f32_slice(normals)
}

/// Hash UV coordinates.
#[allow(dead_code)]
pub fn hash_uvs(uvs: &[[f32; 2]]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for uv in uvs {
        for &f in uv {
            f.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Hash triangle indices.
#[allow(dead_code)]
pub fn hash_indices(indices: &[[u32; 3]]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for tri in indices {
        for &i in tri {
            i.hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Hash all mesh data combined.
#[allow(dead_code)]
pub fn hash_mesh_full(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[[u32; 3]],
) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash_positions(positions).hash(&mut hasher);
    hash_normals(normals).hash(&mut hasher);
    hash_uvs(uvs).hash(&mut hasher);
    hash_indices(indices).hash(&mut hasher);
    hasher.finish()
}

/// Convert a hash to hex string.
#[allow(dead_code)]
pub fn mesh_hash_to_hex(hash: u64) -> String {
    format!("{:016x}", hash)
}

/// Check if two mesh hashes are equal.
#[allow(dead_code)]
pub fn mesh_hashes_equal(a: u64, b: u64) -> bool {
    a == b
}

/// Hash the vertex count as a simple fingerprint.
#[allow(dead_code)]
pub fn hash_vertex_count(count: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    count.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_positions() {
        let p = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let h = hash_positions(&p);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_hash_positions_deterministic() {
        let p = vec![[1.0, 2.0, 3.0]];
        assert_eq!(hash_positions(&p), hash_positions(&p));
    }

    #[test]
    fn test_hash_normals() {
        let n = vec![[0.0, 0.0, 1.0]];
        let h = hash_normals(&n);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_hash_uvs() {
        let uvs = vec![[0.0, 1.0], [0.5, 0.5]];
        let h = hash_uvs(&uvs);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_hash_indices() {
        let i = vec![[0u32, 1, 2]];
        let h = hash_indices(&i);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_hash_mesh_full() {
        let p = vec![[0.0, 0.0, 0.0]];
        let n = vec![[0.0, 0.0, 1.0]];
        let uv = vec![[0.0, 0.0]];
        let i = vec![[0u32, 0, 0]];
        let h = hash_mesh_full(&p, &n, &uv, &i);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_mesh_hash_to_hex() {
        let hex = mesh_hash_to_hex(255);
        assert_eq!(hex, "00000000000000ff");
    }

    #[test]
    fn test_mesh_hashes_equal() {
        assert!(mesh_hashes_equal(42, 42));
        assert!(!mesh_hashes_equal(42, 43));
    }

    #[test]
    fn test_hash_vertex_count() {
        let h1 = hash_vertex_count(100);
        let h2 = hash_vertex_count(200);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_different_positions_different_hash() {
        let p1 = vec![[1.0, 0.0, 0.0]];
        let p2 = vec![[0.0, 1.0, 0.0]];
        assert_ne!(hash_positions(&p1), hash_positions(&p2));
    }
}
