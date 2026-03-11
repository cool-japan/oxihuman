// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The core morph engine: [`HumanEngine`].
//!
//! `HumanEngine` drives procedural human-body generation. It holds the base
//! mesh in Structure-of-Arrays (SoA) form for cache-friendly scatter-add, a
//! [`TargetLibrary`] of morph targets (each paired with a weight function
//! evaluated against [`ParamState`]), a result cache keyed on the current
//! params, and an optional incremental position buffer for fast per-frame
//! updates when only a subset of target weights changes.
//!
//! # Build modes
//!
//! | Method | Description |
//! |---|---|
//! | [`HumanEngine::build_mesh`] | Single-threaded; uses result cache. |
//! | [`HumanEngine::build_mesh_parallel`] | Rayon parallel scatter-add. |
//! | [`HumanEngine::build_mesh_incremental`] | Reuses last position buffer; only re-applies changed-weight targets. |

use std::cell::RefCell;

use anyhow::Result;
use oxihuman_core::parser::obj::ObjMesh;
use oxihuman_core::parser::target::TargetFile;
use oxihuman_core::policy::Policy;

use crate::apply::{apply_target, apply_targets_parallel, reset_from_base, soa_to_aos};
use crate::cache::MeshCache;
use crate::constraint::clamp_params;
use crate::params::ParamState;
use crate::target_lib::TargetLibrary;
use oxihuman_core::parser::target::Delta;

/// Intermediate mesh buffers returned by [`HumanEngine::build_mesh`].
///
/// These are raw SoA→AoS converted buffers straight from the morph engine.
/// The `oxihuman-mesh` crate wraps them in `oxihuman_mesh::MeshBuffers`,
/// which recomputes normals and adds tangents before export.
///
/// `has_suit` is a safety flag: exporters that produce dressed/clothed output
/// check this field and refuse to write if `false`, preventing accidental
/// export of an unclothed mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshBuffers {
    /// Per-vertex XYZ positions after morph application.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals (copied verbatim from base mesh; recomputed by `oxihuman-mesh`).
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex UV texture coordinates.
    pub uvs: Vec<[f32; 2]>,
    /// Triangle index list (groups of 3).
    pub indices: Vec<u32>,
    /// Safety flag: `true` when a suit/clothing mesh has been applied.
    pub has_suit: bool,
}

impl MeshBuffers {
    /// Returns true if all position values between `self` and `other` differ by less than `eps`.
    /// Normals, UVs, and indices are not compared.
    #[allow(dead_code)]
    pub fn approx_eq(&self, other: &Self, eps: f32) -> bool {
        if self.positions.len() != other.positions.len() {
            return false;
        }
        self.positions
            .iter()
            .zip(other.positions.iter())
            .all(|(a, b)| {
                (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps && (a[2] - b[2]).abs() < eps
            })
    }
}

/// Main entry point for the OxiHuman morph engine.
///
/// Manages the base mesh, a library of morph targets, parameter state, and
/// three levels of result caching (exact-params cache, incremental SoA buffer,
/// and rayon parallel build). All parameter values are clamped to `[0.0, 1.0]`
/// before any computation.
///
/// # Examples
///
/// ```rust
/// use oxihuman_core::parser::obj::parse_obj;
/// use oxihuman_core::policy::{Policy, PolicyProfile};
/// use oxihuman_morph::engine::HumanEngine;
/// use oxihuman_morph::params::ParamState;
///
/// let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nvt 0 0\nf 1/1/1 2/1/1 3/1/1\n";
/// let base = parse_obj(obj).unwrap();
/// let policy = Policy::new(PolicyProfile::Standard);
/// let mut engine = HumanEngine::new(base, policy);
///
/// engine.set_params(ParamState::new(0.8, 0.5, 0.5, 0.5));
/// let mesh = engine.build_mesh();
/// assert_eq!(mesh.positions.len(), 3);
/// ```
pub struct HumanEngine {
    /// Base positions stored as SoA (Structure of Arrays) for cache-friendly scatter-add.
    base_x: Vec<f32>,
    base_y: Vec<f32>,
    base_z: Vec<f32>,
    base_normals: Vec<[f32; 3]>,
    base_uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    targets: TargetLibrary,
    policy: Policy,
    params: ParamState,
    cache: RefCell<MeshCache>,
    /// Cached positions from the last incremental or full build.
    cached_positions: Option<Vec<[f32; 3]>>,
    /// Params used for the last incremental build (to compute weight deltas).
    last_params: Option<ParamState>,
}

impl HumanEngine {
    /// Create engine from a parsed base mesh and a policy.
    pub fn new(base: ObjMesh, policy: Policy) -> Self {
        let n = base.positions.len();
        let mut base_x = vec![0.0f32; n];
        let mut base_y = vec![0.0f32; n];
        let mut base_z = vec![0.0f32; n];
        reset_from_base(&mut base_x, &mut base_y, &mut base_z, &base.positions);
        HumanEngine {
            base_x,
            base_y,
            base_z,
            base_normals: base.normals,
            base_uvs: base.uvs,
            indices: base.indices,
            targets: TargetLibrary::new(),
            policy,
            params: ParamState::default(),
            cache: RefCell::new(MeshCache::new()),
            cached_positions: None,
            last_params: None,
        }
    }

    /// Load a morph target with a weight function (if policy permits).
    pub fn load_target(
        &mut self,
        t: TargetFile,
        weight_fn: Box<dyn Fn(&ParamState) -> f32 + Send + Sync>,
    ) {
        if !self.policy.is_target_allowed(&t.name, &[]) {
            return;
        }
        self.targets.add(t, weight_fn);
        self.cache.borrow_mut().invalidate();
        // Invalidate incremental cache: target library changed, positions are stale.
        self.cached_positions = None;
        self.last_params = None;
    }

    /// Load all `.target` files from a directory using a shared weight function factory.
    /// Targets that fail to parse are skipped with a warning (never panics).
    /// Returns the number of targets successfully loaded.
    pub fn load_targets_from_dir<F>(
        &mut self,
        dir: &std::path::Path,
        weight_fn_factory: F,
    ) -> Result<usize>
    where
        F: Fn(&str) -> Box<dyn Fn(&ParamState) -> f32 + Send + Sync>,
    {
        use oxihuman_core::parser::target::parse_target;
        let mut count = 0usize;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "target").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                if let Ok(src) = std::fs::read_to_string(&path) {
                    if let Ok(target) = parse_target(&name, &src) {
                        let wf = weight_fn_factory(&name);
                        self.load_target(target, wf);
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Load all `.target` files from a directory using automatic weight functions
    /// inferred from target filenames (via `weight_curves::auto_weight_fn_for_target`).
    pub fn load_targets_from_dir_auto(&mut self, dir: &std::path::Path) -> Result<usize> {
        use crate::weight_curves::auto_weight_fn_for_target;
        self.load_targets_from_dir(dir, |name| auto_weight_fn_for_target(name))
    }

    /// Set the current morph parameters, clamped to `[0.0, 1.0]`.
    ///
    /// This does **not** invalidate the incremental position cache, so the next
    /// call to [`Self::build_mesh_incremental`] can compute only the delta
    /// against the previous parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oxihuman_core::parser::obj::parse_obj;
    /// # use oxihuman_core::policy::{Policy, PolicyProfile};
    /// # use oxihuman_morph::engine::HumanEngine;
    /// # use oxihuman_morph::params::ParamState;
    /// # let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nvt 0 0\nf 1/1/1 2/1/1 3/1/1\n";
    /// # let base = parse_obj(obj).unwrap();
    /// # let policy = Policy::new(PolicyProfile::Standard);
    /// # let mut engine = HumanEngine::new(base, policy);
    /// // Out-of-range values are silently clamped.
    /// engine.set_params(ParamState::new(2.0, -0.5, 0.5, 0.5));
    /// let mesh = engine.build_mesh();
    /// // height was clamped to 1.0, weight to 0.0
    /// ```
    pub fn set_params(&mut self, mut p: ParamState) {
        clamp_params(&mut p);
        self.params = p;
    }

    /// Explicitly clear the incremental position cache and last-params snapshot.
    /// The next call to `build_mesh_incremental` will fall back to a full rebuild.
    #[allow(dead_code)]
    pub fn clear_incremental_cache(&mut self) {
        self.cached_positions = None;
        self.last_params = None;
    }

    /// Number of vertices in base mesh.
    pub fn vertex_count(&self) -> usize {
        self.base_x.len()
    }

    /// Number of morph targets loaded into the library.
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    /// Replace the policy used for target filtering.
    pub fn set_policy(&mut self, policy: Policy) {
        self.policy = policy;
    }

    /// Apply all active morph targets and return the blended mesh.
    ///
    /// If the current [`ParamState`] matches the last call's state the cached
    /// result is returned immediately without any arithmetic. Otherwise, all
    /// target weight functions are evaluated and their deltas are scatter-added
    /// into a clone of the SoA base buffers.
    ///
    /// Use [`Self::build_mesh_parallel`] when many targets are loaded and you
    /// need maximum throughput. Use [`Self::build_mesh_incremental`] for
    /// interactive sliders where only one or two params change per frame.
    pub fn build_mesh(&self) -> MeshBuffers {
        // Return cached result if params haven't changed
        {
            let cache = self.cache.borrow();
            if cache.is_valid(&self.params) {
                if let Some(mesh) = cache.get() {
                    return mesh.clone();
                }
            }
        }

        let mut x = self.base_x.clone();
        let mut y = self.base_y.clone();
        let mut z = self.base_z.clone();

        for (deltas, weight) in self.targets.iter_weighted(&self.params) {
            apply_target(&mut x, &mut y, &mut z, deltas, weight);
        }

        let mesh = MeshBuffers {
            positions: soa_to_aos(&x, &y, &z),
            normals: self.base_normals.clone(),
            uvs: self.base_uvs.clone(),
            indices: self.indices.clone(),
            has_suit: false,
        };

        self.cache
            .borrow_mut()
            .store(self.params.clone(), mesh.clone());
        mesh
    }

    /// Build the morphed mesh using rayon parallel target application.
    /// Faster than `build_mesh()` when many targets are active.
    /// Uses the same cache as `build_mesh()`.
    pub fn build_mesh_parallel(&self) -> MeshBuffers {
        // Check cache first
        {
            let cache = self.cache.borrow();
            if cache.is_valid(&self.params) {
                if let Some(mesh) = cache.get() {
                    return mesh.clone();
                }
            }
        }

        let mut x = self.base_x.clone();
        let mut y = self.base_y.clone();
        let mut z = self.base_z.clone();

        // Collect all (deltas, weight) pairs
        let weighted: Vec<(&[Delta], f32)> = self.targets.iter_weighted(&self.params).collect();

        apply_targets_parallel(&mut x, &mut y, &mut z, &weighted);

        let mesh = MeshBuffers {
            positions: soa_to_aos(&x, &y, &z),
            normals: self.base_normals.clone(),
            uvs: self.base_uvs.clone(),
            indices: self.indices.clone(),
            has_suit: false,
        };

        self.cache
            .borrow_mut()
            .store(self.params.clone(), mesh.clone());
        mesh
    }

    /// Build the morphed mesh incrementally, reusing the cached position buffer from
    /// the previous call and only reapplying deltas for targets whose weight changed.
    ///
    /// # Strategy
    /// For each target:
    ///   - Compute `old_weight` = weight evaluated at `last_params`
    ///   - Compute `new_weight` = weight evaluated at current `params`
    ///   - If `old_weight == new_weight`: skip (no change)
    ///   - Otherwise: subtract old contribution, add new contribution
    ///
    /// Falls back to a full `build_mesh()` on the first call (no cache) or after
    /// `load_target()` / `clear_incremental_cache()`.
    pub fn build_mesh_incremental(&mut self) -> MeshBuffers {
        // --- First call or cache invalidated: full rebuild ---
        if self.cached_positions.is_none() || self.last_params.is_none() {
            let mesh = self.build_mesh();
            self.cached_positions = Some(mesh.positions.clone());
            self.last_params = Some(self.params.clone());
            return mesh;
        }

        let last = match self.last_params.as_ref() {
            Some(p) => p.clone(),
            None => return self.build_mesh(),
        };

        // Early-out: params haven't changed at all
        if last == self.params {
            let positions = match self.cached_positions.as_ref() {
                Some(p) => p.clone(),
                None => return self.build_mesh(),
            };
            return MeshBuffers {
                positions,
                normals: self.base_normals.clone(),
                uvs: self.base_uvs.clone(),
                indices: self.indices.clone(),
                has_suit: false,
            };
        }

        // Convert cached AoS positions back to SoA for scatter-add
        let cached = match self.cached_positions.as_ref() {
            Some(p) => p,
            None => return self.build_mesh(),
        };
        let n = cached.len();
        let mut x: Vec<f32> = (0..n).map(|i| cached[i][0]).collect();
        let mut y: Vec<f32> = (0..n).map(|i| cached[i][1]).collect();
        let mut z: Vec<f32> = (0..n).map(|i| cached[i][2]).collect();

        // Collect old and new weights for each target (two O(targets) passes).
        // Targets yielded in stable insertion order — indices line up.
        let old_weights: Vec<f32> = self.targets.iter_weighted(&last).map(|(_, w)| w).collect();
        let new_weights: Vec<f32> = self
            .targets
            .iter_weighted(&self.params)
            .map(|(_, w)| w)
            .collect();

        for (i, (deltas, _)) in self.targets.iter_weighted(&self.params).enumerate() {
            let old_w = old_weights[i];
            let new_w = new_weights[i];
            if (old_w - new_w).abs() < f32::EPSILON {
                continue;
            }
            // Subtract old contribution
            if old_w != 0.0 {
                apply_target(&mut x, &mut y, &mut z, deltas, -old_w);
            }
            // Add new contribution
            if new_w != 0.0 {
                apply_target(&mut x, &mut y, &mut z, deltas, new_w);
            }
        }

        let new_positions = soa_to_aos(&x, &y, &z);
        self.cached_positions = Some(new_positions.clone());
        self.last_params = Some(self.params.clone());

        MeshBuffers {
            positions: new_positions,
            normals: self.base_normals.clone(),
            uvs: self.base_uvs.clone(),
            indices: self.indices.clone(),
            has_suit: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::target::{Delta, TargetFile};
    use oxihuman_core::policy::PolicyProfile;

    fn simple_base() -> ObjMesh {
        ObjMesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
        }
    }

    #[test]
    fn build_mesh_no_targets() {
        let policy = Policy::new(PolicyProfile::Standard);
        let engine = HumanEngine::new(simple_base(), policy);
        let mesh = engine.build_mesh();
        assert_eq!(mesh.positions.len(), 3);
        assert!((mesh.positions[0][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn build_mesh_with_target() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        let t = TargetFile {
            name: "height".to_string(),
            deltas: vec![Delta {
                vid: 1,
                dx: 0.5,
                dy: 0.0,
                dz: 0.0,
            }],
        };
        engine.load_target(t, Box::new(|p: &ParamState| p.height));
        engine.set_params(ParamState::new(1.0, 0.5, 0.5, 0.5));

        let mesh = engine.build_mesh();
        assert!((mesh.positions[1][0] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn policy_blocks_explicit_target() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        let t = TargetFile {
            name: "explicit-body".to_string(),
            deltas: vec![Delta {
                vid: 0,
                dx: 100.0,
                dy: 0.0,
                dz: 0.0,
            }],
        };
        engine.load_target(t, Box::new(|_: &ParamState| 1.0));
        let mesh = engine.build_mesh();
        // blocked target → position unchanged
        assert!((mesh.positions[0][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn load_targets_from_dir_loads_real_targets() {
        let policy = Policy::new(PolicyProfile::Standard);
        // Use a small base (3 verts) just to test loading, not positions
        let mut engine = HumanEngine::new(simple_base(), policy);
        let dir = std::path::Path::new(
            "/media/kitasan/Backup/resource/makehuman/makehuman/data/targets/bodyshapes",
        );
        if dir.exists() {
            let count = engine
                .load_targets_from_dir(dir, |_name| Box::new(|_p: &ParamState| 0.5f32))
                .unwrap();
            assert!(count > 0, "should load at least one target");
        }
    }

    #[test]
    fn load_targets_auto_weight() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        let dir = std::path::Path::new(
            "/media/kitasan/Backup/resource/makehuman/makehuman/data/targets/bodyshapes",
        );
        if dir.exists() {
            let count = engine.load_targets_from_dir_auto(dir).unwrap();
            assert!(count > 0);
            // Build with different params — should work without panic
            engine.set_params(ParamState::new(0.3, 0.8, 0.2, 0.6));
            let mesh = engine.build_mesh();
            for pos in &mesh.positions {
                assert!(pos[0].is_finite());
                assert!(pos[1].is_finite());
                assert!(pos[2].is_finite());
            }
        }
    }

    #[test]
    fn build_mesh_returns_cached_result() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.set_params(ParamState::new(0.5, 0.5, 0.5, 0.5));
        let mesh1 = engine.build_mesh();
        let mesh2 = engine.build_mesh(); // should hit cache
        assert_eq!(mesh1.positions, mesh2.positions);
    }

    #[test]
    fn cache_invalidated_after_new_target() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.set_params(ParamState::new(1.0, 0.5, 0.5, 0.5));
        let mesh_before = engine.build_mesh();

        // Add a target that shifts vertex 0
        let t = TargetFile {
            name: "shift".to_string(),
            deltas: vec![Delta {
                vid: 0,
                dx: 5.0,
                dy: 0.0,
                dz: 0.0,
            }],
        };
        engine.load_target(t, Box::new(|_: &ParamState| 1.0));
        let mesh_after = engine.build_mesh(); // must rebuild, not use stale cache

        assert_ne!(mesh_before.positions[0], mesh_after.positions[0]);
    }

    #[test]
    fn parallel_build_matches_sequential() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        let t = TargetFile {
            name: "height".to_string(),
            deltas: vec![Delta {
                vid: 0,
                dx: 0.5,
                dy: 0.0,
                dz: 0.0,
            }],
        };
        engine.load_target(t, Box::new(|p: &ParamState| p.height));
        engine.set_params(ParamState::new(0.8, 0.5, 0.5, 0.5));

        let seq = engine.build_mesh();
        // Invalidate cache by changing params and back
        engine.set_params(ParamState::new(0.8, 0.5, 0.5, 0.5));
        let par = engine.build_mesh_parallel();

        assert_eq!(seq.positions.len(), par.positions.len());
        for (s, p) in seq.positions.iter().zip(par.positions.iter()) {
            assert!((s[0] - p[0]).abs() < 1e-5);
        }
    }

    // ---- Incremental update tests ----

    fn make_target(name: &str, vid: u32, dx: f32, dy: f32, dz: f32) -> TargetFile {
        TargetFile {
            name: name.to_string(),
            deltas: vec![Delta { vid, dx, dy, dz }],
        }
    }

    /// `build_mesh()` and `build_mesh_incremental()` must produce identical positions.
    #[test]
    fn incremental_matches_full_build() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.load_target(
            make_target("height", 0, 0.3, 0.0, 0.0),
            Box::new(|p: &ParamState| p.height),
        );
        engine.set_params(ParamState::new(0.8, 0.5, 0.5, 0.5));

        let full = engine.build_mesh();
        let inc = engine.build_mesh_incremental();

        assert!(
            full.approx_eq(&inc, 1e-5),
            "incremental diverged from full build: {:?} vs {:?}",
            full.positions,
            inc.positions
        );
    }

    /// Change one param, verify result matches a fresh full build.
    #[test]
    fn incremental_updates_correctly() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.load_target(
            make_target("height", 1, 2.0, 0.0, 0.0),
            Box::new(|p: &ParamState| p.height),
        );

        // First call — initialises cache
        engine.set_params(ParamState::new(0.0, 0.5, 0.5, 0.5));
        let _ = engine.build_mesh_incremental();

        // Change height param
        engine.set_params(ParamState::new(0.75, 0.5, 0.5, 0.5));
        let inc = engine.build_mesh_incremental();
        let full = engine.build_mesh();

        assert!(
            full.approx_eq(&inc, 1e-5),
            "after param change: incremental={:?}, full={:?}",
            inc.positions,
            full.positions
        );
        // vertex 1 x should be 1.0 + 2.0 * 0.75 = 2.5
        assert!(
            (inc.positions[1][0] - 2.5).abs() < 1e-5,
            "expected 2.5, got {}",
            inc.positions[1][0]
        );
    }

    /// After `load_target`, the incremental cache must be cleared so the next
    /// `build_mesh_incremental` reflects the newly added target.
    #[test]
    fn incremental_cache_invalidated_on_load_target() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.set_params(ParamState::new(1.0, 0.5, 0.5, 0.5));

        // Build with no targets — seeds the cache
        let _ = engine.build_mesh_incremental();
        assert!(engine.cached_positions.is_some());

        // Add a target that moves vertex 2 significantly
        engine.load_target(
            make_target("new_target", 2, 0.0, 10.0, 0.0),
            Box::new(|_: &ParamState| 1.0),
        );

        // Cache must have been cleared
        assert!(
            engine.cached_positions.is_none(),
            "incremental cache should be None after load_target"
        );

        // Rebuild — must include the new target
        let inc = engine.build_mesh_incremental();
        let full = engine.build_mesh();
        assert!(
            full.approx_eq(&inc, 1e-5),
            "after load_target, incremental should match full build"
        );
        assert!(
            (inc.positions[2][1] - 11.0).abs() < 1e-5,
            "expected y=11.0, got {}",
            inc.positions[2][1]
        );
    }

    /// Make 3 successive param changes; each incremental result must match a full build.
    #[test]
    fn incremental_multiple_param_changes() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.load_target(
            make_target("height", 0, 1.0, 0.0, 0.0),
            Box::new(|p: &ParamState| p.height),
        );
        engine.load_target(
            make_target("weight", 1, 0.0, 1.0, 0.0),
            Box::new(|p: &ParamState| p.weight),
        );

        let param_sets = [
            ParamState::new(0.2, 0.8, 0.5, 0.5),
            ParamState::new(0.6, 0.3, 0.5, 0.5),
            ParamState::new(1.0, 1.0, 0.5, 0.5),
        ];

        for params in &param_sets {
            engine.set_params(params.clone());
            let inc = engine.build_mesh_incremental();
            let full = engine.build_mesh();
            assert!(
                full.approx_eq(&inc, 1e-5),
                "params {:?}: incremental={:?}, full={:?}",
                params,
                inc.positions,
                full.positions
            );
        }
    }

    /// With 0 targets loaded, `build_mesh_incremental` must return base positions.
    #[test]
    fn incremental_with_no_targets() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        engine.set_params(ParamState::new(0.5, 0.5, 0.5, 0.5));

        let inc = engine.build_mesh_incremental();
        let full = engine.build_mesh();

        assert!(
            full.approx_eq(&inc, 1e-6),
            "no-target incremental should equal full build"
        );
        // All positions should equal the base mesh
        assert!((inc.positions[0][0] - 0.0).abs() < 1e-6);
        assert!((inc.positions[1][0] - 1.0).abs() < 1e-6);
        assert!((inc.positions[2][1] - 1.0).abs() < 1e-6);
    }

    /// A target with weight 0.0 must not displace any vertex.
    #[test]
    fn incremental_zero_weight_target_has_no_effect() {
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(simple_base(), policy);
        // height param = 0.0 → weight = 0.0
        engine.load_target(
            make_target("height", 0, 999.0, 999.0, 999.0),
            Box::new(|p: &ParamState| p.height),
        );
        engine.set_params(ParamState::new(0.0, 0.5, 0.5, 0.5));

        let inc = engine.build_mesh_incremental();
        // vertex 0 should be at base position (0, 0, 0)
        assert!(
            (inc.positions[0][0] - 0.0).abs() < 1e-6,
            "zero-weight target shifted vertex 0 x: {}",
            inc.positions[0][0]
        );
        assert!(
            (inc.positions[0][1] - 0.0).abs() < 1e-6,
            "zero-weight target shifted vertex 0 y: {}",
            inc.positions[0][1]
        );
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_params_no_nan(
            h in 0.0f32..=1.0f32,
            w in 0.0f32..=1.0f32,
            m in 0.0f32..=1.0f32,
            a in 0.0f32..=1.0f32,
        ) {
            let policy = Policy::new(PolicyProfile::Standard);
            let base = simple_base();
            let mut engine = HumanEngine::new(base, policy);
            engine.set_params(ParamState::new(h, w, m, a));
            let mesh = engine.build_mesh();
            for pos in &mesh.positions {
                prop_assert!(!pos[0].is_nan(), "NaN in x");
                prop_assert!(!pos[1].is_nan(), "NaN in y");
                prop_assert!(!pos[2].is_nan(), "NaN in z");
            }
        }

        #[test]
        fn params_always_clamped(
            h in -10.0f32..10.0f32,
            w in -10.0f32..10.0f32,
            m in -10.0f32..10.0f32,
            a in -10.0f32..10.0f32,
        ) {
            let policy = Policy::new(PolicyProfile::Standard);
            let mut engine = HumanEngine::new(simple_base(), policy);
            engine.set_params(ParamState::new(h, w, m, a));
            let mesh = engine.build_mesh();
            // After clamping, positions must be finite
            for pos in &mesh.positions {
                prop_assert!(pos[0].is_finite());
                prop_assert!(pos[1].is_finite());
                prop_assert!(pos[2].is_finite());
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use oxihuman_core::parser::obj::parse_obj;
    use oxihuman_core::parser::target::parse_target;
    use oxihuman_core::policy::PolicyProfile;

    const TARGETS_DIR: &str = "/media/kitasan/Backup/resource/makehuman/makehuman/data/targets";
    const BASE_OBJ: &str =
        "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";

    #[allow(dead_code)]
    fn walk_targets(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_targets(&path, out);
                } else if path.extension().and_then(|e| e.to_str()) == Some("target") {
                    out.push(path);
                }
            }
        }
    }

    #[test]
    fn all_targets_parse_without_error() {
        let dir = std::path::Path::new(TARGETS_DIR);
        if !dir.exists() {
            return;
        }
        let mut paths = Vec::new();
        walk_targets(dir, &mut paths);
        let mut count = 0usize;
        for path in &paths {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let src = std::fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Failed to read {:?}", path));
            let result = parse_target(name, &src);
            assert!(
                result.is_ok(),
                "Failed to parse {:?}: {:?}",
                path,
                result.err()
            );
            count += 1;
        }
        println!("Parsed {} target files successfully", count);
    }

    #[test]
    fn all_targets_apply_no_nan() {
        let base_path = std::path::Path::new(BASE_OBJ);
        if !base_path.exists() {
            return;
        }
        let dir = std::path::Path::new(TARGETS_DIR);
        if !dir.exists() {
            return;
        }
        let base_src = std::fs::read_to_string(base_path).expect("Failed to read base.obj");
        let base_mesh = parse_obj(&base_src).expect("Failed to parse base.obj");

        let mut paths = Vec::new();
        walk_targets(dir, &mut paths);
        paths.sort();

        for path in paths.iter().take(50) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let src = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let target = match parse_target(name, &src) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let policy = Policy::new(PolicyProfile::Standard);
            let mut engine = HumanEngine::new(base_mesh.clone(), policy);
            engine.load_target(target, Box::new(|_: &ParamState| 1.0));
            let mesh = engine.build_mesh();
            for pos in &mesh.positions {
                assert!(pos[0].is_finite(), "NaN/Inf in x for {:?}", path);
                assert!(pos[1].is_finite(), "NaN/Inf in y for {:?}", path);
                assert!(pos[2].is_finite(), "NaN/Inf in z for {:?}", path);
            }
        }
    }

    #[test]
    fn multi_target_blend_no_nan() {
        let base_path = std::path::Path::new(BASE_OBJ);
        if !base_path.exists() {
            return;
        }
        let dir = std::path::Path::new(TARGETS_DIR);
        if !dir.exists() {
            return;
        }
        let base_src = std::fs::read_to_string(base_path).expect("Failed to read base.obj");
        let base_mesh = parse_obj(&base_src).expect("Failed to parse base.obj");

        let mut paths = Vec::new();
        walk_targets(dir, &mut paths);
        paths.sort();

        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(base_mesh, policy);

        for path in paths.iter().take(20) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let src = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let target = match parse_target(name, &src) {
                Ok(t) => t,
                Err(_) => continue,
            };
            engine.load_target(target, Box::new(|_: &ParamState| 0.5));
        }

        let mesh = engine.build_mesh();
        for pos in &mesh.positions {
            assert!(pos[0].is_finite(), "NaN/Inf in x after multi-blend");
            assert!(pos[1].is_finite(), "NaN/Inf in y after multi-blend");
            assert!(pos[2].is_finite(), "NaN/Inf in z after multi-blend");
        }
    }

    #[test]
    fn target_count_reasonable() {
        let dir = std::path::Path::new(TARGETS_DIR);
        if !dir.exists() {
            return;
        }
        let mut paths = Vec::new();
        walk_targets(dir, &mut paths);
        assert!(
            paths.len() > 100,
            "Expected more than 100 target files, found {}",
            paths.len()
        );
    }
}
