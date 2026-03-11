// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::engine::MeshBuffers;
use crate::params::ParamState;

/// Cached result of `build_mesh()` with the params that produced it.
pub struct MeshCache {
    cached_params: Option<ParamState>,
    cached_mesh: Option<MeshBuffers>,
}

impl MeshCache {
    pub fn new() -> Self {
        MeshCache {
            cached_params: None,
            cached_mesh: None,
        }
    }

    /// Returns true if cached params match current params (by value).
    pub fn is_valid(&self, params: &ParamState) -> bool {
        self.cached_params
            .as_ref()
            .map(|p| p == params)
            .unwrap_or(false)
    }

    /// Store a newly-built mesh with the params that produced it.
    pub fn store(&mut self, params: ParamState, mesh: MeshBuffers) {
        self.cached_params = Some(params);
        self.cached_mesh = Some(mesh);
    }

    /// Retrieve a reference to the cached mesh (if valid).
    pub fn get(&self) -> Option<&MeshBuffers> {
        self.cached_mesh.as_ref()
    }

    /// Invalidate the cache (e.g., after a new target is loaded).
    pub fn invalidate(&mut self) {
        self.cached_params = None;
        self.cached_mesh = None;
    }
}

impl Default for MeshCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParamState;

    fn sample_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0]],
            normals: vec![[0.0, 1.0, 0.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        }
    }

    #[test]
    fn empty_cache_is_invalid() {
        let cache = MeshCache::new();
        assert!(!cache.is_valid(&ParamState::default()));
        assert!(cache.get().is_none());
    }

    #[test]
    fn cache_valid_after_store() {
        let mut cache = MeshCache::new();
        let params = ParamState::new(0.5, 0.5, 0.5, 0.5);
        cache.store(params.clone(), sample_mesh());
        assert!(cache.is_valid(&params));
        assert!(cache.get().is_some());
    }

    #[test]
    fn different_params_invalidate() {
        let mut cache = MeshCache::new();
        cache.store(ParamState::new(0.5, 0.5, 0.5, 0.5), sample_mesh());
        assert!(!cache.is_valid(&ParamState::new(0.6, 0.5, 0.5, 0.5)));
    }

    #[test]
    fn explicit_invalidate_clears_cache() {
        let mut cache = MeshCache::new();
        let params = ParamState::new(0.5, 0.5, 0.5, 0.5);
        cache.store(params.clone(), sample_mesh());
        cache.invalidate();
        assert!(!cache.is_valid(&params));
    }
}
