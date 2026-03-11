// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashSet;

use crate::params::ParamState;
use oxihuman_core::parser::target::{Delta, TargetFile};

/// A loaded target with its weight function.
pub struct LoadedTarget {
    pub name: String,
    pub deltas: Vec<Delta>,
    pub weight_fn: Box<dyn Fn(&ParamState) -> f32 + Send + Sync>,
}

/// Library of all loaded morph targets.
pub struct TargetLibrary {
    targets: Vec<LoadedTarget>,
    loaded_names: HashSet<String>,
}

impl Default for TargetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetLibrary {
    pub fn new() -> Self {
        TargetLibrary {
            targets: Vec::new(),
            loaded_names: HashSet::new(),
        }
    }

    pub fn add(
        &mut self,
        target: TargetFile,
        weight_fn: Box<dyn Fn(&ParamState) -> f32 + Send + Sync>,
    ) {
        if self.loaded_names.contains(&target.name) {
            return; // deduplicate: same name already loaded
        }
        self.loaded_names.insert(target.name.clone());
        self.targets.push(LoadedTarget {
            name: target.name,
            deltas: target.deltas,
            weight_fn,
        });
    }

    pub fn len(&self) -> usize {
        self.targets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Returns true if a target with this name is already loaded.
    pub fn contains(&self, name: &str) -> bool {
        self.loaded_names.contains(name)
    }

    /// Iterate over all targets, yielding (name, deltas) pairs.
    ///
    /// Used by the delta cache to serialise the library without exposing
    /// the internal `LoadedTarget` type.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[Delta])> {
        self.targets
            .iter()
            .map(|t| (t.name.as_str(), t.deltas.as_slice()))
    }

    /// Iterate over targets with their evaluated weights given params.
    pub fn iter_weighted<'a>(
        &'a self,
        params: &'a ParamState,
    ) -> impl Iterator<Item = (&'a [Delta], f32)> + 'a {
        self.targets
            .iter()
            .map(move |t| (t.deltas.as_slice(), (t.weight_fn)(params)))
    }

    /// Compute library statistics.
    pub fn stats(&self) -> LibraryStats {
        let total_deltas: usize = self.targets.iter().map(|t| t.deltas.len()).sum();
        let max_vid = self
            .targets
            .iter()
            .flat_map(|t| t.deltas.iter().map(|d| d.vid))
            .max()
            .unwrap_or(0);
        LibraryStats {
            target_count: self.targets.len(),
            total_deltas,
            estimated_memory_bytes: total_deltas * 16, // 16 bytes per Delta (u32 + 3×f32)
            max_vertex_index: max_vid,
        }
    }
}

/// Statistics about the loaded target library.
#[derive(Debug, Clone)]
pub struct LibraryStats {
    /// Number of loaded targets.
    pub target_count: usize,
    /// Total number of vertex deltas across all targets.
    pub total_deltas: usize,
    /// Estimated memory usage in bytes.
    pub estimated_memory_bytes: usize,
    /// Highest vertex index referenced by any target.
    pub max_vertex_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::target::{Delta, TargetFile};

    #[test]
    fn add_and_iterate() {
        let mut lib = TargetLibrary::new();
        let t = TargetFile {
            name: "height".to_string(),
            deltas: vec![Delta {
                vid: 0,
                dx: 0.1,
                dy: 0.2,
                dz: 0.0,
            }],
        };
        lib.add(t, Box::new(|p: &ParamState| p.height));
        assert_eq!(lib.len(), 1);

        let params = ParamState::new(0.7, 0.5, 0.5, 0.5);
        let weights: Vec<f32> = lib.iter_weighted(&params).map(|(_, w)| w).collect();
        assert!((weights[0] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn deduplication_prevents_duplicate_targets() {
        let mut lib = TargetLibrary::new();
        let t1 = TargetFile {
            name: "height".to_string(),
            deltas: vec![Delta {
                vid: 0,
                dx: 0.1,
                dy: 0.0,
                dz: 0.0,
            }],
        };
        let t2 = TargetFile {
            name: "height".to_string(), // same name!
            deltas: vec![Delta {
                vid: 1,
                dx: 0.2,
                dy: 0.0,
                dz: 0.0,
            }],
        };
        lib.add(t1, Box::new(|_: &ParamState| 1.0));
        lib.add(t2, Box::new(|_: &ParamState| 1.0));
        assert_eq!(lib.len(), 1, "should deduplicate");
    }

    #[test]
    fn contains_returns_true_after_add() {
        let mut lib = TargetLibrary::new();
        let t = TargetFile {
            name: "muscle".to_string(),
            deltas: vec![Delta {
                vid: 5,
                dx: 0.0,
                dy: 0.1,
                dz: 0.0,
            }],
        };
        lib.add(t, Box::new(|_: &ParamState| 0.5));
        assert!(lib.contains("muscle"));
        assert!(!lib.contains("height"));
    }

    #[test]
    fn stats_reports_correct_counts() {
        let mut lib = TargetLibrary::new();
        for i in 0..3u32 {
            let t = TargetFile {
                name: format!("target_{}", i),
                deltas: vec![
                    Delta {
                        vid: i,
                        dx: 0.1,
                        dy: 0.0,
                        dz: 0.0,
                    },
                    Delta {
                        vid: i + 10,
                        dx: 0.0,
                        dy: 0.1,
                        dz: 0.0,
                    },
                ],
            };
            lib.add(t, Box::new(|_: &ParamState| 1.0));
        }
        let stats = lib.stats();
        assert_eq!(stats.target_count, 3);
        assert_eq!(stats.total_deltas, 6);
        assert_eq!(stats.estimated_memory_bytes, 96); // 6 * 16
        assert_eq!(stats.max_vertex_index, 12); // max of [0,10,1,11,2,12]
    }

    #[test]
    fn iter_yields_name_and_deltas() {
        let mut lib = TargetLibrary::new();
        let t = TargetFile {
            name: "test_iter".to_string(),
            deltas: vec![Delta {
                vid: 7,
                dx: 1.0,
                dy: 2.0,
                dz: 3.0,
            }],
        };
        lib.add(t, Box::new(|_: &ParamState| 1.0));
        let entries: Vec<_> = lib.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "test_iter");
        assert_eq!(entries[0].1.len(), 1);
        assert_eq!(entries[0].1[0].vid, 7);
    }
}
